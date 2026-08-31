import { AzrielCore } from "../components/azriel/AzrielCore";
import { HudPanel } from "../components/hud/HudPanel";
import { Meter } from "../components/hud/Meter";
import { GapDiagnostics } from "../components/knowledge/GapDiagnostics";
import { StarkChart } from "../components/knowledge/StarkChart";
import { ModuleIntro } from "../components/layout/ModuleIntro";
import { useAzrielData } from "../contexts/useAzrielData";
import { azrielStates } from "../data/system";
import type { AzrielState } from "../types";
import { useDailyOperations } from "../contexts/useDailyOperations";
import { useAI } from "../contexts/useAI";

interface CommandCenterProps {
  coreState: AzrielState;
  onOpenAI: () => void;
  onOpenDaily: () => void;
}

export function CommandCenter({ coreState, onOpenAI, onOpenDaily }: CommandCenterProps) {
  const { knowledgeAreas, projects, databaseInfo } = useAzrielData();
  const { counters, loading: dailyLoading, error: dailyError } = useDailyOperations();
  const { status: aiStatus, phase: aiPhase } = useAI();
  const average = (field: "coverage" | "depth") => knowledgeAreas.length
    ? Math.round(knowledgeAreas.reduce((total, area) => total + area[field], 0) / knowledgeAreas.length) : 0;

  return (
    <>
      <ModuleIntro code="CMD-01" title="Command Center" description="Leitura consolidada da formação, dos projetos e das lacunas do operador." metric="V0.6 // AI CORE LOCAL" />
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
          <div className="core-stage__center"><AzrielCore state={coreState} onClick={onOpenAI} /></div>
          <div className="core-stage__metrics">
            <span><strong>{knowledgeAreas.length}</strong>DOMÍNIOS</span>
            <span><strong>{projects.length}</strong>PROJETOS</span>
            <span><strong>12+</strong>FRENTES</span>
            <span><strong>{knowledgeAreas.filter((area) => ["critical", "high"].includes(area.priority)).length}</strong>GAPS ALTOS</span>
          </div>
        </HudPanel>

        <div className="command-grid__right">
          <HudPanel title="AI Core" code="OLLAMA LOCAL">
            <button className="ai-command-summary" onClick={onOpenAI}>
              <span><strong>{aiStatus?.available ? "ONLINE" : "OFFLINE"}</strong>{aiStatus?.available ? aiStatus.models.join(" / ") : "Verifique o Ollama"}</span>
              <small>{aiPhase}</small><i>ABRIR AI CORE →</i>
            </button>
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
          <HudPanel title="Operações Diárias" code="AGORA">
            <button className="daily-command-summary" onClick={onOpenDaily}>
              {dailyError ? <span>OPERAÇÕES INDISPONÍVEIS</span> : dailyLoading ? <span>SINCRONIZANDO...</span> : <>
                <span><strong>{counters.today}</strong>TAREFAS HOJE</span>
                <span><strong>{counters.overdue}</strong>ATRASADAS</span>
                <span><strong>{counters.priority}</strong>ALTA / CRÍTICA</span>
              </>}
              <i>ABRIR CENTRAL →</i>
            </button>
          </HudPanel>
        </div>
      </div>

      <HudPanel title="Mapa Stark // Cobertura × Profundidade" code="LINHA DE BASE" className="command-chart">
        <StarkChart areas={knowledgeAreas} onSelect={() => undefined} />
      </HudPanel>

    </>
  );
}
