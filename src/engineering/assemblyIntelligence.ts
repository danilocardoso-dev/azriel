import type { AssemblyIntelligenceSnapshot, ComponentSemantic, EngineeringSubsystem, ModelComponent, SemanticCoverageStatus } from "./types";

export interface SemanticComponentView {
  component: ModelComponent;
  semantic?: ComponentSemantic;
  subsystem?: EngineeringSubsystem;
  status: SemanticCoverageStatus;
  binding: "exact" | "rebound" | "review_required" | "none";
}

export function semanticStatus(semantic?: ComponentSemantic): SemanticCoverageStatus {
  if (!semantic?.semanticLabel.trim()) return "unclassified";
  return semantic.subsystemId && semantic.role.trim() ? "classified" : "partial";
}

export function bindAssemblyIntelligence(components: ModelComponent[], snapshot: AssemblyIntelligenceSnapshot): SemanticComponentView[] {
  const semantics = new Map(snapshot.semantics.map((item) => [item.componentIdentity, item]));
  const subsystems = new Map(snapshot.subsystems.map((item) => [item.id, item]));
  return components.map((component) => {
    let semantic = semantics.get(component.persistentIdentity);
    let binding: SemanticComponentView["binding"] = semantic ? "exact" : "none";
    if (!semantic) {
      const candidates = snapshot.semantics.filter((item) => item.originalName === component.originalName && item.componentType === component.type);
      const currentMatches = components.filter((item) => item.originalName === component.originalName && item.type === component.type);
      if (candidates.length === 1 && currentMatches.length === 1) { semantic = candidates[0]; binding = "rebound"; }
      else if (candidates.length) binding = "review_required";
    }
    return { component, semantic, binding, subsystem: semantic?.subsystemId ? subsystems.get(semantic.subsystemId) : undefined, status: semanticStatus(semantic) };
  });
}

export function semanticCoverage(views: SemanticComponentView[]) {
  const selectable = views.filter((item) => item.component.selectable && item.component.children.length === 0);
  const counts = { classified: 0, partial: 0, unclassified: 0 };
  selectable.forEach((item) => { counts[item.status] += 1; });
  return { total: selectable.length, ...counts, percent: selectable.length ? Math.round((counts.classified / selectable.length) * 100) : 0 };
}

export function searchSemanticComponents(views: SemanticComponentView[], query: string): SemanticComponentView[] {
  const needle = query.trim().toLocaleLowerCase("pt-BR");
  if (!needle) return views;
  return views.filter(({ component, semantic, subsystem }) => [component.originalName, semantic?.semanticLabel, semantic?.role, subsystem?.name].some((value) => value?.toLocaleLowerCase("pt-BR").includes(needle)));
}

export function semanticLabelsForComponents(components: ModelComponent[], snapshot: AssemblyIntelligenceSnapshot) {
  return new Map(bindAssemblyIntelligence(components, snapshot).filter((item) => item.semantic && item.binding !== "review_required").map((item) => [item.component.persistentIdentity, item.semantic!.semanticLabel]));
}

export function assemblyExport(modelIdentity: string, snapshot: AssemblyIntelligenceSnapshot) {
  return JSON.stringify({ schema: "azriel.assembly-intelligence.v1", exportedAt: new Date().toISOString(), modelIdentity, ...snapshot }, null, 2);
}
