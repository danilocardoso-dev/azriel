import { ENGINEERING_CONFIG, HAND_LANDMARK_INDEX } from "./config";
import { landmarkDistance } from "./trackingMath";
import type { GestureState, HandLandmark } from "./types";

export interface GestureResult {
  state: GestureState;
  distance: number | null;
}

export function evaluatePinch(
  landmarks: HandLandmark[] | null,
  previous: GestureState,
  startThreshold = ENGINEERING_CONFIG.pinchStartThreshold,
  releaseThreshold = ENGINEERING_CONFIG.pinchReleaseThreshold,
): GestureResult {
  if (!landmarks || landmarks.length <= HAND_LANDMARK_INDEX.indexTip) return { state: "none", distance: null };
  if (releaseThreshold <= startThreshold) throw new Error("O threshold de release deve ser maior que o threshold de pinch.");

  const distance = landmarkDistance(landmarks[HAND_LANDMARK_INDEX.thumbTip], landmarks[HAND_LANDMARK_INDEX.indexTip]);
  if (previous === "pinch") return { state: distance >= releaseThreshold ? "none" : "pinch", distance };
  return { state: distance <= startThreshold ? "pinch" : "none", distance };
}
