import { useState, type FormEvent } from "react";
import { useAzrielData } from "../../contexts/useAzrielData";
import { useDailyOperations } from "../../contexts/useDailyOperations";
import type { Task, TaskInput, TaskPriority, TaskStatus } from "../../types";
import { formatLocalDate, formatTimestamp, parseBrazilianDateInput } from "../../services/dateService";
import { DeleteConfirmationDialog } from "./DeleteConfirmationDialog";

const statuses: Array<[TaskStatus, string]> = [["inbox","CAIXA DE ENTRADA"],["pending","PENDENTE"],["in_progress","EM ANDAMENTO"],["completed","CONCLUÍDA"],["cancelled","CANCELADA"]];
const priorities: Array<[TaskPriority, string]> = [["low","BAIXA"],["medium","MÉDIA"],["high","ALTA"],["critical","CRÍTICA"]];
const statusLabel = Object.fromEntries(statuses) as Record<TaskStatus, string>;
const priorityLabel = Object.fromEntries(priorities) as Record<TaskPriority, string>;

const taskInput = (task: Task): TaskInput => ({ id: task.id, title: task.title, description: task.description, status: task.status, priority: task.priority, dueDate: task.dueDate, projectId: task.projectId, knowledgeAreaId: task.knowledgeAreaId });

export function TaskDetails({ task, onClear }: { task: Task; onClear: () => void }) {
  const { projects, knowledgeAreas } = useAzrielData();
  const { saveTask, completeTask, deleteTask } = useDailyOperations();
  const [form, setForm] = useState<TaskInput>(() => taskInput(task));
  const [dueDateText, setDueDateText] = useState(() => task.dueDate ? formatLocalDate(task.dueDate) : "");
  const [editing, setEditing] = useState(false);
  const [busy, setBusy] = useState(false);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const change = <K extends keyof TaskInput>(field: K, value: TaskInput[K]) => setForm((current) => ({ ...current, [field]: value }));
  const projectName = projects.find((project) => project.id === task.projectId)?.name ?? "SEM PROJETO";
  const knowledgeName = knowledgeAreas.find((area) => area.id === task.knowledgeAreaId)?.name ?? "SEM CONHECIMENTO";

  const openEditor = () => {
    setForm(taskInput(task));
    setDueDateText(task.dueDate ? formatLocalDate(task.dueDate) : "");
    setMessage(null);
    setEditing(true);
  };

  async function submit(event: FormEvent) {
    event.preventDefault(); setBusy(true); setMessage(null);
    try {
      await saveTask({ ...form, dueDate: parseBrazilianDateInput(dueDateText) });
      setMessage("Tarefa atualizada.");
      setEditing(false);
    }
    catch (error) { setMessage(error instanceof Error ? error.message : String(error)); }
    finally { setBusy(false); }
  }

  async function complete() { setBusy(true); setMessage(null); try { await completeTask(task.id); onClear(); } catch (error) { setMessage(error instanceof Error ? error.message : String(error)); } finally { setBusy(false); } }
  async function remove() {
    setBusy(true); setMessage(null);
    try { await deleteTask(task.id); onClear(); }
    catch (error) { setMessage(error instanceof Error ? error.message : String(error)); setConfirmDelete(false); }
    finally { setBusy(false); }
  }

  return <>
    <article className="task-overview">
      <header><span>TASK // {task.id.slice(0, 8).toUpperCase()}</span><strong>VISÃO GERAL</strong></header>
      <section className="task-overview__hero">
        <span>TÍTULO</span><h2>{task.title}</h2>
        <span>DESCRIÇÃO</span><p>{task.description || "Nenhuma descrição registrada."}</p>
      </section>
      <dl className="task-overview__facts">
        <div><dt>Status</dt><dd>{statusLabel[task.status]}</dd></div>
        <div><dt>Prioridade</dt><dd data-priority={task.priority}>{priorityLabel[task.priority]}</dd></div>
        <div><dt>Prazo</dt><dd>{task.dueDate ? formatLocalDate(task.dueDate) : "SEM PRAZO"}</dd></div>
        <div><dt>Projeto</dt><dd>{projectName}</dd></div>
        <div className="task-overview__wide"><dt>Conhecimento</dt><dd>{knowledgeName}</dd></div>
      </dl>
      <div className="task-overview__timestamps"><span>CRIADA<strong>{formatTimestamp(task.createdAt)}</strong></span><span>ATUALIZADA<strong>{formatTimestamp(task.updatedAt)}</strong></span>{task.completedAt && <span>CONCLUÍDA<strong>{formatTimestamp(task.completedAt)}</strong></span>}</div>
      <footer className="task-overview__actions"><button type="button" onClick={openEditor} disabled={busy}>EDITAR</button>{task.status !== "completed" && <button type="button" onClick={() => void complete()} disabled={busy}>CONCLUIR</button>}<button type="button" className="danger" onClick={() => setConfirmDelete(true)} disabled={busy}>EXCLUIR</button></footer>
      {message && <p className="task-overview__message" role="status">{message}</p>}
    </article>

    {editing && <div className="record-editor__backdrop" role="dialog" aria-modal="true" aria-label={`Editar tarefa ${task.title}`}>
      <div className="record-editor task-edit-dialog">
        <form onSubmit={submit}>
          <header><span>OPS // EDITAR TAREFA</span><button type="button" onClick={() => setEditing(false)} aria-label="Fechar editor">×</button></header>
          <div className="record-editor__title"><h2>{task.title}</h2><small>Atualize os campos operacionais e salve para retornar à visão geral.</small></div>
          <div className="record-editor__fields">
            <label className="record-editor__wide">Título<input value={form.title} onChange={(event) => change("title", event.target.value)} /></label>
            <label className="record-editor__wide">Descrição<textarea rows={9} value={form.description} onChange={(event) => change("description", event.target.value)} /></label>
            <label>Status<select value={form.status} onChange={(event) => change("status", event.target.value as TaskStatus)}>{statuses.map(([value,label]) => <option value={value} key={value}>{label}</option>)}</select></label>
            <label>Prioridade<select value={form.priority} onChange={(event) => change("priority", event.target.value as TaskPriority)}>{priorities.map(([value,label]) => <option value={value} key={value}>{label}</option>)}</select></label>
            <label>Prazo (DD/MM/AAAA)<input type="text" inputMode="numeric" maxLength={10} placeholder="DD/MM/AAAA" value={dueDateText} onChange={(event) => setDueDateText(event.target.value)} /></label>
            <label>Projeto<select value={form.projectId ?? ""} onChange={(event) => change("projectId", event.target.value || null)}><option value="">SEM PROJETO</option>{projects.map((project) => <option value={project.id} key={project.id}>{project.name}</option>)}</select></label>
            <label className="record-editor__wide">Conhecimento<select value={form.knowledgeAreaId ?? ""} onChange={(event) => change("knowledgeAreaId", event.target.value || null)}><option value="">SEM CONHECIMENTO</option>{knowledgeAreas.map((area) => <option value={area.id} key={area.id}>{area.name}</option>)}</select></label>
          </div>
          {message && <p className="form-error" role="alert">{message}</p>}
          <footer><button type="button" onClick={() => setEditing(false)} disabled={busy}>CANCELAR</button><button disabled={busy}>{busy ? "SALVANDO..." : "SALVAR ALTERAÇÕES"}</button></footer>
        </form>
      </div>
    </div>}

    {confirmDelete && <DeleteConfirmationDialog kind="tarefa" title={task.title} busy={busy} onCancel={() => setConfirmDelete(false)} onConfirm={() => void remove()} />}
  </>;
}
