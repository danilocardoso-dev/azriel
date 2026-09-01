export type EngineeringCoreState = "offline" | "requesting_camera" | "tracking" | "ready" | "error";
export type CameraState = "offline" | "requesting" | "online" | "error";
export type HandState = "not_detected" | "detected";
export type HandSide = "left" | "right";
export type GestureState = "none" | "pinch" | "point" | "open_hand";
export type ManipulationMode = "move" | "rotate" | "scale";
export type ObjectControlState = "none" | "translate" | "rotate" | "scale";
export type EngineeringObjectState = "ready" | "targeted" | "grabbed";

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

export type EngineeringCalibrationInput = Omit<EngineeringCalibration, "updatedAt">;
