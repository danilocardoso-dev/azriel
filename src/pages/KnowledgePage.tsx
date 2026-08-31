import { useMemo, useState } from "react";
import { DetailsDrawer } from "../components/hud/DetailsDrawer";
import { ModuleIntro } from "../components/layout/ModuleIntro";
import { KnowledgeMetrics } from "../components/knowledge/KnowledgeMetrics";
import { useAzrielData } from "../contexts/useAzrielData";
import type { KnowledgeArea } from "../types";

export function KnowledgePage() {
  const { knowledgeAreas, projects } = useAzrielData();
  const [selected, setSelected] = useState<KnowledgeArea | null>(null);
  const groups = useMemo(() => Array.from(new Set(knowledgeAreas.map((area) => area.category))), [knowledgeAreas]);
  return (
    <>
      <ModuleIntro code="KNO-03" title="Knowledge Core" description="Representação dos territórios de conhecimento e suas relações persistidas." metric="SQLITE LOCAL" />
      <div className="knowledge-groups">
        {groups.map((group) => (
          <section className="knowledge-cluster" key={group}>
            <header><span>{group}</span><i>{knowledgeAreas.filter((area) => area.category === group).length} NÓS</i></header>
            <div>
              {knowledgeAreas.filter((area) => area.category === group).map((area) => (
                <button key={area.id} onClick={() => setSelected(area)}>
                  <span className={`priority-light priority-light--${area.priority}`} />
                  <strong>{area.name}</strong>
                  <small>C {area.coverage} / P {area.depth}</small>
                </button>
              ))}
            </div>
          </section>
        ))}
      </div>
      <div className="knowledge-summary">
        <span><strong>{knowledgeAreas.length}</strong>ÁREAS MAPEADAS</span><span><strong>{groups.length}</strong>GRUPOS</span><span><strong>{knowledgeAreas.filter((area) => ["critical", "high"].includes(area.priority)).length}</strong>PRIORIDADES ALTAS</span><span><strong>0.5</strong>CORE PERSISTENTE</span>
      </div>
      {selected && (
        <DetailsDrawer eyebrow={`KNOWLEDGE NODE // ${selected.category}`} title={selected.name} onClose={() => setSelected(null)}>
          <KnowledgeMetrics key={selected.id} area={selected} onUpdated={setSelected} />
          <h3>Prioridade</h3><p className={`priority-text priority-text--${selected.priority}`}>{selected.priority.toUpperCase()}</p>
          <h3>Projetos relacionados</h3>
          <div className="related-list">
            {selected.projectIds.length ? selected.projectIds.map((id) => <span key={id}>{projects.find((project) => project.id === id)?.name ?? id}</span>) : <span>Nenhum projeto relacionado</span>}
          </div>
          <p className="drawer-note">As métricas e cada alteração de histórico são persistidas no SQLite local.</p>
        </DetailsDrawer>
      )}
    </>
  );
}
