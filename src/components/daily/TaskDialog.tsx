import { useState, type FormEvent } from "react";
import { RecordEditorDialog } from "../core/RecordEditorDialog";
import { useDailyOperations } from "../../contexts/useDailyOperations";
import { formatTimestamp } from "../../services/dateService";
import type { Task, TaskInput, TaskPriority, TaskStatus } from "../../types";
import { DeleteConfirmationDialog } from "./DeleteConfirmationDialog";

const statuses: Array<[TaskStatus, string]> = [["inbox", "Entrada"], ["pending", "Pendente"], ["in_progress", "Em andamento"], ["completed", "Concluída"], ["cancelled", "Cancelada"]];
const priorities: Array<[TaskPriority, string]> = [["low", "Baixa"], ["medium", "Média"], ["high", "Alta"], ["critical", "Crítica"]];
const messageOf = (error: unknown) => error instanceof Error ? error.message : String(error);

/** Detalhes e edição de uma tarefa: tudo num modal, com exclusão confirmada. */
export function TaskDialog({ task, onClose }: { task: Task; onClose: () => void }) {
  const { saveTask, completeTask, deleteTask } = useDailyOperations();
  const [form, setForm] = useState<TaskInput>({ id: task.id, title: task.title, description: task.description, status: task.status, priority: task.priority, dueDate: task.dueDate });
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const change = <K extends keyof TaskInput>(field: K, value: TaskInput[K]) => setForm((current) => ({ ...current, [field]: value }));

  async function run(action: () => Promise<unknown>) {
    setBusy(true); setError(null);
    try { await action(); onClose(); }
    catch (reason) { setError(messageOf(reason)); setConfirmDelete(false); }
    finally { setBusy(false); }
  }

  function submit(event: FormEvent) {
    event.preventDefault();
    if (!form.title.trim()) { setError("Dê um título à tarefa."); return; }
    void run(() => saveTask({ ...form, title: form.title.trim(), dueDate: form.dueDate || null }));
  }

  const meta = <>Criada {formatTimestamp(task.createdAt)} · atualizada {formatTimestamp(task.updatedAt)}{task.completedAt ? ` · concluída ${formatTimestamp(task.completedAt)}` : ""}</>;
  return <>
    <RecordEditorDialog
      eyebrow={`OPS // TAREFA ${task.id.slice(0, 8).toUpperCase()}`}
      title={task.title}
      meta={meta}
      busy={busy}
      error={error}
      onCancel={onClose}
      onSubmit={submit}
      submitLabel="SALVAR ALTERAÇÕES"
      actions={<>
        {task.status !== "completed" && <button type="button" disabled={busy} onClick={() => void run(() => completeTask(task.id))}>✓ CONCLUIR</button>}
        <button type="button" className="record-editor__danger" disabled={busy} onClick={() => setConfirmDelete(true)}>EXCLUIR</button>
      </>}
    >
      <label className="record-editor__wide">Título<input value={form.title} onChange={(event) => change("title", event.target.value)} maxLength={200} /></label>
      <label className="record-editor__wide">Descrição<textarea rows={7} value={form.description} placeholder="Detalhes, contexto, próximos passos..." onChange={(event) => change("description", event.target.value)} /></label>
      <label>Prazo<input type="date" value={form.dueDate ?? ""} onChange={(event) => change("dueDate", event.target.value || null)} /></label>
      <label>Prioridade<select value={form.priority} onChange={(event) => change("priority", event.target.value as TaskPriority)}>{priorities.map(([value, label]) => <option value={value} key={value}>{label}</option>)}</select></label>
      <label>Status<select value={form.status} onChange={(event) => change("status", event.target.value as TaskStatus)}>{statuses.map(([value, label]) => <option value={value} key={value}>{label}</option>)}</select></label>
    </RecordEditorDialog>
    {confirmDelete && <DeleteConfirmationDialog kind="tarefa" title={task.title} busy={busy} onCancel={() => setConfirmDelete(false)} onConfirm={() => void run(() => deleteTask(task.id))} />}
  </>;
}
