import { clamp, pointDistance, smoothNumber } from "./trackingMath";
import type { HandSide, ViewportPoint } from "./types";

interface ExplosionHandPoint {
  id: HandSide;
  viewport: ViewportPoint;
}

export interface ExplosionGestureResult {
  factor: number;
  active: boolean;
  cancelled: boolean;
}

export class ExplosionGestureController {
  private session: { initialDistance: number; initialFactor: number } | null = null;

  update(hands: ExplosionHandPoint[], currentFactor: number, smoothingAlpha = 0.3): ExplosionGestureResult {
    const left = hands.find((hand) => hand.id === "left");
    const right = hands.find((hand) => hand.id === "right");
    if (!left || !right) {
      const cancelled = Boolean(this.session);
      this.session = null;
      return { factor: currentFactor, active: false, cancelled };
    }
    const distance = pointDistance(left.viewport, right.viewport);
    if (!this.session) {
      this.session = { initialDistance: distance, initialFactor: currentFactor };
      return { factor: currentFactor, active: true, cancelled: false };
    }
    const target = clamp(this.session.initialFactor + (distance - this.session.initialDistance) * 1.8, 0, 1);
    return { factor: smoothNumber(currentFactor, target, smoothingAlpha), active: true, cancelled: false };
  }

  cancel(currentFactor: number): ExplosionGestureResult {
    const cancelled = Boolean(this.session);
    this.session = null;
    return { factor: currentFactor, active: false, cancelled };
  }
}
