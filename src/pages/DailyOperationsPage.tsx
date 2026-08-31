import { useMemo, useState } from "react";
import { NoteDetails } from "../components/daily/NoteDetails";
import { QuickCapture } from "../components/daily/QuickCapture";
import { TaskDetails } from "../components/daily/TaskDetails";
import { DataState } from "../components/layout/DataState";
import { ModuleIntro } from "../components/layout/ModuleIntro";
import { useAzrielData } from "../contexts/useAzrielData";
import { useDailyOperations } from "../contexts/useDailyOperations";
import { formatLocalDate, groupUpcoming, localDateKey } from "../services/dateService";
import type { DailyView, Task } from "../types";

const views: Array<[DailyView, string]> = [["today","HOJE"],["inbox","CAIXA DE ENTRADA"],["upcoming","PRÓXIMAS"],["completed","CONCLUÍDAS"],["notes","NOTAS"]];
const emptyLabels: Record<DailyView,string> = { today: "Nenhuma tarefa para hoje.", inbox: "Caixa de entrada vazia.", upcoming: "Nenhuma tarefa futura.", completed: "Nenhuma atividade concluída.", notes: "Nenhuma nota ativa." };

export function DailyOperationsPage() {
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
    if (view === "notes") return notes.length ? <div className="daily-list">{notes.map((note) => <button className="daily-note" key={note.id} onClick={() => setSelectedNoteId(note.id)}><span>NOTE</span><strong>{note.title || note.content.slice(0, 55)}</strong><p>{note.content}</p><small>{knowledgeAreas.find((area) => area.id === note.knowledgeAreaId)?.name ?? "SEM VÍNCULO"}</small></button>)}</div> : <div className="daily-empty">{emptyLabels[view]}</div>;
    if (!tasks.length) return <div className="daily-empty">{emptyLabels[view]}</div>;
    if (view === "upcoming") return <div className="upcoming-groups">{(["tomorrow","thisWeek","later"] as const).map((group) => <section key={group}><header>{group === "tomorrow" ? "AMANHÃ" : group === "thisWeek" ? "ESTA SEMANA" : "POSTERIORMENTE"}<span>{upcoming[group].length}</span></header>{upcoming[group].length ? upcoming[group].map(taskRow) : <p>Nenhuma atividade.</p>}</section>)}</div>;
    return <div className="daily-list">{tasks.map(taskRow)}</div>;
  };

  return <>
    <ModuleIntro code="OPS-09" title="Operações Diárias" description="Central operacional para tarefas, prioridades e anotações rápidas." metric={`${counters.pending} PENDENTES / ${counters.priority} PRIORIDADES`} />
    <div className="daily-counters"><span><strong>{counters.pending}</strong>PENDENTES</span><span><strong>{counters.today}</strong>HOJE</span><span><strong>{counters.overdue}</strong>ATRASADAS</span><span><strong>{counters.priority}</strong>PRIORIDADE</span><span><strong>{counters.notes}</strong>NOTAS</span></div>
    <div className="daily-layout">
      <aside className="daily-control"><QuickCapture /><nav>{views.map(([id,label]) => <button className={view === id ? "active" : ""} onClick={() => selectView(id)} key={id}><span>{label}</span><strong>{id === "today" ? counters.today : id === "inbox" ? "IN" : id === "notes" ? counters.notes : "→"}</strong></button>)}</nav></aside>
      <section className="daily-worklist"><header><span>{views.find(([id]) => id === view)?.[1]}</span><strong>{view === "notes" ? notes.length : tasks.length} REGISTROS</strong></header>{content()}</section>
      <aside className="daily-inspector">{selectedTask ? <TaskDetails key={selectedTask.id} task={selectedTask} onClear={() => setSelectedTaskId(null)} /> : selectedNote ? <NoteDetails key={selectedNote.id} note={selectedNote} onClear={() => setSelectedNoteId(null)} /> : <div className="daily-empty"><strong>DETALHES</strong><p>Selecione uma tarefa ou nota para organizar seus campos.</p></div>}</aside>
    </div>
  </>;
}
