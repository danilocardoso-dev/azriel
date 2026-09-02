export type EngineeringCoreState = "offline" | "requesting_camera" | "tracking" | "ready" | "error";
export type CameraState = "offline" | "requesting" | "online" | "error";
export type HandState = "not_detected" | "detected";
export type HandSide = "left" | "right";
export type GestureState = "none" | "pinch" | "point" | "open_hand";
export type ManipulationMode = "move" | "rotate" | "scale";
export type EngineeringControlMode = ManipulationMode | "explode";
export type ObjectControlState = "none" | "translate" | "rotate" | "scale";
export type EngineeringObjectState = "ready" | "targeted" | "grabbed";
export type ModelCoreState = "empty" | "loading" | "ready" | "error";
export type ModelFormat = "GLB" | "GLTF";
export type EngineeringInteractionScope = "model" | "component";
export type ComponentVisualState = "normal" | "targeted" | "selected" | "hidden" | "isolated";
export type ExplosionMode = "all" | "selected";
export type AssemblyState = "assembled" | "exploding" | "exploded" | "reassembling" | "partial";

export interface HandLandmark {
  x: number;
  y: number;
  z: number;
}

export interface TrackedHand {
  id: HandSide;
  landmarks: HandLandmark[];
  handedness: HandSide;
  confidence?: number;
}

export interface NormalizedPoint {
  x: number;
  y: number;
}

export interface ViewportPoint extends NormalizedPoint {
  ndcX: number;
  ndcY: number;
}

export interface ScenePoint {
  x: number;
  y: number;
  z: number;
}

export interface EngineeringObjectSnapshot {
  status: EngineeringObjectState;
  position: ScenePoint;
  rotation: ScenePoint;
  scale: number;
  control: ObjectControlState;
}

export interface TrackingFrame {
  hands: TrackedHand[];
  fps: number;
}

export interface HandInteractionPoint {
  id: HandSide;
  gesture: GestureState;
  viewport: ViewportPoint;
  world: ScenePoint | null;
  hovered: boolean;
}

export interface EngineeringCalibration {
  pinchStartThreshold: number;
  pinchReleaseThreshold: number;
  smoothingAlpha: number;
  rotationSensitivity: number;
  minScale: number;
  maxScale: number;
  comfortableHandDistance: number;
  calibrated: boolean;
  updatedAt: string;
}

export interface ModelDimensions extends ScenePoint {
  largest: number;
}

export interface ModelNode {
  id: string;
  name: string;
  type: string;
  parentId?: string;
  children: string[];
  depth: number;
}

export interface ComponentMaterialInfo {
  name: string;
  type: string;
  color?: string;
  textured: boolean;
}

export interface ModelComponent {
  id: string;
  name: string;
  semanticLabel?: string;
  type: string;
  parentId?: string;
  children: string[];
  depth: number;
  visible: boolean;
  selectable: boolean;
  meshCount: number;
  vertices: number;
  triangles: number;
  originalPosition: ScenePoint;
  originalRotation: ScenePoint;
  originalScale: ScenePoint;
  position: ScenePoint;
  rotation: ScenePoint;
  scale: ScenePoint;
  worldPosition: ScenePoint;
  center: ScenePoint;
  directionFromModelCenter: ScenePoint;
  dimensions: ModelDimensions;
  materials: ComponentMaterialInfo[];
}

export interface ComponentTransformSnapshot {
  componentId: string;
  status: EngineeringObjectState;
  control: ObjectControlState;
  position: ScenePoint;
  rotation: ScenePoint;
  scale: ScenePoint;
}

export interface ExplosionMetadata {
  componentId: string;
  originalPosition: ScenePoint;
  worldCenter: ScenePoint;
  direction: ScenePoint;
  localOffset: ScenePoint;
  originalCenterInParent: ScenePoint;
  distanceMultiplier: number;
  depth: number;
}

export interface ExplosionState {
  enabled: boolean;
  factor: number;
  mode: ExplosionMode;
  selectedRootId?: string;
  assemblyState: AssemblyState;
}

export interface ExplosionGuideLine {
  componentId: string;
  parentComponentId?: string;
  from: ScenePoint;
  to: ScenePoint;
}

export interface ModelMetadata {
  name: string;
  format: ModelFormat;
  objects: number;
  groups: number;
  meshes: number;
  materials: number;
  vertices: number;
  triangles: number;
  dimensions: ModelDimensions;
  complexity: "normal" | "high";
}

export type EngineeringCalibrationInput = Omit<EngineeringCalibration, "updatedAt">;
