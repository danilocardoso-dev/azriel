import * as THREE from "three";
import type { ComponentMaterialInfo, ComponentVisualState, ModelComponent, ModelDimensions, ScenePoint } from "./types";
import { dimensionsFromBox } from "./modelMath";

function scenePoint(vector: THREE.Vector3): ScenePoint {
  return { x: vector.x, y: vector.y, z: vector.z };
}

function fallbackName(object: THREE.Object3D, index: number): string {
  const prefix = object instanceof THREE.Mesh ? "Mesh" : object instanceof THREE.Group ? "Group" : object.type || "Object";
  return `${prefix}_${index.toString().padStart(3, "0")}`;
}

function materialInfo(material: THREE.Material): ComponentMaterialInfo {
  const candidate = material as THREE.Material & { color?: THREE.Color };
  return {
    name: material.name.trim() || material.type,
    type: material.type,
    color: candidate.color ? `#${candidate.color.getHexString().toUpperCase()}` : undefined,
    textured: Object.values(material).some((value) => value instanceof THREE.Texture),
  };
}

function geometryStats(root: THREE.Object3D) {
  let meshCount = 0;
  let vertices = 0;
  let triangles = 0;
  const materials = new Map<string, ComponentMaterialInfo>();
  root.traverse((object) => {
    if (!(object instanceof THREE.Mesh)) return;
    meshCount += 1;
    const position = object.geometry.getAttribute("position");
    vertices += position?.count ?? 0;
    triangles += object.geometry.index ? object.geometry.index.count / 3 : (position?.count ?? 0) / 3;
    const meshMaterials = Array.isArray(object.material) ? object.material : [object.material];
    meshMaterials.forEach((material) => materials.set(material.uuid, materialInfo(material)));
  });
  return { meshCount, vertices, triangles: Math.floor(triangles), materials: [...materials.values()] };
}

export class ComponentService {
  private readonly components = new Map<string, ModelComponent>();
  private readonly objects = new Map<string, THREE.Object3D>();
  private readonly idsByUuid = new Map<string, string>();
  private readonly originalVisibility = new Map<string, boolean>();
  private isolationVisibility: Map<string, boolean> | null = null;
  private isolatedComponentId: string | null = null;
  readonly modelCenter: ScenePoint;

  constructor(sceneRoot: THREE.Object3D) {
    sceneRoot.updateMatrixWorld(true);
    const modelBox = new THREE.Box3().setFromObject(sceneRoot);
    const modelCenter = modelBox.isEmpty() ? new THREE.Vector3() : modelBox.getCenter(new THREE.Vector3());
    this.modelCenter = scenePoint(modelCenter);
    this.mapScene(sceneRoot, undefined, 0, modelCenter);
  }

  list(): ModelComponent[] {
    return [...this.components.values()].map((component) => ({ ...component, visible: this.isEffectivelyVisible(component.id) }));
  }

  get(id: string | null | undefined): ModelComponent | null {
    if (!id) return null;
    const component = this.components.get(id);
    return component ? { ...component, visible: this.isEffectivelyVisible(id) } : null;
  }

  getObject(id: string | null | undefined): THREE.Object3D | null {
    return id ? this.objects.get(id) ?? null : null;
  }

  getRaycastMeshes(): THREE.Mesh[] {
    const meshes: THREE.Mesh[] = [];
    for (const [id, object] of this.objects) {
      if (object instanceof THREE.Mesh && this.isEffectivelyVisible(id)) meshes.push(object);
    }
    return meshes;
  }

  resolveObject(object: THREE.Object3D | null): string | null {
    let current = object;
    while (current) {
      const id = this.idsByUuid.get(current.uuid);
      if (id) return id;
      current = current.parent;
    }
    return null;
  }

  search(query: string): ModelComponent[] {
    const needle = query.trim().toLocaleLowerCase("pt-BR");
    if (!needle) return this.list();
    return this.list().filter((component) => component.name.toLocaleLowerCase("pt-BR").includes(needle));
  }

  ancestors(id: string): string[] {
    const result: string[] = [];
    let parentId = this.components.get(id)?.parentId;
    while (parentId) {
      result.unshift(parentId);
      parentId = this.components.get(parentId)?.parentId;
    }
    return result;
  }

  state(id: string, targetedId: string | null, selectedId: string | null): ComponentVisualState {
    if (!this.isEffectivelyVisible(id)) return "hidden";
    if (this.isolatedComponentId === id) return "isolated";
    if (selectedId === id) return "selected";
    if (targetedId === id) return "targeted";
    return "normal";
  }

  hide(id: string): void {
    if (this.isolationVisibility) this.exitIsolation();
    const object = this.objects.get(id);
    if (object) object.visible = false;
  }

  show(id: string): void {
    if (this.isolationVisibility) this.exitIsolation();
    const object = this.objects.get(id);
    if (object) object.visible = true;
  }

  isolate(id: string): void {
    if (!this.objects.has(id)) return;
    if (this.isolationVisibility) this.exitIsolation();
    this.isolationVisibility = new Map([...this.objects].map(([componentId, object]) => [componentId, object.visible]));
    const ancestorIds = this.ancestors(id);
    const visibleIds = new Set([...ancestorIds, id, ...this.descendants(id)]);
    for (const [componentId, object] of this.objects) {
      object.visible = visibleIds.has(componentId)
        ? (componentId === id || ancestorIds.includes(componentId) || (this.isolationVisibility.get(componentId) ?? true))
        : false;
    }
    this.isolatedComponentId = id;
  }

  exitIsolation(): void {
    if (!this.isolationVisibility) return;
    for (const [id, visible] of this.isolationVisibility) {
      const object = this.objects.get(id);
      if (object) object.visible = visible;
    }
    this.isolationVisibility = null;
    this.isolatedComponentId = null;
  }

  restore(): void {
    this.isolationVisibility = null;
    this.isolatedComponentId = null;
    for (const [id, visible] of this.originalVisibility) {
      const object = this.objects.get(id);
      if (object) object.visible = visible;
    }
  }

  getIsolationId(): string | null {
    return this.isolatedComponentId;
  }

  private mapScene(object: THREE.Object3D, parentId: string | undefined, depth: number, modelCenter: THREE.Vector3): string {
    const index = this.components.size + 1;
    const id = `component-${(index - 1).toString().padStart(4, "0")}`;
    const children: string[] = [];
    object.updateWorldMatrix(true, false);
    const box = new THREE.Box3().setFromObject(object);
    const center = box.isEmpty() ? object.getWorldPosition(new THREE.Vector3()) : box.getCenter(new THREE.Vector3());
    const direction = center.clone().sub(modelCenter);
    if (direction.lengthSq() > 0) direction.normalize();
    const stats = geometryStats(object);
    const dimensions: ModelDimensions = dimensionsFromBox(box);
    const component: ModelComponent = {
      id,
      name: object.name.trim() || fallbackName(object, index),
      type: object.type || "Object3D",
      parentId,
      children,
      depth,
      visible: object.visible,
      selectable: stats.meshCount > 0,
      meshCount: stats.meshCount,
      vertices: stats.vertices,
      triangles: stats.triangles,
      originalPosition: scenePoint(object.position),
      originalRotation: { x: object.rotation.x, y: object.rotation.y, z: object.rotation.z },
      originalScale: scenePoint(object.scale),
      worldPosition: scenePoint(object.getWorldPosition(new THREE.Vector3())),
      center: scenePoint(center),
      directionFromModelCenter: scenePoint(direction),
      dimensions,
      materials: stats.materials,
    };
    this.components.set(id, component);
    this.objects.set(id, object);
    this.idsByUuid.set(object.uuid, id);
    this.originalVisibility.set(id, object.visible);
    for (const child of object.children) children.push(this.mapScene(child, id, depth + 1, modelCenter));
    return id;
  }

  private descendants(id: string): string[] {
    const result: string[] = [];
    const visit = (componentId: string) => {
      for (const childId of this.components.get(componentId)?.children ?? []) {
        result.push(childId);
        visit(childId);
      }
    };
    visit(id);
    return result;
  }

  private isEffectivelyVisible(id: string): boolean {
    let currentId: string | undefined = id;
    while (currentId) {
      const object = this.objects.get(currentId);
      if (!object?.visible) return false;
      currentId = this.components.get(currentId)?.parentId;
    }
    return true;
  }
}
