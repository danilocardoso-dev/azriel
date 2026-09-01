import { ENGINEERING_CONFIG, HAND_LANDMARK_INDEX } from "./config";
import { clamp, pointDistance } from "./trackingMath";
import type { EngineeringCalibration, EngineeringCalibrationInput, TrackedHand } from "./types";

export type CalibrationState = "idle" | "collecting" | "saved" | "error";

export class CalibrationSession {
  private handDistances: number[] = [];

  addSample(hands: TrackedHand[]): boolean {
    const left = hands.find((hand) => hand.id === "left");
    const right = hands.find((hand) => hand.id === "right");
    if (!left || !right) return false;
    const leftIndex = left.landmarks[HAND_LANDMARK_INDEX.indexTip];
    const rightIndex = right.landmarks[HAND_LANDMARK_INDEX.indexTip];
    this.handDistances.push(pointDistance(leftIndex, rightIndex));
    return true;
  }

  sampleCount(): number {
    return this.handDistances.length;
  }

  complete(base: EngineeringCalibration): EngineeringCalibrationInput {
    if (this.handDistances.length < 12) throw new Error("Mantenha as duas mãos visíveis durante toda a calibração.");
    const sorted = [...this.handDistances].sort((a, b) => a - b);
    const middle = Math.floor(sorted.length / 2);
    const median = sorted.length % 2 ? sorted[middle] : (sorted[middle - 1] + sorted[middle]) / 2;
    const mean = this.handDistances.reduce((total, value) => total + value, 0) / this.handDistances.length;
    const deviation = Math.sqrt(this.handDistances.reduce((total, value) => total + (value - mean) ** 2, 0) / this.handDistances.length);
    return {
      pinchStartThreshold: base.pinchStartThreshold,
      pinchReleaseThreshold: base.pinchReleaseThreshold,
      smoothingAlpha: clamp(0.5 - deviation * 4, 0.2, 0.5),
      rotationSensitivity: base.rotationSensitivity,
      minScale: base.minScale,
      maxScale: base.maxScale,
      comfortableHandDistance: clamp(median, 0.05, 1),
      calibrated: true,
    };
  }
}

export function defaultCalibration(): EngineeringCalibration {
  return {
    pinchStartThreshold: ENGINEERING_CONFIG.pinchStartThreshold,
    pinchReleaseThreshold: ENGINEERING_CONFIG.pinchReleaseThreshold,
    smoothingAlpha: ENGINEERING_CONFIG.smoothingAlpha,
    rotationSensitivity: ENGINEERING_CONFIG.rotationSensitivity,
    minScale: ENGINEERING_CONFIG.minimumScale,
    maxScale: ENGINEERING_CONFIG.maximumScale,
    comfortableHandDistance: ENGINEERING_CONFIG.comfortableHandDistance,
    calibrated: false,
    updatedAt: "",
  };
}
