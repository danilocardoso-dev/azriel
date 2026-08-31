import { useMemo, useState } from "react";
import { DetailsDrawer } from "../components/hud/DetailsDrawer";
import { Meter } from "../components/hud/Meter";
import { ModuleIntro } from "../components/layout/ModuleIntro";
import { projects } from "../data/projects";
import type { Project, ProjectStatus } from "../types";

const statusLabels: Record<ProjectStatus, string> = { active: "ATIVO", research: "P&D", paused: "PAUSADO", planned: "PLANEJADO" };

export function ProjectsPage() {
  const [filter, setFilter] = useState<ProjectStatus | "all">("all");
  const [selected, setSelected] = useState<Project | null>(null);
  const filtered = useMemo(() => filter === "all" ? projects : projects.filter((project) => project.status === filter), [filter]);

  return (
    <>
      <ModuleIntro code="PRJ-02" title="Projetos" description="Projetos transformam cobertura de conhecimento em profundidade prática." metric={`${projects.length} PROJETOS REGISTRADOS`} />
      <div className="toolbar" aria-label="Filtrar projetos">
        {(["all", "active", "research", "planned", "paused"] as const).map((status) => (
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
            <div className="tag-row">{project.knowledgeAreas.slice(0, 4).map((area) => <i key={area}>{area}</i>)}</div>
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
          <h3>Áreas conectadas</h3><div className="tag-row">{selected.knowledgeAreas.map((area) => <i key={area}>{area}</i>)}</div>
          <h3>Próximo passo</h3><p className="next-step">{selected.nextStep}</p>
        </DetailsDrawer>
      )}
    </>
  );
}
