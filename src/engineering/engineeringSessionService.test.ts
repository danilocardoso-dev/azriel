import { describe, expect, it } from "vitest";
import * as THREE from "three";
import { ComponentService } from "./componentService";
import { EngineeringSessionService } from "./engineeringSessionService";
import { ExplosionService } from "./explosionService";
import type { LoadedEngineeringModel } from "./modelService";

function model(names = ["Rotor", "Shaft", "Bearing"]): LoadedEngineeringModel {
  const root = new THREE.Group();
  root.name = "MotorAssembly";
  const assembly = new THREE.Group();
  assembly.name = "DriveAssembly";
  names.forEach((name, index) => {
    const mesh = new THREE.Mesh(new THREE.BoxGeometry(1, 1, 1), new THREE.MeshStandardMaterial({ name: `Material_${index}` }));
    mesh.name = name;
    mesh.position.x = index * 1.5;
    assembly.add(mesh);
  });
  root.add(assembly);
  root.updateMatrixWorld(true);
  const components = new ComponentService(root);
  return {
    root,
    components,
    explosion: new ExplosionService(components),
    nodes: [],
    metadata: {
      name: "motor.glb", format: "GLB", objects: names.length + 2, groups: 2, meshes: names.length, materials: names.length,
      vertices: names.length * 24, triangles: names.length * 12,
      dimensions: { x: 4, y: 1, z: 1, largest: 4 }, complexity: "normal",
    },
  };
}

describe("Engineering Session Service", () => {
  it("expõe somente resumos serializáveis do modelo e componentes", () => {
    const service = new EngineeringSessionService();
    service.attachModel(model());
    expect(service.getLoadedModel()).toMatchObject({ loaded: true, name: "motor.glb", format: "GLB", meshes: 3 });
    expect(service.getModelSummary()).toMatchObject({ loaded: true, components: 5, explosionFactor: 0 });
    expect(JSON.stringify(service.listComponents())).not.toContain("Object3D");
  });

  it("resolve nome exato e parcial e não escolhe busca ambígua", () => {
    const service = new EngineeringSessionService();
    service.attachModel(model(["Rotor", "Rotor_Left", "Rotor_Right"]));
    expect(service.findComponent("Rotor")).toMatchObject({ found: true, exact: true, component: { name: "Rotor" } });
    expect(service.findComponent("Left")).toMatchObject({ found: true, exact: false, component: { name: "Rotor_Left" } });
    expect(service.findComponent("Rot")).toMatchObject({ found: false, code: "AMBIGUOUS", ambiguous: true });
    expect(service.findComponent("Compressor")).toMatchObject({ found: false, code: "NOT_FOUND", ambiguous: false });
  });

  it("sincroniza seleção manual/AI e exige seleção para referência contextual", () => {
    const service = new EngineeringSessionService();
    service.attachModel(model());
    expect(service.getComponentDetails("essa peça")).toMatchObject({ success: false, code: "NO_SELECTION" });
    expect(service.selectComponent("Rotor")).toMatchObject({ success: true, component: { name: "Rotor" } });
    expect(service.getSelectedComponent()).toMatchObject({ selected: true, component: { name: "Rotor" } });
    expect(service.getComponentDetails("essa peça")).toMatchObject({ success: true, component: { name: "Rotor", selected: true } });
  });

  it("isola, oculta, mostra e restaura visibilidade", () => {
    const service = new EngineeringSessionService();
    service.attachModel(model());
    expect(service.hideComponent("Rotor")).toMatchObject({ success: true, component: { visible: false } });
    expect(service.showComponent("Rotor")).toMatchObject({ success: true, component: { visible: true } });
    expect(service.isolateComponent("Shaft")).toMatchObject({ success: true });
    expect(service.showAllComponents()).toMatchObject({ success: true });
    expect((service.listComponents() as Array<{ visible: boolean }>).every((component) => component.visible)).toBe(true);
  });

  it("explode, ajusta, limita fator e reconstrói sem drift", () => {
    const service = new EngineeringSessionService();
    service.attachModel(model());
    expect(service.explodeAll()).toMatchObject({ success: true, explosion: { factor: 1 } });
    expect(service.setExplosionFactor(1.2)).toMatchObject({ success: false, code: "INVALID_FACTOR" });
    expect(service.setExplosionFactor(0.5)).toMatchObject({ success: true, explosion: { factor: 0.5 } });
    expect(service.adjustExplosion(0.15)).toMatchObject({ success: true, explosion: { factor: 0.65 } });
    expect(service.reassemble()).toMatchObject({ success: true, explosion: { factor: 0 } });
  });

  it("retorna erro para explosão de folha e ausência de modelo", () => {
    const empty = new EngineeringSessionService();
    expect(empty.explodeAll()).toMatchObject({ success: false, code: "NO_MODEL" });
    const service = new EngineeringSessionService();
    service.attachModel(model());
    expect(service.explodeComponent("Rotor")).toMatchObject({ success: false, code: "LEAF_COMPONENT" });
    expect(service.getExplosionState()).toMatchObject({ available: true, enabled: true, factor: 0, mode: "all" });
  });

  it("explode uma submontagem, solicita foco e reseta a vista", () => {
    const service = new EngineeringSessionService();
    service.attachModel(model());
    expect(service.explodeComponent("DriveAssembly")).toMatchObject({ success: true, explosion: { factor: 1, mode: "selected" } });
    const focusBefore = service.getViewState().focusRequest.sequence;
    expect(service.focusComponent("Rotor")).toMatchObject({ success: true });
    expect(service.getViewState().focusRequest).toMatchObject({ sequence: focusBefore + 1, componentId: expect.any(String) });
    const resetBefore = service.getViewState().resetSignal;
    expect(service.resetModelView()).toMatchObject({ success: true });
    expect(service.getViewState().resetSignal).toBe(resetBefore + 1);
  });

  it("não inventa semântica para nomes Mesh_###", () => {
    const service = new EngineeringSessionService();
    service.attachModel(model(["Mesh_001", "Mesh_002"]));
    expect(service.findComponent("rotor")).toMatchObject({ found: false, poorNames: true });
    expect(JSON.stringify(service.listComponents())).not.toContain("Rotor");
  });

  it("mantém a sessão e registra comandos visuais em memória", () => {
    const service = new EngineeringSessionService();
    const fixture = model();
    service.attachModel(fixture);
    service.selectComponent("Rotor", "ai");
    expect(service.getViewState()).toMatchObject({ model: fixture, selectedComponentId: expect.any(String), commands: [{ command: "select_component", source: "ai", status: "success" }] });
  });
});
