import { CoreTopology } from "../components/azriel/CoreTopology";
import { HudPanel } from "../components/hud/HudPanel";
import { Meter } from "../components/hud/Meter";
import { GapDiagnostics } from "../components/knowledge/GapDiagnostics";
import { StarkChart } from "../components/knowledge/StarkChart";
import { ModuleIntro } from "../components/layout/ModuleIntro";
import { useAI } from "../contexts/useAI";
import { useAutomation } from "../contexts/useAutomation";
import { useAzrielData } from "../contexts/useAzrielData";
import { useDailyOperations } from "../contexts/useDailyOperations";
import { useSystem } from "../contexts/useSystem";
import { formatBytes } from "../services/systemService";
import type { AzrielState } from "../types";

interface CommandCenterProps {
  coreState: AzrielState;
  onOpenAI: () => void;
  onOpenStark: () => void;
  onOpenDaily: () => void;
  onNewProject: () => void;
  onNewTask: () => void;
  onNewNote: () => void;
}

export function CommandCenter({ coreState, onOpenAI, onOpenStark, onOpenDaily, onNewProject, onNewTask, onNewNote }: CommandCenterProps) {
  const { knowledgeAreas, projects, databaseInfo } = useAzrielData();
  const { counters, loading: dailyLoading, error: dailyError } = useDailyOperations();
  const { status: aiStatus, phase: aiPhase } = useAI();
  const { snapshot, workspaces } = useSystem();
  const { state: automationState, actions, applications, routines, routineHistory } = useAutomation();
  const routinesToday = routineHistory.filter((item) => new Date(item.startedAt).toDateString() === new Date().toDateString()).length;
  const memoryPercent = snapshot && snapshot.memory.totalBytes > 0 ? Math.round(snapshot.memory.usedBytes / snapshot.memory.totalBytes * 100) : 0;
  const primaryStorage = snapshot?.storage[0];
  const storagePercent = primaryStorage && primaryStorage.totalBytes > 0
    ? Math.round((primaryStorage.totalBytes - primaryStorage.availableBytes) / primaryStorage.totalBytes * 100) : null;
  const automationOnline = automationState !== "error" && automationState !== "offline";
  const average = (field: "coverage" | "depth") => knowledgeAreas.length
    ? Math.round(knowledgeAreas.reduce((total, area) => total + area[field], 0) / knowledgeAreas.length) : 0;

  return (
    <>
      <ModuleIntro code="CMD-01" title="Command Center" description="Estação integrada de pesquisa, engenharia e inteligência operacional." metric="ENGINEERING HUD // EXP-01" />
      <nav className="command-quick-actions" aria-label="Ações rápidas"><span>CAPTURA OPERACIONAL</span><button onClick={onOpenStark}>MAPA STARK</button><button onClick={onNewProject}>＋ NOVO PROJETO</button><button onClick={onNewTask}>＋ NOVA TAREFA</button><button onClick={onNewNote}>＋ NOVA NOTA</button></nav>

      <div className="engineering-command">
        <div className="engineering-command__deck">
          <aside className="engineering-command__rail engineering-command__rail--left">
            <section className="engineering-instrument operator-instrument">
              <header><span>OPR-00</span><strong>OPERADOR</strong><i>ID / AZ</i></header>
              <div className="operator-instrument__identity"><small>OPERADOR PRINCIPAL</small><strong>AZRIEL</strong><span>KNOWLEDGE · ENGINEERING · INTELLIGENCE · IMPACT</span></div>
              <div className="operator-instrument__metrics">
                <Meter label="Cobertura" value={average("coverage")} />
                <Meter label="Profundidade" value={average("depth")} />
                <Meter label="Integração" value={databaseInfo?.integrationValue ?? 0} />
              </div>
            </section>

            <section className="engineering-instrument project-matrix">
              <header><span>PRJ-MTX</span><strong>PROJECT MATRIX</strong><i>{projects.length.toString().padStart(2, "0")} NODES</i></header>
              <div className="project-matrix__head"><span>ID</span><span>PROJECT</span><span>STATE</span></div>
              <div className="project-matrix__body">
                {projects.map((project, index) => (
                  <div className="project-matrix__row" key={project.id}>
                    <span>P-{String(index + 1).padStart(2, "0")}</span><strong>{project.name}</strong><i data-state={project.status}>{project.status}</i>
                  </div>
                ))}
              </div>
            </section>
          </aside>

          <CoreTopology
            state={coreState}
            aiOnline={Boolean(aiStatus?.available)}
            systemOnline={Boolean(snapshot)}
            automationOnline={automationOnline}
            knowledgeCount={knowledgeAreas.length}
            projectCount={projects.length}
            dailyCount={counters.pending}
            onOpenAI={onOpenAI}
            onOpenDaily={onOpenDaily}
          />

          <aside className="engineering-command__rail engineering-command__rail--right">
            <section className="engineering-instrument diagnostic-stack">
              <header><span>SYS-DIAG</span><strong>SYSTEM CORE</strong><i>{snapshot ? "LIVE" : "UNAVAILABLE"}</i></header>
              <div className="diagnostic-stack__row"><span>CPU LOAD</span><strong>{snapshot ? `${snapshot.cpu.usagePercent.toFixed(0)}%` : "—"}</strong><i data-level={snapshot && snapshot.cpu.usagePercent > 80 ? "warning" : "nominal"} /></div>
              <div className="diagnostic-stack__row"><span>MEMORY</span><strong>{snapshot ? `${memoryPercent}%` : "—"}</strong><i data-level={memoryPercent > 85 ? "warning" : "nominal"} /></div>
              <div className="diagnostic-stack__row"><span>STORAGE</span><strong>{storagePercent === null ? "—" : `${storagePercent}%`}</strong><i data-level={storagePercent !== null && storagePercent > 90 ? "warning" : "nominal"} /></div>
              <div className="diagnostic-stack__row"><span>NETWORK</span><strong>{snapshot?.network.length ? "ONLINE" : "—"}</strong><i data-level={snapshot?.network.length ? "nominal" : "standby"} /></div>
              <footer><span>RAM FREE</span><strong>{snapshot ? formatBytes(snapshot.memory.availableBytes) : "—"}</strong><span>WS</span><strong>{workspaces.filter((workspace) => workspace.enabled).length.toString().padStart(2, "0")}</strong></footer>
            </section>

            <button className="engineering-link-panel" onClick={onOpenAI}>
              <span>AIC-02 // OLLAMA LOCAL</span><strong>AI CORE</strong><i data-state={aiStatus?.available ? "online" : "warning"}>{aiStatus?.available ? "ONLINE" : "OFFLINE"}</i>
              <small>{aiStatus?.available ? aiStatus.models.join(" / ") : "Verifique o Ollama"}</small><em>{aiPhase}</em><b>OPEN CHANNEL →</b>
            </button>

            <section className="engineering-instrument automation-instrument">
              <header><span>AUT-10</span><strong>AUTOMATION</strong><i>{automationOnline ? "SAFE MODE" : "OFFLINE"}</i></header>
              <div><span><strong>{actions.length.toString().padStart(2, "0")}</strong>ACTIONS</span><span><strong>{applications.filter((item) => item.enabled).length.toString().padStart(2, "0")}</strong>APPS</span><span><strong>{routines.filter((item) => item.enabled).length.toString().padStart(2, "0")}</strong>ROUTINES</span><span><strong>{routinesToday.toString().padStart(2, "0")}</strong>TODAY</span></div>
            </section>

            <button className="engineering-link-panel engineering-link-panel--daily" onClick={onOpenDaily}>
              <span>OPS-03 // OPERATIONAL QUEUE</span><strong>DAILY OPERATIONS</strong>
              {dailyError ? <small>OPERATIONS UNAVAILABLE</small> : dailyLoading ? <small>SYNCHRONIZING...</small> : <div><span><b>{counters.today}</b>TODAY</span><span><b>{counters.overdue}</b>OVERDUE</span><span><b>{counters.priority}</b>HIGH / CRITICAL</span></div>}
              <em>OPEN CENTRAL →</em>
            </button>
          </aside>
        </div>

      </div>

      <HudPanel title="Fila de prioridades" code="PRÓXIMA AÇÃO" className="command-flat-panel">
        <div className="priority-queue">
          <article><span>ALTA</span><strong>Matemática + Física</strong><p>Construir a base necessária antes da Mecatrônica.</p></article>
          <article><span>ALTA</span><strong>Eletrônica</strong><p>Circuitos, instrumentação e microcontroladores.</p></article>
          <article><span>ATIVO</span><strong>Biologia Molecular</strong><p>Transformar teoria em domínio aplicado.</p></article>
          <article><span>CONSTRUIR</span><strong>Azriel</strong><p>Consolidar o sistema pessoal de inteligência.</p></article>
        </div>
      </HudPanel>

      <HudPanel title="Diagnóstico de lacunas" code="ANÁLISE" className="command-flat-panel">
        <GapDiagnostics />
      </HudPanel>

      <HudPanel title="Mapa Stark // Cobertura × Profundidade" code="LINHA DE BASE" className="command-chart">
        <StarkChart areas={knowledgeAreas} onSelect={() => undefined} />
      </HudPanel>
    </>
  );
}
