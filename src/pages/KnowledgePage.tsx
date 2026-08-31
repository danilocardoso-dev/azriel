import { useMemo, useState } from "react";
import { DetailsDrawer } from "../components/hud/DetailsDrawer";
import { Meter } from "../components/hud/Meter";
import { ModuleIntro } from "../components/layout/ModuleIntro";
import { knowledgeAreas } from "../data/knowledge";
import { getProjectById } from "../data/projects";
import type { KnowledgeArea } from "../types";

export function KnowledgePage() {
  const [selected, setSelected] = useState<KnowledgeArea | null>(null);
  const groups = useMemo(() => Array.from(new Set(knowledgeAreas.map((area) => area.group))), []);
  return (
    <>
      <ModuleIntro code="KNO-03" title="Knowledge Core" description="Representação visual inicial dos territórios de conhecimento e suas relações." metric="DADOS SIMULADOS" />
      <div className="knowledge-groups">
        {groups.map((group) => (
          <section className="knowledge-cluster" key={group}>
            <header><span>{group}</span><i>{knowledgeAreas.filter((area) => area.group === group).length} NÓS</i></header>
            <div>
              {knowledgeAreas.filter((area) => area.group === group).map((area) => (
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
        <span><strong>13</strong>ÁREAS MAPEADAS</span><span><strong>04</strong>GRUPOS</span><span><strong>05</strong>PRIORIDADES ALTAS</span><span><strong>0.5</strong>CORE REAL NA PRÓXIMA VERSÃO</span>
      </div>
      {selected && (
        <DetailsDrawer eyebrow={`KNOWLEDGE NODE // ${selected.group}`} title={selected.name} onClose={() => setSelected(null)}>
          <Meter label="Cobertura" value={selected.coverage} />
          <Meter label="Profundidade" value={selected.depth} tone="amber" />
          <h3>Prioridade</h3><p className={`priority-text priority-text--${selected.priority}`}>{selected.priority.toUpperCase()}</p>
          <h3>Projetos relacionados</h3>
          <div className="related-list">
            {selected.projectIds.length ? selected.projectIds.map((id) => <span key={id}>{getProjectById(id)?.name}</span>) : <span>Nenhum projeto relacionado</span>}
          </div>
          <p className="drawer-note">As métricas são uma linha de base simulada. Na v0.5 serão persistidas e terão histórico.</p>
        </DetailsDrawer>
      )}
    </>
  );
}
