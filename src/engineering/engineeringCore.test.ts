import { describe, expect, it } from "vitest";
import { CalibrationSession, defaultCalibration } from "./calibration";
import { evaluateGesture } from "./gestureEngine";
import { stabilizeHandIdentity } from "./handIdentity";
import { INITIAL_OBJECT_POSITION, InteractionController } from "./interactionController";
import { clamp, landmarkDistance, mirrorNormalizedPoint, normalizedToViewport, smoothLandmark, smoothNumber } from "./trackingMath";
import type { GestureState, HandInteractionPoint, HandLandmark, HandSide, TrackedHand } from "./types";

function baseHand(): HandLandmark[] {
  return Array.from({ length: 21 }, (_, index) => ({ x: 0.4 + index * 0.001, y: 0.7, z: 0 }));
}

function handWithPinchDistance(distance: number): HandLandmark[] {
  const hand = baseHand();
  hand[4] = { x: 0.4, y: 0.4, z: 0 };
  hand[8] = { x: 0.4 + distance, y: 0.4, z: 0 };
  return hand;
}

function gestureHand(extended: number[]): HandLandmark[] {
  const hand = handWithPinchDistance(0.2);
  for (const [tip, pip] of [[8, 6], [12, 10], [16, 14], [20, 18]]) {
    hand[pip] = { x: hand[pip].x, y: 0.55, z: 0 };
    hand[tip] = { x: hand[tip].x, y: extended.includes(tip) ? 0.3 : 0.7, z: 0 };
  }
  return hand;
}

function trackedHand(id: HandSide, x: number): TrackedHand {
  const landmarks = gestureHand([8, 12, 16, 20]).map((point) => ({ ...point, x: point.x + x - 0.4 }));
  return { id, handedness: id, confidence: 0.9, landmarks };
}

function interactionHand(id: HandSide, gesture: GestureState, x: number, hovered = false): HandInteractionPoint {
  return { id, gesture, hovered, viewport: { x, y: 0.5, ndcX: x * 2 - 1, ndcY: 0 }, world: { x: x * 4 - 2, y: 1, z: 0 } };
}

describe("Engineering Core tracking and gestures", () => {
  it("calcula distância tridimensional e smoothing", () => {
    expect(landmarkDistance({ x: 0, y: 0, z: 0 }, { x: 3, y: 4, z: 0 })).toBe(5);
    expect(smoothLandmark({ x: 0, y: 0, z: 0 }, { x: 1, y: 1, z: 1 }, 0.25)).toEqual({ x: 0.25, y: 0.25, z: 0.25 });
    expect(smoothNumber(1, 2, 0.25)).toBe(1.25);
  });

  it("preserva pinch, release e hysteresis", () => {
    expect(evaluateGesture(handWithPinchDistance(0.04), "none").state).toBe("pinch");
    expect(evaluateGesture(handWithPinchDistance(0.055), "none").state).not.toBe("pinch");
    expect(evaluateGesture(handWithPinchDistance(0.055), "pinch").state).toBe("pinch");
    expect(evaluateGesture(handWithPinchDistance(0.07), "pinch").state).not.toBe("pinch");
  });

  it("reconhece POINT e OPEN_HAND com prioridade para PINCH", () => {
    expect(evaluateGesture(gestureHand([8]), "none").state).toBe("point");
    expect(evaluateGesture(gestureHand([8, 12, 16, 20]), "none").state).toBe("open_hand");
    const pinchingOpenHand = gestureHand([8, 12, 16, 20]);
    pinchingOpenHand[4] = { ...pinchingOpenHand[8], x: pinchingOpenHand[8].x - 0.02 };
    expect(evaluateGesture(pinchingOpenHand, "open_hand").state).toBe("pinch");
  });

  it("espelha e converte coordenadas normalizadas", () => {
    expect(mirrorNormalizedPoint({ x: 0.2, y: 0.4 })).toEqual({ x: 0.8, y: 0.4 });
    expect(normalizedToViewport({ x: 0.2, y: 0.25 })).toEqual({ x: 0.8, y: 0.25, ndcX: 0.6000000000000001, ndcY: 0.5 });
  });

  it("estabiliza duas mãos por handedness e posição anterior", () => {
    const left = trackedHand("left", 0.75);
    const right = trackedHand("right", 0.25);
    expect(stabilizeHandIdentity([right, left]).map((hand) => hand.id)).toEqual(["left", "right"]);
    const unidentified = [{ landmarks: trackedHand("right", 0.26).landmarks }, { landmarks: trackedHand("left", 0.74).landmarks }];
    expect(stabilizeHandIdentity(unidentified, { left: left.landmarks[0], right: right.landmarks[0] }).map((hand) => hand.id)).toEqual(["left", "right"]);
  });

  it("resolve handedness duplicado sem produzir dois IDs iguais", () => {
    const first = trackedHand("left", 0.7);
    const second = trackedHand("left", 0.3);
    const result = stabilizeHandIdentity([first, second]);
    expect(new Set(result.map((hand) => hand.id)).size).toBe(2);
  });
});

describe("Engineering Core spatial interaction", () => {
  it("mantém MOVE da v0.1, aplica bounds e cancela ao perder tracking", () => {
    const controller = new InteractionController();
    expect(controller.update({ mode: "move", hands: [interactionHand("right", "none", 0.5, true)] }).status).toBe("targeted");
    expect(controller.update({ mode: "move", hands: [interactionHand("right", "pinch", 0.5, true)] }).control).toBe("translate");
    const moved = controller.update({ mode: "move", hands: [{ ...interactionHand("right", "pinch", 1), world: { x: 20, y: 20, z: 0 } }] });
    expect(moved.position).toEqual({ x: 3.25, y: 3.5, z: 0 });
    expect(controller.update({ mode: "move", hands: [] }).position).toEqual(moved.position);
  });

  it("converte deltas em rotação X/Y e preserva ao soltar", () => {
    const controller = new InteractionController();
    controller.update({ mode: "rotate", hands: [interactionHand("right", "pinch", 0.4, true)] });
    const rotated = controller.update({ mode: "rotate", hands: [{ ...interactionHand("right", "pinch", 0.7), viewport: { x: 0.7, y: 0.8, ndcX: 0.4, ndcY: -0.6 } }] });
    expect(rotated.rotation.x).toBeGreaterThan(0);
    expect(rotated.rotation.y).toBeGreaterThan(0);
    expect(controller.update({ mode: "rotate", hands: [interactionHand("right", "none", 0.7)] }).rotation).toEqual(rotated.rotation);
  });

  it("aplica sensibilidade configurada à rotação", () => {
    const slow = new InteractionController({ rotationSensitivity: 1, minScale: 0.5, maxScale: 2, rotationSmoothingAlpha: 1, scaleSmoothingAlpha: 1 });
    const fast = new InteractionController({ rotationSensitivity: 4, minScale: 0.5, maxScale: 2, rotationSmoothingAlpha: 1, scaleSmoothingAlpha: 1 });
    for (const controller of [slow, fast]) controller.update({ mode: "rotate", hands: [interactionHand("right", "pinch", 0.4, true)] });
    const moved = { ...interactionHand("right", "pinch", 0.6), viewport: { x: 0.6, y: 0.5, ndcX: 0.2, ndcY: 0 } };
    expect(fast.update({ mode: "rotate", hands: [moved] }).rotation.y).toBeGreaterThan(slow.update({ mode: "rotate", hands: [moved] }).rotation.y);
  });

  it("inicia Scale Session, limita, suaviza e cancela sem salto", () => {
    const controller = new InteractionController({ rotationSensitivity: 3, minScale: 0.5, maxScale: 1.5, rotationSmoothingAlpha: 1, scaleSmoothingAlpha: 1 });
    const initial = [interactionHand("left", "open_hand", 0.35), interactionHand("right", "open_hand", 0.65)];
    expect(controller.update({ mode: "scale", hands: initial }).control).toBe("scale");
    const enlarged = controller.update({ mode: "scale", hands: [interactionHand("left", "open_hand", 0), interactionHand("right", "open_hand", 1)] });
    expect(enlarged.scale).toBe(1.5);
    const cancelled = controller.update({ mode: "scale", hands: [interactionHand("left", "open_hand", 0)] });
    expect(cancelled.scale).toBe(1.5);
    expect(cancelled.control).toBe("none");
  });

  it("calcula scale factor relativo à distância inicial", () => {
    const controller = new InteractionController({ rotationSensitivity: 3, minScale: 0.2, maxScale: 3, rotationSmoothingAlpha: 1, scaleSmoothingAlpha: 1 });
    controller.update({ mode: "scale", hands: [interactionHand("left", "open_hand", 0.4), interactionHand("right", "open_hand", 0.6)] });
    expect(controller.update({ mode: "scale", hands: [interactionHand("left", "open_hand", 0.3), interactionHand("right", "open_hand", 0.7)] }).scale).toBeCloseTo(2);
  });

  it("aplica smoothing de escala sem alcançar o alvo no primeiro frame", () => {
    const controller = new InteractionController({ rotationSensitivity: 3, minScale: 0.2, maxScale: 3, rotationSmoothingAlpha: 1, scaleSmoothingAlpha: 0.25 });
    controller.update({ mode: "scale", hands: [interactionHand("left", "open_hand", 0.4), interactionHand("right", "open_hand", 0.6)] });
    const result = controller.update({ mode: "scale", hands: [interactionHand("left", "open_hand", 0.3), interactionHand("right", "open_hand", 0.7)] });
    expect(result.scale).toBeGreaterThan(1);
    expect(result.scale).toBeLessThan(2);
  });

  it("troca de modo cancela a sessão e reset restaura transformação", () => {
    const controller = new InteractionController();
    controller.update({ mode: "move", hands: [interactionHand("right", "pinch", 0.6, true)] });
    expect(controller.update({ mode: "rotate", hands: [interactionHand("right", "none", 0.6)] }).control).toBe("none");
    expect(controller.reset()).toEqual({ status: "ready", position: INITIAL_OBJECT_POSITION, rotation: { x: 0, y: 0, z: 0 }, scale: 1, control: "none" });
  });

  it("restaura transformação global ao recriar o renderer da sessão", () => {
    const controller = new InteractionController();
    expect(controller.restore({ status: "grabbed", control: "rotate", position: { x: 1, y: 2, z: 0 }, rotation: { x: 0.2, y: 0.4, z: 0 }, scale: 1.4 }))
      .toEqual({ status: "ready", control: "none", position: { x: 1, y: 2, z: 0 }, rotation: { x: 0.2, y: 0.4, z: 0 }, scale: 1.4 });
  });

  it("clamp respeita limites de escala", () => {
    expect(clamp(0.1, 0.5, 2)).toBe(0.5);
    expect(clamp(3, 0.5, 2)).toBe(2);
  });
});

describe("Engineering Core calibration", () => {
  it("coleta duas mãos, calcula mediana e marca calibração", () => {
    const session = new CalibrationSession();
    for (let index = 0; index < 12; index += 1) session.addSample([trackedHand("left", 0.3), trackedHand("right", 0.7)]);
    const result = session.complete(defaultCalibration());
    expect(result.calibrated).toBe(true);
    expect(result.comfortableHandDistance).toBeCloseTo(0.4);
  });

  it("calibração ajusta smoothing dentro dos limites seguros", () => {
    const session = new CalibrationSession();
    for (let index = 0; index < 12; index += 1) session.addSample([trackedHand("left", 0.25 + index * 0.002), trackedHand("right", 0.75)]);
    expect(session.complete(defaultCalibration()).smoothingAlpha).toBeGreaterThanOrEqual(0.2);
    expect(session.complete(defaultCalibration()).smoothingAlpha).toBeLessThanOrEqual(0.5);
  });

  it("recusa calibração sem amostras suficientes", () => {
    expect(() => new CalibrationSession().complete(defaultCalibration())).toThrow(/duas mãos/i);
  });
});
