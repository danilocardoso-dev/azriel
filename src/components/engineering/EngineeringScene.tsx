import { useEffect, useRef } from "react";
import * as THREE from "three";
import { InteractionController } from "../../engineering/interactionController";
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
  resetSignal: number;
  onObjectChange: (snapshot: EngineeringObjectSnapshot) => void;
  onRendererReady: (ready: boolean) => void;
}

interface SceneRuntime {
  renderer: THREE.WebGLRenderer;
  scene: THREE.Scene;
  camera: THREE.PerspectiveCamera;
  mesh: THREE.Mesh<THREE.BoxGeometry, THREE.MeshStandardMaterial>;
  grid: THREE.GridHelper;
  controller: InteractionController;
  raycaster: THREE.Raycaster;
  dragPlane: THREE.Plane;
  resizeObserver: ResizeObserver;
}

const colorByStatus: Record<EngineeringObjectState, { color: number; emissive: number }> = {
  ready: { color: 0x164959, emissive: 0x06242f },
  targeted: { color: 0x27cce5, emissive: 0x0b5968 },
  grabbed: { color: 0x4bdb8f, emissive: 0x0b5030 },
};

export function EngineeringScene({ hands, mode, calibration, resetSignal, onObjectChange, onRendererReady }: EngineeringSceneProps) {
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
      const scene = new THREE.Scene();
      scene.fog = new THREE.FogExp2(0x020a0f, 0.055);
      const camera = new THREE.PerspectiveCamera(43, 1, 0.1, 100);
      camera.position.set(0, 3.5, 7);
      camera.lookAt(0, 0.7, 0);

      const grid = new THREE.GridHelper(16, 24, 0x1f9db1, 0x0b3944);
      scene.add(grid);
      scene.add(new THREE.HemisphereLight(0x7defff, 0x061017, 2.1));
      const keyLight = new THREE.DirectionalLight(0x46e9ff, 3.2);
      keyLight.position.set(3, 6, 5);
      scene.add(keyLight);
      const rimLight = new THREE.PointLight(0x4bdb8f, 16, 12);
      rimLight.position.set(-3, 2, 3);
      scene.add(rimLight);

      const geometry = new THREE.BoxGeometry(1.15, 1.15, 1.15, 2, 2, 2);
      const material = new THREE.MeshStandardMaterial({ color: 0x164959, emissive: 0x06242f, roughness: 0.42, metalness: 0.72, wireframe: true });
      const mesh = new THREE.Mesh(geometry, material);
      mesh.position.set(0, 0.65, 0);
      scene.add(mesh);

      const render = () => renderer.render(scene, camera);
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
        renderer, scene, camera, mesh, grid,
        controller: new InteractionController(),
        raycaster: new THREE.Raycaster(),
        dragPlane: new THREE.Plane(new THREE.Vector3(0, 0, 1), 0),
        resizeObserver,
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
      runtime.mesh.geometry.dispose();
      runtime.mesh.material.dispose();
      runtime.grid.geometry.dispose();
      const gridMaterials = Array.isArray(runtime.grid.material) ? runtime.grid.material : [runtime.grid.material];
      gridMaterials.forEach((material) => material.dispose());
      runtime.renderer.dispose();
      runtimeRef.current = null;
      onRendererReady(false);
    };
  }, [onRendererReady]);

  useEffect(() => {
    const runtime = runtimeRef.current;
    if (!runtime) return;
    runtime.controller.updateSettings({
      rotationSensitivity: calibration.rotationSensitivity,
      minScale: calibration.minScale,
      maxScale: calibration.maxScale,
    });
    const interactionHands: HandInteractionPoint[] = hands.map((hand) => {
      const pointer = new THREE.Vector2(hand.cursor.ndcX, hand.cursor.ndcY);
      runtime.raycaster.setFromCamera(pointer, runtime.camera);
      const hovered = (hand.gesture === "point" || hand.gesture === "pinch") && runtime.raycaster.intersectObject(runtime.mesh, false).length > 0;
      const intersection = new THREE.Vector3();
      const world = runtime.raycaster.ray.intersectPlane(runtime.dragPlane, intersection)
        ? { x: intersection.x, y: intersection.y, z: 0 }
        : null;
      return { id: hand.id, gesture: hand.gesture, viewport: hand.cursor, world, hovered };
    });

    const snapshot = runtime.controller.update({ mode, hands: interactionHands });
    runtime.mesh.position.set(snapshot.position.x, snapshot.position.y, snapshot.position.z);
    runtime.mesh.rotation.set(snapshot.rotation.x, snapshot.rotation.y, snapshot.rotation.z);
    runtime.mesh.scale.setScalar(snapshot.scale);
    const colors = colorByStatus[snapshot.status];
    runtime.mesh.material.color.setHex(colors.color);
    runtime.mesh.material.emissive.setHex(colors.emissive);
    runtime.renderer.render(runtime.scene, runtime.camera);
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
    runtime.mesh.position.set(snapshot.position.x, snapshot.position.y, snapshot.position.z);
    runtime.mesh.rotation.set(snapshot.rotation.x, snapshot.rotation.y, snapshot.rotation.z);
    runtime.mesh.scale.setScalar(snapshot.scale);
    const colors = colorByStatus.ready;
    runtime.mesh.material.color.setHex(colors.color);
    runtime.mesh.material.emissive.setHex(colors.emissive);
    runtime.renderer.render(runtime.scene, runtime.camera);
    lastSnapshotRef.current = JSON.stringify(snapshot);
    onObjectChange(snapshot);
  }, [resetSignal, onObjectChange]);

  return <canvas ref={canvasRef} className="engineering-scene" aria-label="Cena 3D do objeto TEST-01" />;
}
