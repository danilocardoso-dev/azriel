import { ENGINEERING_CONFIG, HAND_LANDMARK_INDEX } from "./config";
import { landmarkDistance } from "./trackingMath";
import type { GestureState, HandLandmark } from "./types";

export interface GestureResult {
  state: GestureState;
  pinchDistance: number | null;
  extendedFingers: number;
}

function fingerExtended(landmarks: HandLandmark[], tip: number, pip: number): boolean {
  return landmarks[tip].y < landmarks[pip].y - 0.012;
}

export function evaluateGesture(
  landmarks: HandLandmark[] | null,
  previous: GestureState,
  startThreshold: number = ENGINEERING_CONFIG.pinchStartThreshold,
  releaseThreshold: number = ENGINEERING_CONFIG.pinchReleaseThreshold,
): GestureResult {
  if (!landmarks || landmarks.length < 21) return { state: "none", pinchDistance: null, extendedFingers: 0 };
  if (releaseThreshold <= startThreshold) throw new Error("O threshold de release deve ser maior que o threshold de pinch.");

  const pinchDistance = landmarkDistance(landmarks[HAND_LANDMARK_INDEX.thumbTip], landmarks[HAND_LANDMARK_INDEX.indexTip]);
  if (previous === "pinch" && pinchDistance < releaseThreshold) return { state: "pinch", pinchDistance, extendedFingers: 0 };
  if (pinchDistance <= startThreshold) return { state: "pinch", pinchDistance, extendedFingers: 0 };

  const index = fingerExtended(landmarks, HAND_LANDMARK_INDEX.indexTip, HAND_LANDMARK_INDEX.indexPip);
  const middle = fingerExtended(landmarks, HAND_LANDMARK_INDEX.middleTip, HAND_LANDMARK_INDEX.middlePip);
  const ring = fingerExtended(landmarks, HAND_LANDMARK_INDEX.ringTip, HAND_LANDMARK_INDEX.ringPip);
  const pinky = fingerExtended(landmarks, HAND_LANDMARK_INDEX.pinkyTip, HAND_LANDMARK_INDEX.pinkyPip);
  const extendedFingers = [index, middle, ring, pinky].filter(Boolean).length;
  if (index && [middle, ring, pinky].filter(Boolean).length <= 1) return { state: "point", pinchDistance, extendedFingers };
  if (extendedFingers >= 3) return { state: "open_hand", pinchDistance, extendedFingers };
  return { state: "none", pinchDistance, extendedFingers };
}

export const evaluatePinch = evaluateGesture;
