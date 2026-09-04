import { useEffect, useMemo, useState } from "react";
import { KnowledgeEditor } from "../components/core/KnowledgeEditor";
import { DeleteConfirmationDialog } from "../components/daily/DeleteConfirmationDialog";
import { DetailsDrawer } from "../components/hud/DetailsDrawer";
import { HudPanel } from "../components/hud/HudPanel";
import { GapDiagnostics } from "../components/knowledge/GapDiagnostics";
import { KnowledgeMetrics } from "../components/knowledge/KnowledgeMetrics";
import { StarkChart } from "../components/knowledge/StarkChart";
import { ModuleIntro } from "../components/layout/ModuleIntro";
import { RoadmapEditor } from "../components/stark/RoadmapEditor";
import { ResearchEditor } from "../components/stark/ResearchEditor";
import { useAzrielData } from "../contexts/useAzrielData";
import { knowledgeService } from "../services/knowledgeService";
import { starkService } from "../services/starkService";
import type { KnowledgeArea, KnowledgeBaseline, KnowledgeEvent, KnowledgeHistory, KnowledgeInput, ResearchInput, ResearchItem, StarkSummary, StudyRoadmap, StudyRoadmapInput } from "../types";

type StarkTab = "overview" | "knowledge" | "roadmaps" | "research" | "evolution" | "gaps";
const tabs: Array<{ id: StarkTab; label: string }> = [{ id: "overview", label: "VISÃO GERAL" }, { id: "knowledge", label: "CONHECIMENTO" }, { id: "roadmaps", label: "ROADMAPS" }, { id: "research", label: "PESQUISA" }, { id: "evolution", label: "EVOLUÇÃO" }, { id: "gaps", label: "LACUNAS" }];
const roadmapStatus = { planned: "PLANEJADO", active: "ATIVO", paused: "PAUSADO", completed: "CONCLUÍDO" };
const researchStatus = { planned: "PLANEJADA", active: "ATIVA", paused: "PAUSADA", completed: "CONCLUÍDA" };

export function StarkMapPage() {
  const { knowledgeAreas, projects, databaseInfo, saveKnowledge, deleteKnowledge } = useAzrielData();
  const [tab, setTab] = useState<StarkTab>("overview");
  const [roadmaps, setRoadmaps] = useState<StudyRoadmap[]>([]);
  const [research, setResearch] = useState<ResearchItem[]>([]);
  const [baselines, setBaselines] = useState<KnowledgeBaseline[]>([]);
  const [events, setEvents] = useState<KnowledgeEvent[]>([]);
  const [summary, setSummary] = useState<StarkSummary | null>(null);
  const [selectedKnowledge, setSelectedKnowledge] = useState<KnowledgeArea | null>(null);
  const [editingKnowledge, setEditingKnowledge] = useState<KnowledgeArea | "new" | null>(null);
  const [editingRoadmap, setEditingRoadmap] = useState<StudyRoadmap | "new" | null>(null);
  const [editingResearch, setEditingResearch] = useState<ResearchItem | "new" | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<{ kind: "knowledge" | "roadmap" | "research"; id: string; title: string } | null>(null);
  const [evolutionAreaId, setEvolutionAreaId] = useState("");
  const [history, setHistory] = useState<KnowledgeHistory[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadStark = async () => {
    setLoading(true); setError(null);
    try {
      const [nextRoadmaps, nextResearch, nextBaselines, nextEvents, nextSummary] = await Promise.all([starkService.roadmaps(), starkService.research(), starkService.baselines(), starkService.events(), starkService.summary()]);
      setRoadmaps(nextRoadmaps); setResearch(nextResearch); setBaselines(nextBaselines); setEvents(nextEvents); setSummary(nextSummary);
    } catch (reason) { setError(reason instanceof Error ? reason.message : String(reason)); }
    finally { setLoading(false); }
  };
  useEffect(() => {
    let active = true;
    Promise.all([starkService.roadmaps(), starkService.research(), starkService.baselines(), starkService.events(), starkService.summary()])
      .then(([nextRoadmaps, nextResearch, nextBaselines, nextEvents, nextSummary]) => {
        if (!active) return;
        setRoadmaps(nextRoadmaps); setResearch(nextResearch); setBaselines(nextBaselines); setEvents(nextEvents); setSummary(nextSummary); setLoading(false);
      })
      .catch((reason) => {
        if (!active) return;
        setError(reason instanceof Error ? reason.message : String(reason)); setLoading(false);
      });
    return () => { active = false; };
  }, []);

  async function selectEvolutionArea(id: string) {
    setEvolutionAreaId(id);
    if (!id) { setHistory([]); return; }
    try { setHistory(await knowledgeService.history(id)); }
    catch (reason) { setError(reason instanceof Error ? reason.message : String(reason)); }
  }

  const averages = useMemo(() => ({ coverage: knowledgeAreas.length ? Math.round(knowledgeAreas.reduce((sum, item) => sum + item.coverage, 0) / knowledgeAreas.length) : 0, depth: knowledgeAreas.length ? Math.round(knowledgeAreas.reduce((sum, item) => sum + item.depth, 0) / knowledgeAreas.length) : 0 }), [knowledgeAreas]);
  const groups = useMemo(() => [...new Set(knowledgeAreas.map((area) => area.category))], [knowledgeAreas]);
  const activeRoadmaps = roadmaps.filter((item) => item.status === "active");
  const activeResearch = research.filter((item) => item.status === "active");

  async function saveRoadmap(input: StudyRoadmapInput) {
    const result = await starkService.saveRoadmap(input);
    setRoadmaps(result.roadmaps);
    setEvents(await starkService.events());
    setSummary(await starkService.summary());
    setEditingRoadmap(null);
  }
  async function saveResearch(input: ResearchInput) { setResearch(await starkService.saveResearch(input)); setSummary(await starkService.summary()); setEditingResearch(null); }
  async function saveKnowledgeInput(input: KnowledgeInput) { await saveKnowledge(input); setEditingKnowledge(null); setSelectedKnowledge(null); }
  async function remove() {
    if (!deleteTarget) return;
    if (deleteTarget.kind === "knowledge") { await deleteKnowledge(deleteTarget.id); setSelectedKnowledge(null); }
    if (deleteTarget.kind === "roadmap") setRoadmaps(await starkService.deleteRoadmap(deleteTarget.id));
    if (deleteTarget.kind === "research") setResearch(await starkService.deleteResearch(deleteTarget.id));
    setSummary(await starkService.summary()); setDeleteTarget(null);
  }

  const knowledgeName = (id: string | null) => knowledgeAreas.find((item) => item.id === id)?.name ?? "SEM VÍNCULO";
  const roadmapName = (id: string | null) => roadmaps.find((item) => item.id === id)?.name ?? "SEM VÍNCULO";
  const projectName = (id: string | null) => projects.find((item) => item.id === id)?.name ?? "SEM VÍNCULO";

  return <>
    <ModuleIntro code="STK-08" title="Mapa Stark" description="Central integrada de conhecimento, aprendizado, pesquisa e evolução." metric="STARK KNOWLEDGE SYSTEM" />
    <nav className="stark-tabs" aria-label="Áreas do Mapa Stark">{tabs.map((item) => <button key={item.id} className={tab === item.id ? "active" : ""} onClick={() => setTab(item.id)}>{item.label}</button>)}</nav>
    {error && <div className="form-error stark-error">{error}<button onClick={() => void loadStark()}>TENTAR NOVAMENTE</button></div>}
    {loading ? <div className="core-empty">SINCRONIZANDO STARK KNOWLEDGE SYSTEM...</div> : <>
      {tab === "overview" && <div className="stark-dashboard">
        <div className="stark-kpis"><article><span>COBERTURA</span><strong>{averages.coverage}%</strong><i>BASE REAL</i></article><article><span>PROFUNDIDADE</span><strong>{averages.depth}%</strong><i>BASE REAL</i></article><article><span>INTEGRAÇÃO</span><strong>{Math.round(databaseInfo?.integrationValue ?? 0)}%</strong><i>MÉTRICA GLOBAL</i></article><article><span>ROADMAPS ATIVOS</span><strong>{summary?.activeRoadmapCount ?? 0}</strong><i>{summary?.roadmapCount ?? 0} CADASTRADOS</i></article><article><span>PESQUISAS ATIVAS</span><strong>{summary?.activeResearchCount ?? 0}</strong><i>{summary?.researchCount ?? 0} REGISTRADAS</i></article><article><span>BASELINES</span><strong>{summary?.baselineCount ?? 0}</strong><i>ESTADO PRESERVADO</i></article></div>
        <div className="stark-layout"><HudPanel title="Cobertura × Profundidade" code={`${knowledgeAreas.length} NÓS`}><StarkChart areas={knowledgeAreas} selectedId={selectedKnowledge?.id} onSelect={setSelectedKnowledge} /></HudPanel><HudPanel title="Prioridades e lacunas" code="DIAGNÓSTICO"><GapDiagnostics onSelect={setSelectedKnowledge} /></HudPanel></div>
        <div className="stark-overview-grid"><HudPanel title="Roadmaps ativos" code={`${activeRoadmaps.length} EM CURSO`}><div className="stark-compact-list">{activeRoadmaps.length ? activeRoadmaps.map((item) => <button key={item.id} onClick={() => { setTab("roadmaps"); setEditingRoadmap(item); }}><strong>{item.name}</strong><span>{item.completedActivities}/{item.totalActivities} ATIVIDADES</span><i>{item.progress}%</i></button>) : <p>Nenhum roadmap ativo.</p>}</div></HudPanel><HudPanel title="Pesquisas ativas" code={`${activeResearch.length} FRENTES`}><div className="stark-compact-list">{activeResearch.length ? activeResearch.map((item) => <button key={item.id} onClick={() => { setTab("research"); setEditingResearch(item); }}><strong>{item.title}</strong><span>{item.domain}</span><i>{knowledgeName(item.knowledgeNodeId)}</i></button>) : <p>Nenhuma pesquisa ativa.</p>}</div></HudPanel></div>
      </div>}

      {tab === "knowledge" && <div className="stark-section"><div className="core-actions"><button onClick={() => setEditingKnowledge("new")}>＋ NOVO CONHECIMENTO</button></div><div className="knowledge-groups">{groups.map((group) => <section className="knowledge-cluster" key={group}><header><span>{group}</span><i>{knowledgeAreas.filter((area) => area.category === group).length} NÓS</i></header><div>{knowledgeAreas.filter((area) => area.category === group).map((area) => <button key={area.id} onClick={() => setSelectedKnowledge(area)}><span className={`priority-light priority-light--${area.priority}`} /><strong>{area.name}</strong><small>{area.nodeType.toUpperCase()} · C {area.coverage} / P {area.depth}</small></button>)}</div></section>)}</div></div>}

      {tab === "roadmaps" && <div className="stark-section"><div className="core-actions"><button onClick={() => setEditingRoadmap("new")}>＋ NOVO ROADMAP</button></div><div className="roadmap-grid">{roadmaps.map((roadmap) => <article className="roadmap-record" key={roadmap.id}><header><span>{roadmapStatus[roadmap.status]}</span><b>{roadmap.progress}%</b></header><h2>{roadmap.name}</h2><p>{roadmap.description || "Sem descrição."}</p><div className="roadmap-progress"><i style={{ width: `${roadmap.progress}%` }} /></div><small>{roadmap.completedActivities} / {roadmap.totalActivities} ATIVIDADES · EVIDÊNCIAS AUDITÁVEIS</small><div className="roadmap-tree">{roadmap.stages.map((stage) => <section key={stage.id}><strong>{String(stage.order).padStart(2, "0")} · {stage.name}</strong>{stage.topics.map((topic) => <div key={topic.id}><span>{topic.name}</span><i>{topic.state}</i><small>{knowledgeName(topic.knowledgeNodeId)}</small></div>)}</section>)}</div><footer><button onClick={() => setEditingRoadmap(roadmap)}>EDITAR</button><button className="danger-link" onClick={() => setDeleteTarget({ kind: "roadmap", id: roadmap.id, title: roadmap.name })}>EXCLUIR</button></footer></article>)}{!roadmaps.length && <div className="core-empty">Nenhum roadmap cadastrado. Crie o primeiro caminho de estudo.</div>}</div></div>}

      {tab === "research" && <div className="stark-section"><div className="core-actions"><button onClick={() => setEditingResearch("new")}>＋ NOVA PESQUISA</button></div><div className="research-table-wrap"><table className="research-table"><thead><tr><th>ID</th><th>Pesquisa</th><th>Domínio</th><th>Status</th><th>Conhecimento</th><th>Roadmap</th><th>Projeto</th><th /></tr></thead><tbody>{research.map((item) => <tr key={item.id}><td>{item.id}</td><td><strong>{item.title}</strong><small>{item.objective}</small></td><td>{item.domain}</td><td><span data-status={item.status}>{researchStatus[item.status]}</span></td><td>{knowledgeName(item.knowledgeNodeId)}</td><td>{roadmapName(item.roadmapId)}</td><td>{projectName(item.projectId)}</td><td><button onClick={() => setEditingResearch(item)}>EDITAR</button><button className="danger-link" onClick={() => setDeleteTarget({ kind: "research", id: item.id, title: item.title })}>×</button></td></tr>)}</tbody></table></div><p className="stark-boundary">CONCLUIR PESQUISA NÃO MODIFICA CONHECIMENTO AUTOMATICAMENTE NA v0.8.2.</p></div>}

      {tab === "evolution" && <div className="stark-section evolution-layout"><HudPanel title="Origem dos valores" code="AUDITORIA"><label className="stark-select">Conhecimento<select value={evolutionAreaId} onChange={(event) => void selectEvolutionArea(event.target.value)}><option value="">Selecione um conhecimento</option>{knowledgeAreas.map((area) => <option key={area.id} value={area.id}>{area.name}</option>)}</select></label>{evolutionAreaId && <div className="evolution-origin"><article><span>BASELINE</span><strong>{baselines.find((item) => item.knowledgeAreaId === evolutionAreaId)?.coverage ?? "—"} / {baselines.find((item) => item.knowledgeAreaId === evolutionAreaId)?.depth ?? "—"}</strong></article><article><span>EVENTOS</span><strong>{events.filter((item) => item.knowledgeNodeId === evolutionAreaId).length}</strong></article><article><span>ALTERAÇÕES MANUAIS</span><strong>{history.length}</strong></article></div>}</HudPanel><HudPanel title="Timeline real" code="SEM DADOS FABRICADOS"><div className="evolution-timeline">{history.map((item) => <article key={`history-${item.id}`}><span>{new Date(item.recordedAt).toLocaleString("pt-BR")}</span><strong>C {item.coverage} · P {item.depth}</strong><p>{item.reason}</p></article>)}{events.filter((item) => !evolutionAreaId || item.knowledgeNodeId === evolutionAreaId).map((item) => <article key={item.id}><span>{new Date(item.createdAt).toLocaleString("pt-BR")} · {item.sourceType.toUpperCase()}</span><strong>Δ C {item.coverageDelta} · P {item.depthDelta} · I {item.integrationDelta}</strong><p>{item.description || item.eventType}</p></article>)}{!history.length && !events.filter((item) => !evolutionAreaId || item.knowledgeNodeId === evolutionAreaId).length && <p>Selecione um conhecimento ou aguarde eventos reais. Nenhum evento automático é gerado nesta versão.</p>}</div></HudPanel></div>}

      {tab === "gaps" && <div className="stark-section gaps-integrated"><HudPanel title="Gap Diagnostics" code="DADOS REAIS"><GapDiagnostics onSelect={setSelectedKnowledge} /></HudPanel><HudPanel title="Roadmaps relacionados" code="RELAÇÃO POR TÓPICO"><div className="gap-roadmaps">{knowledgeAreas.filter((area) => ["critical", "high"].includes(area.priority)).map((area) => { const related = roadmaps.filter((roadmap) => roadmap.stages.some((stage) => stage.topics.some((topic) => topic.knowledgeNodeId === area.id))); return <article key={area.id}><strong>{area.name}</strong><span>C {area.coverage} · P {area.depth}</span><p>{related.length ? related.map((item) => item.name).join(" · ") : "Nenhum roadmap relacionado"}</p></article>; })}</div></HudPanel></div>}
    </>}

    {selectedKnowledge && <DetailsDrawer eyebrow={`STARK KNOWLEDGE // ${selectedKnowledge.nodeType.toUpperCase()}`} title={selectedKnowledge.name} onClose={() => setSelectedKnowledge(null)}><div className="drawer-actions"><button onClick={() => setEditingKnowledge(selectedKnowledge)}>EDITAR CONHECIMENTO</button><button className="danger-link" onClick={() => setDeleteTarget({ kind: "knowledge", id: selectedKnowledge.id, title: selectedKnowledge.name })}>EXCLUIR</button></div><KnowledgeMetrics key={selectedKnowledge.id} area={selectedKnowledge} onUpdated={setSelectedKnowledge} /><h3>Hierarquia</h3><p>{selectedKnowledge.parentId ? `${knowledgeName(selectedKnowledge.parentId)} → ${selectedKnowledge.name}` : "Nó raiz"}</p><h3>Projetos relacionados</h3><div className="related-list">{selectedKnowledge.projectIds.length ? selectedKnowledge.projectIds.map((id) => <span key={id}>{projectName(id)}</span>) : <span>Nenhum projeto relacionado</span>}</div></DetailsDrawer>}
    {editingKnowledge && <KnowledgeEditor area={editingKnowledge === "new" ? null : editingKnowledge} areas={knowledgeAreas} onCancel={() => setEditingKnowledge(null)} onSave={saveKnowledgeInput} />}
    {editingRoadmap && <RoadmapEditor roadmap={editingRoadmap === "new" ? null : editingRoadmap} knowledge={knowledgeAreas} projects={projects} research={research} onCancel={() => setEditingRoadmap(null)} onSave={saveRoadmap} />}
    {editingResearch && <ResearchEditor item={editingResearch === "new" ? null : editingResearch} knowledge={knowledgeAreas} projects={projects} roadmaps={roadmaps} onCancel={() => setEditingResearch(null)} onSave={saveResearch} />}
    {deleteTarget && <DeleteConfirmationDialog kind={deleteTarget.kind === "roadmap" ? "roadmap" : deleteTarget.kind === "research" ? "pesquisa" : "conhecimento"} title={deleteTarget.title} description="A exclusão será persistida. Registros relacionados protegidos permanecerão preservados quando aplicável." busy={false} onCancel={() => setDeleteTarget(null)} onConfirm={() => void remove()} />}
  </>;
}
