import type { LoadedEngineeringModel } from "./modelService";
import type { EngineeringInteractionScope, EngineeringObjectSnapshot, ExplosionState, ModelComponent } from "./types";

export type EngineeringCommandSource = "ai" | "ui";
export type EngineeringCommandStatus = "success" | "error";

export interface EngineeringCommandLog {
  id: number;
  timestamp: string;
  command: string;
  target: string | null;
  source: EngineeringCommandSource;
  status: EngineeringCommandStatus;
  message: string;
}

export interface EngineeringSessionState {
  modelLoaded: boolean;
  modelName?: string;
  componentCount: number;
  selectedComponentId?: string;
  explosionFactor: number;
  explosionMode: "all" | "selected";
  modelMode: EngineeringInteractionScope;
}

export interface EngineeringViewSessionState {
  model: LoadedEngineeringModel | null;
  selectedComponentId: string | null;
  explosion: ExplosionState;
  modelMode: EngineeringInteractionScope;
  componentRevision: number;
  focusRequest: { sequence: number; componentId: string | null };
  resetSignal: number;
  commands: EngineeringCommandLog[];
  objectSnapshot: EngineeringObjectSnapshot;
}

export interface EngineeringComponentSummary {
  id: string;
  name: string;
  semanticLabel?: string;
  type: string;
  parent: string | null;
  children: number;
  visible: boolean;
}

export interface EngineeringActionResult {
  success: boolean;
  code: string;
  message: string;
  component?: EngineeringComponentSummary;
  candidates?: EngineeringComponentSummary[];
  explosion?: ExplosionState;
}

export interface EngineeringToolGateway {
  getLoadedModel(): unknown;
  getModelSummary(): unknown;
  listComponents(): unknown;
  findComponent(query: string): unknown;
  getComponentDetails(reference?: string): unknown;
  getSelectedComponent(): unknown;
  getExplosionState(): unknown;
  selectComponent(reference?: string, source?: EngineeringCommandSource): EngineeringActionResult;
  focusComponent(reference?: string, source?: EngineeringCommandSource): EngineeringActionResult;
  isolateComponent(reference?: string, source?: EngineeringCommandSource): EngineeringActionResult;
  showAllComponents(source?: EngineeringCommandSource): EngineeringActionResult;
  hideComponent(reference?: string, source?: EngineeringCommandSource): EngineeringActionResult;
  showComponent(reference?: string, source?: EngineeringCommandSource): EngineeringActionResult;
  setExplosionFactor(factor: number, source?: EngineeringCommandSource): EngineeringActionResult;
  adjustExplosion(delta: number, source?: EngineeringCommandSource): EngineeringActionResult;
  explodeAll(source?: EngineeringCommandSource): EngineeringActionResult;
  explodeComponent(reference?: string, source?: EngineeringCommandSource): EngineeringActionResult;
  reassemble(source?: EngineeringCommandSource): EngineeringActionResult;
  resetModelView(source?: EngineeringCommandSource): EngineeringActionResult;
}

const emptyExplosion = (): ExplosionState => ({ enabled: false, factor: 0, mode: "all", assemblyState: "assembled" });
const initialObject = (): EngineeringObjectSnapshot => ({ status: "ready", position: { x: 0, y: 0.65, z: 0 }, rotation: { x: 0, y: 0, z: 0 }, scale: 1, control: "none" });
const normalize = (value: string) => value.normalize("NFD").replace(/[\u0300-\u036f]/g, "").trim().toLocaleLowerCase("pt-BR");
const contextualReference = /^(?:essa|esta|esse|este|a)?\s*(?:peca|componente)?\s*(?:selecionad[ao])?$|^(?:isso|ela|ele)$/;
const poorName = /^(?:mesh|group|object)[ _.-]*\d+$/i;

function cloneExplosion(state: ExplosionState): ExplosionState {
  return { ...state };
}

export class EngineeringSessionService {
  private model: LoadedEngineeringModel | null = null;
  private selectedComponentId: string | null = null;
  private modelMode: EngineeringInteractionScope = "model";
  private componentRevision = 0;
  private focusRequest = { sequence: 0, componentId: null as string | null };
  private resetSignal = 0;
  private commandSequence = 0;
  private commands: EngineeringCommandLog[] = [];
  private objectSnapshot = initialObject();
  private readonly listeners = new Set<() => void>();

  subscribe = (listener: () => void) => {
    this.listeners.add(listener);
    return () => { this.listeners.delete(listener); };
  };

  getViewState(): EngineeringViewSessionState {
    return {
      model: this.model,
      selectedComponentId: this.selectedComponentId,
      explosion: cloneExplosion(this.model?.explosion.getState() ?? emptyExplosion()),
      modelMode: this.modelMode,
      componentRevision: this.componentRevision,
      focusRequest: { ...this.focusRequest },
      resetSignal: this.resetSignal,
      commands: this.commands.map((command) => ({ ...command })),
      objectSnapshot: { ...this.objectSnapshot, position: { ...this.objectSnapshot.position }, rotation: { ...this.objectSnapshot.rotation } },
    };
  }

  getSessionState(): EngineeringSessionState {
    const explosion = this.model?.explosion.getState() ?? emptyExplosion();
    return {
      modelLoaded: Boolean(this.model),
      modelName: this.model?.metadata.name,
      componentCount: this.model?.components.list().length ?? 0,
      selectedComponentId: this.selectedComponentId ?? undefined,
      explosionFactor: explosion.factor,
      explosionMode: explosion.mode,
      modelMode: this.modelMode,
    };
  }

  attachModel(model: LoadedEngineeringModel): LoadedEngineeringModel | null {
    const previous = this.model;
    this.model = model;
    this.selectedComponentId = null;
    this.modelMode = "model";
    this.componentRevision += 1;
    this.focusRequest = { sequence: this.focusRequest.sequence + 1, componentId: null };
    this.resetSignal += 1;
    this.commands = [];
    this.objectSnapshot = initialObject();
    this.emit();
    return previous;
  }

  detachModel(): LoadedEngineeringModel | null {
    const previous = this.model;
    this.model = null;
    this.selectedComponentId = null;
    this.modelMode = "model";
    this.componentRevision += 1;
    this.focusRequest = { sequence: this.focusRequest.sequence + 1, componentId: null };
    this.resetSignal += 1;
    this.commands = [];
    this.objectSnapshot = initialObject();
    this.emit();
    return previous;
  }

  setModelMode(mode: EngineeringInteractionScope): void {
    if (this.modelMode === mode) return;
    this.modelMode = mode;
    this.emit();
  }

  setSelectedComponent(componentId: string | null): void {
    if (componentId && !this.model?.components.get(componentId)) return;
    this.selectedComponentId = componentId;
    if (componentId) this.modelMode = "component";
    this.componentRevision += 1;
    this.emit();
  }

  focusModel(): void {
    this.focusRequest = { sequence: this.focusRequest.sequence + 1, componentId: null };
    this.emit();
  }

  touch(): void {
    this.componentRevision += 1;
    this.emit();
  }

  updateObjectSnapshot(snapshot: EngineeringObjectSnapshot): void {
    this.objectSnapshot = { ...snapshot, position: { ...snapshot.position }, rotation: { ...snapshot.rotation } };
  }

  getLoadedModel() {
    if (!this.model) return { loaded: false, code: "NO_MODEL", message: "Nenhum modelo está carregado no Engineering Core." };
    const { metadata } = this.model;
    return {
      loaded: true,
      name: metadata.name,
      format: metadata.format,
      components: this.model.components.list().length,
      meshes: metadata.meshes,
      materials: metadata.materials,
      dimensions: { ...metadata.dimensions },
      status: "ready",
    };
  }

  getModelSummary() {
    if (!this.model) return { loaded: false, code: "NO_MODEL", message: "Nenhum modelo está carregado no Engineering Core." };
    const selected = this.model.components.get(this.selectedComponentId);
    const explosion = this.model.explosion.getState();
    return {
      loaded: true,
      name: this.model.metadata.name,
      components: this.model.components.list().length,
      meshes: this.model.metadata.meshes,
      materials: this.model.metadata.materials,
      selectedComponent: selected ? this.summary(selected) : null,
      explosionFactor: explosion.factor,
      explosionMode: explosion.mode,
    };
  }

  listComponents(): EngineeringComponentSummary[] | { code: string; message: string } {
    if (!this.model) return { code: "NO_MODEL", message: "Nenhum modelo está carregado no Engineering Core." };
    return this.model.components.list().map((component) => this.summary(component));
  }

  findComponent(query: string) {
    if (!this.model) return { found: false, code: "NO_MODEL", message: "Nenhum modelo está carregado no Engineering Core.", candidates: [] as EngineeringComponentSummary[] };
    const resolution = this.resolve(query, false);
    if (resolution.component) return { found: true, exact: resolution.exact, ambiguous: false, component: this.summary(resolution.component), candidates: [this.summary(resolution.component)] };
    if (resolution.candidates.length > 1) return { found: false, code: "AMBIGUOUS", ambiguous: true, message: "Encontrei mais de um componente compatível. Escolha um deles.", candidates: resolution.candidates.map((candidate) => this.summary(candidate)) };
    const leafComponents = this.model.components.list().filter((component) => component.selectable && component.children.length === 0);
    const hasOnlyPoorNames = leafComponents.length > 0 && leafComponents.every((component) => poorName.test(component.name));
    return {
      found: false,
      code: "NOT_FOUND",
      ambiguous: false,
      poorNames: hasOnlyPoorNames,
      message: hasOnlyPoorNames
        ? "O modelo não possui nomes descritivos suficientes para identificar esse componente."
        : `Não encontrei nenhum componente chamado "${query.trim()}" no modelo atual.`,
      candidates: [] as EngineeringComponentSummary[],
    };
  }

  getComponentDetails(reference?: string) {
    const resolution = this.resolve(reference, true);
    if (!resolution.component) return this.resolutionError(resolution, reference);
    const component = resolution.component;
    return {
      success: true,
      component: {
        ...this.summary(component),
        parentId: component.parentId ?? null,
        childrenIds: [...component.children],
        selected: component.id === this.selectedComponentId,
        transform: { position: { ...component.position }, rotation: { ...component.rotation }, scale: { ...component.scale } },
        materials: component.materials.map((material) => ({ ...material })),
        boundingBox: { ...component.dimensions },
        meshes: component.meshCount,
        vertices: component.vertices,
        triangles: component.triangles,
      },
    };
  }

  getSelectedComponent() {
    if (!this.model) return { selected: false, code: "NO_MODEL", message: "Nenhum modelo está carregado no Engineering Core." };
    const component = this.model.components.get(this.selectedComponentId);
    return component
      ? { selected: true, component: this.summary(component) }
      : { selected: false, code: "NO_SELECTION", message: "Nenhum componente está selecionado." };
  }

  getExplosionState() {
    if (!this.model) return { available: false, code: "NO_MODEL", message: "Nenhum modelo está carregado no Engineering Core." };
    const state = this.model.explosion.getState();
    return { available: true, ...state, selectedRoot: state.selectedRootId ?? null };
  }

  selectComponent(reference: string | undefined, source: EngineeringCommandSource = "ai"): EngineeringActionResult {
    const resolution = this.resolve(reference, true);
    if (!resolution.component) return this.fail("select_component", resolution, reference, source);
    this.selectedComponentId = resolution.component.id;
    this.modelMode = "component";
    this.componentRevision += 1;
    const result = this.ok("select_component", resolution.component, `Componente ${resolution.component.name} selecionado.`, source);
    return result;
  }

  focusComponent(reference: string | undefined, source: EngineeringCommandSource = "ai"): EngineeringActionResult {
    const resolution = this.resolve(reference, true);
    if (!resolution.component) return this.fail("focus_component", resolution, reference, source);
    this.focusRequest = { sequence: this.focusRequest.sequence + 1, componentId: resolution.component.id };
    const result = this.ok("focus_component", resolution.component, `Foco ajustado para ${resolution.component.name}.`, source);
    return result;
  }

  isolateComponent(reference: string | undefined, source: EngineeringCommandSource = "ai"): EngineeringActionResult {
    const resolution = this.resolve(reference, true);
    if (!resolution.component || !this.model) return this.fail("isolate_component", resolution, reference, source);
    this.model.components.isolate(resolution.component.id);
    this.selectedComponentId = resolution.component.id;
    this.modelMode = "component";
    this.componentRevision += 1;
    const result = this.ok("isolate_component", resolution.component, `Componente ${resolution.component.name} isolado.`, source);
    return result;
  }

  showAllComponents(source: EngineeringCommandSource = "ai"): EngineeringActionResult {
    if (!this.model) return this.noModel("show_all_components", source);
    this.model.components.showAll();
    this.componentRevision += 1;
    const result = this.recordResult("show_all_components", null, true, "Visibilidade de todos os componentes restaurada.", source);
    return result;
  }

  hideComponent(reference: string | undefined, source: EngineeringCommandSource = "ai"): EngineeringActionResult {
    return this.changeVisibility("hide_component", reference, false, source);
  }

  showComponent(reference: string | undefined, source: EngineeringCommandSource = "ai"): EngineeringActionResult {
    return this.changeVisibility("show_component", reference, true, source);
  }

  setExplosionFactor(factor: number, source: EngineeringCommandSource = "ai"): EngineeringActionResult {
    if (!this.model) return this.noModel("set_explosion_factor", source);
    if (!Number.isFinite(factor) || factor < 0 || factor > 1) return this.recordResult("set_explosion_factor", null, false, "O fator de explosão deve estar entre 0 e 1.", source, "INVALID_FACTOR");
    const current = this.model.explosion.getState();
    if (!current.enabled) return this.recordResult("set_explosion_factor", null, false, "A montagem atual não possui subcomponentes suficientes para explosão.", source, "EXPLOSION_UNAVAILABLE");
    const next = this.model.explosion.applyFactor(factor);
    this.componentRevision += 1;
    const result = this.recordResult("set_explosion_factor", `${Math.round(factor * 100)}%`, true, `Explosão ajustada para ${Math.round(factor * 100)}%.`, source, "OK", next);
    return result;
  }

  adjustExplosion(delta: number, source: EngineeringCommandSource = "ai"): EngineeringActionResult {
    if (!this.model) return this.noModel("set_explosion_factor", source);
    const current = this.model.explosion.getState().factor;
    return this.setExplosionFactor(Math.max(0, Math.min(1, current + delta)), source);
  }

  explodeAll(source: EngineeringCommandSource = "ai"): EngineeringActionResult {
    if (!this.model) return this.noModel("explode_all", source);
    const configured = this.model.explosion.configure("all");
    if (!configured.success) return this.recordResult("explode_all", null, false, configured.error ?? "Não foi possível explodir a montagem.", source, "EXPLOSION_UNAVAILABLE");
    const state = this.model.explosion.applyFactor(1, "exploded");
    this.componentRevision += 1;
    const result = this.recordResult("explode_all", this.model.metadata.name, true, "Montagem explodida.", source, "OK", state);
    return result;
  }

  explodeComponent(reference: string | undefined, source: EngineeringCommandSource = "ai"): EngineeringActionResult {
    const resolution = this.resolve(reference, true);
    if (!resolution.component || !this.model) return this.fail("explode_component", resolution, reference, source);
    const previous = this.model.explosion.getState();
    const configured = this.model.explosion.configure("selected", resolution.component.id);
    if (!configured.success) {
      const restored = this.model.explosion.configure(previous.mode, previous.selectedRootId);
      if (restored.success) this.model.explosion.applyFactor(previous.factor, previous.assemblyState);
      this.componentRevision += 1;
      return this.recordResult("explode_component", resolution.component.name, false, configured.error ?? "Não foi possível explodir o componente.", source, "LEAF_COMPONENT");
    }
    const state = this.model.explosion.applyFactor(1, "exploded");
    this.selectedComponentId = resolution.component.id;
    this.modelMode = "component";
    this.componentRevision += 1;
    const result = this.recordResult("explode_component", resolution.component.name, true, `Submontagem ${resolution.component.name} explodida.`, source, "OK", state, resolution.component);
    return result;
  }

  reassemble(source: EngineeringCommandSource = "ai"): EngineeringActionResult {
    if (!this.model) return this.noModel("reassemble", source);
    const current = this.model.explosion.getState();
    const state = current.enabled ? this.model.explosion.applyFactor(0, "assembled") : current;
    this.componentRevision += 1;
    const result = this.recordResult("reassemble", this.model.metadata.name, true, "Montagem reconstruída.", source, "OK", state);
    return result;
  }

  resetModelView(source: EngineeringCommandSource = "ai"): EngineeringActionResult {
    if (!this.model) return this.noModel("reset_model_view", source);
    this.focusRequest = { sequence: this.focusRequest.sequence + 1, componentId: null };
    this.resetSignal += 1;
    const result = this.recordResult("reset_model_view", this.model.metadata.name, true, "Visualização do modelo restaurada.", source);
    return result;
  }

  private resolve(reference: string | undefined, allowSelected: boolean): { component: ModelComponent | null; candidates: ModelComponent[]; exact: boolean; code?: string; message?: string } {
    if (!this.model) return { component: null, candidates: [], exact: false, code: "NO_MODEL", message: "Nenhum modelo está carregado no Engineering Core." };
    const raw = reference?.trim() ?? "";
    const normalized = normalize(raw);
    if (allowSelected && (!normalized || contextualReference.test(normalized))) {
      const selected = this.model.components.get(this.selectedComponentId);
      return selected
        ? { component: selected, candidates: [selected], exact: true }
        : { component: null, candidates: [], exact: false, code: "NO_SELECTION", message: "Nenhum componente está selecionado." };
    }
    const components = this.model.components.list();
    const byId = components.find((component) => component.id === raw);
    if (byId) return { component: byId, candidates: [byId], exact: true };
    const exact = components.filter((component) => normalize(component.name) === normalized || normalize(component.semanticLabel ?? "") === normalized);
    if (exact.length === 1) return { component: exact[0], candidates: exact, exact: true };
    if (exact.length > 1) return { component: null, candidates: exact, exact: true, code: "AMBIGUOUS", message: "Encontrei mais de um componente com esse nome." };
    const partial = components.filter((component) => normalize(component.name).includes(normalized) || normalize(component.semanticLabel ?? "").includes(normalized));
    if (partial.length === 1) return { component: partial[0], candidates: partial, exact: false };
    if (partial.length > 1) return { component: null, candidates: partial, exact: false, code: "AMBIGUOUS", message: "Encontrei mais de um componente compatível. Escolha um deles." };
    return { component: null, candidates: [], exact: false, code: "NOT_FOUND", message: `Não encontrei nenhum componente chamado "${raw}" no modelo atual.` };
  }

  private summary(component: ModelComponent): EngineeringComponentSummary {
    const parent = this.model?.components.get(component.parentId);
    return { id: component.id, name: component.name, semanticLabel: component.semanticLabel, type: component.type, parent: parent?.name ?? null, children: component.children.length, visible: component.visible };
  }

  private resolutionError(resolution: ReturnType<EngineeringSessionService["resolve"]>, reference?: string): EngineeringActionResult {
    return { success: false, code: resolution.code ?? "NOT_FOUND", message: resolution.message ?? `Não encontrei o componente "${reference ?? ""}".`, candidates: resolution.candidates.map((candidate) => this.summary(candidate)) };
  }

  private fail(command: string, resolution: ReturnType<EngineeringSessionService["resolve"]>, reference: string | undefined, source: EngineeringCommandSource): EngineeringActionResult {
    const error = this.resolutionError(resolution, reference);
    return this.recordResult(command, reference ?? null, false, error.message, source, error.code, undefined, undefined, error.candidates);
  }

  private noModel(command: string, source: EngineeringCommandSource): EngineeringActionResult {
    return this.recordResult(command, null, false, "Nenhum modelo está carregado no Engineering Core.", source, "NO_MODEL");
  }

  private ok(command: string, component: ModelComponent, message: string, source: EngineeringCommandSource): EngineeringActionResult {
    return this.recordResult(command, component.name, true, message, source, "OK", undefined, component);
  }

  private changeVisibility(command: "hide_component" | "show_component", reference: string | undefined, visible: boolean, source: EngineeringCommandSource): EngineeringActionResult {
    const resolution = this.resolve(reference, true);
    if (!resolution.component || !this.model) return this.fail(command, resolution, reference, source);
    if (visible) this.model.components.show(resolution.component.id); else this.model.components.hide(resolution.component.id);
    this.componentRevision += 1;
    const current = this.model.components.get(resolution.component.id) ?? resolution.component;
    const result = this.ok(command, current, `${resolution.component.name} agora está ${visible ? "visível" : "oculto"}.`, source);
    return result;
  }

  private recordResult(command: string, target: string | null, success: boolean, message: string, source: EngineeringCommandSource, code = success ? "OK" : "ERROR", explosion?: ExplosionState, component?: ModelComponent, candidates?: EngineeringComponentSummary[]): EngineeringActionResult {
    this.commandSequence += 1;
    const entry: EngineeringCommandLog = { id: this.commandSequence, timestamp: new Date().toISOString(), command, target, source, status: success ? "success" : "error", message };
    this.commands = [entry, ...this.commands].slice(0, 12);
    this.emit();
    return { success, code, message, component: component ? this.summary(component) : undefined, candidates, explosion: explosion ? cloneExplosion(explosion) : undefined };
  }

  private emit(): void {
    this.listeners.forEach((listener) => listener());
  }
}

export const engineeringSessionService = new EngineeringSessionService();
