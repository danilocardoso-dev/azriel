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
import { ModelService, resolveModelCoreState, type LoadedEngineeringModel } from "../engineering/modelService";
import { normalizedToViewport, smoothLandmark } from "../engineering/trackingMath";
import type { CameraState, EngineeringCalibration, EngineeringCalibrationInput, EngineeringInteractionScope, EngineeringObjectSnapshot, GestureState, HandLandmark, HandSide, ManipulationMode, ModelComponent, ModelCoreState, TrackedHand, ViewportPoint } from "../engineering/types";
import { engineeringRepository } from "../repositories/engineeringRepository";

const futureInterfaces = [
  { code: "CAD", label: "Modelos e componentes" },
  { code: "DTW", label: "Digital Twins" },
  { code: "IOT", label: "Sensores e telemetria" },
];

const cameraStateLabel: Record<CameraState, string> = { offline: "OFFLINE", requesting: "REQUESTING", online: "ONLINE", error: "ERROR" };
const modelStateLabel: Record<ModelCoreState, string> = { empty: "SEM MODELO", loading: "CARREGANDO", ready: "ONLINE", error: "ERROR" };
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
  const modelService = useRef(new ModelService());
  const loadedModelRef = useRef<LoadedEngineeringModel | null>(null);
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
  const [interactionScope, setInteractionScope] = useState<EngineeringInteractionScope>("model");
  const [objectSnapshot, setObjectSnapshot] = useState<EngineeringObjectSnapshot>(initialObject);
  const [resetSignal, setResetSignal] = useState(0);
  const [loadedModel, setLoadedModel] = useState<LoadedEngineeringModel | null>(null);
  const [modelState, setModelState] = useState<ModelCoreState>("empty");
  const [wireframe, setWireframe] = useState(false);
  const [gridVisible, setGridVisible] = useState(true);
  const [axesVisible, setAxesVisible] = useState(true);
  const [selectedComponentId, setSelectedComponentId] = useState<string | null>(null);
  const [targetedComponentId, setTargetedComponentId] = useState<string | null>(null);
  const [componentRevision, setComponentRevision] = useState(0);
  const [expandedComponentIds, setExpandedComponentIds] = useState<Set<string>>(new Set());
  const [componentSearch, setComponentSearch] = useState("");
  const [boundingBoxVisible, setBoundingBoxVisible] = useState(true);
  const [focusRequest, setFocusRequest] = useState({ sequence: 0, componentId: null as string | null });
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
    modelService.current.unload(loadedModelRef.current);
    loadedModelRef.current = null;
  }, []);

  const loadModel = useCallback(async () => {
    setModelState(resolveModelCoreState("load_started", Boolean(loadedModelRef.current)));
    setError(null);
    try {
      const next = await modelService.current.selectAndLoad();
      if (!next) {
        setModelState(resolveModelCoreState("load_cancelled", Boolean(loadedModelRef.current)));
        return;
      }
      const previous = loadedModelRef.current;
      loadedModelRef.current = next;
      setLoadedModel(next);
      setSelectedComponentId(null);
      setTargetedComponentId(null);
      setComponentSearch("");
      setExpandedComponentIds(new Set(next.components.list().filter((component) => component.depth < 2).map((component) => component.id)));
      setComponentRevision((revision) => revision + 1);
      setModelState(resolveModelCoreState("load_succeeded", true));
      if (previous) window.setTimeout(() => modelService.current.unload(previous), 0);
    } catch (reason) {
      setModelState(resolveModelCoreState("load_failed", Boolean(loadedModelRef.current)));
      setError(reason instanceof Error ? reason.message : String(reason));
    }
  }, []);

  const unloadModel = useCallback(() => {
    const previous = loadedModelRef.current;
    loadedModelRef.current = null;
    setLoadedModel(null);
    setSelectedComponentId(null);
    setTargetedComponentId(null);
    setComponentSearch("");
    setExpandedComponentIds(new Set());
    setModelState(resolveModelCoreState("unloaded", false));
    setError(null);
    if (previous) window.setTimeout(() => modelService.current.unload(previous), 0);
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

  const components = useMemo(() => { void componentRevision; return loadedModel?.components.list() ?? []; }, [loadedModel, componentRevision]);
  const selectedComponent = useMemo(() => { void componentRevision; return loadedModel?.components.get(selectedComponentId) ?? null; }, [loadedModel, selectedComponentId, componentRevision]);
  const selectedParent = selectedComponent?.parentId ? loadedModel?.components.get(selectedComponent.parentId) : null;
  const isolationId = loadedModel?.components.getIsolationId() ?? null;
  const searchMatches = useMemo(() => { void componentRevision; return loadedModel?.components.search(componentSearch) ?? []; }, [loadedModel, componentSearch, componentRevision]);
  const searchVisibleIds = useMemo(() => {
    if (!loadedModel || !componentSearch.trim()) return null;
    const ids = new Set<string>();
    searchMatches.forEach((component) => {
      ids.add(component.id);
      loadedModel.components.ancestors(component.id).forEach((id) => ids.add(id));
    });
    return ids;
  }, [loadedModel, componentSearch, searchMatches]);
  const visibleComponents = useMemo(() => components.filter((component) => {
    if (searchVisibleIds) return searchVisibleIds.has(component.id);
    return loadedModel?.components.ancestors(component.id).every((id) => expandedComponentIds.has(id));
  }), [components, expandedComponentIds, loadedModel, searchVisibleIds]);

  const selectComponent = useCallback((id: string | null) => {
    setSelectedComponentId(id);
    if (!id || !loadedModelRef.current) return;
    setExpandedComponentIds((current) => new Set([...current, ...loadedModelRef.current!.components.ancestors(id)]));
  }, []);

  const mutateComponents = useCallback((operation: (model: LoadedEngineeringModel) => void) => {
    const current = loadedModelRef.current;
    if (!current) return;
    operation(current);
    setTargetedComponentId(null);
    setComponentRevision((revision) => revision + 1);
  }, []);

  const focusComponent = useCallback((componentId: string | null) => {
    setFocusRequest((current) => ({ sequence: current.sequence + 1, componentId }));
  }, []);

  const toggleExpanded = useCallback((component: ModelComponent) => {
    if (!component.children.length) return;
    setExpandedComponentIds((current) => {
      const next = new Set(current);
      if (next.has(component.id)) next.delete(component.id); else next.add(component.id);
      return next;
    });
  }, []);

  return (
    <section className="engineering-module">
      <ModuleIntro code="ENG-03" title="Engineering View" description="Modelos 3D reais e manipulação espacial processados localmente." metric={`MODEL CORE // ${modelStateLabel[modelState]}`} />

      {error && <div className="engineering-error"><strong>ENGINEERING CORE ERROR</strong><span>{error}</span></div>}

      <div className="engineering-controls" aria-label="Controles do Engineering Core">
        <button onClick={() => void loadModel()} disabled={modelState === "loading"}>CARREGAR MODELO</button>
        <button onClick={unloadModel} disabled={!loadedModel}>DESCARREGAR</button>
        <button onClick={() => setResetSignal((signal) => signal + 1)}>RESET MODEL</button>
        <button onClick={() => void startCamera()} disabled={cameraState === "requesting" || cameraState === "online"}>INICIAR CÂMERA</button>
        <button onClick={stopCamera} disabled={cameraState === "offline"}>PARAR CÂMERA</button>
        <button className={previewVisible ? "active" : ""} onClick={() => setPreviewVisible((visible) => !visible)}>PREVIEW {previewVisible ? "ON" : "OFF"}</button>
        <button className={debugVisible ? "active" : ""} onClick={() => setDebugVisible((visible) => !visible)}>DEBUG {debugVisible ? "ON" : "OFF"}</button>
        <span>MODEL + VISION PROCESSING // LOCAL</span>
      </div>

      <div className="engineering-mode-switch" aria-label="Modo de manipulação">
        <span>SCOPE</span>
        {(["model", "component"] as const).map((candidate) => <button key={candidate} className={interactionScope === candidate ? "active" : ""} onClick={() => { setInteractionScope(candidate); setTargetedComponentId(null); }}>{candidate.toUpperCase()}</button>)}
        <i />
        <span>MODEL CONTROL</span>
        {(["move", "rotate", "scale"] as const).map((candidate) => <button key={candidate} className={mode === candidate ? "active" : ""} onClick={() => setMode(candidate)}>{candidate.toUpperCase()}</button>)}
        <button className={wireframe ? "active" : ""} onClick={() => setWireframe((active) => !active)}>WIREFRAME {wireframe ? "ON" : "OFF"}</button>
        <button className={gridVisible ? "active" : ""} onClick={() => setGridVisible((visible) => !visible)}>GRID {gridVisible ? "ON" : "OFF"}</button>
        <button className={axesVisible ? "active" : ""} onClick={() => setAxesVisible((visible) => !visible)}>AXES {axesVisible ? "ON" : "OFF"}</button>
      </div>

      <div className="engineering-module__layout">
        <section className="engineering-canvas" aria-label="Viewport tridimensional do Engineering Core">
          <header><span>VIEWPORT // PRIMARY</span><strong>{loadedModel?.metadata.name ?? "TEST-01"} // {objectSnapshot.status.toUpperCase()}</strong></header>
          <div className="engineering-canvas__stage">
            <EngineeringScene hands={sceneHands} mode={mode} interactionScope={interactionScope} calibration={calibration} model={loadedModel} wireframe={wireframe} gridVisible={gridVisible} axesVisible={axesVisible} resetSignal={resetSignal} selectedComponentId={selectedComponentId} targetedComponentId={targetedComponentId} componentRevision={componentRevision} boundingBoxVisible={boundingBoxVisible} focusRequest={focusRequest} onComponentTarget={setTargetedComponentId} onComponentSelect={selectComponent} onObjectChange={setObjectSnapshot} onRendererReady={setRendererReady} />
            <span className="engineering-canvas__axis engineering-canvas__axis--x">AXIS X // INTERACTION PLANE</span>
            <span className="engineering-canvas__axis engineering-canvas__axis--y">AXIS Y // SCREEN SPACE</span>
            {sceneHands.map((hand) => <div key={hand.id} className={`engineering-hand-cursor engineering-hand-cursor--${hand.id} ${hand.gesture === "pinch" ? "engineering-hand-cursor--pinch" : ""}`} style={{ left: `${hand.cursor.x * 100}%`, top: `${hand.cursor.y * 100}%` }} aria-hidden="true"><i /><small>{hand.id.toUpperCase()}</small></div>)}
            <div className="engineering-object-readout engineering-object-inspector">
              <span>OBJECT</span><strong>{loadedModel?.metadata.name ?? "TEST-01"}</strong>
              <span>STATUS</span><strong data-status={objectSnapshot.status}>{objectSnapshot.status.toUpperCase()}</strong>
              <span>CONTROL</span><strong>{objectSnapshot.control.toUpperCase()}</strong>
              <span>POSITION</span><strong>{objectSnapshot.position.x.toFixed(2)} / {objectSnapshot.position.y.toFixed(2)} / {objectSnapshot.position.z.toFixed(2)}</strong>
              <span>ROTATION</span><strong>{objectSnapshot.rotation.x.toFixed(2)} / {objectSnapshot.rotation.y.toFixed(2)} / {objectSnapshot.rotation.z.toFixed(2)}</strong>
              <span>SCALE</span><strong>{objectSnapshot.scale.toFixed(2)}</strong>
            </div>
          </div>
          <footer><span>SCOPE {interactionScope.toUpperCase()} // {mode.toUpperCase()}</span><span>MOUSE DRAG ORBIT // WHEEL ZOOM</span><span>RENDERER {rendererReady ? "ONLINE" : "ERROR"}</span></footer>
        </section>

        <aside className="engineering-module__rail">
          <section className="engineering-register engineering-model-status">
            <header><strong>MODEL CORE</strong><span data-state={modelState}>{modelStateLabel[modelState]}</span></header>
            {modelState === "loading" && <p className="engineering-model-loading">CARREGANDO MODELO...</p>}
            {loadedModel ? <dl>
              <div><dt>NAME</dt><dd>{loadedModel.metadata.name}</dd></div>
              <div><dt>FORMAT</dt><dd>{loadedModel.metadata.format}</dd></div>
              <div><dt>OBJECTS</dt><dd>{loadedModel.metadata.objects}</dd></div>
              <div><dt>MESHES</dt><dd>{loadedModel.metadata.meshes}</dd></div>
              <div><dt>MATERIALS</dt><dd>{loadedModel.metadata.materials}</dd></div>
              <div><dt>VERTICES</dt><dd>{loadedModel.metadata.vertices.toLocaleString("pt-BR")}</dd></div>
              <div><dt>TRIANGLES</dt><dd>{loadedModel.metadata.triangles.toLocaleString("pt-BR")}</dd></div>
              <div><dt>DIMENSIONS</dt><dd>{loadedModel.metadata.dimensions.x.toFixed(2)} × {loadedModel.metadata.dimensions.y.toFixed(2)} × {loadedModel.metadata.dimensions.z.toFixed(2)}</dd></div>
              <div><dt>STATUS</dt><dd data-state="online">{loadedModel.metadata.complexity === "high" ? "HIGH COMPLEXITY" : "READY"}</dd></div>
            </dl> : <p className="engineering-model-empty">SEM MODELO CARREGADO<br /><small>TEST-01 ATIVO COMO FALLBACK</small></p>}
          </section>

          {loadedModel && <section className="engineering-register engineering-model-tree">
            <header><strong>MODEL STRUCTURE</strong><span>{components.length} COMPONENTS</span></header>
            <label className="engineering-component-search">
              <span>BUSCAR COMPONENTE</span>
              <input value={componentSearch} onChange={(event) => setComponentSearch(event.target.value)} placeholder="Nome da peça..." />
            </label>
            <div>{visibleComponents.map((component) => {
              const state = loadedModel.components.state(component.id, targetedComponentId, selectedComponentId);
              const expanded = expandedComponentIds.has(component.id) || Boolean(searchVisibleIds);
              return <article key={component.id} className={selectedComponentId === component.id ? "selected" : ""} data-state={state} style={{ paddingLeft: `${8 + Math.min(component.depth, 7) * 12}px` }}>
                <button className="engineering-tree-toggle" onClick={() => toggleExpanded(component)} disabled={!component.children.length} aria-label={expanded ? "Recolher componente" : "Expandir componente"}>{component.children.length ? (expanded ? "−" : "+") : "·"}</button>
                <button className="engineering-tree-select" onClick={() => selectComponent(component.id)} onDoubleClick={() => focusComponent(component.id)}>
                  <span>{component.type}</span><strong>{component.name}</strong><small>{state.toUpperCase()}</small>
                </button>
              </article>;
            })}</div>
          </section>}

          {loadedModel && <section className="engineering-register engineering-component-inspector">
            <header><strong>COMPONENT INSPECTOR</strong><span>{selectedComponent ? "SELECTED" : "NONE"}</span></header>
            {selectedComponent ? <>
              <dl>
                <div><dt>NAME</dt><dd>{selectedComponent.name}</dd></div>
                <div><dt>TYPE</dt><dd>{selectedComponent.type}</dd></div>
                <div><dt>PARENT</dt><dd>{selectedParent?.name ?? "ROOT"}</dd></div>
                <div><dt>CHILDREN</dt><dd>{selectedComponent.children.length}</dd></div>
                <div><dt>MESHES</dt><dd>{selectedComponent.meshCount}</dd></div>
                <div><dt>VISIBLE</dt><dd>{selectedComponent.visible ? "YES" : "NO"}</dd></div>
                <div><dt>VERTICES</dt><dd>{selectedComponent.vertices.toLocaleString("pt-BR")}</dd></div>
                <div><dt>TRIANGLES</dt><dd>{selectedComponent.triangles.toLocaleString("pt-BR")}</dd></div>
                <div><dt>POSITION</dt><dd>{selectedComponent.originalPosition.x.toFixed(2)} / {selectedComponent.originalPosition.y.toFixed(2)} / {selectedComponent.originalPosition.z.toFixed(2)}</dd></div>
                <div><dt>ROTATION</dt><dd>{selectedComponent.originalRotation.x.toFixed(2)} / {selectedComponent.originalRotation.y.toFixed(2)} / {selectedComponent.originalRotation.z.toFixed(2)}</dd></div>
                <div><dt>SCALE</dt><dd>{selectedComponent.originalScale.x.toFixed(2)} / {selectedComponent.originalScale.y.toFixed(2)} / {selectedComponent.originalScale.z.toFixed(2)}</dd></div>
                <div><dt>DIMENSIONS</dt><dd>{selectedComponent.dimensions.x.toFixed(2)} × {selectedComponent.dimensions.y.toFixed(2)} × {selectedComponent.dimensions.z.toFixed(2)}</dd></div>
              </dl>
              <div className="engineering-material-list">
                <strong>MATERIALS</strong>
                {selectedComponent.materials.length ? selectedComponent.materials.map((material, index) => <p key={`${material.name}-${index}`}><span>{material.name}</span><small>{material.type} // {material.color ?? "NO COLOR"} // TEXTURE {material.textured ? "YES" : "NO"}</small></p>) : <p><span>NONE</span></p>}
              </div>
              <div className="engineering-component-actions">
                <button onClick={() => mutateComponents((current) => selectedComponent.visible ? current.components.hide(selectedComponent.id) : current.components.show(selectedComponent.id))}>{selectedComponent.visible ? "OCULTAR" : "MOSTRAR"}</button>
                <button className={isolationId ? "active" : ""} onClick={() => mutateComponents((current) => isolationId ? current.components.exitIsolation() : current.components.isolate(selectedComponent.id))}>{isolationId ? "SAIR DO ISOLAMENTO" : "ISOLAR"}</button>
                <button onClick={() => focusComponent(selectedComponent.id)}>FOCAR</button>
                <button className={boundingBoxVisible ? "active" : ""} onClick={() => setBoundingBoxVisible((visible) => !visible)}>BOUNDING BOX {boundingBoxVisible ? "ON" : "OFF"}</button>
                <button onClick={() => focusComponent(null)}>FOCAR MODELO</button>
                <button onClick={() => { mutateComponents((current) => current.components.restore()); selectComponent(null); }}>RESTAURAR COMPONENTES</button>
              </div>
            </> : <p className="engineering-model-empty">SELECIONE UMA PEÇA NA ÁRVORE OU NO VIEWPORT</p>}
          </section>}

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
              <div><dt>SCOPE</dt><dd>{interactionScope.toUpperCase()}</dd></div>
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
              <div><dt>MODELO</dt><dd data-state={modelState}>{loadedModel ? "ONLINE" : "NÃO CARREGADO"}</dd></div><div><dt>TELEMETRIA</dt><dd>DESVINCULADA</dd></div>
              <div><dt>RENDERIZADOR 3D</dt><dd>{rendererReady ? "ONLINE" : "ERROR"}</dd></div><div><dt>HAND TRACKING</dt><dd>{trackingOnline ? "ONLINE" : "OFFLINE"}</dd></div>
              <div><dt>COMPONENT CORE</dt><dd>{loadedModel ? "ONLINE" : "OFFLINE"}</dd></div><div><dt>SELECTED</dt><dd>{selectedComponent?.name ?? "NONE"}</dd></div>
            </dl>
          </section>
        </aside>
      </div>
    </section>
  );
}
