import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { useMemo, useState } from "react";
import { assemblyExport, bindAssemblyIntelligence, searchSemanticComponents, semanticCoverage } from "../../engineering/assemblyIntelligence";
import type { AssemblyIntelligenceSnapshot, ComponentRelationshipType, ModelComponent, ModelFormat, SemanticCoverageStatus } from "../../engineering/types";
import { engineeringRepository } from "../../repositories/engineeringRepository";

type Tab = "model" | "components" | "assembly" | "semantics";
const emptySnapshot: AssemblyIntelligenceSnapshot = { semantics: [], subsystems: [], relationships: [] };
const relationshipTypes: ComponentRelationshipType[] = ["connected_to", "contains", "supports", "drives", "mounted_on", "adjacent_to", "depends_on", "custom"];

interface Props {
  modelIdentity: string;
  modelName: string;
  modelFormat: ModelFormat;
  components: ModelComponent[];
  selectedComponentId: string | null;
  snapshot?: AssemblyIntelligenceSnapshot;
  onSnapshotChange: (snapshot: AssemblyIntelligenceSnapshot) => void;
  onSelectComponent: (sessionId: string) => void;
}

export function AssemblyIntelligencePanel({ modelIdentity, modelName, modelFormat, components, selectedComponentId, snapshot = emptySnapshot, onSnapshotChange, onSelectComponent }: Props) {
  const [tab, setTab] = useState<Tab>("components");
  const [query, setQuery] = useState("");
  const [statusFilter, setStatusFilter] = useState<SemanticCoverageStatus | "all">("all");
  const [subsystemFilter, setSubsystemFilter] = useState("all");
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const selected = components.find((item) => item.id === selectedComponentId) ?? null;
  const views = useMemo(() => bindAssemblyIntelligence(components, snapshot), [components, snapshot]);
  const coverage = useMemo(() => semanticCoverage(views), [views]);
  const selectedView = selected ? views.find((item) => item.component.id === selected.id) : undefined;
  const filtered = useMemo(() => searchSemanticComponents(views, query).filter((item) => (statusFilter === "all" || item.status === statusFilter) && (subsystemFilter === "all" || item.semantic?.subsystemId === subsystemFilter)), [query, statusFilter, subsystemFilter, views]);
  const [semanticDrafts, setSemanticDrafts] = useState<Record<string, { semanticLabel: string; subsystemId: string; role: string; description: string; notes: string }>>({});
  const [subsystemForm, setSubsystemForm] = useState({ id: "", name: "", description: "", parentSubsystemId: "" });
  const [relationshipForm, setRelationshipForm] = useState({ source: "", target: "", type: "connected_to" as ComponentRelationshipType, description: "" });

  const semanticForm = selected ? semanticDrafts[selected.persistentIdentity] ?? { semanticLabel: selectedView?.semantic?.semanticLabel ?? "", subsystemId: selectedView?.semantic?.subsystemId ?? "", role: selectedView?.semantic?.role ?? "", description: selectedView?.semantic?.description ?? "", notes: selectedView?.semantic?.notes ?? "" } : { semanticLabel: "", subsystemId: "", role: "", description: "", notes: "" };
  const setSemanticForm = (value: typeof semanticForm) => { if (selected) setSemanticDrafts((current) => ({ ...current, [selected.persistentIdentity]: value })); };
  const relationshipSource = relationshipForm.source || selected?.persistentIdentity || "";

  const refresh = async () => onSnapshotChange(await engineeringRepository.getAssemblyIntelligence(modelIdentity));
  const run = async (operation: () => Promise<void>, success: string) => {
    setBusy(true); setMessage(null);
    try { await operation(); await refresh(); setMessage(success); }
    catch (reason) { setMessage(reason instanceof Error ? reason.message : String(reason)); }
    finally { setBusy(false); }
  };

  const saveSemantic = () => {
    if (!selected) return;
    void run(async () => { await engineeringRepository.saveSemantic({ modelIdentity, componentIdentity: selected.persistentIdentity, semanticLabel: semanticForm.semanticLabel, subsystemId: semanticForm.subsystemId || null, role: semanticForm.role, description: semanticForm.description, notes: semanticForm.notes }); }, "SEMÂNTICA SALVA");
  };
  const saveSubsystem = () => {
    const id = subsystemForm.id || crypto.randomUUID();
    void run(async () => { await engineeringRepository.saveSubsystem({ id, modelIdentity, name: subsystemForm.name, description: subsystemForm.description, parentSubsystemId: subsystemForm.parentSubsystemId || null }); setSubsystemForm({ id: "", name: "", description: "", parentSubsystemId: "" }); }, "SUBSISTEMA SALVO");
  };
  const saveRelationship = () => void run(async () => {
    await engineeringRepository.saveRelationship({ id: crypto.randomUUID(), modelIdentity, sourceComponentIdentity: relationshipSource, targetComponentIdentity: relationshipForm.target, relationshipType: relationshipForm.type, description: relationshipForm.description });
    setRelationshipForm((current) => ({ ...current, target: "", description: "" }));
  }, "RELAÇÃO SALVA");
  const exportJson = async () => {
    const path = await save({ title: "Exportar Assembly Intelligence", defaultPath: `${modelName.replace(/\.[^.]+$/, "")}-assembly.json`, filters: [{ name: "JSON", extensions: ["json"] }] });
    if (!path) return;
    await invoke("write_engineering_semantics_export", { path, content: assemblyExport(modelIdentity, snapshot) });
    setMessage("JSON EXPORTADO");
  };

  const graphComponents = views.filter((item) => item.component.selectable).slice(0, 80);
  const graphPositions = new Map(graphComponents.map((item, index) => {
    const angle = (index / Math.max(graphComponents.length, 1)) * Math.PI * 2 - Math.PI / 2;
    return [item.component.persistentIdentity, { x: 300 + Math.cos(angle) * 220, y: 190 + Math.sin(angle) * 145 }] as const;
  }));

  return <section className="assembly-intelligence">
    <header className="assembly-intelligence__header">
      <div><span>ENGINEERING CORE // V0.7</span><strong>ASSEMBLY INTELLIGENCE</strong></div>
      <div className="assembly-intelligence__coverage"><b>{coverage.percent}%</b><span>CLASSIFIED</span><small>{coverage.classified} OK / {coverage.partial} PARTIAL / {coverage.unclassified} OPEN</small></div>
    </header>
    <nav>{(["model", "components", "assembly", "semantics"] as Tab[]).map((item) => <button key={item} className={tab === item ? "active" : ""} onClick={() => setTab(item)}>{item.toUpperCase()}</button>)}<button onClick={() => void exportJson()}>EXPORT JSON</button></nav>
    {message && <p className="assembly-intelligence__message">{message}</p>}

    {tab === "model" && <div className="assembly-intelligence__model"><dl>
      <div><dt>MODEL</dt><dd>{modelName}</dd></div><div><dt>FORMAT</dt><dd>{modelFormat}</dd></div><div><dt>IDENTITY</dt><dd>{modelIdentity.slice(0, 20)}…</dd></div><div><dt>COMPONENTS</dt><dd>{components.length}</dd></div><div><dt>SELECTABLE</dt><dd>{coverage.total}</dd></div><div><dt>RELATIONSHIPS</dt><dd>{snapshot.relationships.length}</dd></div>
    </dl><p>A identidade usa o conteúdo do arquivo; nenhum caminho local é persistido.</p></div>}

    {tab === "components" && <div className="assembly-intelligence__workspace">
      <section className="assembly-intelligence__list"><div className="assembly-intelligence__filters"><input value={query} onChange={(e) => setQuery(e.target.value)} placeholder="Buscar nome, rótulo, função ou subsistema"/><select value={statusFilter} onChange={(e) => setStatusFilter(e.target.value as typeof statusFilter)}><option value="all">TODOS STATUS</option><option value="unclassified">UNCLASSIFIED</option><option value="partial">PARTIAL</option><option value="classified">CLASSIFIED</option></select><select value={subsystemFilter} onChange={(e) => setSubsystemFilter(e.target.value)}><option value="all">TODOS SUBSISTEMAS</option>{snapshot.subsystems.map((item) => <option key={item.id} value={item.id}>{item.name}</option>)}</select></div>
        <div className="assembly-intelligence__rows">{filtered.map((item) => <button key={item.component.id} className={item.component.id === selectedComponentId ? "selected" : ""} onClick={() => onSelectComponent(item.component.id)}><span data-status={item.binding === "review_required" ? "review" : item.status}>{item.binding === "review_required" ? "REVIEW REQUIRED" : item.status.toUpperCase()}</span><strong>{item.semantic?.semanticLabel || item.component.originalName}</strong><small>{item.subsystem?.name ?? "SEM SUBSISTEMA"} // {item.semantic?.role || item.component.type}{item.binding === "rebound" ? " // SAFE REBIND" : ""}</small></button>)}</div>
      </section>
      <SemanticEditor selected={selected} form={semanticForm} setForm={setSemanticForm} subsystems={snapshot.subsystems} busy={busy} onSave={saveSemantic}/>
    </div>}

    {tab === "semantics" && <div className="assembly-intelligence__workspace">
      <section className="assembly-intelligence__list"><header><strong>SUBSYSTEMS</strong><span>{snapshot.subsystems.length}</span></header><div className="assembly-intelligence__rows">{snapshot.subsystems.map((item) => <button key={item.id} onClick={() => setSubsystemForm({ id: item.id, name: item.name, description: item.description, parentSubsystemId: item.parentSubsystemId ?? "" })}><strong>{item.name}</strong><small>{snapshot.subsystems.find((candidate) => candidate.id === item.parentSubsystemId)?.name ?? "ROOT"}</small></button>)}</div></section>
      <section className="assembly-intelligence__editor"><header><strong>{subsystemForm.id ? "EDIT SUBSYSTEM" : "NEW SUBSYSTEM"}</strong></header><label>NAME<input value={subsystemForm.name} onChange={(e) => setSubsystemForm({ ...subsystemForm, name: e.target.value })}/></label><label>PARENT<select value={subsystemForm.parentSubsystemId} onChange={(e) => setSubsystemForm({ ...subsystemForm, parentSubsystemId: e.target.value })}><option value="">ROOT</option>{snapshot.subsystems.filter((item) => item.id !== subsystemForm.id).map((item) => <option key={item.id} value={item.id}>{item.name}</option>)}</select></label><label>DESCRIPTION<textarea value={subsystemForm.description} onChange={(e) => setSubsystemForm({ ...subsystemForm, description: e.target.value })}/></label><div><button disabled={busy || !subsystemForm.name.trim()} onClick={saveSubsystem}>SAVE</button>{subsystemForm.id && <button disabled={busy} onClick={() => void run(async () => { await engineeringRepository.deleteSubsystem(modelIdentity, subsystemForm.id); setSubsystemForm({ id: "", name: "", description: "", parentSubsystemId: "" }); }, "SUBSISTEMA EXCLUÍDO")}>DELETE</button>}<button onClick={() => setSubsystemForm({ id: "", name: "", description: "", parentSubsystemId: "" })}>CLEAR</button></div></section>
    </div>}

    {tab === "assembly" && <div className="assembly-intelligence__assembly">
      <section className="assembly-intelligence__graph"><svg viewBox="0 0 600 380" role="img" aria-label="Assembly Graph">{snapshot.relationships.map((relation) => { const from = graphPositions.get(relation.sourceComponentIdentity); const to = graphPositions.get(relation.targetComponentIdentity); return from && to ? <line key={relation.id} x1={from.x} y1={from.y} x2={to.x} y2={to.y}/> : null; })}{graphComponents.map((item) => { const point = graphPositions.get(item.component.persistentIdentity)!; return <g key={item.component.id} transform={`translate(${point.x} ${point.y})`} onClick={() => onSelectComponent(item.component.id)} className={item.component.id === selectedComponentId ? "selected" : ""}><circle r="8"/><text y="-13" textAnchor="middle">{(item.semantic?.semanticLabel || item.component.originalName).slice(0, 22)}</text></g>; })}</svg>{components.filter((item) => item.selectable).length > 80 && <small>GRAPH LIMIT // 80 OF {coverage.total}</small>}</section>
      <section className="assembly-intelligence__relations"><header><strong>RELATIONSHIPS</strong><span>{snapshot.relationships.length}</span></header><div className="assembly-intelligence__relation-form"><select value={relationshipSource} onChange={(e) => setRelationshipForm({ ...relationshipForm, source: e.target.value })}><option value="">SOURCE</option>{components.filter((item) => item.selectable).map((item) => <option key={item.id} value={item.persistentIdentity}>{item.semanticLabel || item.originalName}</option>)}</select><select value={relationshipForm.type} onChange={(e) => setRelationshipForm({ ...relationshipForm, type: e.target.value as ComponentRelationshipType })}>{relationshipTypes.map((item) => <option key={item}>{item.toUpperCase()}</option>)}</select><select value={relationshipForm.target} onChange={(e) => setRelationshipForm({ ...relationshipForm, target: e.target.value })}><option value="">TARGET</option>{components.filter((item) => item.selectable && item.persistentIdentity !== relationshipSource).map((item) => <option key={item.id} value={item.persistentIdentity}>{item.semanticLabel || item.originalName}</option>)}</select><input value={relationshipForm.description} onChange={(e) => setRelationshipForm({ ...relationshipForm, description: e.target.value })} placeholder="Descrição opcional"/><button disabled={busy || !relationshipSource || !relationshipForm.target} onClick={saveRelationship}>ADD RELATION</button></div><div className="assembly-intelligence__rows">{snapshot.relationships.map((relation) => <article key={relation.id}><button onClick={() => { const component = components.find((item) => item.persistentIdentity === relation.sourceComponentIdentity); if (component) onSelectComponent(component.id); }}>{relation.relationshipType.toUpperCase()} // {labelOf(views, relation.sourceComponentIdentity)} → {labelOf(views, relation.targetComponentIdentity)}</button><button onClick={() => void run(() => engineeringRepository.deleteRelationship(modelIdentity, relation.id), "RELAÇÃO EXCLUÍDA")}>×</button></article>)}</div></section>
    </div>}
  </section>;
}

function labelOf(views: ReturnType<typeof bindAssemblyIntelligence>, identity: string) { const item = views.find((view) => view.component.persistentIdentity === identity); return item?.semantic?.semanticLabel || item?.component.originalName || identity.slice(-18); }

function SemanticEditor({ selected, form, setForm, subsystems, busy, onSave }: { selected: ModelComponent | null; form: { semanticLabel: string; subsystemId: string; role: string; description: string; notes: string }; setForm: (value: typeof form) => void; subsystems: AssemblyIntelligenceSnapshot["subsystems"]; busy: boolean; onSave: () => void }) {
  return <section className="assembly-intelligence__editor"><header><strong>COMPONENT SEMANTICS</strong><span>{selected?.originalName ?? "NONE"}</span></header>{selected ? <><label>SEMANTIC LABEL<input value={form.semanticLabel} onChange={(e) => setForm({ ...form, semanticLabel: e.target.value })}/></label><label>SUBSYSTEM<select value={form.subsystemId} onChange={(e) => setForm({ ...form, subsystemId: e.target.value })}><option value="">SEM SUBSISTEMA</option>{subsystems.map((item) => <option key={item.id} value={item.id}>{item.name}</option>)}</select></label><label>ROLE<input value={form.role} onChange={(e) => setForm({ ...form, role: e.target.value })}/></label><label>DESCRIPTION<textarea value={form.description} onChange={(e) => setForm({ ...form, description: e.target.value })}/></label><label>NOTES<textarea value={form.notes} onChange={(e) => setForm({ ...form, notes: e.target.value })}/></label><button disabled={busy} onClick={onSave}>SAVE SEMANTICS</button></> : <p>SELECIONE UM COMPONENTE NA LISTA, ÁRVORE OU VIEWPORT.</p>}</section>;
}
