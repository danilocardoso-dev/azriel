import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { CameraPreview } from "../components/engineering/CameraPreview";
import { AssemblyIntelligencePanel } from "../components/engineering/AssemblyIntelligencePanel";
import { EngineeringScene, type EngineeringHandControl } from "../components/engineering/EngineeringScene";
import { EngineeringSettings } from "../components/engineering/EngineeringSettings";
import { ModuleIntro } from "../components/layout/ModuleIntro";
import { defaultCalibration } from "../engineering/calibration";
import { CameraService } from "../engineering/cameraService";
import { HAND_LANDMARK_INDEX } from "../engineering/config";
import { evaluateGesture } from "../engineering/gestureEngine";
import { HandTrackingService } from "../engineering/handTrackingService";
import { engineeringSessionService, type EngineeringCommandLog } from "../engineering/engineeringSessionService";
import { ModelService, resolveModelCoreState, type LoadedEngineeringModel } from "../engineering/modelService";
import { normalizedToViewport, smoothLandmark } from "../engineering/trackingMath";
import { semanticLabelsForComponents } from "../engineering/assemblyIntelligence";
import type { AssemblyIntelligenceSnapshot, AssemblyState, CameraState, ComponentTransformSnapshot, EngineeringCalibration, EngineeringCalibrationInput, EngineeringControlMode, EngineeringInteractionScope, EngineeringObjectSnapshot, ExplosionMode, ExplosionState, GestureState, HandLandmark, HandSide, ModelComponent, ModelCoreState, TrackedHand, ViewportPoint } from "../engineering/types";
import { componentRegistryInput, engineeringRepository } from "../repositories/engineeringRepository";

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
  const initialSession = engineeringSessionService.getViewState();
  const videoRef = useRef<HTMLVideoElement>(null);
  const cameraService = useRef(new CameraService());
  const trackingService = useRef(new HandTrackingService());
  const modelService = useRef(new ModelService());
  const loadedModelRef = useRef<LoadedEngineeringModel | null>(initialSession.model);
  const explosionAnimationRef = useRef<number | null>(null);
  const explosionFactorRef = useRef(0);
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
  const [mode, setMode] = useState<EngineeringControlMode>("move");
  const [interactionScope, setInteractionScope] = useState<EngineeringInteractionScope>(initialSession.modelMode);
  const [objectSnapshot, setObjectSnapshot] = useState<EngineeringObjectSnapshot>(initialSession.objectSnapshot ?? initialObject);
  const [resetSignal, setResetSignal] = useState(initialSession.resetSignal);
  const [loadedModel, setLoadedModel] = useState<LoadedEngineeringModel | null>(initialSession.model);
  const [modelState, setModelState] = useState<ModelCoreState>(initialSession.model ? "ready" : "empty");
  const [wireframe, setWireframe] = useState(false);
  const [gridVisible, setGridVisible] = useState(true);
  const [axesVisible, setAxesVisible] = useState(true);
  const [selectedComponentId, setSelectedComponentId] = useState<string | null>(initialSession.selectedComponentId);
  const [targetedComponentId, setTargetedComponentId] = useState<string | null>(null);
  const [componentRevision, setComponentRevision] = useState(initialSession.componentRevision);
  const [explosionState, setExplosionState] = useState<ExplosionState>(initialSession.explosion);
  const [guideLinesVisible, setGuideLinesVisible] = useState(true);
  const [explosionError, setExplosionError] = useState<string | null>(null);
  const [explosionGestureState, setExplosionGestureState] = useState<"idle" | "active" | "cancelled">("idle");
  const [componentTransform, setComponentTransform] = useState<ComponentTransformSnapshot | null>(null);
  const [expandedComponentIds, setExpandedComponentIds] = useState<Set<string>>(() => new Set(initialSession.model?.components.list().filter((component) => component.depth < 2).map((component) => component.id) ?? []));
  const [componentSearch, setComponentSearch] = useState("");
  const [boundingBoxVisible, setBoundingBoxVisible] = useState(true);
  const [focusRequest, setFocusRequest] = useState(initialSession.focusRequest);
  const [engineeringCommands, setEngineeringCommands] = useState<EngineeringCommandLog[]>(initialSession.commands);
  const [calibration, setCalibration] = useState<EngineeringCalibration>(() => defaultCalibration());
  const [savingCalibration, setSavingCalibration] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [assemblySnapshot, setAssemblySnapshot] = useState<AssemblyIntelligenceSnapshot | undefined>();

  useEffect(() => { calibrationRef.current = calibration; }, [calibration]);

  useEffect(() => engineeringSessionService.subscribe(() => {
    const session = engineeringSessionService.getViewState();
    loadedModelRef.current = session.model;
    explosionFactorRef.current = session.explosion.factor;
    setLoadedModel(session.model);
    setSelectedComponentId(session.selectedComponentId);
    setInteractionScope(session.modelMode);
    setComponentRevision(session.componentRevision);
    setExplosionState(session.explosion);
    setFocusRequest(session.focusRequest);
    setResetSignal(session.resetSignal);
    setEngineeringCommands(session.commands);
    setObjectSnapshot(session.objectSnapshot);
    if (session.model && session.selectedComponentId) {
      setExpandedComponentIds((current) => new Set([...current, ...session.model!.components.ancestors(session.selectedComponentId!)]));
    }
    setModelState(session.model ? "ready" : "empty");
  }), []);

  useEffect(() => {
    let active = true;
    void engineeringRepository.getCalibration()
      .then((settings) => { if (active) setCalibration(settings); })
      .catch((reason) => { if (active) setError(reason instanceof Error ? reason.message : String(reason)); });
    return () => { active = false; };
  }, []);

  useEffect(() => {
    if (!loadedModel) return;
    let active = true;
    void engineeringRepository.registerModel({ modelIdentity: loadedModel.identity, fileName: loadedModel.metadata.name, format: loadedModel.metadata.format, byteSize: loadedModel.byteSize, components: componentRegistryInput(loadedModel.components.list()) })
      .then((snapshot) => {
        if (!active) return;
        loadedModel.components.applySemanticLabels(semanticLabelsForComponents(loadedModel.components.list(), snapshot));
        setAssemblySnapshot(snapshot);
        engineeringSessionService.setAssemblyIntelligence(snapshot);
        engineeringSessionService.touch();
      })
      .catch((reason) => { if (active) setError(reason instanceof Error ? reason.message : String(reason)); });
    return () => { active = false; };
  }, [loadedModel]);

  const updateAssemblySnapshot = useCallback((snapshot: AssemblyIntelligenceSnapshot) => {
    const current = loadedModelRef.current;
    if (current) current.components.applySemanticLabels(semanticLabelsForComponents(current.components.list(), snapshot));
    setAssemblySnapshot(snapshot);
    engineeringSessionService.setAssemblyIntelligence(snapshot);
    engineeringSessionService.touch();
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
    if (explosionAnimationRef.current !== null) cancelAnimationFrame(explosionAnimationRef.current);
    startAttempt.current += 1;
    trackingService.current.stop();
    cameraService.current.stop(videoRef.current);
  }, []);

  const loadModel = useCallback(async () => {
    if (explosionAnimationRef.current !== null) cancelAnimationFrame(explosionAnimationRef.current);
    explosionAnimationRef.current = null;
    setModelState(resolveModelCoreState("load_started", Boolean(loadedModelRef.current)));
    setError(null);
    try {
      const next = await modelService.current.selectAndLoad();
      if (!next) {
        setModelState(resolveModelCoreState("load_cancelled", Boolean(loadedModelRef.current)));
        return;
      }
      const previous = loadedModelRef.current;
      setAssemblySnapshot(undefined);
      engineeringSessionService.attachModel(next);
      setComponentTransform(null);
      setTargetedComponentId(null);
      setComponentSearch("");
      setExpandedComponentIds(new Set(next.components.list().filter((component) => component.depth < 2).map((component) => component.id)));
      setExplosionError(null);
      setModelState(resolveModelCoreState("load_succeeded", true));
      if (previous) window.setTimeout(() => modelService.current.unload(previous), 0);
    } catch (reason) {
      setModelState(resolveModelCoreState("load_failed", Boolean(loadedModelRef.current)));
      setError(reason instanceof Error ? reason.message : String(reason));
    }
  }, []);

  const unloadModel = useCallback(() => {
    const previous = engineeringSessionService.detachModel();
    setComponentTransform(null);
    setTargetedComponentId(null);
    setComponentSearch("");
    setExpandedComponentIds(new Set());
    if (explosionAnimationRef.current !== null) cancelAnimationFrame(explosionAnimationRef.current);
    explosionAnimationRef.current = null;
    explosionFactorRef.current = 0;
    setExplosionError(null);
    setExplosionGestureState("idle");
    setAssemblySnapshot(undefined);
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
    engineeringSessionService.setSelectedComponent(id);
    if (!id || !loadedModelRef.current) return;
    setExpandedComponentIds((current) => new Set([...current, ...loadedModelRef.current!.components.ancestors(id)]));
  }, []);

  const mutateComponents = useCallback((operation: (model: LoadedEngineeringModel) => void) => {
    const current = loadedModelRef.current;
    if (!current) return;
    operation(current);
    setTargetedComponentId(null);
    engineeringSessionService.touch();
  }, []);

  const focusComponent = useCallback((componentId: string | null) => {
    if (componentId) engineeringSessionService.focusComponent(componentId, "ui");
    else engineeringSessionService.focusModel();
  }, []);

  const handleComponentTransform = useCallback((snapshot: ComponentTransformSnapshot | null) => {
    setComponentTransform(snapshot);
    engineeringSessionService.touch();
  }, []);

  const handleObjectChange = useCallback((snapshot: EngineeringObjectSnapshot) => {
    setObjectSnapshot(snapshot);
    engineeringSessionService.updateObjectSnapshot(snapshot);
  }, []);

  const stopExplosionAnimation = useCallback(() => {
    if (explosionAnimationRef.current !== null) cancelAnimationFrame(explosionAnimationRef.current);
    explosionAnimationRef.current = null;
  }, []);

  const applyExplosionFactor = useCallback((factor: number, assemblyState?: AssemblyState) => {
    const current = loadedModelRef.current;
    if (!current?.explosion.getState().enabled) return;
    const next = current.explosion.applyFactor(factor, assemblyState);
    explosionFactorRef.current = next.factor;
    setComponentTransform(null);
    engineeringSessionService.touch();
  }, []);

  const configureExplosion = useCallback((explosionMode: ExplosionMode) => {
    const current = loadedModelRef.current;
    if (!current) return false;
    stopExplosionAnimation();
    const selectedRootId = explosionMode === "selected" ? selectedComponentId : undefined;
    const result = current.explosion.configure(explosionMode, selectedRootId ?? undefined);
    explosionFactorRef.current = 0;
    setExplosionError(result.error ?? null);
    setComponentTransform(null);
    engineeringSessionService.touch();
    return result.success;
  }, [selectedComponentId, stopExplosionAnimation]);

  const animateExplosion = useCallback((target: 0 | 1, explosionMode?: ExplosionMode) => {
    const current = loadedModelRef.current;
    if (!current) return;
    if (explosionMode && !configureExplosion(explosionMode)) return;
    if (!current.explosion.getState().enabled) return;
    stopExplosionAnimation();
    setMode("explode");
    setExplosionError(null);
    const from = explosionFactorRef.current;
    const startedAt = performance.now();
    const duration = 620 * Math.max(0.25, Math.abs(target - from));
    const state: AssemblyState = target === 1 ? "exploding" : "reassembling";
    const frame = (now: number) => {
      const progress = Math.min(1, (now - startedAt) / duration);
      const eased = 1 - Math.pow(1 - progress, 3);
      applyExplosionFactor(from + (target - from) * eased, progress === 1 ? (target === 1 ? "exploded" : "assembled") : state);
      if (progress < 1) explosionAnimationRef.current = requestAnimationFrame(frame);
      else explosionAnimationRef.current = null;
    };
    explosionAnimationRef.current = requestAnimationFrame(frame);
  }, [applyExplosionFactor, configureExplosion, stopExplosionAnimation]);

  const handleExplosionSlider = useCallback((factor: number) => {
    stopExplosionAnimation();
    setMode("explode");
    applyExplosionFactor(factor);
  }, [applyExplosionFactor, stopExplosionAnimation]);

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
        <button onClick={() => engineeringSessionService.resetModelView("ui")}>RESET MODEL</button>
        <button onClick={() => void startCamera()} disabled={cameraState === "requesting" || cameraState === "online"}>INICIAR CÂMERA</button>
        <button onClick={stopCamera} disabled={cameraState === "offline"}>PARAR CÂMERA</button>
        <button className={previewVisible ? "active" : ""} onClick={() => setPreviewVisible((visible) => !visible)}>PREVIEW {previewVisible ? "ON" : "OFF"}</button>
        <button className={debugVisible ? "active" : ""} onClick={() => setDebugVisible((visible) => !visible)}>DEBUG {debugVisible ? "ON" : "OFF"}</button>
        <span>MODEL + VISION PROCESSING // LOCAL</span>
      </div>

      <div className="engineering-mode-switch" aria-label="Modo de manipulação">
        <span>SCOPE</span>
        {(["model", "component"] as const).map((candidate) => <button key={candidate} className={interactionScope === candidate ? "active" : ""} onClick={() => { engineeringSessionService.setModelMode(candidate); setTargetedComponentId(null); }}>{candidate.toUpperCase()}</button>)}
        <i />
        <span>TRANSFORM</span>
        {(["move", "rotate", "scale"] as const).map((candidate) => <button key={candidate} className={mode === candidate ? "active" : ""} onClick={() => setMode(candidate)}>{candidate.toUpperCase()}</button>)}
        <button className={mode === "explode" ? "active" : ""} disabled={!loadedModel || !explosionState.enabled} onClick={() => setMode("explode")}>EXPLODE</button>
        <button className={wireframe ? "active" : ""} onClick={() => setWireframe((active) => !active)}>WIREFRAME {wireframe ? "ON" : "OFF"}</button>
        <button className={gridVisible ? "active" : ""} onClick={() => setGridVisible((visible) => !visible)}>GRID {gridVisible ? "ON" : "OFF"}</button>
        <button className={axesVisible ? "active" : ""} onClick={() => setAxesVisible((visible) => !visible)}>AXES {axesVisible ? "ON" : "OFF"}</button>
      </div>

      <div className="engineering-module__layout">
        <section className="engineering-canvas" aria-label="Viewport tridimensional do Engineering Core">
          <header><span>VIEWPORT // PRIMARY</span><strong>{loadedModel?.metadata.name ?? "TEST-01"} // {objectSnapshot.status.toUpperCase()}</strong></header>
          <div className="engineering-canvas__stage">
            <EngineeringScene hands={sceneHands} mode={mode} interactionScope={interactionScope} calibration={calibration} model={loadedModel} wireframe={wireframe} gridVisible={gridVisible} axesVisible={axesVisible} resetSignal={resetSignal} selectedComponentId={selectedComponentId} targetedComponentId={targetedComponentId} componentRevision={componentRevision} explosionFactor={explosionState.factor} guideLinesVisible={guideLinesVisible} boundingBoxVisible={boundingBoxVisible} focusRequest={focusRequest} persistedObjectSnapshot={objectSnapshot} onComponentTarget={setTargetedComponentId} onComponentSelect={selectComponent} onComponentTransform={handleComponentTransform} onExplosionFactorChange={handleExplosionSlider} onExplosionGestureState={setExplosionGestureState} onObjectChange={handleObjectChange} onRendererReady={setRendererReady} />
            <span className="engineering-canvas__axis engineering-canvas__axis--x">AXIS X // INTERACTION PLANE</span>
            <span className="engineering-canvas__axis engineering-canvas__axis--y">AXIS Y // SCREEN SPACE</span>
            {sceneHands.map((hand) => <div key={hand.id} className={`engineering-hand-cursor engineering-hand-cursor--${hand.id} ${hand.gesture === "pinch" ? "engineering-hand-cursor--pinch" : ""}`} style={{ left: `${hand.cursor.x * 100}%`, top: `${hand.cursor.y * 100}%` }} aria-hidden="true"><i /><small>{hand.id.toUpperCase()}</small></div>)}
            <div className="engineering-object-readout engineering-object-inspector">
              {interactionScope === "component" && selectedComponent ? <>
                <span>COMPONENT</span><strong>{selectedComponent.name}</strong>
                <span>STATUS</span><strong data-status={componentTransform?.status ?? "ready"}>{(componentTransform?.status ?? "selected").toUpperCase()}</strong>
                <span>CONTROL</span><strong>{(componentTransform?.control ?? "none").toUpperCase()}</strong>
                <span>POSITION</span><strong>{selectedComponent.position.x.toFixed(2)} / {selectedComponent.position.y.toFixed(2)} / {selectedComponent.position.z.toFixed(2)}</strong>
                <span>ROTATION</span><strong>{selectedComponent.rotation.x.toFixed(2)} / {selectedComponent.rotation.y.toFixed(2)} / {selectedComponent.rotation.z.toFixed(2)}</strong>
                <span>SCALE</span><strong>{selectedComponent.scale.x.toFixed(2)} / {selectedComponent.scale.y.toFixed(2)} / {selectedComponent.scale.z.toFixed(2)}</strong>
                <span>EXPLOSION OFFSET</span><strong>{loadedModel!.explosion.getOffset(selectedComponent.id).x.toFixed(2)} / {loadedModel!.explosion.getOffset(selectedComponent.id).y.toFixed(2)} / {loadedModel!.explosion.getOffset(selectedComponent.id).z.toFixed(2)}</strong>
              </> : <>
                <span>OBJECT</span><strong>{loadedModel?.metadata.name ?? "TEST-01"}</strong>
                <span>STATUS</span><strong data-status={objectSnapshot.status}>{objectSnapshot.status.toUpperCase()}</strong>
                <span>CONTROL</span><strong>{objectSnapshot.control.toUpperCase()}</strong>
                <span>POSITION</span><strong>{objectSnapshot.position.x.toFixed(2)} / {objectSnapshot.position.y.toFixed(2)} / {objectSnapshot.position.z.toFixed(2)}</strong>
                <span>ROTATION</span><strong>{objectSnapshot.rotation.x.toFixed(2)} / {objectSnapshot.rotation.y.toFixed(2)} / {objectSnapshot.rotation.z.toFixed(2)}</strong>
                <span>SCALE</span><strong>{objectSnapshot.scale.toFixed(2)}</strong>
              </>}
            </div>
          </div>
          <footer><span>SCOPE {interactionScope.toUpperCase()} // {mode.toUpperCase()}</span><span>{mode === "explode" ? "TWO HAND DISTANCE // SLIDER FALLBACK" : interactionScope === "component" ? "PINCH SELECT // RELEASE // PINCH TRANSFORM" : "MOUSE DRAG ORBIT // WHEEL ZOOM"}</span><span>RENDERER {rendererReady ? "ONLINE" : "ERROR"}</span></footer>
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

          <section className="engineering-register engineering-explosion-panel">
            <header><strong>EXPLOSION CORE</strong><span data-state={loadedModel && explosionState.enabled ? "online" : "offline"}>{loadedModel && explosionState.enabled ? "ONLINE" : "OFFLINE"}</span></header>
            <dl>
              <div><dt>ASSEMBLY</dt><dd>{explosionState.assemblyState.toUpperCase()}</dd></div>
              <div><dt>FACTOR</dt><dd>{Math.round(explosionState.factor * 100)}%</dd></div>
              <div><dt>MODE</dt><dd>{explosionState.mode.toUpperCase()}</dd></div>
              <div><dt>UNITS</dt><dd>{loadedModel?.explosion.getMetadata().length ?? 0}</dd></div>
              <div><dt>GESTURE</dt><dd>{explosionGestureState.toUpperCase()}</dd></div>
            </dl>
            <div className="engineering-explosion-controls">
              <div className="engineering-explosion-mode">
                <button className={explosionState.mode === "all" ? "active" : ""} disabled={!loadedModel} onClick={() => configureExplosion("all")}>ALL</button>
                <button className={explosionState.mode === "selected" ? "active" : ""} disabled={!loadedModel || !selectedComponent} onClick={() => configureExplosion("selected")}>SELECTED</button>
              </div>
              <label><span>EXPLOSION FACTOR</span><output>{Math.round(explosionState.factor * 100)}%</output><input type="range" min="0" max="100" step="1" value={Math.round(explosionState.factor * 100)} disabled={!loadedModel || !explosionState.enabled} onChange={(event) => handleExplosionSlider(Number(event.target.value) / 100)} /></label>
              <div className="engineering-explosion-presets">
                {[25, 50, 100].map((value) => <button key={value} disabled={!explosionState.enabled} onClick={() => handleExplosionSlider(value / 100)}>{value}%</button>)}
              </div>
              <button disabled={!loadedModel} onClick={() => animateExplosion(1, "all")}>EXPLODIR MONTAGEM</button>
              <button disabled={!loadedModel || !selectedComponent} onClick={() => animateExplosion(1, "selected")}>EXPLODIR SELECIONADO</button>
              <button disabled={!loadedModel || explosionState.factor === 0} onClick={() => animateExplosion(0)}>RECONSTRUIR</button>
              <button className={guideLinesVisible ? "active" : ""} disabled={!loadedModel} onClick={() => setGuideLinesVisible((visible) => !visible)}>GUIDE LINES {guideLinesVisible ? "ON" : "OFF"}</button>
              {explosionState.factor > 0 && <small>TRANSFORMAÇÃO INDIVIDUAL SUSPENSA // MODEL ROOT DISPONÍVEL</small>}
              {explosionError && <small className="engineering-explosion-error">{explosionError}</small>}
            </div>
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
                <div><dt>POSITION</dt><dd>{selectedComponent.position.x.toFixed(2)} / {selectedComponent.position.y.toFixed(2)} / {selectedComponent.position.z.toFixed(2)}</dd></div>
                <div><dt>ROTATION</dt><dd>{selectedComponent.rotation.x.toFixed(2)} / {selectedComponent.rotation.y.toFixed(2)} / {selectedComponent.rotation.z.toFixed(2)}</dd></div>
                <div><dt>SCALE</dt><dd>{selectedComponent.scale.x.toFixed(2)} / {selectedComponent.scale.y.toFixed(2)} / {selectedComponent.scale.z.toFixed(2)}</dd></div>
                <div><dt>ORIGINAL POS.</dt><dd>{selectedComponent.originalPosition.x.toFixed(2)} / {selectedComponent.originalPosition.y.toFixed(2)} / {selectedComponent.originalPosition.z.toFixed(2)}</dd></div>
                <div><dt>ORIGINAL ROT.</dt><dd>{selectedComponent.originalRotation.x.toFixed(2)} / {selectedComponent.originalRotation.y.toFixed(2)} / {selectedComponent.originalRotation.z.toFixed(2)}</dd></div>
                <div><dt>ORIGINAL SCALE</dt><dd>{selectedComponent.originalScale.x.toFixed(2)} / {selectedComponent.originalScale.y.toFixed(2)} / {selectedComponent.originalScale.z.toFixed(2)}</dd></div>
                <div><dt>EXPLOSION OFFSET</dt><dd>{loadedModel.explosion.getOffset(selectedComponent.id).x.toFixed(2)} / {loadedModel.explosion.getOffset(selectedComponent.id).y.toFixed(2)} / {loadedModel.explosion.getOffset(selectedComponent.id).z.toFixed(2)}</dd></div>
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
                <button onClick={() => { mutateComponents((current) => { current.explosion.applyFactor(0); current.components.restore(); explosionFactorRef.current = 0; }); selectComponent(null); }}>RESTAURAR COMPONENTES</button>
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

          <section className="engineering-register engineering-ai-commands">
            <header><strong>AZRIEL COMMAND</strong><span>SESSION</span></header>
            <div>
              {engineeringCommands.filter((command) => command.source === "ai").slice(0, 4).map((command) => <article key={command.id} data-state={command.status}>
                <time>{new Date(command.timestamp).toLocaleTimeString("pt-BR", { hour: "2-digit", minute: "2-digit", second: "2-digit" })}</time>
                <strong>{command.command.replaceAll("_", " ").toUpperCase()}</strong>
                <span>{command.target ?? "ENGINEERING CORE"}</span>
                <small>{command.status.toUpperCase()} // {command.message}</small>
              </article>)}
              {!engineeringCommands.some((command) => command.source === "ai") && <p className="engineering-model-empty">NENHUM COMANDO DA IA NESTA SESSÃO</p>}
            </div>
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
              <div><dt>EXPLOSION CORE</dt><dd>{loadedModel && explosionState.enabled ? "ONLINE" : "OFFLINE"}</dd></div><div><dt>ASSEMBLY</dt><dd>{explosionState.assemblyState.toUpperCase()}</dd></div>
            </dl>
          </section>
        </aside>
      </div>
      {loadedModel && <AssemblyIntelligencePanel modelIdentity={loadedModel.identity} modelName={loadedModel.metadata.name} modelFormat={loadedModel.metadata.format} components={components} selectedComponentId={selectedComponentId} snapshot={assemblySnapshot} onSnapshotChange={updateAssemblySnapshot} onSelectComponent={selectComponent} />}
    </section>
  );
}
