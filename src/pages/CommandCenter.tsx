import { useState } from "react";
import { AzrielCore } from "../components/azriel/AzrielCore";
import { DetailsDrawer } from "../components/hud/DetailsDrawer";
import { HudPanel } from "../components/hud/HudPanel";
import { Meter } from "../components/hud/Meter";
import { GapDiagnostics } from "../components/knowledge/GapDiagnostics";
import { StarkChart } from "../components/knowledge/StarkChart";
import { ModuleIntro } from "../components/layout/ModuleIntro";
import { useAzrielData } from "../contexts/useAzrielData";
import { azrielStates, modules } from "../data/system";
import type { AzrielState } from "../types";

interface CommandCenterProps {
  coreState: AzrielState;
  setCoreState: (state: AzrielState) => void;
}

export function CommandCenter({ coreState, setCoreState }: CommandCenterProps) {
  const { knowledgeAreas, projects, databaseInfo } = useAzrielData();
  const [coreOpen, setCoreOpen] = useState(false);
  const average = (field: "coverage" | "depth") => knowledgeAreas.length
    ? Math.round(knowledgeAreas.reduce((total, area) => total + area[field], 0) / knowledgeAreas.length) : 0;

  return (
    <>
      <ModuleIntro code="CMD-01" title="Command Center" description="Leitura consolidada da formação, dos projetos e das lacunas do operador." metric="V0.5 // SQLITE LOCAL" />
      <div className="command-grid">
        <div className="command-grid__left">
          <HudPanel title="Perfil do operador" code="ID/AZ">
            <div className="operator-card">
              <span className="eyebrow">OPERADOR PRINCIPAL</span>
              <h3>AZRIEL</h3>
              <p>Conhecimento · Engenharia · Inteligência · Impacto</p>
              <Meter label="Cobertura" value={average("coverage")} />
              <Meter label="Profundidade" value={average("depth")} />
              <Meter label="Integração" value={databaseInfo?.integrationValue ?? 0} />
            </div>
          </HudPanel>
          <HudPanel title="Projetos" code="07 NÚCLEOS">
            <div className="signal-list">
              {projects.map((project, index) => (
                <div className="signal-row" key={project.id}>
                  <span>P-{String(index + 1).padStart(2, "0")}</span>
                  <strong>{project.name}</strong>
                  <i data-state={project.status}>{project.status}</i>
                </div>
              ))}
            </div>
          </HudPanel>
        </div>

        <HudPanel title="Núcleo central de conhecimento" code={`ESTADO / ${azrielStates[coreState].label}`} className="core-stage">
          <span className="satellite satellite--one">BIOSSISTEMAS<br />SINCRONIA 42%</span>
          <span className="satellite satellite--two">ENGENHARIA<br />LACUNA ALTA</span>
          <span className="satellite satellite--three">IA / SOFTWARE<br />NÚCLEO ESTÁVEL</span>
          <span className="satellite satellite--four">ENERGIA<br />P&D EM FILA</span>
          <div className="core-stage__center"><AzrielCore state={coreState} onClick={() => setCoreOpen(true)} /></div>
          <div className="core-stage__metrics">
            <span><strong>{knowledgeAreas.length}</strong>DOMÍNIOS</span>
            <span><strong>{projects.length}</strong>PROJETOS</span>
            <span><strong>12+</strong>FRENTES</span>
            <span><strong>{knowledgeAreas.filter((area) => ["critical", "high"].includes(area.priority)).length}</strong>GAPS ALTOS</span>
          </div>
        </HudPanel>

        <div className="command-grid__right">
          <HudPanel title="Simulador de estado" code="CORE CTRL">
            <div className="state-switcher">
              {(Object.keys(azrielStates) as AzrielState[]).map((state) => (
                <button className={coreState === state ? "active" : ""} onClick={() => setCoreState(state)} key={state}>{azrielStates[state].label}</button>
              ))}
            </div>
            <p className="system-message">{azrielStates[coreState].message}</p>
          </HudPanel>
          <HudPanel title="Fila de prioridades" code="PRÓXIMA AÇÃO">
            <div className="priority-queue">
              <article><span>ALTA</span><strong>Matemática + Física</strong><p>Construir a base necessária antes da Mecatrônica.</p></article>
              <article><span>ALTA</span><strong>Eletrônica</strong><p>Circuitos, instrumentação e microcontroladores.</p></article>
              <article><span>ATIVO</span><strong>Biologia Molecular</strong><p>Transformar teoria em domínio aplicado.</p></article>
              <article><span>CONSTRUIR</span><strong>Azriel</strong><p>Consolidar o sistema pessoal de inteligência.</p></article>
            </div>
          </HudPanel>
          <HudPanel title="Diagnóstico de lacunas" code="ANÁLISE">
            <GapDiagnostics />
          </HudPanel>
        </div>
      </div>

      <HudPanel title="Mapa Stark // Cobertura × Profundidade" code="LINHA DE BASE" className="command-chart">
        <StarkChart areas={knowledgeAreas} onSelect={() => undefined} />
      </HudPanel>

      {coreOpen && (
        <DetailsDrawer eyebrow="AZRIEL CORE // V0.4" title={azrielStates[coreState].label} onClose={() => setCoreOpen(false)}>
          <p className="drawer-lead">{azrielStates[coreState].message}</p>
          <div className="drawer-kpis"><span><strong>8</strong>MÓDULOS</span><span><strong>{projects.length}</strong>PROJETOS</span><span><strong>{knowledgeAreas.length}</strong>ÁREAS</span></div>
          <h3>Estado dos módulos</h3>
          <div className="module-status-list">
            {modules.map((module) => <div key={module.id}><span>{module.code}</span><strong>{module.label}</strong><i>DISPONÍVEL</i></div>)}
          </div>
          <p className="drawer-note">Os estados são simulados nesta versão. Integração real com o computador entra no System Core v0.7.</p>
        </DetailsDrawer>
      )}
    </>
  );
}
