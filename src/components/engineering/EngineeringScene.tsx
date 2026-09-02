import { useEffect, useRef } from "react";
import * as THREE from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
import { ComponentInteractionController } from "../../engineering/componentInteractionController";
import { ExplosionGestureController } from "../../engineering/explosionGestureController";
import { InteractionController } from "../../engineering/interactionController";
import type { LoadedEngineeringModel } from "../../engineering/modelService";
import type { ComponentTransformSnapshot, EngineeringCalibration, EngineeringControlMode, EngineeringInteractionScope, EngineeringObjectSnapshot, EngineeringObjectState, GestureState, HandInteractionPoint, HandSide, ViewportPoint } from "../../engineering/types";

export interface EngineeringHandControl {
  id: HandSide;
  gesture: GestureState;
  cursor: ViewportPoint;
}

interface EngineeringSceneProps {
  hands: EngineeringHandControl[];
  mode: EngineeringControlMode;
  interactionScope: EngineeringInteractionScope;
  calibration: EngineeringCalibration;
  model: LoadedEngineeringModel | null;
  wireframe: boolean;
  gridVisible: boolean;
  axesVisible: boolean;
  resetSignal: number;
  selectedComponentId: string | null;
  targetedComponentId: string | null;
  componentRevision: number;
  explosionFactor: number;
  guideLinesVisible: boolean;
  boundingBoxVisible: boolean;
  focusRequest: { sequence: number; componentId: string | null };
  onComponentTarget: (id: string | null) => void;
  onComponentSelect: (id: string | null) => void;
  onComponentTransform: (snapshot: ComponentTransformSnapshot | null) => void;
  onExplosionFactorChange: (factor: number) => void;
  onExplosionGestureState: (state: "idle" | "active" | "cancelled") => void;
  onObjectChange: (snapshot: EngineeringObjectSnapshot) => void;
  onRendererReady: (ready: boolean) => void;
  persistedObjectSnapshot: EngineeringObjectSnapshot;
}

interface SceneRuntime {
  renderer: THREE.WebGLRenderer;
  scene: THREE.Scene;
  camera: THREE.PerspectiveCamera;
  controls: OrbitControls;
  manipulator: THREE.Group;
  fallback: THREE.Mesh<THREE.BoxGeometry, THREE.MeshStandardMaterial>;
  loadedRoot: THREE.Group | null;
  grid: THREE.GridHelper;
  axes: THREE.AxesHelper;
  selectionBox: THREE.BoxHelper;
  componentTargetBox: THREE.BoxHelper;
  componentSelectionBox: THREE.BoxHelper;
  componentBoundsBox: THREE.BoxHelper;
  controller: InteractionController;
  componentController: ComponentInteractionController;
  explosionController: ExplosionGestureController;
  guideLines: Map<string, THREE.Line>;
  raycaster: THREE.Raycaster;
  dragPlane: THREE.Plane;
  resizeObserver: ResizeObserver;
  originalWireframe: Map<THREE.Material, boolean>;
  render: () => void;
}

const colorByStatus: Record<EngineeringObjectState, number> = {
  ready: 0x164959,
  targeted: 0x27cce5,
  grabbed: 0x4bdb8f,
};

function setWireframe(root: THREE.Object3D, enabled: boolean, originals: Map<THREE.Material, boolean>) {
  root.traverse((object) => {
    if (!(object instanceof THREE.Mesh)) return;
    const materials = Array.isArray(object.material) ? object.material : [object.material];
    materials.forEach((material) => {
      if (!("wireframe" in material)) return;
      const compatible = material as THREE.Material & { wireframe: boolean; needsUpdate: boolean };
      if (!originals.has(material)) originals.set(material, compatible.wireframe);
      compatible.wireframe = enabled ? true : (originals.get(material) ?? false);
      compatible.needsUpdate = true;
    });
  });
}

function restoreWireframe(originals: Map<THREE.Material, boolean>) {
  originals.forEach((wireframe, material) => {
    if (!("wireframe" in material)) return;
    const compatible = material as THREE.Material & { wireframe: boolean; needsUpdate: boolean };
    compatible.wireframe = wireframe;
    compatible.needsUpdate = true;
  });
  originals.clear();
}

function clearGuideLines(runtime: SceneRuntime) {
  runtime.guideLines.forEach((line) => {
    line.removeFromParent();
    line.geometry.dispose();
    (line.material as THREE.Material).dispose();
  });
  runtime.guideLines.clear();
}

function frameObject(runtime: SceneRuntime, target: THREE.Object3D = runtime.manipulator) {
  target.updateMatrixWorld(true);
  const box = new THREE.Box3().setFromObject(target);
  if (box.isEmpty()) return;
  const center = box.getCenter(new THREE.Vector3());
  const size = box.getSize(new THREE.Vector3());
  const radius = Math.max(size.length() * 0.5, 0.7);
  const verticalFov = THREE.MathUtils.degToRad(runtime.camera.fov);
  const distance = Math.max(radius / Math.tan(verticalFov / 2), 2.5) * 1.25;
  runtime.camera.near = Math.max(distance / 100, 0.01);
  runtime.camera.far = Math.max(distance * 100, 100);
  runtime.camera.position.copy(center).add(new THREE.Vector3(0.8, 0.55, 1).normalize().multiplyScalar(distance));
  runtime.camera.updateProjectionMatrix();
  runtime.controls.target.copy(center);
  runtime.controls.update();
  runtime.render();
}

export function EngineeringScene({ hands, mode, interactionScope, calibration, model, wireframe, gridVisible, axesVisible, resetSignal, selectedComponentId, targetedComponentId, componentRevision, explosionFactor, guideLinesVisible, boundingBoxVisible, focusRequest, onComponentTarget, onComponentSelect, onComponentTransform, onExplosionFactorChange, onExplosionGestureState, onObjectChange, onRendererReady, persistedObjectSnapshot }: EngineeringSceneProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const runtimeRef = useRef<SceneRuntime | null>(null);
  const lastSnapshotRef = useRef<string>("");
  const previousHandGestureRef = useRef<Record<HandSide, GestureState>>({ left: "none", right: "none" });
  const mouseComponentTargetRef = useRef<string | null>(null);
  const handComponentTargetRef = useRef<string | null>(null);
  const componentManipulationArmedRef = useRef(true);
  const lastComponentSnapshotRef = useRef("");
  const explosionFactorRef = useRef(explosionFactor);
  const persistedObjectSnapshotRef = useRef(persistedObjectSnapshot);

  useEffect(() => { persistedObjectSnapshotRef.current = persistedObjectSnapshot; }, [persistedObjectSnapshot]);

  useEffect(() => {
    const canvas = canvasRef.current;
    const container = canvas?.parentElement;
    if (!canvas || !container) return;

    try {
      const renderer = new THREE.WebGLRenderer({ canvas, alpha: true, antialias: true, powerPreference: "high-performance" });
      renderer.setClearColor(0x000000, 0);
      renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
      renderer.outputColorSpace = THREE.SRGBColorSpace;
      renderer.toneMapping = THREE.ACESFilmicToneMapping;
      renderer.toneMappingExposure = 1.05;
      const scene = new THREE.Scene();
      scene.fog = new THREE.FogExp2(0x020a0f, 0.055);
      const camera = new THREE.PerspectiveCamera(43, 1, 0.1, 100);
      camera.position.set(0, 3.5, 7);

      const grid = new THREE.GridHelper(16, 24, 0x1f9db1, 0x0b3944);
      const axes = new THREE.AxesHelper(1.5);
      axes.position.set(-3.3, 0.02, -2.4);
      scene.add(grid, axes);
      scene.add(new THREE.HemisphereLight(0x7defff, 0x061017, 2.1));
      const keyLight = new THREE.DirectionalLight(0xffffff, 3.2);
      keyLight.position.set(3, 6, 5);
      scene.add(keyLight);
      const rimLight = new THREE.PointLight(0x4bdb8f, 16, 12);
      rimLight.position.set(-3, 2, 3);
      scene.add(rimLight);

      const manipulator = new THREE.Group();
      manipulator.name = "ModelRoot";
      const geometry = new THREE.BoxGeometry(1.15, 1.15, 1.15, 2, 2, 2);
      const material = new THREE.MeshStandardMaterial({ color: 0x164959, emissive: 0x06242f, roughness: 0.42, metalness: 0.72, wireframe: true });
      const fallback = new THREE.Mesh(geometry, material);
      fallback.name = "TEST-01";
      manipulator.add(fallback);
      scene.add(manipulator);

      const selectionBox = new THREE.BoxHelper(manipulator, colorByStatus.targeted);
      selectionBox.visible = false;
      scene.add(selectionBox);
      const componentTargetBox = new THREE.BoxHelper(manipulator, 0x27cce5);
      const componentSelectionBox = new THREE.BoxHelper(manipulator, 0x4bdb8f);
      const componentBoundsBox = new THREE.BoxHelper(manipulator, 0xffb64b);
      componentTargetBox.visible = false;
      componentSelectionBox.visible = false;
      componentBoundsBox.visible = false;
      scene.add(componentTargetBox, componentSelectionBox, componentBoundsBox);

      const controls = new OrbitControls(camera, canvas);
      controls.enableDamping = false;
      controls.minDistance = 1.2;
      controls.maxDistance = 30;
      controls.target.set(0, 0.65, 0);
      controls.update();

      const render = () => renderer.render(scene, camera);
      controls.addEventListener("change", render);
      const resize = () => {
        const width = Math.max(1, container.clientWidth);
        const height = Math.max(1, container.clientHeight);
        renderer.setSize(width, height, false);
        camera.aspect = width / height;
        camera.updateProjectionMatrix();
        render();
      };
      const resizeObserver = new ResizeObserver(resize);
      resizeObserver.observe(container);
      runtimeRef.current = {
        renderer, scene, camera, controls, manipulator, fallback, loadedRoot: null, grid, axes, selectionBox, componentTargetBox, componentSelectionBox, componentBoundsBox,
        controller: new InteractionController(), componentController: new ComponentInteractionController(), explosionController: new ExplosionGestureController(), guideLines: new Map(), raycaster: new THREE.Raycaster(),
        dragPlane: new THREE.Plane(new THREE.Vector3(0, 0, 1), 0), resizeObserver,
        originalWireframe: new Map(), render,
      };
      resize();
      onRendererReady(true);
    } catch {
      onRendererReady(false);
    }

    return () => {
      const runtime = runtimeRef.current;
      if (!runtime) return;
      runtime.resizeObserver.disconnect();
      runtime.controls.removeEventListener("change", runtime.render);
      runtime.controls.dispose();
      clearGuideLines(runtime);
      restoreWireframe(runtime.originalWireframe);
      runtime.fallback.geometry.dispose();
      runtime.fallback.material.dispose();
      runtime.grid.geometry.dispose();
      const gridMaterials = Array.isArray(runtime.grid.material) ? runtime.grid.material : [runtime.grid.material];
      gridMaterials.forEach((material) => material.dispose());
      runtime.axes.geometry.dispose();
      (Array.isArray(runtime.axes.material) ? runtime.axes.material : [runtime.axes.material]).forEach((material) => material.dispose());
      runtime.selectionBox.geometry.dispose();
      (runtime.selectionBox.material as THREE.Material).dispose();
      [runtime.componentTargetBox, runtime.componentSelectionBox, runtime.componentBoundsBox].forEach((helper) => {
        helper.geometry.dispose();
        (helper.material as THREE.Material).dispose();
      });
      runtime.renderer.dispose();
      runtimeRef.current = null;
      onRendererReady(false);
    };
  }, [onRendererReady]);

  useEffect(() => {
    const runtime = runtimeRef.current;
    if (!runtime) return;
    restoreWireframe(runtime.originalWireframe);
    clearGuideLines(runtime);
    if (runtime.loadedRoot) runtime.manipulator.remove(runtime.loadedRoot);
    runtime.loadedRoot = model?.root ?? null;
    runtime.manipulator.remove(runtime.fallback);
    if (runtime.loadedRoot) runtime.manipulator.add(runtime.loadedRoot);
    else runtime.manipulator.add(runtime.fallback);
    const snapshot = runtime.controller.restore(persistedObjectSnapshotRef.current);
    runtime.manipulator.position.set(snapshot.position.x, snapshot.position.y, snapshot.position.z);
    runtime.manipulator.rotation.set(snapshot.rotation.x, snapshot.rotation.y, snapshot.rotation.z);
    runtime.manipulator.scale.setScalar(snapshot.scale);
    runtime.selectionBox.visible = false;
    runtime.componentTargetBox.visible = false;
    runtime.componentSelectionBox.visible = false;
    runtime.componentBoundsBox.visible = false;
    frameObject(runtime);
    lastSnapshotRef.current = JSON.stringify(snapshot);
    onObjectChange(snapshot);
  }, [model, onObjectChange]);

  useEffect(() => { explosionFactorRef.current = explosionFactor; }, [explosionFactor]);

  useEffect(() => {
    const runtime = runtimeRef.current;
    if (!runtime) return;
    if (!model || !guideLinesVisible || explosionFactor <= 0) {
      clearGuideLines(runtime);
      runtime.render();
      return;
    }
    const activeIds = new Set<string>();
    for (const guide of model.explosion.getGuideLines()) {
      const object = model.components.getObject(guide.componentId);
      const component = model.components.get(guide.componentId);
      if (!object?.parent || !component?.visible) continue;
      activeIds.add(guide.componentId);
      let line = runtime.guideLines.get(guide.componentId);
      if (!line) {
        const geometry = new THREE.BufferGeometry();
        geometry.setAttribute("position", new THREE.Float32BufferAttribute(new Float32Array(6), 3));
        const material = new THREE.LineBasicMaterial({ color: 0x46e9ff, transparent: true, opacity: 0.46, depthTest: false });
        line = new THREE.Line(geometry, material);
        line.name = `ExplosionGuide-${guide.componentId}`;
        line.renderOrder = 20;
        object.parent.add(line);
        runtime.guideLines.set(guide.componentId, line);
      } else if (line.parent !== object.parent) object.parent.add(line);
      const positions = line.geometry.getAttribute("position") as THREE.BufferAttribute;
      positions.setXYZ(0, guide.from.x, guide.from.y, guide.from.z);
      positions.setXYZ(1, guide.to.x, guide.to.y, guide.to.z);
      positions.needsUpdate = true;
      line.geometry.computeBoundingSphere();
    }
    [...runtime.guideLines].forEach(([id, line]) => {
      if (activeIds.has(id)) return;
      line.removeFromParent();
      line.geometry.dispose();
      (line.material as THREE.Material).dispose();
      runtime.guideLines.delete(id);
    });
    runtime.render();
  }, [model, explosionFactor, guideLinesVisible, componentRevision]);

  useEffect(() => {
    const runtime = runtimeRef.current;
    if (!runtime?.loadedRoot) return;
    restoreWireframe(runtime.originalWireframe);
    setWireframe(runtime.loadedRoot, wireframe, runtime.originalWireframe);
    runtime.render();
  }, [model, wireframe]);

  useEffect(() => {
    const runtime = runtimeRef.current;
    if (!runtime) return;
    runtime.grid.visible = gridVisible;
    runtime.axes.visible = axesVisible;
    runtime.render();
  }, [gridVisible, axesVisible]);

  useEffect(() => {
    const runtime = runtimeRef.current;
    if (!runtime) return;
    const updateHelper = (helper: THREE.BoxHelper, componentId: string | null, visible: boolean) => {
      const object = model?.components.getObject(componentId);
      const component = model?.components.get(componentId);
      helper.visible = Boolean(object && component?.visible && visible);
      if (helper.visible && object) helper.setFromObject(object);
    };
    updateHelper(runtime.componentTargetBox, targetedComponentId, interactionScope === "component" && targetedComponentId !== selectedComponentId);
    updateHelper(runtime.componentSelectionBox, selectedComponentId, interactionScope === "component");
    updateHelper(runtime.componentBoundsBox, selectedComponentId, interactionScope === "component" && boundingBoxVisible);
    runtime.selectionBox.visible = interactionScope === "model" && runtime.selectionBox.visible;
    runtime.render();
  }, [model, interactionScope, targetedComponentId, selectedComponentId, boundingBoxVisible, componentRevision]);

  useEffect(() => {
    const runtime = runtimeRef.current;
    const canvas = canvasRef.current;
    if (!runtime || !canvas || interactionScope !== "component" || !model) return;
    let pointerDown: { x: number; y: number } | null = null;
    const raycast = (event: PointerEvent) => {
      const bounds = canvas.getBoundingClientRect();
      const pointer = new THREE.Vector2(
        ((event.clientX - bounds.left) / bounds.width) * 2 - 1,
        -((event.clientY - bounds.top) / bounds.height) * 2 + 1,
      );
      runtime.raycaster.setFromCamera(pointer, runtime.camera);
      const hit = runtime.raycaster.intersectObjects(model.components.getRaycastMeshes(), false)[0]?.object ?? null;
      return model.components.resolveObject(hit);
    };
    const publishTarget = () => onComponentTarget(handComponentTargetRef.current ?? mouseComponentTargetRef.current);
    const move = (event: PointerEvent) => { mouseComponentTargetRef.current = raycast(event); publishTarget(); };
    const leave = () => { mouseComponentTargetRef.current = null; publishTarget(); };
    const down = (event: PointerEvent) => { pointerDown = { x: event.clientX, y: event.clientY }; };
    const up = (event: PointerEvent) => {
      if (!pointerDown) return;
      const moved = Math.hypot(event.clientX - pointerDown.x, event.clientY - pointerDown.y) > 5;
      pointerDown = null;
      if (moved) return;
      onComponentSelect(raycast(event));
    };
    canvas.addEventListener("pointermove", move);
    canvas.addEventListener("pointerleave", leave);
    canvas.addEventListener("pointerdown", down);
    canvas.addEventListener("pointerup", up);
    return () => {
      canvas.removeEventListener("pointermove", move);
      canvas.removeEventListener("pointerleave", leave);
      canvas.removeEventListener("pointerdown", down);
      canvas.removeEventListener("pointerup", up);
    };
  }, [interactionScope, model, onComponentSelect, onComponentTarget]);

  useEffect(() => {
    const runtime = runtimeRef.current;
    if (!runtime) return;
    runtime.controller.updateSettings({
      rotationSensitivity: calibration.rotationSensitivity,
      minScale: calibration.minScale,
      maxScale: calibration.maxScale,
    });
    if (mode === "explode") {
      runtime.controller.update({ mode: "move", hands: [] });
      runtime.componentController.release();
      const result = runtime.explosionController.update(
        hands.map((hand) => ({ id: hand.id, viewport: hand.cursor })),
        explosionFactorRef.current,
        calibration.smoothingAlpha,
      );
      if (result.cancelled) onExplosionGestureState("cancelled");
      else if (result.active) onExplosionGestureState("active");
      runtime.controls.enabled = true;
      if (Math.abs(result.factor - explosionFactorRef.current) > 0.0005) {
        explosionFactorRef.current = result.factor;
        onExplosionFactorChange(result.factor);
      }
      runtime.render();
      return;
    }
    runtime.explosionController.cancel(explosionFactorRef.current);
    onExplosionGestureState("idle");
    if (interactionScope === "component") {
      const emptySnapshot = runtime.controller.update({ mode, hands: [] });
      runtime.selectionBox.visible = false;
      let handTarget: string | null = null;
      const selectedObject = model?.components.getObject(selectedComponentId) ?? null;
      const selectedComponent = model?.components.get(selectedComponentId) ?? null;
      if (!selectedObject || !selectedComponent?.visible) {
        runtime.componentController.clear();
        if (lastComponentSnapshotRef.current) {
          lastComponentSnapshotRef.current = "";
          onComponentTransform(null);
        }
      }
      else {
        runtime.componentController.bind(selectedComponent.id, selectedComponent.position, selectedComponent.rotation, selectedComponent.scale);
        runtime.componentController.updateSettings({
          rotationSensitivity: calibration.rotationSensitivity,
          minScaleFactor: calibration.minScale,
          maxScaleFactor: calibration.maxScale,
        });
      }
      const interactionHands: HandInteractionPoint[] = hands.map((hand) => {
        const pointer = new THREE.Vector2(hand.cursor.ndcX, hand.cursor.ndcY);
        runtime.raycaster.setFromCamera(pointer, runtime.camera);
        const hit = runtime.raycaster.intersectObjects(model?.components.getRaycastMeshes() ?? [], false)[0]?.object ?? null;
        const componentId = model?.components.resolveObject(hit) ?? null;
        if ((hand.gesture === "point" || hand.gesture === "pinch") && componentId) handTarget = componentId;
        const hitWithinSelected = Boolean(componentId && selectedComponentId && (componentId === selectedComponentId || model?.components.ancestors(componentId).includes(selectedComponentId)));
        const previous = previousHandGestureRef.current[hand.id];
        if (hand.gesture === "pinch" && previous !== "pinch" && componentId && !hitWithinSelected) {
          componentManipulationArmedRef.current = false;
          onComponentSelect(componentId);
        }
        previousHandGestureRef.current[hand.id] = hand.gesture;
        const intersection = new THREE.Vector3();
        const worldIntersection = runtime.raycaster.ray.intersectPlane(runtime.dragPlane, intersection);
        let localPoint = worldIntersection ? intersection.clone() : null;
        if (localPoint && selectedObject?.parent) {
          selectedObject.parent.updateWorldMatrix(true, false);
          localPoint = selectedObject.parent.worldToLocal(localPoint);
        }
        return { id: hand.id, gesture: hand.gesture, viewport: hand.cursor, world: localPoint ? { x: localPoint.x, y: localPoint.y, z: localPoint.z } : null, hovered: hitWithinSelected };
      });
      if (!hands.some((hand) => hand.gesture === "pinch")) componentManipulationArmedRef.current = true;
      const componentSnapshot = explosionFactor <= 0 && selectedObject && componentManipulationArmedRef.current
        ? runtime.componentController.update(mode, interactionHands)
        : runtime.componentController.release();
      if (selectedObject && componentSnapshot) {
        selectedObject.position.set(componentSnapshot.position.x, componentSnapshot.position.y, componentSnapshot.position.z);
        selectedObject.rotation.set(componentSnapshot.rotation.x, componentSnapshot.rotation.y, componentSnapshot.rotation.z);
        selectedObject.scale.set(componentSnapshot.scale.x, componentSnapshot.scale.y, componentSnapshot.scale.z);
        selectedObject.updateMatrixWorld(true);
        runtime.controls.enabled = componentSnapshot.status !== "grabbed";
        const serializedComponent = JSON.stringify(componentSnapshot);
        if (lastComponentSnapshotRef.current !== serializedComponent) {
          lastComponentSnapshotRef.current = serializedComponent;
          onComponentTransform(componentSnapshot);
        }
      } else runtime.controls.enabled = true;
      handComponentTargetRef.current = handTarget;
      onComponentTarget(handTarget ?? mouseComponentTargetRef.current);
      lastSnapshotRef.current = JSON.stringify(emptySnapshot);
      runtime.render();
      return;
    }
    previousHandGestureRef.current = { left: "none", right: "none" };
    handComponentTargetRef.current = null;
    const target = runtime.loadedRoot ?? runtime.fallback;
    const interactionHands: HandInteractionPoint[] = hands.map((hand) => {
      const pointer = new THREE.Vector2(hand.cursor.ndcX, hand.cursor.ndcY);
      runtime.raycaster.setFromCamera(pointer, runtime.camera);
      const hovered = (hand.gesture === "point" || hand.gesture === "pinch") && runtime.raycaster.intersectObject(target, true).length > 0;
      const intersection = new THREE.Vector3();
      const world = runtime.raycaster.ray.intersectPlane(runtime.dragPlane, intersection)
        ? { x: intersection.x, y: intersection.y, z: 0 }
        : null;
      return { id: hand.id, gesture: hand.gesture, viewport: hand.cursor, world, hovered };
    });

    const snapshot = runtime.controller.update({ mode, hands: interactionHands });
    runtime.manipulator.position.set(snapshot.position.x, snapshot.position.y, snapshot.position.z);
    runtime.manipulator.rotation.set(snapshot.rotation.x, snapshot.rotation.y, snapshot.rotation.z);
    runtime.manipulator.scale.setScalar(snapshot.scale);
    runtime.controls.enabled = snapshot.status !== "grabbed";
    runtime.selectionBox.visible = snapshot.status !== "ready";
    if (runtime.selectionBox.visible) {
      runtime.selectionBox.setFromObject(runtime.manipulator);
      runtime.selectionBox.material.color.setHex(colorByStatus[snapshot.status]);
    }
    runtime.fallback.material.color.setHex(colorByStatus[snapshot.status]);
    runtime.render();
    const serialized = JSON.stringify(snapshot);
    if (lastSnapshotRef.current !== serialized) {
      lastSnapshotRef.current = serialized;
      onObjectChange(snapshot);
    }
  }, [hands, mode, interactionScope, calibration, model, selectedComponentId, explosionFactor, onComponentSelect, onComponentTarget, onComponentTransform, onExplosionFactorChange, onExplosionGestureState, onObjectChange]);

  useEffect(() => {
    const runtime = runtimeRef.current;
    if (!runtime || focusRequest.sequence === 0) return;
    const target = focusRequest.componentId ? model?.components.getObject(focusRequest.componentId) : runtime.manipulator;
    if (target) frameObject(runtime, target);
  }, [focusRequest, model]);

  useEffect(() => {
    const runtime = runtimeRef.current;
    if (!runtime) return;
    const snapshot = runtime.controller.reset();
    runtime.manipulator.position.set(snapshot.position.x, snapshot.position.y, snapshot.position.z);
    runtime.manipulator.rotation.set(snapshot.rotation.x, snapshot.rotation.y, snapshot.rotation.z);
    runtime.manipulator.scale.setScalar(snapshot.scale);
    runtime.controls.enabled = true;
    runtime.selectionBox.visible = false;
    frameObject(runtime);
    lastSnapshotRef.current = JSON.stringify(snapshot);
    onObjectChange(snapshot);
  }, [resetSignal, onObjectChange]);

  return <canvas ref={canvasRef} className="engineering-scene" aria-label="Cena 3D do Engineering Core" />;
}
