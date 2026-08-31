import { useMemo, useState } from "react";
import { ProjectEditor } from "../components/core/ProjectEditor";
import { DeleteConfirmationDialog } from "../components/daily/DeleteConfirmationDialog";
import { DetailsDrawer } from "../components/hud/DetailsDrawer";
import { Meter } from "../components/hud/Meter";
import { ModuleIntro } from "../components/layout/ModuleIntro";
import { useAzrielData } from "../contexts/useAzrielData";
import type { Project, ProjectInput, ProjectStatus } from "../types";

const statusLabels: Record<ProjectStatus, string> = { active: "ATIVO", research: "P&D", paused: "PAUSADO", planned: "PLANEJADO", completed: "CONCLUÍDO" };

export function ProjectsPage({ openCreate = false }: { openCreate?: boolean }) {
  const { projects, knowledgeAreas, saveProject, deleteProject } = useAzrielData();
  const [filter, setFilter] = useState<ProjectStatus | "all">("all");
  const [selected, setSelected] = useState<Project | null>(null);
  const [editing, setEditing] = useState<Project | "new" | null>(() => openCreate ? "new" : null);
  const [deleteTarget, setDeleteTarget] = useState<Project | null>(null);
  const [deleting, setDeleting] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);
  const filtered = useMemo(() => filter === "all" ? projects : projects.filter((project) => project.status === filter), [filter, projects]);

  async function save(input: ProjectInput) { await saveProject(input); setEditing(null); setSelected(null); setActionError(null); }
  async function remove() {
    if (!deleteTarget) return;
    setDeleting(true); setActionError(null);
    try { await deleteProject(deleteTarget.id); setDeleteTarget(null); setSelected(null); }
    catch (reason) { setActionError(reason instanceof Error ? reason.message : String(reason)); setDeleteTarget(null); }
    finally { setDeleting(false); }
  }

  return <>
    <ModuleIntro code="PRJ-02" title="Projetos" description="Projetos transformam cobertura de conhecimento em profundidade prática." metric={`${projects.length} PROJETOS REGISTRADOS`} />
    <div className="core-actions"><button onClick={() => setEditing("new")}>＋ NOVO PROJETO</button>{actionError && <p className="form-error">{actionError}</p>}</div>
    <div className="toolbar" aria-label="Filtrar projetos">{(["all", "active", "research", "planned", "paused", "completed"] as const).map((status) => <button className={filter === status ? "active" : ""} onClick={() => setFilter(status)} key={status}>{status === "all" ? "TODOS" : statusLabels[status]}</button>)}</div>
    {filtered.length ? <div className="project-grid">{filtered.map((project, index) => <button className="project-record" key={project.id} onClick={() => setSelected(project)}>
      <span className="project-record__index">PRJ-{String(index + 1).padStart(2, "0")}</span><span className={`status-tag status-tag--${project.status}`}>{statusLabels[project.status]}</span><h2>{project.name}</h2><small>{project.category}</small><p>{project.description}</p><div className="tag-row">{project.knowledgeAreaIds.slice(0, 4).map((id) => <i key={id}>{knowledgeAreas.find((area) => area.id === id)?.name ?? id}</i>)}</div><Meter label="Evolução estimada" value={project.progress} compact /><span className="project-record__action">ABRIR DOSSIÊ →</span>
    </button>)}</div> : <div className="core-empty">Nenhum projeto encontrado neste filtro.</div>}
    {selected && <DetailsDrawer eyebrow={`PROJETO // ${selected.category}`} title={selected.name} onClose={() => setSelected(null)}>
      <div className="drawer-actions"><button onClick={() => setEditing(selected)}>EDITAR PROJETO</button><button className="danger-link" onClick={() => setDeleteTarget(selected)}>EXCLUIR</button></div><span className={`status-tag status-tag--${selected.status}`}>{statusLabels[selected.status]}</span><p className="drawer-lead">{selected.description}</p><Meter label="Evolução estimada" value={selected.progress} /><h3>Objetivo</h3><p>{selected.objective || "Objetivo ainda não informado."}</p><h3>Áreas conectadas</h3><div className="tag-row">{selected.knowledgeAreaIds.length ? selected.knowledgeAreaIds.map((id) => <i key={id}>{knowledgeAreas.find((area) => area.id === id)?.name ?? id}</i>) : <i>NENHUMA RELAÇÃO</i>}</div><h3>Próximo passo</h3><p className="next-step">{selected.nextStep || "Próximo passo ainda não informado."}</p>
    </DetailsDrawer>}
    {editing && <ProjectEditor project={editing === "new" ? null : editing} knowledgeAreas={knowledgeAreas} onCancel={() => setEditing(null)} onSave={save} />}
    {deleteTarget && <DeleteConfirmationDialog kind="projeto" title={deleteTarget.name} description="O registro será removido. Tarefas, notas e workspaces serão preservados e apenas perderão o vínculo; nenhuma pasta ou arquivo físico será excluído." busy={deleting} onCancel={() => setDeleteTarget(null)} onConfirm={() => void remove()} />}
  </>;
}
