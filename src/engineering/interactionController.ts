import { ENGINEERING_CONFIG } from "./config";
import { clamp, clampScenePoint, pointDistance, smoothNumber } from "./trackingMath";
import type { EngineeringObjectSnapshot, HandInteractionPoint, HandSide, ManipulationMode, ScenePoint } from "./types";

export const INITIAL_OBJECT_POSITION: ScenePoint = { x: 0, y: 0.65, z: 0 };
export const INITIAL_OBJECT_ROTATION: ScenePoint = { x: 0, y: 0, z: 0 };

export interface InteractionSettings {
  rotationSensitivity: number;
  minScale: number;
  maxScale: number;
  rotationSmoothingAlpha: number;
  scaleSmoothingAlpha: number;
}

export interface InteractionInput {
  mode: ManipulationMode;
  hands: HandInteractionPoint[];
}

interface RotationSession {
  handId: HandSide;
  startX: number;
  startY: number;
  initialRotation: ScenePoint;
}

interface ScaleSession {
  initialHandDistance: number;
  initialObjectScale: number;
}

const defaultSettings: InteractionSettings = {
  rotationSensitivity: ENGINEERING_CONFIG.rotationSensitivity,
  minScale: ENGINEERING_CONFIG.minimumScale,
  maxScale: ENGINEERING_CONFIG.maximumScale,
  rotationSmoothingAlpha: ENGINEERING_CONFIG.rotationSmoothingAlpha,
  scaleSmoothingAlpha: ENGINEERING_CONFIG.scaleSmoothingAlpha,
};

export class InteractionController {
  private snapshot: EngineeringObjectSnapshot = this.initialSnapshot();
  private activeHand: HandSide | null = null;
  private rotationSession: RotationSession | null = null;
  private scaleSession: ScaleSession | null = null;
  private currentMode: ManipulationMode = "move";

  constructor(private settings: InteractionSettings = defaultSettings) {}

  update(input: InteractionInput): EngineeringObjectSnapshot {
    if (input.mode !== this.currentMode) {
      this.cancelSession();
      this.currentMode = input.mode;
    }
    if (input.mode === "scale") return this.updateScale(input.hands);

    this.scaleSession = null;
    const hand = this.selectPrimaryHand(input.hands);
    if (!hand) {
      this.cancelSession();
      return this.getSnapshot();
    }
    return input.mode === "rotate" ? this.updateRotation(hand, input.hands) : this.updateMove(hand, input.hands);
  }

  updateSettings(settings: Partial<InteractionSettings>): void {
    this.settings = { ...this.settings, ...settings };
    this.snapshot = { ...this.snapshot, scale: clamp(this.snapshot.scale, this.settings.minScale, this.settings.maxScale) };
  }

  reset(): EngineeringObjectSnapshot {
    this.cancelSession();
    this.snapshot = this.initialSnapshot();
    return this.getSnapshot();
  }

  restore(snapshot: EngineeringObjectSnapshot): EngineeringObjectSnapshot {
    this.cancelSession();
    this.snapshot = {
      ...snapshot,
      status: "ready",
      control: "none",
      position: { ...snapshot.position },
      rotation: { ...snapshot.rotation },
      scale: clamp(snapshot.scale, this.settings.minScale, this.settings.maxScale),
    };
    return this.getSnapshot();
  }

  getSnapshot(): EngineeringObjectSnapshot {
    return { ...this.snapshot, position: { ...this.snapshot.position }, rotation: { ...this.snapshot.rotation } };
  }

  private updateMove(hand: HandInteractionPoint, hands: HandInteractionPoint[]): EngineeringObjectSnapshot {
    if (this.activeHand) {
      const active = hands.find((candidate) => candidate.id === this.activeHand);
      if (active?.gesture === "pinch" && active.world) {
        this.snapshot = { ...this.snapshot, status: "grabbed", control: "translate", position: clampScenePoint(active.world, ENGINEERING_CONFIG.workspaceBounds) };
      } else {
        this.cancelSession();
      }
      return this.getSnapshot();
    }

    if (hand.hovered && hand.gesture === "pinch" && hand.world) {
      this.activeHand = hand.id;
      this.snapshot = { ...this.snapshot, status: "grabbed", control: "translate", position: clampScenePoint(hand.world, ENGINEERING_CONFIG.workspaceBounds) };
    } else {
      this.snapshot = { ...this.snapshot, status: hands.some((candidate) => candidate.hovered) ? "targeted" : "ready", control: "none" };
    }
    return this.getSnapshot();
  }

  private updateRotation(hand: HandInteractionPoint, hands: HandInteractionPoint[]): EngineeringObjectSnapshot {
    if (this.rotationSession) {
      const active = hands.find((candidate) => candidate.id === this.rotationSession?.handId);
      if (!active || active.gesture !== "pinch") {
        this.cancelSession();
        return this.getSnapshot();
      }
      const targetX = this.rotationSession.initialRotation.x + (active.viewport.y - this.rotationSession.startY) * this.settings.rotationSensitivity;
      const targetY = this.rotationSession.initialRotation.y + (active.viewport.x - this.rotationSession.startX) * this.settings.rotationSensitivity;
      this.snapshot = {
        ...this.snapshot,
        status: "grabbed",
        control: "rotate",
        rotation: {
          x: smoothNumber(this.snapshot.rotation.x, targetX, this.settings.rotationSmoothingAlpha),
          y: smoothNumber(this.snapshot.rotation.y, targetY, this.settings.rotationSmoothingAlpha),
          z: this.snapshot.rotation.z,
        },
      };
      return this.getSnapshot();
    }

    if (hand.hovered && hand.gesture === "pinch") {
      this.activeHand = hand.id;
      this.rotationSession = { handId: hand.id, startX: hand.viewport.x, startY: hand.viewport.y, initialRotation: { ...this.snapshot.rotation } };
      this.snapshot = { ...this.snapshot, status: "grabbed", control: "rotate" };
    } else {
      this.snapshot = { ...this.snapshot, status: hands.some((candidate) => candidate.hovered) ? "targeted" : "ready", control: "none" };
    }
    return this.getSnapshot();
  }

  private updateScale(hands: HandInteractionPoint[]): EngineeringObjectSnapshot {
    this.activeHand = null;
    this.rotationSession = null;
    const left = hands.find((hand) => hand.id === "left");
    const right = hands.find((hand) => hand.id === "right");
    if (!left || !right) {
      this.scaleSession = null;
      this.snapshot = { ...this.snapshot, status: "ready", control: "none" };
      return this.getSnapshot();
    }

    const distance = pointDistance(left.viewport, right.viewport);
    if (!this.scaleSession) {
      if (distance < 0.05) return this.getSnapshot();
      this.scaleSession = { initialHandDistance: distance, initialObjectScale: this.snapshot.scale };
      this.snapshot = { ...this.snapshot, status: "grabbed", control: "scale" };
      return this.getSnapshot();
    }

    const target = clamp(this.scaleSession.initialObjectScale * (distance / this.scaleSession.initialHandDistance), this.settings.minScale, this.settings.maxScale);
    this.snapshot = { ...this.snapshot, status: "grabbed", control: "scale", scale: smoothNumber(this.snapshot.scale, target, this.settings.scaleSmoothingAlpha) };
    return this.getSnapshot();
  }

  private selectPrimaryHand(hands: HandInteractionPoint[]): HandInteractionPoint | undefined {
    if (this.activeHand) return hands.find((hand) => hand.id === this.activeHand);
    return hands.find((hand) => hand.id === "right") ?? hands[0];
  }

  private cancelSession(): void {
    this.activeHand = null;
    this.rotationSession = null;
    this.scaleSession = null;
    this.snapshot = { ...this.snapshot, status: "ready", control: "none" };
  }

  private initialSnapshot(): EngineeringObjectSnapshot {
    return { status: "ready", position: { ...INITIAL_OBJECT_POSITION }, rotation: { ...INITIAL_OBJECT_ROTATION }, scale: 1, control: "none" };
  }
}
