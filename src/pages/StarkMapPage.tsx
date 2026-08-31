import { useState } from "react";
import { DetailsDrawer } from "../components/hud/DetailsDrawer";
import { Meter } from "../components/hud/Meter";
import { GapDiagnostics } from "../components/knowledge/GapDiagnostics";
import { StarkChart } from "../components/knowledge/StarkChart";
import { ModuleIntro } from "../components/layout/ModuleIntro";
import { HudPanel } from "../components/hud/HudPanel";
import { knowledgeAreas } from "../data/knowledge";
import { getProjectById } from "../data/projects";
import type { KnowledgeArea } from "../types";

export function StarkMapPage() {
  const [selected, setSelected] = useState<KnowledgeArea | null>(null);
  return (
    <>
      <ModuleIntro code="STK-04" title="Mapa Stark" description="Comparação interativa entre território conhecido e capacidade prática." metric="COBERTURA × PROFUNDIDADE" />
      <div className="stark-layout">
        <HudPanel title="Mapa de domínios" code="13 ÁREAS"><StarkChart areas={knowledgeAreas} selectedId={selected?.id} onSelect={setSelected} /></HudPanel>
        <HudPanel title="Diagnóstico de lacunas" code="PRIORIDADE"><GapDiagnostics onSelect={setSelected} /></HudPanel>
      </div>
      {selected && (
        <DetailsDrawer eyebrow={`MAPA STARK // ${selected.group}`} title={selected.name} onClose={() => setSelected(null)}>
          <Meter label="Cobertura" value={selected.coverage} />
          <Meter label="Profundidade" value={selected.depth} tone="amber" />
          <div className="gap-index"><strong>{selected.coverage - selected.depth}</strong><span>PONTOS ENTRE COBERTURA E PROFUNDIDADE</span></div>
          <h3>Prioridade</h3><p className={`priority-text priority-text--${selected.priority}`}>{selected.priority.toUpperCase()}</p>
          <h3>Projetos relacionados</h3><div className="related-list">{selected.projectIds.length ? selected.projectIds.map((id) => <span key={id}>{getProjectById(id)?.name}</span>) : <span>Projeto prático ainda necessário</span>}</div>
        </DetailsDrawer>
      )}
    </>
  );
}
