import * as THREE from "three";
import type { ModelDimensions, ModelMetadata, ModelNode } from "./types";

export const MODEL_DISPLAY_SIZE = 2.4;
export const HIGH_COMPLEXITY_TRIANGLES = 500_000;

export interface ModelAnalysis {
  metadata: Omit<ModelMetadata, "name" | "format">;
  nodes: ModelNode[];
  center: THREE.Vector3;
  normalizationScale: number;
}

export function dimensionsFromBox(box: THREE.Box3): ModelDimensions {
  if (box.isEmpty()) return { x: 0, y: 0, z: 0, largest: 0 };
  const size = box.getSize(new THREE.Vector3());
  return { x: size.x, y: size.y, z: size.z, largest: Math.max(size.x, size.y, size.z) };
}

export function normalizationScale(dimensions: ModelDimensions, targetSize = MODEL_DISPLAY_SIZE): number {
  return dimensions.largest > 0 && Number.isFinite(dimensions.largest) ? targetSize / dimensions.largest : 1;
}

export function buildModelHierarchy(root: THREE.Object3D): ModelNode[] {
  const nodes: ModelNode[] = [];
  let sequence = 0;
  const visit = (object: THREE.Object3D, parentId: string | undefined, depth: number) => {
    const id = `node-${sequence.toString().padStart(4, "0")}`;
    sequence += 1;
    const type = object.type || "Object3D";
    const name = object.name.trim() || `${type.toUpperCase()}-${sequence.toString().padStart(3, "0")}`;
    const node: ModelNode = { id, name, type, parentId, children: [], depth };
    nodes.push(node);
    for (const child of object.children) {
      const childId = `node-${sequence.toString().padStart(4, "0")}`;
      node.children.push(childId);
      visit(child, id, depth + 1);
    }
  };
  visit(root, undefined, 0);
  return nodes;
}

export function analyzeModel(root: THREE.Object3D): ModelAnalysis {
  root.updateMatrixWorld(true);
  const box = new THREE.Box3().setFromObject(root);
  const dimensions = dimensionsFromBox(box);
  const center = box.isEmpty() ? new THREE.Vector3() : box.getCenter(new THREE.Vector3());
  const materialIds = new Set<string>();
  let objects = 0;
  let groups = 0;
  let meshes = 0;
  let vertices = 0;
  let triangles = 0;

  root.traverse((object) => {
    objects += 1;
    if (object instanceof THREE.Group) groups += 1;
    if (!(object instanceof THREE.Mesh)) return;
    meshes += 1;
    const geometry = object.geometry;
    const position = geometry.getAttribute("position");
    if (position) vertices += position.count;
    triangles += geometry.index ? geometry.index.count / 3 : (position?.count ?? 0) / 3;
    const materials = Array.isArray(object.material) ? object.material : [object.material];
    materials.forEach((material) => materialIds.add(material.uuid));
  });

  return {
    metadata: {
      objects,
      groups,
      meshes,
      materials: materialIds.size,
      vertices,
      triangles: Math.floor(triangles),
      dimensions,
      complexity: triangles >= HIGH_COMPLEXITY_TRIANGLES ? "high" : "normal",
    },
    nodes: buildModelHierarchy(root),
    center,
    normalizationScale: normalizationScale(dimensions),
  };
}

export function prepareModelRoot(source: THREE.Object3D, center: THREE.Vector3, scale: number): THREE.Group {
  const normalized = new THREE.Group();
  normalized.name = "ModelNormalization";
  const centered = new THREE.Group();
  centered.name = "ModelCentering";
  centered.position.copy(center).multiplyScalar(-1);
  centered.add(source);
  normalized.scale.setScalar(scale);
  normalized.add(centered);
  normalized.updateMatrixWorld(true);
  return normalized;
}
