import type { HandLandmark, NormalizedPoint, ViewportPoint } from "./types";

export function landmarkDistance(first: HandLandmark, second: HandLandmark): number {
  return Math.hypot(first.x - second.x, first.y - second.y, first.z - second.z);
}

export function smoothLandmark(previous: HandLandmark | null, next: HandLandmark, alpha: number): HandLandmark {
  if (!previous) return { ...next };
  const factor = Math.min(1, Math.max(0, alpha));
  return {
    x: previous.x + (next.x - previous.x) * factor,
    y: previous.y + (next.y - previous.y) * factor,
    z: previous.z + (next.z - previous.z) * factor,
  };
}

export function mirrorNormalizedPoint(point: NormalizedPoint): NormalizedPoint {
  return { x: 1 - point.x, y: point.y };
}

export function normalizedToViewport(point: NormalizedPoint, mirrored = true): ViewportPoint {
  const normalized = mirrored ? mirrorNormalizedPoint(point) : point;
  return {
    x: normalized.x,
    y: normalized.y,
    ndcX: normalized.x * 2 - 1,
    ndcY: 1 - normalized.y * 2,
  };
}
