import { useState } from "react";
import { NoteDialog } from "../components/daily/NoteDialog";
import { QuickCapture } from "../components/daily/QuickCapture";
import { TaskDialog } from "../components/daily/TaskDialog";
import { ModuleIntro } from "../components/layout/ModuleIntro";
import { useDailyOperations } from "../contexts/useDailyOperations";
import { formatLocalDate, localDateKey } from "../services/dateService";
import type { DailyView, TaskPriority } from "../types";

const views: Array<[DailyView, string]> = [
  ["today", "HOJE"],
  // Única aba com todas as tarefas abertas: sem ela, tarefas sem prazo e sem prioridade alta somem da tela.
  ["pending", "PENDENTES"],
  ["priority", "PRIORIDADE"],
  ["inbox", "ENTRADA"],
  ["upcoming", "PRÓXIMAS"],
  ["completed", "CONCLUÍDAS"],
  ["notes", "NOTAS"],
  ["archived_notes", "ARQUIVADAS"],
];

const priorityLabels: Record<TaskPriority, string> = {
  critical: "CRÍTICA",
  high: "ALTA",
  medium: "MÉDIA",
  low: "BAIXA",
};

export function DailyOperationsPage({ initialCapture }: { initialCapture?: "task" | "note" }) {
  const { view, setView, tasks, notes, counters, loading, error, completeTask } = useDailyOperations();
  const [busy, setBusy] = useState<string | null>(null);
  const [selectedTaskId, setSelectedTaskId] = useState<string | null>(null);
  const [selectedNoteId, setSelectedNoteId] = useState<string | null>(null);
  const selectedTask = tasks.find((task) => task.id === selectedTaskId) ?? null;
  const selectedNote = notes.find((note) => note.id === selectedNoteId) ?? null;
  const today = localDateKey();

  const finish = async (id: string) => {
    setBusy(id);
    try {
      await completeTask(id);
    } finally {
      setBusy(null);
    }
  };

  const isNotesView = view === "notes" || view === "archived_notes";

  return (
    <section className="module-page daily-page">
      <ModuleIntro
        code="OPS-09"
        title="Operações Diárias"
        description="Clique em uma tarefa ou nota para ver os detalhes, editar, definir prazo ou excluir."
        metric={`${counters.pending} PENDENTES // ${counters.today} HOJE`}
      />

      <QuickCapture initialKind={initialCapture} autoFocus={Boolean(initialCapture)} />

      <nav className="daily-page__tabs" aria-label="Filtros operacionais">
        {views.map(([id, label]) => (
          <button
            key={id}
            type="button"
            className={view === id ? "active" : ""}
            onClick={() => setView(id)}
          >
            {label}
            {id === "today" && counters.today > 0 ? ` (${counters.today})` : ""}
            {id === "pending" && counters.pending > 0 ? ` (${counters.pending})` : ""}
            {id === "priority" && counters.priority > 0 ? ` (${counters.priority})` : ""}
          </button>
        ))}
      </nav>

      {error && <p role="alert" style={{ color: "var(--red-critical)", margin: "4px 0" }}>{error}</p>}

      {loading ? (
        <p style={{ color: "var(--text-muted)", font: "11px var(--mono)" }}>
          Carregando registros locais do Azriel…
        </p>
      ) : isNotesView ? (
        notes.length ? (
          <div className="daily-page__list">
            {notes.map((note) => (
              <article key={note.id} className="daily-page__item" role="button" tabIndex={0} onClick={() => setSelectedNoteId(note.id)} onKeyDown={(event) => { if (event.key === "Enter") setSelectedNoteId(note.id); }}>
                <div>
                  <h3>{note.title || "NOTA RÁPIDA"}</h3>
                  <p>{note.content}</p>
                </div>
                <div>
                  <button type="button" onClick={(event) => { event.stopPropagation(); setSelectedNoteId(note.id); }}>ABRIR</button>
                </div>
              </article>
            ))}
          </div>
        ) : (
          <p style={{ color: "var(--text-dim)", font: "11px var(--mono)", padding: "16px 0" }}>
            {view === "archived_notes" ? "NENHUMA ANOTAÇÃO ARQUIVADA." : "NENHUMA ANOTAÇÃO ATIVA ENCONTRADA."}
          </p>
        )
      ) : tasks.length ? (
        <div className="daily-page__list">
          {tasks.map((task) => (
            <article key={task.id} className="daily-page__item" role="button" tabIndex={0} onClick={() => setSelectedTaskId(task.id)} onKeyDown={(event) => { if (event.key === "Enter") setSelectedTaskId(task.id); }}>
              <div>
                <b>{task.title}</b>
                <small>
                  <span
                    style={{
                      color:
                        task.priority === "critical"
                          ? "var(--red-critical)"
                          : task.priority === "high"
                          ? "var(--amber-alert)"
                          : "var(--cyan)",
                      fontWeight: 700,
                    }}
                  >
                    {priorityLabels[task.priority]}
                  </span>
                  {" · "}
                  {task.dueDate
                    ? `PRAZO: ${formatLocalDate(task.dueDate)}${task.dueDate < today && task.status !== "completed" ? " · ATRASADA" : ""}`
                    : "SEM PRAZO"}
                </small>
                {task.description && <p>{task.description}</p>}
              </div>
              <div style={{ display: "flex", gap: "8px" }}>
                {task.status !== "completed" && (
                  <button
                    type="button"
                    disabled={busy === task.id}
                    onClick={(event) => { event.stopPropagation(); void finish(task.id); }}
                  >
                    CONCLUIR
                  </button>
                )}
                <button type="button" onClick={(event) => { event.stopPropagation(); setSelectedTaskId(task.id); }}>ABRIR</button>
              </div>
            </article>
          ))}
        </div>
      ) : (
        <p style={{ color: "var(--text-dim)", font: "11px var(--mono)", padding: "16px 0" }}>
          NENHUMA TAREFA NESTE FILTRO.
        </p>
      )}

      {selectedTask && <TaskDialog key={selectedTask.id} task={selectedTask} onClose={() => setSelectedTaskId(null)} />}
      {selectedNote && <NoteDialog key={selectedNote.id} note={selectedNote} onClose={() => setSelectedNoteId(null)} />}
    </section>
  );
}
