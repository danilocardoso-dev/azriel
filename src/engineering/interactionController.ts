import type { EngineeringObjectSnapshot, GestureState, ScenePoint } from "./types";

export const INITIAL_OBJECT_POSITION: ScenePoint = { x: 0, y: 0.65, z: 0 };

export interface InteractionInput {
  handDetected: boolean;
  gesture: GestureState;
  hovered: boolean;
  cursorWorld: ScenePoint | null;
}

export class InteractionController {
  private snapshot: EngineeringObjectSnapshot = { status: "ready", position: { ...INITIAL_OBJECT_POSITION } };

  update(input: InteractionInput): EngineeringObjectSnapshot {
    if (!input.handDetected) {
      this.snapshot = { ...this.snapshot, status: "ready" };
      return this.getSnapshot();
    }

    if (this.snapshot.status === "grabbed") {
      if (input.gesture === "pinch" && input.cursorWorld) {
        this.snapshot = { status: "grabbed", position: { ...input.cursorWorld } };
      } else {
        this.snapshot = { ...this.snapshot, status: "ready" };
      }
      return this.getSnapshot();
    }

    if (input.hovered && input.gesture === "pinch" && input.cursorWorld) {
      this.snapshot = { status: "grabbed", position: { ...input.cursorWorld } };
    } else {
      this.snapshot = { ...this.snapshot, status: input.hovered ? "targeted" : "ready" };
    }
    return this.getSnapshot();
  }

  reset(): EngineeringObjectSnapshot {
    this.snapshot = { status: "ready", position: { ...INITIAL_OBJECT_POSITION } };
    return this.getSnapshot();
  }

  getSnapshot(): EngineeringObjectSnapshot {
    return { status: this.snapshot.status, position: { ...this.snapshot.position } };
  }
}
