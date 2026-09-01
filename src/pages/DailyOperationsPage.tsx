import { useMemo, useState } from "react";
import { NoteDetails } from "../components/daily/NoteDetails";
import { QuickCapture } from "../components/daily/QuickCapture";
import { TaskDetails } from "../components/daily/TaskDetails";
import { DataState } from "../components/layout/DataState";
import { ModuleIntro } from "../components/layout/ModuleIntro";
import { useAzrielData } from "../contexts/useAzrielData";
import { useDailyOperations } from "../contexts/useDailyOperations";
import { formatLocalDate, groupUpcoming, localDateKey } from "../services/dateService";
import type { DailyCounters, DailyView, Task } from "../types";

const navigationViews: Array<[DailyView, string]> = [["today","HOJE"],["inbox","CAIXA DE ENTRADA"],["upcoming","PRÓXIMAS"],["completed","CONCLUÍDAS"],["notes","NOTAS"],["archived_notes","ARQUIVADAS"]];
const counterViews: Array<[DailyView, keyof DailyCounters, string]> = [["pending","pending","PENDENTES"],["today","today","HOJE"],["overdue","overdue","ATRASADAS"],["priority","priority","PRIORIDADE"],["notes","notes","NOTAS"],["completed","completed","CONCLUÍDAS"]];
const viewLabels: Record<DailyView,string> = { pending: "PENDENTES", today: "HOJE", overdue: "ATRASADAS", priority: "PRIORIDADE", inbox: "CAIXA DE ENTRADA", upcoming: "PRÓXIMAS", completed: "CONCLUÍDAS", notes: "NOTAS", archived_notes: "ARQUIVADAS" };
const emptyLabels: Record<DailyView,string> = { pending: "Nenhuma tarefa pendente.", today: "Nenhuma tarefa para hoje.", overdue: "Nenhuma tarefa atrasada.", priority: "Nenhuma tarefa de alta prioridade.", inbox: "Caixa de entrada vazia.", upcoming: "Nenhuma tarefa futura.", completed: "Nenhuma atividade concluída.", notes: "Nenhuma nota ativa.", archived_notes: "Nenhuma nota arquivada." };

export function DailyOperationsPage({ initialCapture }: { initialCapture?: "task" | "note" }) {
  const { view, setView, tasks, notes, counters, loading, error, reload, completeTask } = useDailyOperations();
  const { projects, knowledgeAreas } = useAzrielData();
  const [selectedTaskId, setSelectedTaskId] = useState<string | null>(null);
  const [selectedNoteId, setSelectedNoteId] = useState<string | null>(null);
  const selectedTask = tasks.find((task) => task.id === selectedTaskId) ?? null;
  const selectedNote = notes.find((note) => note.id === selectedNoteId) ?? null;
  const upcoming = useMemo(() => groupUpcoming(tasks), [tasks]);
  const today = localDateKey();

  const selectView = (next: DailyView) => { setSelectedTaskId(null); setSelectedNoteId(null); setView(next); };
  const taskRow = (task: Task) => <div className={`daily-item daily-item--${task.priority}`} key={task.id} role="button" tabIndex={0} onClick={() => setSelectedTaskId(task.id)} onKeyDown={(event) => { if (event.key === "Enter" || event.key === " ") setSelectedTaskId(task.id); }}>
    <button className="daily-item__check" aria-label={`Concluir ${task.title}`} disabled={task.status === "completed"} onClick={(event) => { event.stopPropagation(); if (task.status !== "completed") void completeTask(task.id).catch(() => undefined); }}>{task.status === "completed" ? "✓" : ""}</button>
    <span><strong>{task.title}</strong><small>{task.dueDate ? `${formatLocalDate(task.dueDate)}${task.dueDate < today && task.status !== "completed" ? " · ATRASADA" : ""}` : "SEM PRAZO"}{task.projectId ? ` · ${projects.find((project) => project.id === task.projectId)?.name ?? task.projectId}` : ""}</small></span>
    <i>{task.priority}</i>
  </div>;

  const content = () => {
    if (loading || error) return <DataState loading={loading} error={error} onRetry={() => void reload()} />;
    if (view === "notes" || view === "archived_notes") return notes.length ? <div className="daily-list">{notes.map((note) => <button className="daily-note" key={note.id} onClick={() => setSelectedNoteId(note.id)}><span>{note.status === "archived" ? "ARQ" : "NOTE"}</span><strong>{note.title || note.content.slice(0, 55)}</strong><p>{note.content}</p><small>{knowledgeAreas.find((area) => area.id === note.knowledgeAreaId)?.name ?? "SEM VÍNCULO"}</small></button>)}</div> : <div className="daily-empty">{emptyLabels[view]}</div>;
    if (!tasks.length) return <div className="daily-empty">{emptyLabels[view]}</div>;
    if (view === "upcoming") return <div className="upcoming-groups">{(["tomorrow","thisWeek","later"] as const).map((group) => <section key={group}><header>{group === "tomorrow" ? "AMANHÃ" : group === "thisWeek" ? "ESTA SEMANA" : "POSTERIORMENTE"}<span>{upcoming[group].length}</span></header>{upcoming[group].length ? upcoming[group].map(taskRow) : <p>Nenhuma atividade.</p>}</section>)}</div>;
    return <div className="daily-list">{tasks.map(taskRow)}</div>;
  };

  return <>
    <ModuleIntro code="OPS-09" title="Operações Diárias" description="Central operacional para tarefas, prioridades e anotações rápidas." metric={`${counters.pending} PENDENTES / ${counters.priority} PRIORIDADES`} />
    <div className="daily-counters" aria-label="Filtros rápidos">{counterViews.map(([nextView, field, label]) => <button className={view === nextView ? "active" : ""} onClick={() => selectView(nextView)} aria-pressed={view === nextView} key={nextView}><strong>{counters[field]}</strong>{label}</button>)}</div>
    <div className="daily-layout">
      <aside className="daily-control"><QuickCapture initialKind={initialCapture} autoFocus={Boolean(initialCapture)} /><nav>{navigationViews.map(([id,label]) => <button className={view === id ? "active" : ""} onClick={() => selectView(id)} key={id}><span>{label}</span><strong>{id === "today" ? counters.today : id === "inbox" ? "IN" : id === "notes" ? counters.notes : id === "archived_notes" ? "ARQ" : "→"}</strong></button>)}</nav></aside>
      <section className="daily-worklist"><header><span>{viewLabels[view]}</span><strong>{view === "notes" || view === "archived_notes" ? notes.length : tasks.length} REGISTROS</strong></header>{content()}</section>
      <aside className="daily-inspector">{selectedTask ? <TaskDetails key={selectedTask.id} task={selectedTask} onClear={() => setSelectedTaskId(null)} /> : selectedNote ? <NoteDetails key={selectedNote.id} note={selectedNote} onClear={() => setSelectedNoteId(null)} /> : <div className="daily-empty"><strong>DETALHES</strong><p>Selecione uma tarefa ou nota para organizar seus campos.</p></div>}</aside>
    </div>
  </>;
}
