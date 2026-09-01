import { describe, expect, it } from "vitest";
import * as THREE from "three";
import { ComponentService } from "./componentService";
import { ComponentInteractionController } from "./componentInteractionController";
import type { GestureState, HandInteractionPoint, HandSide } from "./types";

function interactionHand(id: HandSide, gesture: GestureState, x: number, y = 0.5, hovered = false): HandInteractionPoint {
  return { id, gesture, hovered, viewport: { x, y, ndcX: x * 2 - 1, ndcY: 1 - y * 2 }, world: { x: x * 4 - 2, y: 2 - y * 4, z: 0 } };
}

function assemblyFixture() {
  const root = new THREE.Group();
  root.name = "MotorAssembly";
  const housing = new THREE.Group();
  housing.name = "Housing";
  const rotorMaterial = new THREE.MeshStandardMaterial({ name: "Steel", color: 0x87949b });
  const rotor = new THREE.Mesh(new THREE.BoxGeometry(2, 2, 2), rotorMaterial);
  rotor.name = "Rotor";
  rotor.position.set(2, 0, 0);
  const anonymousGroup = new THREE.Group();
  const shaft = new THREE.Mesh(new THREE.BoxGeometry(1, 1, 4), new THREE.MeshBasicMaterial());
  shaft.position.set(-2, 0, 0);
  anonymousGroup.add(shaft);
  housing.add(rotor, anonymousGroup);
  root.add(housing);
  root.updateMatrixWorld(true);
  return { root, housing, rotor, rotorMaterial, anonymousGroup, shaft };
}

describe("Engineering Core Component Core", () => {
  it("mapeia a hierarquia com IDs e nomes fallback determinísticos", () => {
    const firstFixture = assemblyFixture();
    const secondFixture = assemblyFixture();
    const first = new ComponentService(firstFixture.root).list();
    const second = new ComponentService(secondFixture.root).list();
    expect(first.map(({ id, name, parentId, children }) => ({ id, name, parentId, children })))
      .toEqual(second.map(({ id, name, parentId, children }) => ({ id, name, parentId, children })));
    expect(first[0].name).toBe("MotorAssembly");
    expect(first.find((component) => component.name === "Housing")?.children).toHaveLength(2);
    expect(first.some((component) => component.name.startsWith("Group_"))).toBe(true);
    expect(first.some((component) => component.name.startsWith("Mesh_"))).toBe(true);
  });

  it("resolve a mesh exata, busca por nome e lista ancestrais", () => {
    const { root, rotor } = assemblyFixture();
    const service = new ComponentService(root);
    const rotorId = service.resolveObject(rotor);
    expect(rotorId).not.toBeNull();
    expect(service.get(rotorId)?.name).toBe("Rotor");
    expect(service.search("rot").map((component) => component.id)).toEqual([rotorId]);
    expect(service.ancestors(rotorId!)).toHaveLength(2);
    expect(service.getRaycastMeshes()).toContain(rotor);
  });

  it("calcula geometria, materiais, bounding box, centro e direção uma vez no carregamento", () => {
    const { root, rotor } = assemblyFixture();
    const service = new ComponentService(root);
    const component = service.get(service.resolveObject(rotor));
    expect(component?.vertices).toBe(24);
    expect(component?.triangles).toBe(12);
    expect(component?.dimensions).toMatchObject({ x: 2, y: 2, z: 2, largest: 2 });
    expect(component?.center.x).toBeCloseTo(2);
    expect(component?.directionFromModelCenter.x).toBeGreaterThan(0);
    expect(component?.materials[0]).toMatchObject({ name: "Steel", type: "MeshStandardMaterial", color: "#87949B", textured: false });
  });

  it("representa targeted, selected, hidden e isolated sem estados contraditórios", () => {
    const { root, rotor } = assemblyFixture();
    const service = new ComponentService(root);
    const id = service.resolveObject(rotor)!;
    expect(service.state(id, id, null)).toBe("targeted");
    expect(service.state(id, id, id)).toBe("selected");
    service.hide(id);
    expect(service.state(id, id, id)).toBe("hidden");
    service.show(id);
    service.isolate(id);
    expect(service.state(id, null, id)).toBe("isolated");
  });

  it("oculta, mostra e restaura componentes sem alterar o material", () => {
    const { root, rotor, rotorMaterial } = assemblyFixture();
    const originalColor = rotorMaterial.color.getHex();
    const service = new ComponentService(root);
    const id = service.resolveObject(rotor)!;
    service.hide(id);
    expect(service.get(id)?.visible).toBe(false);
    expect(service.getRaycastMeshes()).not.toContain(rotor);
    service.show(id);
    expect(service.get(id)?.visible).toBe(true);
    rotor.position.set(7, 8, 9);
    rotor.rotation.set(0.3, 0.4, 0.5);
    rotor.scale.set(2, 3, 4);
    expect(service.get(id)).toMatchObject({ position: { x: 7, y: 8, z: 9 }, scale: { x: 2, y: 3, z: 4 } });
    service.hide(id);
    service.restore();
    expect(service.get(id)?.visible).toBe(true);
    expect(service.get(id)).toMatchObject({ position: { x: 2, y: 0, z: 0 }, rotation: { x: 0, y: 0, z: 0 }, scale: { x: 1, y: 1, z: 1 } });
    expect(rotorMaterial.color.getHex()).toBe(originalColor);
  });

  it("isola uma peça e devolve exatamente a visibilidade anterior", () => {
    const { root, rotor, shaft } = assemblyFixture();
    const service = new ComponentService(root);
    const rotorId = service.resolveObject(rotor)!;
    const shaftId = service.resolveObject(shaft)!;
    service.hide(shaftId);
    service.isolate(rotorId);
    expect(service.getIsolationId()).toBe(rotorId);
    expect(service.get(rotorId)?.visible).toBe(true);
    expect(service.get(shaftId)?.visible).toBe(false);
    service.exitIsolation();
    expect(service.getIsolationId()).toBeNull();
    expect(service.get(rotorId)?.visible).toBe(true);
    expect(service.get(shaftId)?.visible).toBe(false);
  });

  it("funciona com uma única mesh sem inventar uma montagem", () => {
    const mesh = new THREE.Mesh(new THREE.SphereGeometry(1), new THREE.MeshBasicMaterial());
    const service = new ComponentService(mesh);
    expect(service.list()).toHaveLength(1);
    expect(service.resolveObject(mesh)).toBe("component-0000");
    expect(service.get("component-0000")?.selectable).toBe(true);
  });

  it("move o componente por delta local sem saltar para o cursor", () => {
    const controller = new ComponentInteractionController();
    controller.bind("rotor", { x: 5, y: 1, z: 0 }, { x: 0, y: 0, z: 0 }, { x: 1, y: 1, z: 1 });
    controller.update("move", [interactionHand("right", "pinch", 0.5, 0.5, true)]);
    const moved = controller.update("move", [interactionHand("right", "pinch", 0.75, 0.25, false)])!;
    expect(moved.position).toEqual({ x: 6, y: 2, z: 0 });
    expect(moved.control).toBe("translate");
    expect(controller.update("move", [])!.position).toEqual(moved.position);
  });

  it("rotaciona somente o snapshot vinculado e preserva ao soltar", () => {
    const controller = new ComponentInteractionController({ rotationSensitivity: 2, minScaleFactor: 0.2, maxScaleFactor: 3, rotationSmoothingAlpha: 1, scaleSmoothingAlpha: 1 });
    controller.bind("rotor", { x: 0, y: 0, z: 0 }, { x: 0.2, y: 0.1, z: 0 }, { x: 1, y: 1, z: 1 });
    controller.update("rotate", [interactionHand("right", "pinch", 0.4, 0.5, true)]);
    const rotated = controller.update("rotate", [interactionHand("right", "pinch", 0.7, 0.8)])!;
    expect(rotated.rotation.x).toBeCloseTo(0.8);
    expect(rotated.rotation.y).toBeCloseTo(0.7);
    expect(controller.update("rotate", [])!.rotation).toEqual(rotated.rotation);
  });

  it("escala os três eixos relativamente com duas mãos", () => {
    const controller = new ComponentInteractionController({ rotationSensitivity: 2, minScaleFactor: 0.5, maxScaleFactor: 2, rotationSmoothingAlpha: 1, scaleSmoothingAlpha: 1 });
    controller.bind("rotor", { x: 0, y: 0, z: 0 }, { x: 0, y: 0, z: 0 }, { x: 1, y: 2, z: 3 });
    controller.update("scale", [interactionHand("left", "open_hand", 0.4), interactionHand("right", "open_hand", 0.6)]);
    const scaled = controller.update("scale", [interactionHand("left", "open_hand", 0.3), interactionHand("right", "open_hand", 0.7)])!;
    expect(scaled.scale).toEqual({ x: 2, y: 4, z: 6 });
  });

  it("trocar de componente e perder tracking encerram a sessão anterior", () => {
    const controller = new ComponentInteractionController();
    controller.bind("rotor", { x: 0, y: 0, z: 0 }, { x: 0, y: 0, z: 0 }, { x: 1, y: 1, z: 1 });
    controller.update("move", [interactionHand("right", "pinch", 0.5, 0.5, true)]);
    expect(controller.update("move", [])!.control).toBe("none");
    const next = controller.bind("housing", { x: 4, y: 0, z: 0 }, { x: 0, y: 1, z: 0 }, { x: 2, y: 2, z: 2 });
    expect(next).toMatchObject({ componentId: "housing", position: { x: 4, y: 0, z: 0 }, control: "none" });
  });
});
