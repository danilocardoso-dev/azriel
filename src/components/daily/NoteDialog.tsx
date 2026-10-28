import { useState, type FormEvent } from "react";
import { RecordEditorDialog } from "../core/RecordEditorDialog";
import { useDailyOperations } from "../../contexts/useDailyOperations";
import { formatTimestamp } from "../../services/dateService";
import type { Note, NoteInput } from "../../types";
import { DeleteConfirmationDialog } from "./DeleteConfirmationDialog";

const messageOf = (error: unknown) => error instanceof Error ? error.message : String(error);

/** Leitura e edição de uma nota, com arquivar/restaurar e exclusão confirmada. */
export function NoteDialog({ note, onClose }: { note: Note; onClose: () => void }) {
  const { saveNote, archiveNote, restoreNote, deleteNote } = useDailyOperations();
  const [form, setForm] = useState<NoteInput>({ id: note.id, title: note.title, content: note.content, status: note.status });
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [confirmDelete, setConfirmDelete] = useState(false);

  async function run(action: () => Promise<unknown>) {
    setBusy(true); setError(null);
    try { await action(); onClose(); }
    catch (reason) { setError(messageOf(reason)); setConfirmDelete(false); }
    finally { setBusy(false); }
  }

  function submit(event: FormEvent) {
    event.preventDefault();
    if (!form.content.trim()) { setError("A nota não pode ficar vazia."); return; }
    void run(() => saveNote({ ...form, title: form.title?.trim() || null }));
  }

  const label = note.title || note.content.slice(0, 60);
  return <>
    <RecordEditorDialog
      eyebrow={`OPS // NOTA ${note.id.slice(0, 8).toUpperCase()}`}
      title={label}
      meta={<>Criada {formatTimestamp(note.createdAt)} · atualizada {formatTimestamp(note.updatedAt)}{note.status === "archived" ? " · arquivada" : ""}</>}
      busy={busy}
      error={error}
      onCancel={onClose}
      onSubmit={submit}
      submitLabel="SALVAR ALTERAÇÕES"
      actions={<>
        {note.status === "archived"
          ? <button type="button" disabled={busy} onClick={() => void run(() => restoreNote(note.id))}>RESTAURAR</button>
          : <button type="button" disabled={busy} onClick={() => void run(() => archiveNote(note.id))}>ARQUIVAR</button>}
        <button type="button" className="record-editor__danger" disabled={busy} onClick={() => setConfirmDelete(true)}>EXCLUIR</button>
      </>}
    >
      <label className="record-editor__wide">Título (opcional)<input value={form.title ?? ""} maxLength={200} onChange={(event) => setForm({ ...form, title: event.target.value || null })} /></label>
      <label className="record-editor__wide">Conteúdo<textarea rows={11} value={form.content} onChange={(event) => setForm({ ...form, content: event.target.value })} /></label>
    </RecordEditorDialog>
    {confirmDelete && <DeleteConfirmationDialog kind="anotação" title={label} busy={busy} onCancel={() => setConfirmDelete(false)} onConfirm={() => void run(() => deleteNote(note.id))} />}
  </>;
}
