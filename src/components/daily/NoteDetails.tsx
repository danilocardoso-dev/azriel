import { useState, type FormEvent } from "react";
import { useAzrielData } from "../../contexts/useAzrielData";
import { useDailyOperations } from "../../contexts/useDailyOperations";
import type { Note, NoteInput } from "../../types";
import { formatTimestamp } from "../../services/dateService";
import { DeleteConfirmationDialog } from "./DeleteConfirmationDialog";

export function NoteDetails({ note, onClear }: { note: Note; onClear: () => void }) {
  const { projects, knowledgeAreas } = useAzrielData();
  const { saveNote, archiveNote, deleteNote } = useDailyOperations();
  const [form, setForm] = useState<NoteInput>({ id: note.id, title: note.title, content: note.content, status: note.status, projectId: note.projectId, knowledgeAreaId: note.knowledgeAreaId });
  const [busy, setBusy] = useState(false);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const change = <K extends keyof NoteInput>(field: K, value: NoteInput[K]) => setForm((current) => ({ ...current, [field]: value }));

  async function submit(event: FormEvent) { event.preventDefault(); setBusy(true); setMessage(null); try { await saveNote(form); setMessage("Nota atualizada."); } catch (error) { setMessage(error instanceof Error ? error.message : String(error)); } finally { setBusy(false); } }
  async function archive() { setBusy(true); setMessage(null); try { await archiveNote(note.id); onClear(); } catch (error) { setMessage(error instanceof Error ? error.message : String(error)); } finally { setBusy(false); } }
  async function remove() { setBusy(true); setMessage(null); try { await deleteNote(note.id); onClear(); } catch (error) { setMessage(error instanceof Error ? error.message : String(error)); setConfirmDelete(false); } finally { setBusy(false); } }

  return <form className="daily-details" onSubmit={submit}>
    <header><span>NOTE // {note.id.slice(0, 8).toUpperCase()}</span><strong>DETALHES</strong></header>
    <label>Título opcional<input value={form.title ?? ""} onChange={(event) => change("title", event.target.value || null)} /></label>
    <label>Conteúdo<textarea rows={9} value={form.content} onChange={(event) => change("content", event.target.value)} /></label>
    <label>Projeto<select value={form.projectId ?? ""} onChange={(event) => change("projectId", event.target.value || null)}><option value="">SEM PROJETO</option>{projects.map((project) => <option value={project.id} key={project.id}>{project.name}</option>)}</select></label>
    <label>Conhecimento<select value={form.knowledgeAreaId ?? ""} onChange={(event) => change("knowledgeAreaId", event.target.value || null)}><option value="">SEM CONHECIMENTO</option>{knowledgeAreas.map((area) => <option value={area.id} key={area.id}>{area.name}</option>)}</select></label>
    <div className="daily-timestamps"><span>CRIADA<strong>{formatTimestamp(note.createdAt)}</strong></span><span>ATUALIZADA<strong>{formatTimestamp(note.updatedAt)}</strong></span></div>
    <div className="daily-details__actions"><button disabled={busy}>SALVAR</button><button type="button" onClick={() => void archive()} disabled={busy}>ARQUIVAR</button><button type="button" className="danger" onClick={() => setConfirmDelete(true)} disabled={busy}>EXCLUIR</button></div>
    {message && <p role="status">{message}</p>}
    {confirmDelete && <DeleteConfirmationDialog kind="anotação" title={note.title || note.content.slice(0, 90)} busy={busy} onCancel={() => setConfirmDelete(false)} onConfirm={() => void remove()} />}
  </form>;
}
