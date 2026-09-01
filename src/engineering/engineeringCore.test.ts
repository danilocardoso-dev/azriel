import { describe, expect, it } from "vitest";
import { evaluatePinch } from "./gestureEngine";
import { INITIAL_OBJECT_POSITION, InteractionController } from "./interactionController";
import { landmarkDistance, mirrorNormalizedPoint, normalizedToViewport, smoothLandmark } from "./trackingMath";
import type { HandLandmark } from "./types";

const handWithDistance = (distance: number): HandLandmark[] => Array.from({ length: 21 }, (_, index) => ({
  x: index === 8 ? distance : 0,
  y: 0,
  z: 0,
}));

describe("Engineering Core tracking math", () => {
  it("calcula distância tridimensional entre landmarks", () => {
    expect(landmarkDistance({ x: 0, y: 0, z: 0 }, { x: 3, y: 4, z: 0 })).toBe(5);
  });

  it("inicia pinch abaixo do threshold", () => {
    expect(evaluatePinch(handWithDistance(0.04), "none").state).toBe("pinch");
  });

  it("libera pinch acima do threshold de release", () => {
    expect(evaluatePinch(handWithDistance(0.07), "pinch").state).toBe("none");
  });

  it("mantém hysteresis entre os thresholds", () => {
    expect(evaluatePinch(handWithDistance(0.055), "none").state).toBe("none");
    expect(evaluatePinch(handWithDistance(0.055), "pinch").state).toBe("pinch");
  });

  it("suaviza coordenadas por interpolação exponencial", () => {
    expect(smoothLandmark({ x: 0, y: 0, z: 0 }, { x: 1, y: 1, z: 1 }, 0.25)).toEqual({ x: 0.25, y: 0.25, z: 0.25 });
  });

  it("espelha e converte coordenadas normalizadas para NDC", () => {
    expect(mirrorNormalizedPoint({ x: 0.2, y: 0.4 })).toEqual({ x: 0.8, y: 0.4 });
    expect(normalizedToViewport({ x: 0.2, y: 0.25 })).toEqual({ x: 0.8, y: 0.25, ndcX: 0.6000000000000001, ndcY: 0.5 });
  });
});

describe("Engineering Core interaction controller", () => {
  it("faz ready, targeted, grabbed e release mantendo a posição", () => {
    const controller = new InteractionController();
    expect(controller.getSnapshot().status).toBe("ready");
    expect(controller.update({ handDetected: true, gesture: "none", hovered: true, cursorWorld: { x: 1, y: 1, z: 0 } }).status).toBe("targeted");
    expect(controller.update({ handDetected: true, gesture: "pinch", hovered: true, cursorWorld: { x: 1, y: 2, z: 0 } })).toEqual({ status: "grabbed", position: { x: 1, y: 2, z: 0 } });
    expect(controller.update({ handDetected: true, gesture: "pinch", hovered: false, cursorWorld: { x: 2, y: 3, z: 0 } }).position).toEqual({ x: 2, y: 3, z: 0 });
    expect(controller.update({ handDetected: true, gesture: "none", hovered: false, cursorWorld: { x: 4, y: 4, z: 0 } })).toEqual({ status: "ready", position: { x: 2, y: 3, z: 0 } });
  });

  it("cancela grab quando perde tracking sem mover o objeto", () => {
    const controller = new InteractionController();
    controller.update({ handDetected: true, gesture: "pinch", hovered: true, cursorWorld: { x: 1, y: 2, z: 0 } });
    expect(controller.update({ handDetected: false, gesture: "pinch", hovered: false, cursorWorld: { x: 9, y: 9, z: 0 } })).toEqual({ status: "ready", position: { x: 1, y: 2, z: 0 } });
  });

  it("restaura posição e estado no reset", () => {
    const controller = new InteractionController();
    controller.update({ handDetected: true, gesture: "pinch", hovered: true, cursorWorld: { x: 2, y: 2, z: 0 } });
    expect(controller.reset()).toEqual({ status: "ready", position: INITIAL_OBJECT_POSITION });
  });
});
