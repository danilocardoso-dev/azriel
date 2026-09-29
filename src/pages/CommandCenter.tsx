import { useEffect, useMemo, useState } from "react";
import { AzrielCore } from "../components/azriel/AzrielCore";
import { localAiCardState } from "../components/ai/localAiLifecycle";
import { ModuleIntro } from "../components/layout/ModuleIntro";
import { useAI } from "../contexts/useAI";
import { useAutomation } from "../contexts/useAutomation";
import { useAzrielData } from "../contexts/useAzrielData";
import { useDailyOperations } from "../contexts/useDailyOperations";
import { useSystem } from "../contexts/useSystem";
import { starkService } from "../services/starkService";
import { formatBytes, formatUptime } from "../services/systemService";
import type { AzrielState, StudyRoadmap } from "../types";

interface CommandCenterProps {
  coreState: AzrielState;
  onOpenAI: () => void;
  onOpenStudies: () => void;
  onOpenDaily: () => void;
  onOpenMarket: () => void;
  onOpenAutomation: () => void;
  onNewProject: () => void;
  onNewTask: () => void;
  onNewNote: () => void;
}

export function CommandCenter({ coreState, onOpenAI, onOpenStudies, onOpenDaily, onOpenMarket, onOpenAutomation, onNewProject, onNewTask, onNewNote }: CommandCenterProps) {
  const { projects, databaseInfo } = useAzrielData();
  const { counters, loading: dailyLoading, error: dailyError } = useDailyOperations();
  const { settings: aiSettings, status: aiStatus, modelLifecycle, modelTransition, refreshModelLifecycle, loadLocalModel, unloadLocalModel } = useAI();
  const { snapshot, workspaces } = useSystem();
  const { state: automationState, actions, applications, routines, routineHistory } = useAutomation();
  const [roadmaps, setRoadmaps] = useState<StudyRoadmap[]>([]);
  const [roadmapError, setRoadmapError] = useState(false);
  const [modelActionError, setModelActionError] = useState<string | null>(null);

  useEffect(() => {
    let active = true;
    void starkService.roadmaps().then((items) => {
      if (active) setRoadmaps(items);
    }).catch(() => {
      if (active) setRoadmapError(true);
    });
    return () => { active = false; };
  }, []);

  useEffect(() => {
    let active = true;
    const refresh = async () => { if (active) await refreshModelLifecycle(); };
    const first = window.setTimeout(() => void refresh(), 0);
    const interval = window.setInterval(() => void refresh(), 20_000);
    return () => { active = false; window.clearTimeout(first); window.clearInterval(interval); };
  }, [refreshModelLifecycle]);

  const activeProjects = projects.filter((project) => project.status === "active" || project.status === "research");
  const activeRoadmaps = roadmaps.filter((roadmap) => roadmap.status === "active");
  const currentRoadmap = activeRoadmaps[0] ?? roadmaps.find((roadmap) => roadmap.status !== "completed") ?? roadmaps[0];
  const routinesToday = routineHistory.filter((item) => new Date(item.startedAt).toDateString() === new Date().toDateString()).length;
  const memoryPercent = snapshot && snapshot.memory.totalBytes > 0 ? Math.round(snapshot.memory.usedBytes / snapshot.memory.totalBytes * 100) : 0;
  const storage = snapshot?.storage[0];
  const storagePercent = storage && storage.totalBytes > 0 ? Math.round((storage.totalBytes - storage.availableBytes) / storage.totalBytes * 100) : null;
  const automationOnline = automationState !== "error" && automationState !== "offline";
  const localAi = localAiCardState(modelLifecycle, modelTransition);
  const localAiMessage = modelActionError || modelLifecycle?.error || (localAi.effectiveState === "NOT_AVAILABLE" ? "O modelo configurado nao esta instalado no Ollama." : null);
  const systemStatus = snapshot ? "TELEMETRIA ATIVA" : "AGUARDANDO SISTEMA";
  const systemSummary = useMemo(() => [
    { label: "CPU", value: snapshot ? `${snapshot.cpu.usagePercent.toFixed(0)}%` : "—", detail: `${snapshot?.details.logicalCores ?? 0} PROCESSADORES` },
    { label: "MEMÓRIA", value: snapshot ? `${memoryPercent}%` : "—", detail: snapshot ? `${formatBytes(snapshot.memory.availableBytes)} LIVRES` : "SEM LEITURA" },
    { label: "ARMAZENAMENTO", value: storagePercent === null ? "—" : `${storagePercent}%`, detail: storage?.name ?? "SEM VOLUME" },
    { label: "TEMPO ATIVO", value: snapshot ? formatUptime(snapshot.details.uptimeSeconds) : "—", detail: "DESDE A INICIALIZAÇÃO" },
  ], [memoryPercent, snapshot, storage, storagePercent]);

  async function runModelAction() {
    if (localAi.disabled) return;
    setModelActionError(null);
    try {
      if (localAi.action === "load") await loadLocalModel();
      else if (localAi.action === "unload") await unloadLocalModel();
      else await refreshModelLifecycle();
    } catch (reason) {
      setModelActionError(reason instanceof Error ? reason.message : String(reason));
      await refreshModelLifecycle();
    }
  }

  return <>
    <ModuleIntro code="CMD-01" title="Command Center" description="Visão operacional dos módulos ativos, atividades e recursos locais." metric={systemStatus} />
    <nav className="command-quick-actions" aria-label="Ações rápidas"><span>ACESSO RÁPIDO</span><button onClick={onOpenStudies}>ESTUDOS</button><button onClick={onNewProject}>＋ PROJETO</button><button onClick={onNewTask}>＋ TAREFA</button><button onClick={onNewNote}>＋ NOTA</button></nav>

    <div className="command-center-v2">
      <section className="command-center-v2__hero">
        <article className="command-core-card">
          <header><span>AZRIEL CORE</span><i>{coreState.toUpperCase()}</i></header>
          <div className="command-core-card__body"><AzrielCore state={coreState} onClick={onOpenAI} compact /><div><small>SESSÃO LOCAL</small><strong>PERSONAL INTELLIGENCE SYSTEM</strong><p>Projetos, estudos, operações, automações e experimentos reunidos em uma única leitura.</p></div></div>
          <footer><span>SQLITE <b>S{databaseInfo?.schemaVersion ?? "—"}</b></span><span>AI <b>{aiStatus?.available ? "ONLINE" : "OFFLINE"}</b></span><span>REDE <b>{snapshot?.network.length ? "ATIVA" : "—"}</b></span></footer>
        </article>

        <section className="command-system-panel" aria-label="Telemetria do dispositivo">
          <header><span>DISPOSITIVO</span><strong>{snapshot?.details.hostname ?? "COMPUTADOR LOCAL"}</strong><i>{snapshot?.details.osName ?? "SISTEMA INDISPONÍVEL"}</i></header>
          <div>{systemSummary.map((item) => <article key={item.label}><span>{item.label}</span><strong>{item.value}</strong><small>{item.detail}</small></article>)}</div>
        </section>
      </section>

      <section className="command-module-grid">
        <button className="command-module-card command-module-card--studies" onClick={onOpenStudies}>
          <header><span>STD</span><strong>ESTUDOS</strong><i>{activeRoadmaps.length} ATIVOS</i></header>
          {roadmapError ? <p>ROADMAPS INDISPONÍVEIS</p> : currentRoadmap ? <><h2>{currentRoadmap.name}</h2><p>{currentRoadmap.completedActivities} de {currentRoadmap.totalActivities} atividades concluídas</p><div className="command-progress"><i style={{ width: `${currentRoadmap.progress}%` }} /></div><footer><b>{currentRoadmap.progress}%</b><span>CONTINUAR ESTUDO →</span></footer></> : <><h2>Nenhum roadmap</h2><p>Crie uma trilha para começar.</p><footer><span>ABRIR ESTUDOS →</span></footer></>}
        </button>

        <button className="command-module-card" onClick={onOpenDaily}>
          <header><span>OPS</span><strong>OPERAÇÕES DIÁRIAS</strong><i>{dailyError ? "ERRO" : dailyLoading ? "SINCRONIZANDO" : "ATIVO"}</i></header>
          <div className="command-stat-row"><span><b>{counters.today}</b>HOJE</span><span data-alert={counters.overdue > 0}><b>{counters.overdue}</b>ATRASADAS</span><span><b>{counters.priority}</b>PRIORITÁRIAS</span></div>
          <footer><span>ABRIR OPERAÇÕES →</span></footer>
        </button>

        <article className="command-module-card">
          <header><span>PRJ</span><strong>PROJETOS</strong><i>{activeProjects.length} ATIVOS</i></header>
          <div className="command-project-list">{activeProjects.slice(0, 4).map((project) => <span key={project.id}><strong>{project.name}</strong><i>{project.progress}%</i></span>)}{!activeProjects.length && <p>Nenhum projeto ativo.</p>}</div>
          <footer><button onClick={onNewProject}>NOVO PROJETO</button></footer>
        </article>

        <button className="command-module-card" onClick={onOpenAutomation}>
          <header><span>AUT</span><strong>AUTOMAÇÃO</strong><i>{automationOnline ? "SAFE MODE" : "OFFLINE"}</i></header>
          <div className="command-stat-row"><span><b>{actions.length}</b>AÇÕES</span><span><b>{applications.filter((item) => item.enabled).length}</b>APPS</span><span><b>{routines.filter((item) => item.enabled).length}</b>ROTINAS</span></div>
          <footer><span>{routinesToday} EXECUÇÕES HOJE</span><span>ABRIR →</span></footer>
        </button>

        <article className="command-module-card command-local-ai-card" data-state={localAi.effectiveState.toLowerCase()}>
          <header><span>AIC</span><strong>IA LOCAL</strong><i>{localAi.headline}</i></header>
          <div className="command-local-ai-card__status">
            <h2>{modelLifecycle?.model || aiSettings?.model || "MODELO CONFIGURADO"}</h2>
            <span><b>OLLAMA SERVER</b><i>{modelLifecycle?.serverStatus || (aiStatus?.available ? "ONLINE" : "VERIFICANDO")}</i></span>
            <span><b>MODELO</b><i>{localAi.effectiveState}</i></span>
            <span><b>PROVIDER</b><i>{modelLifecycle?.provider || "OLLAMA"}</i></span>
            {localAiMessage && <p>{localAiMessage}</p>}
          </div>
          <footer><button onClick={onOpenAI}>ABRIR CONVERSA</button><button className="command-local-ai-card__action" onClick={() => void runModelAction()} disabled={localAi.disabled}>{localAi.actionLabel}</button></footer>
        </article>

        <button className="command-module-card command-module-card--market" onClick={onOpenMarket}>
          <header><span>LAB</span><strong>MARKET LAB</strong><i>LOCAL</i></header>
          <h2>Laboratório quantitativo</h2><p>Backtests, validação e observação de agentes sem execução financeira real.</p>
          <footer><span>ABRIR LABORATÓRIO →</span></footer>
        </button>
      </section>

      <footer className="command-center-v2__footer"><span>{workspaces.filter((workspace) => workspace.enabled).length} WORKSPACES AUTORIZADOS</span><span>DADOS LOCAIS</span><span>SEM CONTROLE IRRESTRITO</span></footer>
    </div>
  </>;
}
