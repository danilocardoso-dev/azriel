import { useEffect, useRef } from "react";
import * as THREE from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
import { InteractionController } from "../../engineering/interactionController";
import type { LoadedEngineeringModel } from "../../engineering/modelService";
import type { EngineeringCalibration, EngineeringObjectSnapshot, EngineeringObjectState, GestureState, HandInteractionPoint, HandSide, ManipulationMode, ViewportPoint } from "../../engineering/types";

export interface EngineeringHandControl {
  id: HandSide;
  gesture: GestureState;
  cursor: ViewportPoint;
}

interface EngineeringSceneProps {
  hands: EngineeringHandControl[];
  mode: ManipulationMode;
  calibration: EngineeringCalibration;
  model: LoadedEngineeringModel | null;
  wireframe: boolean;
  gridVisible: boolean;
  axesVisible: boolean;
  resetSignal: number;
  onObjectChange: (snapshot: EngineeringObjectSnapshot) => void;
  onRendererReady: (ready: boolean) => void;
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
  controller: InteractionController;
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

function frameObject(runtime: SceneRuntime) {
  runtime.manipulator.updateMatrixWorld(true);
  const box = new THREE.Box3().setFromObject(runtime.manipulator);
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

export function EngineeringScene({ hands, mode, calibration, model, wireframe, gridVisible, axesVisible, resetSignal, onObjectChange, onRendererReady }: EngineeringSceneProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const runtimeRef = useRef<SceneRuntime | null>(null);
  const lastSnapshotRef = useRef<string>("");

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
        renderer, scene, camera, controls, manipulator, fallback, loadedRoot: null, grid, axes, selectionBox,
        controller: new InteractionController(), raycaster: new THREE.Raycaster(),
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
      runtime.renderer.dispose();
      runtimeRef.current = null;
      onRendererReady(false);
    };
  }, [onRendererReady]);

  useEffect(() => {
    const runtime = runtimeRef.current;
    if (!runtime) return;
    restoreWireframe(runtime.originalWireframe);
    if (runtime.loadedRoot) runtime.manipulator.remove(runtime.loadedRoot);
    runtime.loadedRoot = model?.root ?? null;
    runtime.manipulator.remove(runtime.fallback);
    if (runtime.loadedRoot) runtime.manipulator.add(runtime.loadedRoot);
    else runtime.manipulator.add(runtime.fallback);
    const snapshot = runtime.controller.reset();
    runtime.manipulator.position.set(snapshot.position.x, snapshot.position.y, snapshot.position.z);
    runtime.manipulator.rotation.set(snapshot.rotation.x, snapshot.rotation.y, snapshot.rotation.z);
    runtime.manipulator.scale.setScalar(snapshot.scale);
    runtime.selectionBox.visible = false;
    frameObject(runtime);
    lastSnapshotRef.current = JSON.stringify(snapshot);
    onObjectChange(snapshot);
  }, [model, onObjectChange]);

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
    runtime.controller.updateSettings({
      rotationSensitivity: calibration.rotationSensitivity,
      minScale: calibration.minScale,
      maxScale: calibration.maxScale,
    });
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
  }, [hands, mode, calibration, onObjectChange]);

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
