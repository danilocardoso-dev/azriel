export type EngineeringCoreState = "offline" | "requesting_camera" | "tracking" | "ready" | "error";
export type CameraState = "offline" | "requesting" | "online" | "error";
export type HandState = "not_detected" | "detected";
export type GestureState = "none" | "pinch";
export type EngineeringObjectState = "ready" | "targeted" | "grabbed";

export interface HandLandmark {
  x: number;
  y: number;
  z: number;
}

export interface TrackedHand {
  landmarks: HandLandmark[];
  handedness?: "left" | "right";
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
}

export interface TrackingFrame {
  hand: TrackedHand | null;
  fps: number;
}
