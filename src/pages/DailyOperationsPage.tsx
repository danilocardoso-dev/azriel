import { useState } from "react";
import { QuickCapture } from "../components/daily/QuickCapture";
import { ModuleIntro } from "../components/layout/ModuleIntro";
import { useDailyOperations } from "../contexts/useDailyOperations";
import type { DailyView, TaskPriority } from "../types";

const views: Array<[DailyView, string]> = [
  ["today", "HOJE"],
  ["priority", "PRIORIDADE"],
  ["inbox", "ENTRADA"],
  ["upcoming", "PRÓXIMAS"],
  ["completed", "CONCLUÍDAS"],
  ["notes", "NOTAS"],
];

const priorityLabels: Record<TaskPriority, string> = {
  critical: "CRÍTICA",
  high: "ALTA",
  medium: "MÉDIA",
  low: "BAIXA",
};

export function DailyOperationsPage({ initialCapture }: { initialCapture?: "task" | "note" }) {
  const { view, setView, tasks, notes, counters, loading, error, completeTask, deleteTask, deleteNote } =
    useDailyOperations();
  const [busy, setBusy] = useState<string | null>(null);

  const removeTask = async (id: string) => {
    setBusy(id);
    try {
      await deleteTask(id);
    } finally {
      setBusy(null);
    }
  };

  const finish = async (id: string) => {
    setBusy(id);
    try {
      await completeTask(id);
    } finally {
      setBusy(null);
    }
  };

  const isNotesView = view === "notes";

  return (
    <section className="module-page daily-page">
      <ModuleIntro
        code="OPS-09"
        title="Operações Diárias"
        description="Gestão tática de tarefas, prioridades e anotações rápidas com persistência local."
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
              <article key={note.id}>
                <div>
                  <h3>{note.title || "NOTA RÁPIDA"}</h3>
                  <p>{note.content}</p>
                </div>
                <div>
                  <button
                    type="button"
                    disabled={busy === note.id}
                    onClick={() => void deleteNote(note.id)}
                  >
                    EXCLUIR
                  </button>
                </div>
              </article>
            ))}
          </div>
        ) : (
          <p style={{ color: "var(--text-dim)", font: "11px var(--mono)", padding: "16px 0" }}>
            NENHUMA ANOTAÇÃO ATIVA ENCONTRADA.
          </p>
        )
      ) : tasks.length ? (
        <div className="daily-page__list">
          {tasks.map((task) => (
            <article key={task.id}>
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
                  {task.dueDate ? `PRAZO: ${task.dueDate}` : "SEM PRAZO"}
                </small>
                {task.description && <p>{task.description}</p>}
              </div>
              <div style={{ display: "flex", gap: "8px" }}>
                {task.status !== "completed" && (
                  <button
                    type="button"
                    disabled={busy === task.id}
                    onClick={() => void finish(task.id)}
                  >
                    CONCLUIR
                  </button>
                )}
                <button
                  type="button"
                  disabled={busy === task.id}
                  onClick={() => void removeTask(task.id)}
                >
                  EXCLUIR
                </button>
              </div>
            </article>
          ))}
        </div>
      ) : (
        <p style={{ color: "var(--text-dim)", font: "11px var(--mono)", padding: "16px 0" }}>
          NENHUMA TAREFA PENDENTE NESTE FILTRO.
        </p>
      )}
    </section>
  );
}
