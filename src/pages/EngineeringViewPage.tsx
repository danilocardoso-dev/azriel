import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { CameraPreview } from "../components/engineering/CameraPreview";
import { EngineeringScene } from "../components/engineering/EngineeringScene";
import { ModuleIntro } from "../components/layout/ModuleIntro";
import { CameraService } from "../engineering/cameraService";
import { ENGINEERING_CONFIG, HAND_LANDMARK_INDEX } from "../engineering/config";
import { evaluatePinch } from "../engineering/gestureEngine";
import { HandTrackingService } from "../engineering/handTrackingService";
import { normalizedToViewport, smoothLandmark } from "../engineering/trackingMath";
import type { CameraState, EngineeringCoreState, EngineeringObjectState, GestureState, HandLandmark, TrackedHand, ViewportPoint } from "../engineering/types";

const futureInterfaces = [
  { code: "CAD", label: "Modelos e componentes" },
  { code: "DTW", label: "Digital Twins" },
  { code: "IOT", label: "Sensores e telemetria" },
];

const cameraStateLabel: Record<CameraState, string> = {
  offline: "OFFLINE",
  requesting: "REQUESTING",
  online: "ONLINE",
  error: "ERROR",
};

export function EngineeringViewPage() {
  const videoRef = useRef<HTMLVideoElement>(null);
  const cameraService = useRef(new CameraService());
  const trackingService = useRef(new HandTrackingService());
  const startAttempt = useRef(0);
  const gestureRef = useRef<GestureState>("none");
  const smoothedCursorRef = useRef<HandLandmark | null>(null);
  const [cameraState, setCameraState] = useState<CameraState>("offline");
  const [trackingOnline, setTrackingOnline] = useState(false);
  const [trackedHand, setTrackedHand] = useState<TrackedHand | null>(null);
  const [gesture, setGesture] = useState<GestureState>("none");
  const [cursor, setCursor] = useState<ViewportPoint | null>(null);
  const [fps, setFps] = useState(0);
  const [previewVisible, setPreviewVisible] = useState(true);
  const [debugVisible, setDebugVisible] = useState(false);
  const [rendererReady, setRendererReady] = useState(false);
  const [objectStatus, setObjectStatus] = useState<EngineeringObjectState>("ready");
  const [resetSignal, setResetSignal] = useState(0);
  const [error, setError] = useState<string | null>(null);

  const coreState: EngineeringCoreState = useMemo(() => {
    if (cameraState === "error") return "error";
    if (cameraState === "requesting") return "requesting_camera";
    if (cameraState === "online") return trackedHand ? "tracking" : "ready";
    return "offline";
  }, [cameraState, trackedHand]);

  const handleTrackingFrame = useCallback(({ hand, fps: currentFps }: { hand: TrackedHand | null; fps: number }) => {
    setFps(currentFps);
    if (!hand) {
      smoothedCursorRef.current = null;
      gestureRef.current = "none";
      setTrackedHand(null);
      setGesture("none");
      setCursor(null);
      return;
    }

    const indexTip = hand.landmarks[HAND_LANDMARK_INDEX.indexTip];
    const smoothed = smoothLandmark(smoothedCursorRef.current, indexTip, ENGINEERING_CONFIG.smoothingAlpha);
    const gestureResult = evaluatePinch(hand.landmarks, gestureRef.current);
    smoothedCursorRef.current = smoothed;
    gestureRef.current = gestureResult.state;
    setTrackedHand(hand);
    setGesture(gestureResult.state);
    setCursor(normalizedToViewport(smoothed, true));
  }, []);

  const handleTrackingError = useCallback((reason: Error) => {
    trackingService.current.stop();
    cameraService.current.stop(videoRef.current);
    setTrackingOnline(false);
    setTrackedHand(null);
    setGesture("none");
    setCursor(null);
    setCameraState("error");
    setError(`Falha no tracking local: ${reason.message}`);
  }, []);

  const stopCamera = useCallback(() => {
    startAttempt.current += 1;
    trackingService.current.stop();
    cameraService.current.stop(videoRef.current);
    smoothedCursorRef.current = null;
    gestureRef.current = "none";
    setCameraState("offline");
    setTrackingOnline(false);
    setTrackedHand(null);
    setGesture("none");
    setCursor(null);
    setFps(0);
    setObjectStatus("ready");
    setError(null);
  }, []);

  const startCamera = useCallback(async () => {
    const video = videoRef.current;
    if (!video || cameraState === "requesting" || cameraState === "online") return;
    const attempt = startAttempt.current + 1;
    startAttempt.current = attempt;
    setCameraState("requesting");
    setError(null);
    try {
      await cameraService.current.start(video);
      if (startAttempt.current !== attempt) return cameraService.current.stop(video);
      setCameraState("online");
      await trackingService.current.initialize();
      if (startAttempt.current !== attempt) return trackingService.current.stop();
      setTrackingOnline(true);
      trackingService.current.start(video, handleTrackingFrame, handleTrackingError);
    } catch (reason) {
      trackingService.current.stop();
      cameraService.current.stop(video);
      setTrackingOnline(false);
      setCameraState("error");
      setError(reason instanceof Error ? reason.message : String(reason));
    }
  }, [cameraState, handleTrackingError, handleTrackingFrame]);

  useEffect(() => () => {
    startAttempt.current += 1;
    trackingService.current.stop();
    cameraService.current.stop(videoRef.current);
  }, []);

  const setRendererState = useCallback((ready: boolean) => setRendererReady(ready), []);
  const setObjectState = useCallback((status: EngineeringObjectState) => setObjectStatus(status), []);

  return (
    <section className="engineering-module">
      <ModuleIntro
        code="ENG-01"
        title="Engineering View"
        description="Computer vision, hand tracking e interação tridimensional processados localmente."
        metric={`ENGINEERING CORE // ${coreState.toUpperCase()}`}
      />

      {error && <div className="engineering-error"><strong>ENGINEERING CORE ERROR</strong><span>{error}</span></div>}

      <div className="engineering-controls" aria-label="Controles do Engineering Core">
        <button onClick={() => void startCamera()} disabled={cameraState === "requesting" || cameraState === "online"}>INICIAR CÂMERA</button>
        <button onClick={stopCamera} disabled={cameraState === "offline"}>PARAR CÂMERA</button>
        <button className={previewVisible ? "active" : ""} onClick={() => setPreviewVisible((visible) => !visible)}>CAMERA PREVIEW {previewVisible ? "ON" : "OFF"}</button>
        <button className={debugVisible ? "active" : ""} onClick={() => setDebugVisible((visible) => !visible)}>HAND DEBUG {debugVisible ? "ON" : "OFF"}</button>
        <button onClick={() => setResetSignal((signal) => signal + 1)}>RESET OBJECT</button>
        <span>PROCESSAMENTO LOCAL // FRAMES NÃO SAEM DO DISPOSITIVO</span>
      </div>

      <div className="engineering-module__layout">
        <section className="engineering-canvas" aria-label="Viewport tridimensional do Engineering Core">
          <header>
            <span>VIEWPORT // PRIMARY</span>
            <strong>TEST-01 // {objectStatus.toUpperCase()}</strong>
          </header>

          <div className="engineering-canvas__stage">
            <EngineeringScene
              cursor={cursor}
              gesture={gesture}
              handDetected={Boolean(trackedHand)}
              resetSignal={resetSignal}
              onObjectStatusChange={setObjectState}
              onRendererReady={setRendererState}
            />
            <span className="engineering-canvas__axis engineering-canvas__axis--x">AXIS X // INTERACTION PLANE</span>
            <span className="engineering-canvas__axis engineering-canvas__axis--y">AXIS Y // SCREEN SPACE</span>
            {cursor && <div className={`engineering-hand-cursor ${gesture === "pinch" ? "engineering-hand-cursor--pinch" : ""}`} style={{ left: `${cursor.x * 100}%`, top: `${cursor.y * 100}%` }} aria-hidden="true"><i /></div>}
            <div className="engineering-object-readout">
              <span>OBJECT</span><strong>TEST-01</strong>
              <span>STATUS</span><strong data-status={objectStatus}>{objectStatus.toUpperCase()}</strong>
              <span>CONTROL</span><strong>HAND</strong>
            </div>
          </div>

          <footer>
            <span>GRID 10 MM</span>
            <span>ORIGIN 0 / 0 / 0</span>
            <span>RENDERER {rendererReady ? "ONLINE" : "ERROR"}</span>
          </footer>
        </section>

        <aside className="engineering-module__rail">
          <section className="engineering-register engineering-tracking-status">
            <header><strong>HAND TRACKING</strong><span>{trackingOnline ? "ONLINE" : "OFFLINE"}</span></header>
            <dl>
              <div><dt>CAMERA</dt><dd data-state={cameraState}>{cameraStateLabel[cameraState]}</dd></div>
              <div><dt>PROCESSAMENTO</dt><dd>{cameraState === "online" ? "LOCAL" : "—"}</dd></div>
              <div><dt>HAND</dt><dd>{trackedHand ? "DETECTED" : "NOT DETECTED"}</dd></div>
              <div><dt>SIDE</dt><dd>{trackedHand?.handedness?.toUpperCase() ?? "—"}</dd></div>
              <div><dt>GESTURE</dt><dd data-state={gesture}>{gesture.toUpperCase()}</dd></div>
              <div><dt>CONFIDENCE</dt><dd>{trackedHand?.confidence !== undefined ? trackedHand.confidence.toFixed(2) : "—"}</dd></div>
              <div><dt>FPS</dt><dd>{fps || "—"}</dd></div>
            </dl>
          </section>

          <CameraPreview videoRef={videoRef} hand={trackedHand} visible={previewVisible} debug={debugVisible} fps={fps} />

          <section className="engineering-register">
            <header><strong>REGISTRO DE INTERFACES</strong><span>FUTURO</span></header>
            <div>
              {futureInterfaces.map((item) => (
                <article key={item.code}>
                  <span>{item.code}</span>
                  <strong>{item.label}</strong>
                  <small>NÃO CONECTADO</small>
                </article>
              ))}
            </div>
          </section>

          <section className="engineering-register engineering-register--status">
            <header><strong>ESTADO DA ESTAÇÃO</strong><span>LOCAL</span></header>
            <dl>
              <div><dt>MODELO</dt><dd>TEST-01</dd></div>
              <div><dt>TELEMETRIA</dt><dd>DESVINCULADA</dd></div>
              <div><dt>RENDERIZADOR 3D</dt><dd>{rendererReady ? "ONLINE" : "ERROR"}</dd></div>
              <div><dt>HAND TRACKING</dt><dd>{trackingOnline ? "ONLINE" : "OFFLINE"}</dd></div>
            </dl>
          </section>
        </aside>
      </div>
    </section>
  );
}
