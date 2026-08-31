import { useMemo, useState } from "react";
import { KnowledgeEditor } from "../components/core/KnowledgeEditor";
import { DeleteConfirmationDialog } from "../components/daily/DeleteConfirmationDialog";
import { DetailsDrawer } from "../components/hud/DetailsDrawer";
import { ModuleIntro } from "../components/layout/ModuleIntro";
import { KnowledgeMetrics } from "../components/knowledge/KnowledgeMetrics";
import { useAzrielData } from "../contexts/useAzrielData";
import type { KnowledgeArea, KnowledgeInput } from "../types";

export function KnowledgePage() {
  const { knowledgeAreas, projects, saveKnowledge, deleteKnowledge } = useAzrielData();
  const [selected, setSelected] = useState<KnowledgeArea | null>(null);
  const [editing, setEditing] = useState<KnowledgeArea | "new" | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<KnowledgeArea | null>(null);
  const [deleting, setDeleting] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);
  const groups = useMemo(() => Array.from(new Set(knowledgeAreas.map((area) => area.category))), [knowledgeAreas]);
  async function save(input: KnowledgeInput) { await saveKnowledge(input); setEditing(null); setSelected(null); setActionError(null); }
  async function remove() { if (!deleteTarget) return; setDeleting(true); setActionError(null); try { await deleteKnowledge(deleteTarget.id); setDeleteTarget(null); setSelected(null); } catch (reason) { setActionError(reason instanceof Error ? reason.message : String(reason)); setDeleteTarget(null); } finally { setDeleting(false); } }

  return <>
    <ModuleIntro code="KNO-03" title="Knowledge Core" description="Representação dos territórios de conhecimento e suas relações persistidas." metric="SQLITE LOCAL" />
    <div className="core-actions"><button onClick={() => setEditing("new")}>＋ NOVO CONHECIMENTO</button>{actionError && <p className="form-error">{actionError}</p>}</div>
    {groups.length ? <div className="knowledge-groups">{groups.map((group) => <section className="knowledge-cluster" key={group}><header><span>{group}</span><i>{knowledgeAreas.filter((area) => area.category === group).length} NÓS</i></header><div>{knowledgeAreas.filter((area) => area.category === group).map((area) => <button key={area.id} onClick={() => setSelected(area)}><span className={`priority-light priority-light--${area.priority}`} /><strong>{area.name}</strong><small>C {area.coverage} / P {area.depth}</small></button>)}</div></section>)}</div> : <div className="core-empty">Nenhum conhecimento registrado.</div>}
    <div className="knowledge-summary"><span><strong>{knowledgeAreas.length}</strong>ÁREAS MAPEADAS</span><span><strong>{groups.length}</strong>GRUPOS</span><span><strong>{knowledgeAreas.filter((area) => ["critical", "high"].includes(area.priority)).length}</strong>PRIORIDADES ALTAS</span><span><strong>0.8.1</strong>CORE OPERACIONAL</span></div>
    {selected && <DetailsDrawer eyebrow={`KNOWLEDGE NODE // ${selected.category}`} title={selected.name} onClose={() => setSelected(null)}><div className="drawer-actions"><button onClick={() => setEditing(selected)}>EDITAR DADOS</button><button className="danger-link" onClick={() => setDeleteTarget(selected)}>EXCLUIR</button></div><KnowledgeMetrics key={selected.id} area={selected} onUpdated={setSelected} /><h3>Descrição</h3><p>{selected.description || "Descrição ainda não informada."}</p><h3>Prioridade</h3><p className={`priority-text priority-text--${selected.priority}`}>{selected.priority.toUpperCase()}</p><h3>Projetos relacionados</h3><div className="related-list">{selected.projectIds.length ? selected.projectIds.map((id) => <span key={id}>{projects.find((project) => project.id === id)?.name ?? id}</span>) : <span>Nenhum projeto relacionado</span>}</div><p className="drawer-note">As métricas e cada alteração de histórico são persistidas no SQLite local.</p></DetailsDrawer>}
    {editing && <KnowledgeEditor area={editing === "new" ? null : editing} onCancel={() => setEditing(null)} onSave={save} />}
    {deleteTarget && <DeleteConfirmationDialog kind="conhecimento" title={deleteTarget.name} description="A área e seu histórico serão removidos. Projetos, tarefas e notas serão preservados; apenas seus vínculos com este conhecimento serão desfeitos." busy={deleting} onCancel={() => setDeleteTarget(null)} onConfirm={() => void remove()} />}
  </>;
}
