import { clamp, pointDistance, smoothNumber } from "./trackingMath";
import type { ComponentTransformSnapshot, HandInteractionPoint, HandSide, ManipulationMode, ScenePoint } from "./types";

interface ComponentInteractionSettings {
  rotationSensitivity: number;
  minScaleFactor: number;
  maxScaleFactor: number;
  rotationSmoothingAlpha: number;
  scaleSmoothingAlpha: number;
}

interface MoveSession {
  handId: HandSide;
  startPoint: ScenePoint;
  initialPosition: ScenePoint;
}

interface RotationSession {
  handId: HandSide;
  startX: number;
  startY: number;
  initialRotation: ScenePoint;
}

interface ScaleSession {
  initialHandDistance: number;
  initialScale: ScenePoint;
}

const defaults: ComponentInteractionSettings = {
  rotationSensitivity: 3,
  minScaleFactor: 0.2,
  maxScaleFactor: 3,
  rotationSmoothingAlpha: 0.34,
  scaleSmoothingAlpha: 0.28,
};

function copy(point: ScenePoint): ScenePoint {
  return { ...point };
}

export class ComponentInteractionController {
  private snapshot: ComponentTransformSnapshot | null = null;
  private moveSession: MoveSession | null = null;
  private rotationSession: RotationSession | null = null;
  private scaleSession: ScaleSession | null = null;
  private currentMode: ManipulationMode = "move";

  constructor(private settings: ComponentInteractionSettings = defaults) {}

  bind(componentId: string, position: ScenePoint, rotation: ScenePoint, scale: ScenePoint): ComponentTransformSnapshot {
    if (this.snapshot?.componentId === componentId) return this.getSnapshot()!;
    this.cancelSession();
    this.snapshot = { componentId, status: "ready", control: "none", position: copy(position), rotation: copy(rotation), scale: copy(scale) };
    return this.getSnapshot()!;
  }

  update(mode: ManipulationMode, hands: HandInteractionPoint[]): ComponentTransformSnapshot | null {
    if (!this.snapshot) return null;
    if (mode !== this.currentMode) {
      this.cancelSession();
      this.currentMode = mode;
    }
    if (mode === "scale") return this.updateScale(hands);
    this.scaleSession = null;
    const hand = this.primaryHand(hands);
    if (!hand) {
      this.cancelSession();
      return this.getSnapshot();
    }
    return mode === "rotate" ? this.updateRotation(hand, hands) : this.updateMove(hand, hands);
  }

  updateSettings(settings: Partial<ComponentInteractionSettings>): void {
    this.settings = { ...this.settings, ...settings };
  }

  release(): ComponentTransformSnapshot | null {
    this.cancelSession();
    return this.getSnapshot();
  }

  clear(): void {
    this.cancelSession();
    this.snapshot = null;
  }

  getSnapshot(): ComponentTransformSnapshot | null {
    return this.snapshot ? { ...this.snapshot, position: copy(this.snapshot.position), rotation: copy(this.snapshot.rotation), scale: copy(this.snapshot.scale) } : null;
  }

  private updateMove(hand: HandInteractionPoint, hands: HandInteractionPoint[]): ComponentTransformSnapshot {
    if (this.moveSession) {
      const active = hands.find((candidate) => candidate.id === this.moveSession?.handId);
      if (!active || active.gesture !== "pinch" || !active.world) {
        this.cancelSession();
        return this.getSnapshot()!;
      }
      this.snapshot = {
        ...this.snapshot!, status: "grabbed", control: "translate",
        position: {
          x: this.moveSession.initialPosition.x + active.world.x - this.moveSession.startPoint.x,
          y: this.moveSession.initialPosition.y + active.world.y - this.moveSession.startPoint.y,
          z: this.moveSession.initialPosition.z + active.world.z - this.moveSession.startPoint.z,
        },
      };
      return this.getSnapshot()!;
    }
    if (hand.hovered && hand.gesture === "pinch" && hand.world) {
      this.moveSession = { handId: hand.id, startPoint: copy(hand.world), initialPosition: copy(this.snapshot!.position) };
      this.snapshot = { ...this.snapshot!, status: "grabbed", control: "translate" };
    } else this.setTargetState(hands);
    return this.getSnapshot()!;
  }

  private updateRotation(hand: HandInteractionPoint, hands: HandInteractionPoint[]): ComponentTransformSnapshot {
    if (this.rotationSession) {
      const active = hands.find((candidate) => candidate.id === this.rotationSession?.handId);
      if (!active || active.gesture !== "pinch") {
        this.cancelSession();
        return this.getSnapshot()!;
      }
      const targetX = this.rotationSession.initialRotation.x + (active.viewport.y - this.rotationSession.startY) * this.settings.rotationSensitivity;
      const targetY = this.rotationSession.initialRotation.y + (active.viewport.x - this.rotationSession.startX) * this.settings.rotationSensitivity;
      this.snapshot = {
        ...this.snapshot!, status: "grabbed", control: "rotate",
        rotation: {
          x: smoothNumber(this.snapshot!.rotation.x, targetX, this.settings.rotationSmoothingAlpha),
          y: smoothNumber(this.snapshot!.rotation.y, targetY, this.settings.rotationSmoothingAlpha),
          z: this.snapshot!.rotation.z,
        },
      };
      return this.getSnapshot()!;
    }
    if (hand.hovered && hand.gesture === "pinch") {
      this.rotationSession = { handId: hand.id, startX: hand.viewport.x, startY: hand.viewport.y, initialRotation: copy(this.snapshot!.rotation) };
      this.snapshot = { ...this.snapshot!, status: "grabbed", control: "rotate" };
    } else this.setTargetState(hands);
    return this.getSnapshot()!;
  }

  private updateScale(hands: HandInteractionPoint[]): ComponentTransformSnapshot {
    this.moveSession = null;
    this.rotationSession = null;
    const left = hands.find((hand) => hand.id === "left");
    const right = hands.find((hand) => hand.id === "right");
    if (!left || !right) {
      this.cancelSession();
      return this.getSnapshot()!;
    }
    const distance = pointDistance(left.viewport, right.viewport);
    if (!this.scaleSession) {
      if (distance < 0.05) return this.getSnapshot()!;
      this.scaleSession = { initialHandDistance: distance, initialScale: copy(this.snapshot!.scale) };
      this.snapshot = { ...this.snapshot!, status: "grabbed", control: "scale" };
      return this.getSnapshot()!;
    }
    const factor = clamp(distance / this.scaleSession.initialHandDistance, this.settings.minScaleFactor, this.settings.maxScaleFactor);
    const target = {
      x: this.scaleSession.initialScale.x * factor,
      y: this.scaleSession.initialScale.y * factor,
      z: this.scaleSession.initialScale.z * factor,
    };
    this.snapshot = {
      ...this.snapshot!, status: "grabbed", control: "scale",
      scale: {
        x: smoothNumber(this.snapshot!.scale.x, target.x, this.settings.scaleSmoothingAlpha),
        y: smoothNumber(this.snapshot!.scale.y, target.y, this.settings.scaleSmoothingAlpha),
        z: smoothNumber(this.snapshot!.scale.z, target.z, this.settings.scaleSmoothingAlpha),
      },
    };
    return this.getSnapshot()!;
  }

  private primaryHand(hands: HandInteractionPoint[]): HandInteractionPoint | undefined {
    const activeId = this.moveSession?.handId ?? this.rotationSession?.handId;
    return activeId ? hands.find((hand) => hand.id === activeId) : hands.find((hand) => hand.id === "right") ?? hands[0];
  }

  private setTargetState(hands: HandInteractionPoint[]): void {
    this.snapshot = { ...this.snapshot!, status: hands.some((hand) => hand.hovered) ? "targeted" : "ready", control: "none" };
  }

  private cancelSession(): void {
    this.moveSession = null;
    this.rotationSession = null;
    this.scaleSession = null;
    if (this.snapshot) this.snapshot = { ...this.snapshot, status: "ready", control: "none" };
  }
}
