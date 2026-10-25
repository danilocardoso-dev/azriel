import { useEffect, useMemo, useState, type FormEvent } from "react";
import { AzrielCore } from "../components/azriel/AzrielCore";
import { useDailyOperations } from "../contexts/useDailyOperations";
import { useAI } from "../contexts/useAI";
import { azrielStates } from "../data/system";
import { formatBytes, formatUptime, systemService } from "../services/systemService";
import type { AzrielState, SystemSnapshot, Task } from "../types";

interface CommandCenterProps {
  coreState: AzrielState;
  onOpenAI: () => void;
  onOpenDaily: () => void;
  onNewTask: () => void;
  onNewNote: () => void;
}

export function CommandCenter({
  coreState,
  onOpenAI,
  onOpenDaily,
  onNewTask,
  onNewNote,
}: CommandCenterProps) {
  const { counters, tasks, notes, loading, completeTask } = useDailyOperations();
  const { send } = useAI();
  const [snapshot, setSnapshot] = useState<SystemSnapshot | null>(null);
  const [quickInput, setQuickInput] = useState("");
  const [busyTaskId, setBusyTaskId] = useState<string | null>(null);

  useEffect(() => {
    let active = true;
    const updateSnapshot = () => {
      systemService
        .snapshot()
        .then((data) => {
          if (active) setSnapshot(data);
        })
        .catch(() => undefined);
    };

    updateSnapshot();
    const interval = window.setInterval(updateSnapshot, 10_000);
    return () => {
      active = false;
      window.clearInterval(interval);
    };
  }, []);

  const memoryPercent = useMemo(() => {
    if (!snapshot || snapshot.memory.totalBytes <= 0) return 0;
    return Math.round((snapshot.memory.usedBytes / snapshot.memory.totalBytes) * 100);
  }, [snapshot]);

  const memoryFree = useMemo(() => {
    if (!snapshot) return "—";
    return formatBytes(snapshot.memory.availableBytes);
  }, [snapshot]);

  const uptime = useMemo(() => {
    if (!snapshot) return "—";
    return formatUptime(snapshot.details.uptimeSeconds);
  }, [snapshot]);

  const stateInfo = azrielStates[coreState] ?? {
    label: coreState.toUpperCase(),
    message: "Operações táticas ativas.",
  };

  const handleCommandSubmit = async (event: FormEvent) => {
    event.preventDefault();
    const query = quickInput.trim();
    if (!query) return;
    setQuickInput("");
    onOpenAI();
    try {
      await send(query);
    } catch {
      // Handled in context
    }
  };

  const handleQuickQuestion = async (query: string) => {
    onOpenAI();
    try {
      await send(query);
    } catch {
      // Handled in context
    }
  };

  const handleFinishTask = async (id: string) => {
    setBusyTaskId(id);
    try {
      await completeTask(id);
    } finally {
      setBusyTaskId(null);
    }
  };

  const activeTodayTasks = useMemo(() => {
    return tasks.filter((t) => t.status !== "completed").slice(0, 4);
  }, [tasks]);

  const recentNotes = useMemo(() => {
    return notes.filter((n) => n.status !== "archived").slice(0, 3);
  }, [notes]);

  return (
    <div className="stark-hud-deck">
      {/* Top Reactor Cockpit Stage */}
      <section className="stark-cockpit">
        {/* Left Telemetry Wing */}
        <aside className="stark-hud-wing stark-hud-wing--left">
          <header className="stark-wing-head">
            <span className="stark-bracket">┌</span>
            <strong>HARDWARE TELEMETRY</strong>
            <i>SYS // READOUT</i>
          </header>
          <div className="stark-telemetry-body">
            <div className="stark-telemetry-gauge">
              <div className="stark-gauge-label">
                <span>CPU UTILIZATION</span>
                <strong>{snapshot ? `${snapshot.cpu.usagePercent.toFixed(0)}%` : "—"}</strong>
              </div>
              <div className="stark-gauge-bar">
                <i style={{ width: `${snapshot ? Math.min(snapshot.cpu.usagePercent, 100) : 0}%` }} />
              </div>
              <small>{snapshot ? `${snapshot.details.logicalCores} NÚCLEOS ATIVOS` : "AGUARDANDO"}</small>
            </div>

            <div className="stark-telemetry-gauge">
              <div className="stark-gauge-label">
                <span>RAM ALLOCATION</span>
                <strong>{memoryPercent}%</strong>
              </div>
              <div className="stark-gauge-bar">
                <i style={{ width: `${memoryPercent}%` }} />
              </div>
              <small>{memoryFree} DISPONÍVEIS</small>
            </div>

            <div className="stark-telemetry-stat">
              <span>TEMPO ATIVO</span>
              <strong>{uptime}</strong>
            </div>
            <div className="stark-telemetry-stat">
              <span>ESTAÇÃO HOST</span>
              <strong style={{ fontSize: "11px", letterSpacing: "0.05em" }}>
                {snapshot?.details.hostname ?? "AZRIEL LOCAL"}
              </strong>
            </div>
          </div>
          <footer className="stark-wing-footer">
            <span className="stark-bracket">└</span>
            <small>TELEMETRIA LOCAL</small>
          </footer>
        </aside>

        {/* Center: The Holographic Arc Reactor */}
        <div className="stark-reactor-column">
          <div className="stark-reactor-glow-field">
            <AzrielCore state={coreState} onClick={onOpenAI} />
          </div>

          <div className="stark-reactor-info">
            <span className="stark-reactor-badge">ARC REACTOR // {stateInfo.label}</span>
            <h2>{stateInfo.message}</h2>
          </div>

          {/* Holographic Sound Waveform */}
          <div className="stark-waveform" aria-hidden="true">
            <span /><span /><span /><span /><span /><span /><span /><span /><span /><span /><span /><span />
          </div>
        </div>

        {/* Right Mission Directives Wing */}
        <aside className="stark-hud-wing stark-hud-wing--right">
          <header className="stark-wing-head">
            <i>OPS // OVERVIEW</i>
            <strong>DIRETRIZES TÁTICAS</strong>
            <span className="stark-bracket">┐</span>
          </header>
          <div className="stark-telemetry-body">
            <button className="stark-mission-metric" onClick={onOpenDaily} type="button">
              <span>TAREFAS PENDENTES</span>
              <strong>{loading ? "…" : counters.pending}</strong>
              <small>{counters.priority} DE ALTA PRIORIDADE</small>
            </button>

            <button
              className={`stark-mission-metric ${counters.overdue > 0 ? "stark-mission-metric--alert" : ""}`}
              onClick={onOpenDaily}
              type="button"
            >
              <span>FOCO PARA HOJE</span>
              <strong style={{ color: counters.overdue > 0 ? "var(--amber-alert)" : undefined }}>
                {loading ? "…" : counters.today}
              </strong>
              <small>
                {counters.overdue > 0 ? `⚠ ${counters.overdue} ATRASADAS` : "AGENDA REGULAR"}
              </small>
            </button>

            <button className="stark-mission-metric" onClick={onOpenDaily} type="button">
              <span>NOTAS NO REGISTRO</span>
              <strong>{loading ? "…" : counters.notes}</strong>
              <small>PERSISTÊNCIA SQLITE</small>
            </button>

            <div className="stark-telemetry-stat">
              <span>CANAL OLLAMA</span>
              <strong style={{ color: coreState === "offline" ? "var(--red-critical)" : "var(--green-online)" }}>
                {coreState === "offline" ? "OFFLINE" : "ATIVO"}
              </strong>
            </div>
          </div>
          <footer className="stark-wing-footer">
            <small>ESTADO DO FLUXO</small>
            <span className="stark-bracket">┘</span>
          </footer>
        </aside>
      </section>

      {/* Center Command Console */}
      <section className="stark-command-deck">
        <form className="stark-command-bar" onSubmit={handleCommandSubmit}>
          <span className="stark-prompt-symbol">&gt;</span>
          <input
            type="text"
            placeholder="AZRIEL // DIGITE UMA ORDEM, PERGUNTA OU COMANDO..."
            value={quickInput}
            onChange={(e) => setQuickInput(e.target.value)}
          />
          <button type="submit" disabled={!quickInput.trim()}>
            TRANSMITIR
          </button>
        </form>

        <div className="stark-quick-chips">
          <button type="button" onClick={() => void handleQuickQuestion("Azriel, relatório do sistema.")}>
            ⚡ RELATÓRIO DO SISTEMA
          </button>
          <button type="button" onClick={() => void handleQuickQuestion("O que tenho agendado para hoje?")}>
            ⚡ AGENDA DE HOJE
          </button>
          <button type="button" onClick={() => void handleQuickQuestion("Quais tarefas são prioritárias?")}>
            ⚡ PRIORIDADES
          </button>
          <button type="button" onClick={onNewTask}>
            ＋ NOVA TAREFA
          </button>
          <button type="button" onClick={onNewNote}>
            ＋ NOVA NOTA
          </button>
          <button type="button" onClick={onOpenAI}>
            🧠 TERMINAL COMPLETO
          </button>
        </div>
      </section>

      {/* Holographic Tactical Data Streams */}
      <section className="stark-data-streams">
        {/* Stream 1: Urgent Tasks */}
        <div className="stark-stream-panel">
          <header className="stark-stream-header">
            <span>DIRETRIZES ATIVAS // FILTRO RÁPIDO</span>
            <button type="button" onClick={onOpenDaily}>VER TODAS →</button>
          </header>
          <div className="stark-stream-list">
            {activeTodayTasks.length > 0 ? (
              activeTodayTasks.map((task: Task) => (
                <article key={task.id} className="stark-stream-item">
                  <div className="stark-item-info">
                    <strong>{task.title}</strong>
                    <small>
                      <span className={`stark-priority-tag priority--${task.priority}`}>
                        {task.priority.toUpperCase()}
                      </span>
                      {" · "}
                      {task.dueDate ? `PRAZO: ${task.dueDate}` : "SEM PRAZO DEFINIDO"}
                    </small>
                  </div>
                  <button
                    type="button"
                    className="stark-check-btn"
                    disabled={busyTaskId === task.id}
                    onClick={() => void handleFinishTask(task.id)}
                    title="Concluir tarefa"
                  >
                    ✓ CONCLUIR
                  </button>
                </article>
              ))
            ) : (
              <p className="stark-empty-msg">NENHUMA TAREFA PENDENTE NO RADAR ATUAL.</p>
            )}
          </div>
        </div>

        {/* Stream 2: Recent Notes */}
        <div className="stark-stream-panel">
          <header className="stark-stream-header">
            <span>MEMÓRIA OPERACIONAL // ANOTAÇÕES RECENTES</span>
            <button type="button" onClick={onOpenDaily}>NOTAS →</button>
          </header>
          <div className="stark-stream-list">
            {recentNotes.length > 0 ? (
              recentNotes.map((note) => (
                <article key={note.id} className="stark-stream-item">
                  <div className="stark-item-info">
                    <strong>{note.title || "ANOTAÇÃO"}</strong>
                    <p>{note.content}</p>
                  </div>
                </article>
              ))
            ) : (
              <p className="stark-empty-msg">NENHUMA ANOTAÇÃO SALVA RECENTEMENTE.</p>
            )}
          </div>
        </div>
      </section>
    </div>
  );
}
