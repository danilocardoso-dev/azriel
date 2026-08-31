import { useState } from "react";
import { DetailsDrawer } from "../components/hud/DetailsDrawer";
import { GapDiagnostics } from "../components/knowledge/GapDiagnostics";
import { StarkChart } from "../components/knowledge/StarkChart";
import { ModuleIntro } from "../components/layout/ModuleIntro";
import { HudPanel } from "../components/hud/HudPanel";
import { KnowledgeMetrics } from "../components/knowledge/KnowledgeMetrics";
import { useAzrielData } from "../contexts/useAzrielData";
import type { KnowledgeArea } from "../types";

export function StarkMapPage() {
  const { knowledgeAreas, projects } = useAzrielData();
  const [selected, setSelected] = useState<KnowledgeArea | null>(null);
  return (
    <>
      <ModuleIntro code="STK-04" title="Mapa Stark" description="Comparação interativa entre território conhecido e capacidade prática." metric="COBERTURA × PROFUNDIDADE" />
      <div className="stark-layout">
        <HudPanel title="Mapa de domínios" code={`${knowledgeAreas.length} ÁREAS`}><StarkChart areas={knowledgeAreas} selectedId={selected?.id} onSelect={setSelected} /></HudPanel>
        <HudPanel title="Diagnóstico de lacunas" code="PRIORIDADE"><GapDiagnostics onSelect={setSelected} /></HudPanel>
      </div>
      {selected && (
        <DetailsDrawer eyebrow={`MAPA STARK // ${selected.category}`} title={selected.name} onClose={() => setSelected(null)}>
          <KnowledgeMetrics key={selected.id} area={selected} onUpdated={setSelected} />
          <div className="gap-index"><strong>{selected.coverage - selected.depth}</strong><span>PONTOS ENTRE COBERTURA E PROFUNDIDADE</span></div>
          <h3>Prioridade</h3><p className={`priority-text priority-text--${selected.priority}`}>{selected.priority.toUpperCase()}</p>
          <h3>Projetos relacionados</h3><div className="related-list">{selected.projectIds.length ? selected.projectIds.map((id) => <span key={id}>{projects.find((project) => project.id === id)?.name ?? id}</span>) : <span>Projeto prático ainda necessário</span>}</div>
        </DetailsDrawer>
      )}
    </>
  );
}
