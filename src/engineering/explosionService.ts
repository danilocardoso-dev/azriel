import * as THREE from "three";
import { clamp } from "./trackingMath";
import type { AssemblyState, ExplosionGuideLine, ExplosionMetadata, ExplosionMode, ExplosionState, ModelComponent, ScenePoint } from "./types";
import { ComponentService } from "./componentService";

const FALLBACK_DIRECTIONS = [
  new THREE.Vector3(1, 0, 0), new THREE.Vector3(-1, 0, 0),
  new THREE.Vector3(0, 1, 0), new THREE.Vector3(0, -1, 0),
  new THREE.Vector3(0, 0, 1), new THREE.Vector3(0, 0, -1),
];

function point(vector: THREE.Vector3): ScenePoint {
  return { x: vector.x, y: vector.y, z: vector.z };
}

function addScaled(origin: ScenePoint, offset: ScenePoint, factor: number): ScenePoint {
  return { x: origin.x + offset.x * factor, y: origin.y + offset.y * factor, z: origin.z + offset.z * factor };
}

export interface ExplosionConfigurationResult {
  success: boolean;
  unitCount: number;
  error?: string;
}

export class ExplosionService {
  private metadata: ExplosionMetadata[] = [];
  private state: ExplosionState = { enabled: false, factor: 0, mode: "all", assemblyState: "assembled" };
  private readonly rootId: string | null;
  private readonly modelSize: number;

  constructor(private readonly components: ComponentService) {
    const roots = components.list().filter((component) => !component.parentId);
    this.rootId = roots[0]?.id ?? null;
    this.modelSize = Math.max(roots[0]?.dimensions.largest ?? 1, 0.001);
    this.configure("all");
  }

  configure(mode: ExplosionMode, selectedRootId?: string): ExplosionConfigurationResult {
    this.restoreUnitPositions();
    this.components.restoreTransforms();
    const rootId = mode === "selected" ? selectedRootId : this.rootId ?? undefined;
    if (!rootId || !this.components.get(rootId)) {
      this.metadata = [];
      this.state = { enabled: false, factor: 0, mode, selectedRootId, assemblyState: "assembled" };
      return { success: false, unitCount: 0, error: mode === "selected" ? "Selecione um grupo para explodir." : "O modelo não possui uma raiz utilizável." };
    }
    const units = this.explosionUnits(rootId);
    if (units.length < 2) {
      this.metadata = [];
      this.state = { enabled: false, factor: 0, mode, selectedRootId: mode === "selected" ? rootId : undefined, assemblyState: "assembled" };
      return { success: false, unitCount: units.length, error: "Sem subcomponentes suficientes para explodir." };
    }
    const root = this.components.get(rootId)!;
    const rootCenter = new THREE.Vector3(root.center.x, root.center.y, root.center.z);
    this.metadata = units.map((component, index) => this.createMetadata(component, rootCenter, index));
    this.state = { enabled: true, factor: 0, mode, selectedRootId: mode === "selected" ? rootId : undefined, assemblyState: "assembled" };
    return { success: true, unitCount: units.length };
  }

  applyFactor(value: number, assemblyState?: AssemblyState): ExplosionState {
    const factor = clamp(value, 0, 1);
    for (const metadata of this.metadata) {
      const object = this.components.getObject(metadata.componentId);
      if (!object) continue;
      const position = addScaled(metadata.originalPosition, metadata.localOffset, factor);
      object.position.set(position.x, position.y, position.z);
      object.updateMatrixWorld(true);
    }
    this.state = {
      ...this.state,
      enabled: this.metadata.length > 1,
      factor,
      assemblyState: assemblyState ?? (factor === 0 ? "assembled" : factor === 1 ? "exploded" : "partial"),
    };
    return this.getState();
  }

  reset(): ExplosionState {
    this.restoreUnitPositions();
    this.state = { enabled: this.metadata.length > 1, factor: 0, mode: "all", assemblyState: "assembled" };
    this.configure("all");
    return this.getState();
  }

  getState(): ExplosionState {
    return { ...this.state };
  }

  getMetadata(): ExplosionMetadata[] {
    return this.metadata.map((metadata) => ({ ...metadata, originalPosition: { ...metadata.originalPosition }, worldCenter: { ...metadata.worldCenter }, direction: { ...metadata.direction }, localOffset: { ...metadata.localOffset }, originalCenterInParent: { ...metadata.originalCenterInParent } }));
  }

  getGuideLines(): ExplosionGuideLine[] {
    return this.metadata.map((metadata) => ({
      componentId: metadata.componentId,
      parentComponentId: this.components.get(metadata.componentId)?.parentId,
      from: { ...metadata.originalCenterInParent },
      to: addScaled(metadata.originalCenterInParent, metadata.localOffset, this.state.factor),
    }));
  }

  getOffset(componentId: string): ScenePoint {
    const ancestors = this.components.ancestors(componentId);
    const metadata = this.metadata.find((candidate) => candidate.componentId === componentId || ancestors.includes(candidate.componentId));
    return metadata ? point(new THREE.Vector3(metadata.localOffset.x, metadata.localOffset.y, metadata.localOffset.z).multiplyScalar(this.state.factor)) : { x: 0, y: 0, z: 0 };
  }

  private explosionUnits(rootId: string): ModelComponent[] {
    let children = this.childrenWithGeometry(rootId);
    while (children.length === 1 && children[0].children.length) children = this.childrenWithGeometry(children[0].id);
    return children;
  }

  private childrenWithGeometry(id: string): ModelComponent[] {
    const component = this.components.get(id);
    return (component?.children ?? []).map((childId) => this.components.get(childId)).filter((child): child is ModelComponent => Boolean(child?.meshCount));
  }

  private createMetadata(component: ModelComponent, rootCenter: THREE.Vector3, index: number): ExplosionMetadata {
    const object = this.components.getObject(component.id)!;
    object.updateWorldMatrix(true, false);
    const center = new THREE.Vector3(component.center.x, component.center.y, component.center.z);
    const direction = center.clone().sub(rootCenter);
    if (direction.lengthSq() < 1e-8) direction.copy(FALLBACK_DIRECTIONS[index % FALLBACK_DIRECTIONS.length]);
    else direction.normalize();
    const distanceMultiplier = 1 + Math.max(0, component.depth - 1) * 0.14 + index * 0.025;
    const worldOffset = direction.clone().multiplyScalar(this.modelSize * 0.55 * distanceMultiplier);
    const parent = object.parent;
    const localCenter = center.clone();
    const localEnd = center.clone().add(worldOffset);
    if (parent) {
      parent.updateWorldMatrix(true, false);
      parent.worldToLocal(localCenter);
      parent.worldToLocal(localEnd);
    }
    return {
      componentId: component.id,
      originalPosition: { ...component.originalPosition },
      worldCenter: { ...component.center },
      direction: point(direction),
      localOffset: point(localEnd.sub(localCenter)),
      originalCenterInParent: point(localCenter),
      distanceMultiplier,
      depth: component.depth,
    };
  }

  private restoreUnitPositions(): void {
    for (const metadata of this.metadata) {
      const object = this.components.getObject(metadata.componentId);
      if (object) object.position.set(metadata.originalPosition.x, metadata.originalPosition.y, metadata.originalPosition.z);
    }
  }
}
