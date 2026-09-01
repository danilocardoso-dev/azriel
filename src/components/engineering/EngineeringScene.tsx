import { useEffect, useRef } from "react";
import * as THREE from "three";
import { InteractionController } from "../../engineering/interactionController";
import type { EngineeringObjectState, GestureState, ViewportPoint } from "../../engineering/types";

interface EngineeringSceneProps {
  cursor: ViewportPoint | null;
  gesture: GestureState;
  handDetected: boolean;
  resetSignal: number;
  onObjectStatusChange: (status: EngineeringObjectState) => void;
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

export function EngineeringScene({ cursor, gesture, handDetected, resetSignal, onObjectStatusChange, onRendererReady }: EngineeringSceneProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const runtimeRef = useRef<SceneRuntime | null>(null);
  const lastStatusRef = useRef<EngineeringObjectState>("ready");

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
    const pointer = cursor ? new THREE.Vector2(cursor.ndcX, cursor.ndcY) : null;
    let hovered = false;
    let cursorWorld = null;
    if (pointer) {
      runtime.raycaster.setFromCamera(pointer, runtime.camera);
      hovered = runtime.raycaster.intersectObject(runtime.mesh, false).length > 0;
      const intersection = new THREE.Vector3();
      if (runtime.raycaster.ray.intersectPlane(runtime.dragPlane, intersection)) cursorWorld = { x: intersection.x, y: intersection.y, z: 0 };
    }

    const snapshot = runtime.controller.update({ handDetected, gesture, hovered, cursorWorld });
    runtime.mesh.position.set(snapshot.position.x, snapshot.position.y, snapshot.position.z);
    const colors = colorByStatus[snapshot.status];
    runtime.mesh.material.color.setHex(colors.color);
    runtime.mesh.material.emissive.setHex(colors.emissive);
    runtime.renderer.render(runtime.scene, runtime.camera);
    if (lastStatusRef.current !== snapshot.status) {
      lastStatusRef.current = snapshot.status;
      onObjectStatusChange(snapshot.status);
    }
  }, [cursor, gesture, handDetected, onObjectStatusChange]);

  useEffect(() => {
    const runtime = runtimeRef.current;
    if (!runtime) return;
    const snapshot = runtime.controller.reset();
    runtime.mesh.position.set(snapshot.position.x, snapshot.position.y, snapshot.position.z);
    const colors = colorByStatus.ready;
    runtime.mesh.material.color.setHex(colors.color);
    runtime.mesh.material.emissive.setHex(colors.emissive);
    runtime.renderer.render(runtime.scene, runtime.camera);
    lastStatusRef.current = "ready";
    onObjectStatusChange("ready");
  }, [resetSignal, onObjectStatusChange]);

  return <canvas ref={canvasRef} className="engineering-scene" aria-label="Cena 3D do objeto TEST-01" />;
}
