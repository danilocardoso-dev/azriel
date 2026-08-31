import { useMemo, useState } from "react";
import { DetailsDrawer } from "../components/hud/DetailsDrawer";
import { Meter } from "../components/hud/Meter";
import { ModuleIntro } from "../components/layout/ModuleIntro";
import { useAzrielData } from "../contexts/useAzrielData";
import type { Project, ProjectStatus } from "../types";

const statusLabels: Record<ProjectStatus, string> = { active: "ATIVO", research: "P&D", paused: "PAUSADO", planned: "PLANEJADO", completed: "CONCLUÍDO" };

export function ProjectsPage() {
  const { projects, knowledgeAreas } = useAzrielData();
  const [filter, setFilter] = useState<ProjectStatus | "all">("all");
  const [selected, setSelected] = useState<Project | null>(null);
  const filtered = useMemo(() => filter === "all" ? projects : projects.filter((project) => project.status === filter), [filter, projects]);

  return (
    <>
      <ModuleIntro code="PRJ-02" title="Projetos" description="Projetos transformam cobertura de conhecimento em profundidade prática." metric={`${projects.length} PROJETOS REGISTRADOS`} />
      <div className="toolbar" aria-label="Filtrar projetos">
        {(["all", "active", "research", "planned", "paused", "completed"] as const).map((status) => (
          <button className={filter === status ? "active" : ""} onClick={() => setFilter(status)} key={status}>{status === "all" ? "TODOS" : statusLabels[status]}</button>
        ))}
      </div>
      <div className="project-grid">
        {filtered.map((project, index) => (
          <button className="project-record" key={project.id} onClick={() => setSelected(project)}>
            <span className="project-record__index">PRJ-{String(index + 1).padStart(2, "0")}</span>
            <span className={`status-tag status-tag--${project.status}`}>{statusLabels[project.status]}</span>
            <h2>{project.name}</h2>
            <small>{project.category}</small>
            <p>{project.description}</p>
            <div className="tag-row">{project.knowledgeAreaIds.slice(0, 4).map((id) => <i key={id}>{knowledgeAreas.find((area) => area.id === id)?.name ?? id}</i>)}</div>
            <Meter label="Evolução estimada" value={project.progress} compact />
            <span className="project-record__action">ABRIR DOSSIÊ →</span>
          </button>
        ))}
      </div>
      {selected && (
        <DetailsDrawer eyebrow={`PROJETO // ${selected.category}`} title={selected.name} onClose={() => setSelected(null)}>
          <span className={`status-tag status-tag--${selected.status}`}>{statusLabels[selected.status]}</span>
          <p className="drawer-lead">{selected.description}</p>
          <Meter label="Evolução estimada" value={selected.progress} />
          <h3>Objetivo</h3><p>{selected.objective}</p>
          <h3>Áreas conectadas</h3><div className="tag-row">{selected.knowledgeAreaIds.map((id) => <i key={id}>{knowledgeAreas.find((area) => area.id === id)?.name ?? id}</i>)}</div>
          <h3>Próximo passo</h3><p className="next-step">{selected.nextStep}</p>
        </DetailsDrawer>
      )}
    </>
  );
}
