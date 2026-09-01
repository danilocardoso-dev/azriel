import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { CameraPreview } from "../components/engineering/CameraPreview";
import { EngineeringScene, type EngineeringHandControl } from "../components/engineering/EngineeringScene";
import { EngineeringSettings } from "../components/engineering/EngineeringSettings";
import { ModuleIntro } from "../components/layout/ModuleIntro";
import { defaultCalibration } from "../engineering/calibration";
import { CameraService } from "../engineering/cameraService";
import { HAND_LANDMARK_INDEX } from "../engineering/config";
import { evaluateGesture } from "../engineering/gestureEngine";
import { HandTrackingService } from "../engineering/handTrackingService";
import { normalizedToViewport, smoothLandmark } from "../engineering/trackingMath";
import type { CameraState, EngineeringCalibration, EngineeringCalibrationInput, EngineeringCoreState, EngineeringObjectSnapshot, GestureState, HandLandmark, HandSide, ManipulationMode, TrackedHand, ViewportPoint } from "../engineering/types";
import { engineeringRepository } from "../repositories/engineeringRepository";

const futureInterfaces = [
  { code: "CAD", label: "Modelos e componentes" },
  { code: "DTW", label: "Digital Twins" },
  { code: "IOT", label: "Sensores e telemetria" },
];

const cameraStateLabel: Record<CameraState, string> = { offline: "OFFLINE", requesting: "REQUESTING", online: "ONLINE", error: "ERROR" };
const emptyGestures: Record<HandSide, GestureState> = { left: "none", right: "none" };
const initialObject: EngineeringObjectSnapshot = {
  status: "ready",
  position: { x: 0, y: 0.65, z: 0 },
  rotation: { x: 0, y: 0, z: 0 },
  scale: 1,
  control: "none",
};

export function EngineeringViewPage() {
  const videoRef = useRef<HTMLVideoElement>(null);
  const cameraService = useRef(new CameraService());
  const trackingService = useRef(new HandTrackingService());
  const startAttempt = useRef(0);
  const gestureRef = useRef<Record<HandSide, GestureState>>({ ...emptyGestures });
  const smoothedCursorRef = useRef<Partial<Record<HandSide, HandLandmark>>>({});
  const calibrationRef = useRef<EngineeringCalibration>(defaultCalibration());
  const [cameraState, setCameraState] = useState<CameraState>("offline");
  const [trackingOnline, setTrackingOnline] = useState(false);
  const [trackedHands, setTrackedHands] = useState<TrackedHand[]>([]);
  const [gestures, setGestures] = useState<Record<HandSide, GestureState>>({ ...emptyGestures });
  const [cursors, setCursors] = useState<Partial<Record<HandSide, ViewportPoint>>>({});
  const [fps, setFps] = useState(0);
  const [previewVisible, setPreviewVisible] = useState(true);
  const [debugVisible, setDebugVisible] = useState(false);
  const [rendererReady, setRendererReady] = useState(false);
  const [mode, setMode] = useState<ManipulationMode>("move");
  const [objectSnapshot, setObjectSnapshot] = useState<EngineeringObjectSnapshot>(initialObject);
  const [resetSignal, setResetSignal] = useState(0);
  const [calibration, setCalibration] = useState<EngineeringCalibration>(() => defaultCalibration());
  const [savingCalibration, setSavingCalibration] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => { calibrationRef.current = calibration; }, [calibration]);

  useEffect(() => {
    let active = true;
    void engineeringRepository.getCalibration()
      .then((settings) => { if (active) setCalibration(settings); })
      .catch((reason) => { if (active) setError(reason instanceof Error ? reason.message : String(reason)); });
    return () => { active = false; };
  }, []);

  const coreState: EngineeringCoreState = useMemo(() => {
    if (cameraState === "error") return "error";
    if (cameraState === "requesting") return "requesting_camera";
    if (cameraState === "online") return trackedHands.length ? "tracking" : "ready";
    return "offline";
  }, [cameraState, trackedHands.length]);

  const handleTrackingFrame = useCallback(({ hands, fps: currentFps }: { hands: TrackedHand[]; fps: number }) => {
    setFps(currentFps);
    setTrackedHands(hands);
    if (!hands.length) {
      smoothedCursorRef.current = {};
      gestureRef.current = { ...emptyGestures };
      setGestures({ ...emptyGestures });
      setCursors({});
      return;
    }

    const nextGestures: Record<HandSide, GestureState> = { ...emptyGestures };
    const nextCursors: Partial<Record<HandSide, ViewportPoint>> = {};
    for (const hand of hands) {
      const indexTip = hand.landmarks[HAND_LANDMARK_INDEX.indexTip];
      const smoothed = smoothLandmark(smoothedCursorRef.current[hand.id] ?? null, indexTip, calibrationRef.current.smoothingAlpha);
      const result = evaluateGesture(hand.landmarks, gestureRef.current[hand.id], calibrationRef.current.pinchStartThreshold, calibrationRef.current.pinchReleaseThreshold);
      smoothedCursorRef.current[hand.id] = smoothed;
      nextGestures[hand.id] = result.state;
      nextCursors[hand.id] = normalizedToViewport(smoothed, true);
    }
    gestureRef.current = nextGestures;
    setGestures(nextGestures);
    setCursors(nextCursors);

  }, []);

  const clearTrackingState = useCallback(() => {
    smoothedCursorRef.current = {};
    gestureRef.current = { ...emptyGestures };
    setTrackedHands([]);
    setGestures({ ...emptyGestures });
    setCursors({});
    setFps(0);
  }, []);

  const handleTrackingError = useCallback((reason: Error) => {
    trackingService.current.stop();
    cameraService.current.stop(videoRef.current);
    setTrackingOnline(false);
    clearTrackingState();
    setCameraState("error");
    setError(`Falha no tracking local: ${reason.message}`);
  }, [clearTrackingState]);

  const stopCamera = useCallback(() => {
    startAttempt.current += 1;
    trackingService.current.stop();
    cameraService.current.stop(videoRef.current);
    clearTrackingState();
    setCameraState("offline");
    setTrackingOnline(false);
    setError(null);
  }, [clearTrackingState]);

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

  const sceneHands = useMemo<EngineeringHandControl[]>(() => trackedHands.flatMap((hand) => {
    const cursor = cursors[hand.id];
    return cursor ? [{ id: hand.id, gesture: gestures[hand.id], cursor }] : [];
  }), [cursors, gestures, trackedHands]);

  const changeCalibration = useCallback((input: EngineeringCalibrationInput) => {
    setCalibration((current) => ({ ...input, updatedAt: current.updatedAt }));
  }, []);

  const saveCalibration = useCallback(async () => {
    setSavingCalibration(true);
    setError(null);
    try {
      const current = calibrationRef.current;
      const input: EngineeringCalibrationInput = {
        pinchStartThreshold: current.pinchStartThreshold,
        pinchReleaseThreshold: current.pinchReleaseThreshold,
        smoothingAlpha: current.smoothingAlpha,
        rotationSensitivity: current.rotationSensitivity,
        minScale: current.minScale,
        maxScale: current.maxScale,
        comfortableHandDistance: current.comfortableHandDistance,
        calibrated: current.calibrated,
      };
      setCalibration(await engineeringRepository.updateCalibration(input));
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setSavingCalibration(false);
    }
  }, []);

  const resetCalibration = useCallback(async () => {
    setSavingCalibration(true);
    setError(null);
    try {
      setCalibration(await engineeringRepository.resetCalibration());
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setSavingCalibration(false);
    }
  }, []);

  const handBySide = (side: HandSide) => trackedHands.find((hand) => hand.id === side);

  return (
    <section className="engineering-module">
      <ModuleIntro code="ENG-02" title="Engineering View" description="Manipulação espacial por duas mãos processada localmente." metric={`ENGINEERING CORE // ${coreState.toUpperCase()}`} />

      {error && <div className="engineering-error"><strong>ENGINEERING CORE ERROR</strong><span>{error}</span></div>}

      <div className="engineering-controls" aria-label="Controles do Engineering Core">
        <button onClick={() => void startCamera()} disabled={cameraState === "requesting" || cameraState === "online"}>INICIAR CÂMERA</button>
        <button onClick={stopCamera} disabled={cameraState === "offline"}>PARAR CÂMERA</button>
        <button className={previewVisible ? "active" : ""} onClick={() => setPreviewVisible((visible) => !visible)}>PREVIEW {previewVisible ? "ON" : "OFF"}</button>
        <button className={debugVisible ? "active" : ""} onClick={() => setDebugVisible((visible) => !visible)}>DEBUG {debugVisible ? "ON" : "OFF"}</button>
        <button onClick={() => setResetSignal((signal) => signal + 1)}>RESET OBJECT</button>
        <span>PROCESSAMENTO LOCAL // FRAMES NÃO SAEM DO DISPOSITIVO</span>
      </div>

      <div className="engineering-mode-switch" aria-label="Modo de manipulação">
        <span>MODE</span>
        {(["move", "rotate", "scale"] as const).map((candidate) => <button key={candidate} className={mode === candidate ? "active" : ""} onClick={() => setMode(candidate)}>{candidate.toUpperCase()}</button>)}
      </div>

      <div className="engineering-module__layout">
        <section className="engineering-canvas" aria-label="Viewport tridimensional do Engineering Core">
          <header><span>VIEWPORT // PRIMARY</span><strong>TEST-01 // {objectSnapshot.status.toUpperCase()}</strong></header>
          <div className="engineering-canvas__stage">
            <EngineeringScene hands={sceneHands} mode={mode} calibration={calibration} resetSignal={resetSignal} onObjectChange={setObjectSnapshot} onRendererReady={setRendererReady} />
            <span className="engineering-canvas__axis engineering-canvas__axis--x">AXIS X // INTERACTION PLANE</span>
            <span className="engineering-canvas__axis engineering-canvas__axis--y">AXIS Y // SCREEN SPACE</span>
            {sceneHands.map((hand) => <div key={hand.id} className={`engineering-hand-cursor engineering-hand-cursor--${hand.id} ${hand.gesture === "pinch" ? "engineering-hand-cursor--pinch" : ""}`} style={{ left: `${hand.cursor.x * 100}%`, top: `${hand.cursor.y * 100}%` }} aria-hidden="true"><i /><small>{hand.id.toUpperCase()}</small></div>)}
            <div className="engineering-object-readout engineering-object-inspector">
              <span>OBJECT</span><strong>TEST-01</strong>
              <span>STATUS</span><strong data-status={objectSnapshot.status}>{objectSnapshot.status.toUpperCase()}</strong>
              <span>CONTROL</span><strong>{objectSnapshot.control.toUpperCase()}</strong>
              <span>POSITION</span><strong>{objectSnapshot.position.x.toFixed(2)} / {objectSnapshot.position.y.toFixed(2)} / {objectSnapshot.position.z.toFixed(2)}</strong>
              <span>ROTATION</span><strong>{objectSnapshot.rotation.x.toFixed(2)} / {objectSnapshot.rotation.y.toFixed(2)} / {objectSnapshot.rotation.z.toFixed(2)}</strong>
              <span>SCALE</span><strong>{objectSnapshot.scale.toFixed(2)}</strong>
            </div>
          </div>
          <footer><span>MODE {mode.toUpperCase()}</span><span>ORIGIN 0 / 0 / 0</span><span>RENDERER {rendererReady ? "ONLINE" : "ERROR"}</span></footer>
        </section>

        <aside className="engineering-module__rail">
          <section className="engineering-register engineering-tracking-status">
            <header><strong>HAND TRACKING</strong><span>{trackingOnline ? "ONLINE" : "OFFLINE"}</span></header>
            <dl>
              <div><dt>CAMERA</dt><dd data-state={cameraState}>{cameraStateLabel[cameraState]}</dd></div>
              {(["left", "right"] as const).map((side) => {
                const hand = handBySide(side);
                return <div key={side}><dt>{side.toUpperCase()} HAND</dt><dd>{hand ? "DETECTED" : "NOT DETECTED"}</dd></div>;
              })}
              <div><dt>GESTURE L</dt><dd data-state={gestures.left}>{gestures.left.toUpperCase()}</dd></div>
              <div><dt>GESTURE R</dt><dd data-state={gestures.right}>{gestures.right.toUpperCase()}</dd></div>
              <div><dt>MODE</dt><dd>{mode.toUpperCase()}</dd></div>
              <div><dt>FPS</dt><dd>{fps || "—"}</dd></div>
              <div><dt>CALIBRATION</dt><dd>V0.2.1 / PENDING</dd></div>
            </dl>
          </section>

          <CameraPreview videoRef={videoRef} hands={trackedHands} visible={previewVisible} debug={debugVisible} fps={fps} />
          <EngineeringSettings value={calibration} saving={savingCalibration} onChange={changeCalibration} onSave={() => void saveCalibration()} onReset={() => void resetCalibration()} />

          <section className="engineering-register">
            <header><strong>REGISTRO DE INTERFACES</strong><span>FUTURO</span></header>
            <div>{futureInterfaces.map((item) => <article key={item.code}><span>{item.code}</span><strong>{item.label}</strong><small>NÃO CONECTADO</small></article>)}</div>
          </section>

          <section className="engineering-register engineering-register--status">
            <header><strong>ESTADO DA ESTAÇÃO</strong><span>LOCAL</span></header>
            <dl>
              <div><dt>MODELO</dt><dd>TEST-01</dd></div><div><dt>TELEMETRIA</dt><dd>DESVINCULADA</dd></div>
              <div><dt>RENDERIZADOR 3D</dt><dd>{rendererReady ? "ONLINE" : "ERROR"}</dd></div><div><dt>HAND TRACKING</dt><dd>{trackingOnline ? "ONLINE" : "OFFLINE"}</dd></div>
            </dl>
          </section>
        </aside>
      </div>
    </section>
  );
}
