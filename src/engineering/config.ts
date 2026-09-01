export const ENGINEERING_CONFIG = {
  pinchStartThreshold: 0.045,
  pinchReleaseThreshold: 0.065,
  smoothingAlpha: 0.38,
  minimumConfidence: 0.55,
  maximumTrackingFps: 24,
  cameraWidth: 640,
  cameraHeight: 480,
} as const;

export const HAND_LANDMARK_INDEX = {
  wrist: 0,
  thumbTip: 4,
  indexTip: 8,
} as const;

export const HAND_CONNECTIONS: ReadonlyArray<readonly [number, number]> = [
  [0, 1], [1, 2], [2, 3], [3, 4],
  [0, 5], [5, 6], [6, 7], [7, 8],
  [5, 9], [9, 10], [10, 11], [11, 12],
  [9, 13], [13, 14], [14, 15], [15, 16],
  [13, 17], [17, 18], [18, 19], [19, 20], [0, 17],
];
