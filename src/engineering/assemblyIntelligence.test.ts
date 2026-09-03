import { describe, expect, it } from "vitest";
import * as THREE from "three";
import { ComponentService } from "./componentService";
import { bindAssemblyIntelligence, searchSemanticComponents, semanticCoverage, semanticStatus } from "./assemblyIntelligence";
import type { AssemblyIntelligenceSnapshot } from "./types";

function components() {
  const root = new THREE.Group(); root.name = "Motor";
  for (const name of ["Rotor", "Rotor", "Shaft"]) { const mesh = new THREE.Mesh(new THREE.BoxGeometry(), new THREE.MeshBasicMaterial()); mesh.name = name; root.add(mesh); }
  return new ComponentService(root).list();
}

describe("Assembly Intelligence", () => {
  it("gera identidades estruturais estáveis e distintas para nomes duplicados", () => {
    const first = components(); const second = components();
    expect(first.map((item) => item.persistentIdentity)).toEqual(second.map((item) => item.persistentIdentity));
    expect(new Set(first.map((item) => item.persistentIdentity)).size).toBe(first.length);
    const rotors = first.filter((item) => item.originalName === "Rotor");
    expect(rotors[0].persistentIdentity).not.toBe(rotors[1].persistentIdentity);
  });

  it("calcula cobertura só com componentes selecionáveis e pesquisa campos semânticos", () => {
    const list = components(); const selectable = list.filter((item) => item.selectable && item.children.length === 0);
    const snapshot: AssemblyIntelligenceSnapshot = { subsystems: [{ id: "drive", modelIdentity: "m", name: "Drive", description: "", parentSubsystemId: null, createdAt: "", updatedAt: "" }], relationships: [], semantics: [
      { modelIdentity: "m", componentIdentity: selectable[0].persistentIdentity, originalName: selectable[0].originalName, structuralPath: selectable[0].structuralPath, componentType: selectable[0].type, semanticLabel: "Rotor A", subsystemId: "drive", role: "Girar", description: "", notes: "", createdAt: "", updatedAt: "" },
      { modelIdentity: "m", componentIdentity: selectable[1].persistentIdentity, originalName: selectable[1].originalName, structuralPath: selectable[1].structuralPath, componentType: selectable[1].type, semanticLabel: "Rotor B", subsystemId: null, role: "", description: "", notes: "", createdAt: "", updatedAt: "" },
    ] };
    const views = bindAssemblyIntelligence(list, snapshot);
    expect(semanticCoverage(views)).toEqual({ total: 3, classified: 1, partial: 1, unclassified: 1, percent: 33 });
    expect(searchSemanticComponents(views, "drive")).toHaveLength(1);
    expect(searchSemanticComponents(views, "girar")).toHaveLength(1);
  });

  it("distingue unclassified, partial e classified", () => {
    expect(semanticStatus()).toBe("unclassified");
    expect(semanticStatus({ semanticLabel: "Rotor", subsystemId: null, role: "", modelIdentity: "m", componentIdentity: "c", originalName: "Rotor", structuralPath: "Root/Rotor[0]", componentType: "Mesh", description: "", notes: "", createdAt: "", updatedAt: "" })).toBe("partial");
    expect(semanticStatus({ semanticLabel: "Rotor", subsystemId: "s", role: "Drive", modelIdentity: "m", componentIdentity: "c", originalName: "Rotor", structuralPath: "Root/Rotor[0]", componentType: "Mesh", description: "", notes: "", createdAt: "", updatedAt: "" })).toBe("classified");
  });

  it("faz rebind apenas quando nome e tipo produzem um único candidato", () => {
    const list = components();
    const stale = (originalName: string, componentIdentity: string) => ({ modelIdentity: "m", componentIdentity, originalName, structuralPath: `Legacy/${originalName}`, componentType: "Mesh", semanticLabel: `${originalName} semantic`, subsystemId: null, role: "", description: "", notes: "", createdAt: "", updatedAt: "" });
    const views = bindAssemblyIntelligence(list, { subsystems: [], relationships: [], semantics: [stale("Shaft", "legacy-shaft"), stale("Rotor", "legacy-rotor")] });
    expect(views.find((item) => item.component.originalName === "Shaft")?.binding).toBe("rebound");
    expect(views.filter((item) => item.component.originalName === "Rotor").every((item) => item.binding === "review_required")).toBe(true);
  });
});
