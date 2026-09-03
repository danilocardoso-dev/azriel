import type { EngineeringActionResult, EngineeringComponentSummary } from "./engineeringSessionService";

const fixture: EngineeringComponentSummary[] = [
  { id: "component-root", name: "MotorAssembly", type: "Group", parent: null, children: 3, visible: true },
  { id: "component-housing", name: "Housing", type: "Mesh", parent: "MotorAssembly", children: 0, visible: true },
  { id: "component-rotor", name: "Rotor", type: "Mesh", parent: "MotorAssembly", children: 0, visible: true },
  { id: "component-shaft", name: "Shaft", type: "Mesh", parent: "MotorAssembly", children: 0, visible: true },
];

export class FakeEngineeringService {
  readonly calls: Array<{ command: string; value?: string | number }> = [];
  private selectedId: string | null = null;
  private factor = 0;

  getLoadedModel() { return { loaded: true, name: "MotorAssembly.glb", format: "GLB" as const, components: fixture.length, meshes: 3, materials: 1, dimensions: { x: 2, y: 2, z: 4, largest: 4 }, status: "ready" }; }
  getModelSummary() { return { loaded: true, name: "MotorAssembly.glb", components: fixture.length, meshes: 3, materials: 1, selectedComponent: this.selected(), explosionFactor: this.factor, explosionMode: "all" as const }; }
  listComponents() { return fixture.map((component) => ({ ...component })); }
  findComponent(query: string) {
    const candidates = fixture.filter((component) => component.name.toLowerCase().includes(query.toLowerCase()));
    return candidates.length === 1
      ? { found: true, exact: candidates[0].name.toLowerCase() === query.toLowerCase(), ambiguous: false, component: candidates[0], candidates }
      : { found: false, code: candidates.length ? "AMBIGUOUS" : "NOT_FOUND", ambiguous: candidates.length > 1, message: candidates.length ? "Busca ambígua." : "Componente não encontrado.", candidates };
  }
  getComponentDetails(reference?: string) { const component = this.resolve(reference); return component ? { success: true, component } : { success: false, code: "NOT_FOUND", message: "Componente não encontrado." }; }
  getSelectedComponent() { const component = this.selected(); return component ? { selected: true, component } : { selected: false, code: "NO_SELECTION", message: "Nenhum componente está selecionado." }; }
  getExplosionState() { return { available: true, enabled: true, factor: this.factor, mode: "all", selectedRoot: null, assemblyState: this.factor === 0 ? "assembled" : "partial" }; }
  getComponentSemantics(reference?: string) { return { success: true, component: this.resolve(reference), semantic: null, subsystem: null }; }
  getSubsystems() { return []; }
  getSubsystemComponents() { return { success: false, code: "NOT_FOUND", candidates: [] }; }
  getComponentRelationships(reference?: string) { return { success: true, component: this.resolve(reference), relationships: [] }; }
  getUnclassifiedComponents() { return fixture; }
  getSemanticCoverage() { return { total: 3, classified: 0, partial: 0, unclassified: 3, percent: 0 }; }
  getAssemblyGraphSummary() { return { model: "MotorAssembly.glb", subsystems: 0, relationships: 0 }; }
  selectComponent(reference?: string) { const component = this.resolve(reference); if (component) this.selectedId = component.id; return this.action("select_component", component); }
  focusComponent(reference?: string) { return this.action("focus_component", this.resolve(reference)); }
  isolateComponent(reference?: string) { return this.action("isolate_component", this.resolve(reference)); }
  showAllComponents() { return this.success("show_all_components"); }
  hideComponent(reference?: string) { return this.action("hide_component", this.resolve(reference)); }
  showComponent(reference?: string) { return this.action("show_component", this.resolve(reference)); }
  setExplosionFactor(factor: number) { this.factor = Math.max(0, Math.min(1, factor)); return this.success("set_explosion_factor", this.factor); }
  adjustExplosion(delta: number) { return this.setExplosionFactor(this.factor + delta); }
  explodeAll() { this.factor = 1; return this.success("explode_all", 1); }
  explodeComponent(reference?: string) { return this.action("explode_component", this.resolve(reference)); }
  reassemble() { this.factor = 0; return this.success("reassemble", 0); }
  resetModelView() { return this.success("reset_model_view"); }

  private selected() { return fixture.find((component) => component.id === this.selectedId) ?? null; }
  private resolve(reference?: string) {
    if (!reference) return this.selected();
    const value = reference.toLowerCase();
    return fixture.find((component) => component.id === reference || component.name.toLowerCase() === value) ?? null;
  }
  private action(command: string, component: EngineeringComponentSummary | null): EngineeringActionResult {
    this.calls.push({ command, value: component?.id });
    return component ? { success: true, code: "OK", message: `${command} executado.`, component } : { success: false, code: "NOT_FOUND", message: "Componente não encontrado." };
  }
  private success(command: string, value?: number): EngineeringActionResult {
    this.calls.push({ command, value });
    return { success: true, code: "OK", message: `${command} executado.` };
  }
}
