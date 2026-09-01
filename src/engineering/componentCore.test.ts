import { describe, expect, it } from "vitest";
import * as THREE from "three";
import { ComponentService } from "./componentService";

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
    service.hide(id);
    service.restore();
    expect(service.get(id)?.visible).toBe(true);
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
});
