import { describe, expect, it } from "vitest";
import * as THREE from "three";
import { ComponentService } from "./componentService";
import { ExplosionGestureController } from "./explosionGestureController";
import { ExplosionService } from "./explosionService";
import type { HandSide, ViewportPoint } from "./types";

function hand(id: HandSide, x: number, y = 0.5): { id: HandSide; viewport: ViewportPoint } {
  return { id, viewport: { x, y, ndcX: x * 2 - 1, ndcY: 1 - y * 2 } };
}

function assemblyFixture() {
  const root = new THREE.Group();
  root.name = "ASSEMBLY";
  const wrapper = new THREE.Group();
  wrapper.name = "WRAPPER";
  const leftGroup = new THREE.Group();
  leftGroup.name = "LEFT-GROUP";
  const left = new THREE.Mesh(new THREE.BoxGeometry(1, 1, 1), new THREE.MeshBasicMaterial());
  left.name = "LEFT";
  left.position.set(-2, 0, 0);
  leftGroup.add(left);
  const rightGroup = new THREE.Group();
  rightGroup.name = "RIGHT-GROUP";
  const right = new THREE.Mesh(new THREE.BoxGeometry(1, 1, 1), new THREE.MeshBasicMaterial());
  right.name = "RIGHT";
  right.position.set(2, 0, 0);
  rightGroup.add(right);
  const centered = new THREE.Mesh(new THREE.BoxGeometry(0.5, 0.5, 0.5), new THREE.MeshBasicMaterial());
  centered.name = "CENTER";
  wrapper.add(leftGroup, rightGroup, centered);
  root.add(wrapper);
  root.updateMatrixWorld(true);
  return { root, leftGroup, rightGroup, left, right, centered };
}

describe("Engineering Core Explosion Core", () => {
  it("descobre unidades por ramo e gera metadados determinísticos", () => {
    const first = assemblyFixture();
    const second = assemblyFixture();
    const firstService = new ExplosionService(new ComponentService(first.root));
    const secondService = new ExplosionService(new ComponentService(second.root));
    expect(firstService.getState()).toMatchObject({ enabled: true, factor: 0, mode: "all", assemblyState: "assembled" });
    expect(firstService.getMetadata()).toHaveLength(3);
    expect(firstService.getMetadata().map(({ direction, distanceMultiplier }) => ({ direction, distanceMultiplier })))
      .toEqual(secondService.getMetadata().map(({ direction, distanceMultiplier }) => ({ direction, distanceMultiplier })));
    expect(firstService.getMetadata().every(({ direction }) => Number.isFinite(direction.x + direction.y + direction.z))).toBe(true);
    const firstDirections = firstService.getMetadata().map(({ direction }) => direction.x);
    expect(firstDirections.some((x) => x < 0)).toBe(true);
    expect(firstDirections.some((x) => x > 0)).toBe(true);
    expect(firstService.getMetadata().every(({ depth }) => depth === 2)).toBe(true);
  });

  it("deriva cada posição da origem e reconstrói exatamente sem drift", () => {
    const fixture = assemblyFixture();
    const components = new ComponentService(fixture.root);
    const service = new ExplosionService(components);
    const metadata = service.getMetadata();
    const unit = metadata[0];
    const object = components.getObject(unit.componentId)!;
    service.applyFactor(0.5);
    expect(object.position.x).toBeCloseTo(unit.originalPosition.x + unit.localOffset.x * 0.5);
    service.applyFactor(1);
    service.applyFactor(0.17);
    service.applyFactor(0.83);
    service.applyFactor(0);
    expect(object.position.toArray()).toEqual([unit.originalPosition.x, unit.originalPosition.y, unit.originalPosition.z]);
    expect(service.getState().assemblyState).toBe("assembled");
  });

  it("limita o fator, produz guias coerentes e preserva componente oculto", () => {
    const fixture = assemblyFixture();
    const components = new ComponentService(fixture.root);
    const service = new ExplosionService(components);
    const metadata = service.getMetadata()[0];
    components.hide(metadata.componentId);
    expect(service.applyFactor(4).factor).toBe(1);
    const guide = service.getGuideLines().find(({ componentId }) => componentId === metadata.componentId)!;
    expect(guide.to.x).toBeCloseTo(guide.from.x + metadata.localOffset.x);
    expect(guide.to.y).toBeCloseTo(guide.from.y + metadata.localOffset.y);
    expect(guide.to.z).toBeCloseTo(guide.from.z + metadata.localOffset.z);
    expect(components.get(metadata.componentId)?.visible).toBe(false);
    const descendantId = components.descendantsOf(metadata.componentId)[0];
    if (descendantId) expect(service.getOffset(descendantId)).toEqual(service.getOffset(metadata.componentId));
    expect(service.applyFactor(-3).factor).toBe(0);
  });

  it("preserva isolamento e reaplica o fator vigente ao sair", () => {
    const fixture = assemblyFixture();
    const components = new ComponentService(fixture.root);
    const service = new ExplosionService(components);
    const leftGroupId = components.resolveObject(fixture.leftGroup)!;
    const rightGroupId = components.resolveObject(fixture.rightGroup)!;
    components.isolate(leftGroupId);
    service.applyFactor(0.5);
    expect(components.getIsolationId()).toBe(leftGroupId);
    expect(components.get(rightGroupId)?.visible).toBe(false);
    const expected = service.getMetadata().find(({ componentId }) => componentId === rightGroupId)!;
    components.exitIsolation();
    expect(components.get(rightGroupId)?.visible).toBe(true);
    expect(components.getObject(rightGroupId)?.position.x).toBeCloseTo(expected.originalPosition.x + expected.localOffset.x * 0.5);
  });

  it("explode somente os filhos do grupo selecionado e rejeita uma folha", () => {
    const fixture = assemblyFixture();
    const components = new ComponentService(fixture.root);
    const service = new ExplosionService(components);
    const wrapperId = components.resolveObject(fixture.root.children[0])!;
    const leafId = components.resolveObject(fixture.left)!;
    expect(service.configure("selected", wrapperId)).toEqual({ success: true, unitCount: 3 });
    expect(service.getState().selectedRootId).toBe(wrapperId);
    expect(service.configure("selected", leafId)).toMatchObject({ success: false, unitCount: 0 });
    expect(service.getState()).toMatchObject({ enabled: false, factor: 0, mode: "selected", selectedRootId: leafId });
  });

  it("restaura transformações manuais antes de iniciar a explosão", () => {
    const fixture = assemblyFixture();
    const components = new ComponentService(fixture.root);
    const leftId = components.resolveObject(fixture.left)!;
    fixture.left.position.set(8, 3, -2);
    fixture.left.rotation.set(1, 2, 3);
    fixture.left.scale.set(2, 2, 2);
    const service = new ExplosionService(components);
    service.configure("all");
    expect(components.get(leftId)).toMatchObject({
      position: { x: -2, y: 0, z: 0 },
      rotation: { x: 0, y: 0, z: 0 },
      scale: { x: 1, y: 1, z: 1 },
    });
  });

  it("mantém metadata isolada ao substituir modelo e expõe estados de animação", () => {
    const firstFixture = assemblyFixture();
    const first = new ExplosionService(new ComponentService(firstFixture.root));
    expect(first.applyFactor(0.4, "exploding").assemblyState).toBe("exploding");
    expect(first.applyFactor(1, "exploded").assemblyState).toBe("exploded");
    expect(first.applyFactor(0.5, "reassembling").assemblyState).toBe("reassembling");
    const secondFixture = assemblyFixture();
    const second = new ExplosionService(new ComponentService(secondFixture.root));
    expect(second.getState()).toMatchObject({ factor: 0, mode: "all", assemblyState: "assembled" });
    expect(second.getMetadata()).not.toBe(first.getMetadata());
    first.reset();
    expect(first.getState()).toMatchObject({ factor: 0, mode: "all", assemblyState: "assembled" });
  });

  it("mantém o último fator quando uma das mãos desaparece", () => {
    const controller = new ExplosionGestureController();
    expect(controller.update([hand("left", 0.4), hand("right", 0.6)], 0.25, 1)).toMatchObject({ factor: 0.25, active: true });
    const opened = controller.update([hand("left", 0.2), hand("right", 0.8)], 0.25, 1);
    expect(opened.active).toBe(true);
    expect(opened.factor).toBeCloseTo(0.97);
    expect(controller.update([hand("left", 0.2)], 0.97, 1)).toEqual({ factor: 0.97, active: false, cancelled: true });
  });

  it("suaviza e limita a abertura por gesto entre zero e um", () => {
    const controller = new ExplosionGestureController();
    controller.update([hand("left", 0.45), hand("right", 0.55)], 0.9, 1);
    expect(controller.update([hand("left", 0), hand("right", 1)], 0.9, 0.5).factor).toBeCloseTo(0.95);
    controller.cancel(1);
    controller.update([hand("left", 0), hand("right", 1)], 0.1, 1);
    expect(controller.update([hand("left", 0.49), hand("right", 0.51)], 0.1, 1).factor).toBe(0);
  });
});
