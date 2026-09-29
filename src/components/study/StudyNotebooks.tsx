import { forwardRef, useCallback, useEffect, useImperativeHandle, useRef, useState, type FormEvent } from "react";
import { studyLabService } from "../../services/studyLabService";
import type { StudyNote, StudyNoteContext, StudyNotebook, StudyNotebookInput, StudyNoteSummary } from "../../types";
import { DeleteConfirmationDialog } from "../daily/DeleteConfirmationDialog";
import { StudyNoteEditor, type StudyNoteEditorHandle } from "./StudyNoteEditor";

export type StudyNoteLaunch = StudyNoteContext & {
  token: number;
  mode: "open" | "create";
  noteId?: string;
};

export type StudyNotebooksHandle = {
  flush: () => Promise<boolean>;
};

type Props = {
  launch: StudyNoteLaunch | null;
  onChanged: () => Promise<void>;
  onError: (message: string | null) => void;
  onCreateCard: (note: StudyNote) => void;
  onOpenLibrary: (note: StudyNote) => void;
};

const emptyContext: StudyNoteContext = { roadmapId: null, stageId: null, topicId: null, activityId: null, studySessionId: null };

export const StudyNotebooks = forwardRef<StudyNotebooksHandle, Props>(function StudyNotebooks({ launch, onChanged, onError, onCreateCard, onOpenLibrary }, ref) {
  const [notebooks, setNotebooks] = useState<StudyNotebook[]>([]);
  const [selectedNotebookId, setSelectedNotebookId] = useState<string | null>(null);
  const [notes, setNotes] = useState<StudyNoteSummary[]>([]);
  const [selectedNote, setSelectedNote] = useState<StudyNote | null>(null);
  const [showArchived, setShowArchived] = useState(false);
  const [search, setSearch] = useState("");
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState(false);
  const [contextFiltered, setContextFiltered] = useState(Boolean(!launch?.noteId && (launch?.activityId || launch?.studySessionId)));
  const [notebookDraft, setNotebookDraft] = useState<StudyNotebookInput | null>(null);
  const [notebookDeleteTarget, setNotebookDeleteTarget] = useState<StudyNotebook | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<StudyNote | null>(null);
  const editorRef = useRef<StudyNoteEditorHandle>(null);
  const contextRef = useRef<StudyNoteContext>(launch ?? emptyContext);

  useImperativeHandle(ref, () => ({ flush: () => editorRef.current?.flush() ?? Promise.resolve(true) }), []);

  const refreshNotebooks = useCallback(async () => {
    const next = await studyLabService.notebooks(true);
    setNotebooks(next);
    return next;
  }, []);

  const loadNotes = useCallback(async (notebookId: string | null, context: StudyNoteContext = emptyContext) => {
    if (!notebookId && !context.activityId && !context.studySessionId) {
      setNotes([]);
      return [];
    }
    const next = await studyLabService.notes({
      notebookId,
      activityId: context.activityId,
      studySessionId: context.studySessionId,
      limit: 100,
      offset: 0,
    });
    setNotes(next);
    return next;
  }, []);

  const createNote = useCallback(async (notebook: StudyNotebook, context: StudyNoteContext) => {
    const saved = await studyLabService.saveNote({
      id: crypto.randomUUID(),
      notebookId: notebook.id,
      title: "Nova nota",
      content: "",
      ...context,
    });
    setSelectedNotebookId(notebook.id);
    setSelectedNote(saved);
    await Promise.all([refreshNotebooks(), loadNotes(notebook.id, context), onChanged()]);
  }, [loadNotes, onChanged, refreshNotebooks]);

  useEffect(() => {
    const initialize = async () => {
      setLoading(true);
      onError(null);
      try {
        const available = await studyLabService.notebooks(true);
        setNotebooks(available);
        const active = available.filter((item) => item.status === "ACTIVE");
        if (launch?.mode === "open") {
          if (launch.noteId) {
            const exact = await studyLabService.note(launch.noteId);
            if (exact) {
              setShowArchived(available.some((item) => item.id === exact.notebookId && item.status === "ARCHIVED"));
              setSelectedNotebookId(exact.notebookId);
              await loadNotes(exact.notebookId);
              setSelectedNote(exact);
              return;
            }
          }
          const contextual = await studyLabService.notes({ activityId: launch.activityId, studySessionId: launch.studySessionId, limit: 100, offset: 0 });
          if (contextual.length) {
            setShowArchived(available.some((item) => item.id === contextual[0].notebookId && item.status === "ARCHIVED"));
            setSelectedNotebookId(contextual[0].notebookId);
            setNotes(contextual);
            setSelectedNote(await studyLabService.note(contextual[0].id));
            return;
          }
          if (active.length) {
            setSelectedNotebookId(active[0].id);
            await loadNotes(active[0].id, launch);
            return;
          }
        }
        if (!active.length) {
          setSelectedNotebookId(null);
          setNotes([]);
          return;
        }
        if (launch?.mode === "create") {
          await createNote(active[0], launch);
          return;
        }
        setSelectedNotebookId(active[0].id);
        await loadNotes(active[0].id);
      } catch (reason) {
        onError(reason instanceof Error ? reason.message : String(reason));
      } finally {
        setLoading(false);
      }
    };
    void initialize();
  }, [createNote, launch, loadNotes, onError]);

  async function selectNotebook(notebook: StudyNotebook) {
    if (!(await (editorRef.current?.flush() ?? Promise.resolve(true)))) return;
    setSelectedNote(null);
    setSelectedNotebookId(notebook.id);
    contextRef.current = emptyContext;
    setContextFiltered(false);
    await loadNotes(notebook.id);
  }

  async function selectNote(summary: StudyNoteSummary) {
    if (!(await (editorRef.current?.flush() ?? Promise.resolve(true)))) return;
    setBusy(true);
    onError(null);
    try {
      const fullNote = await studyLabService.note(summary.id);
      setSelectedNotebookId(summary.notebookId);
      setSelectedNote(fullNote);
    } catch (reason) {
      onError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setBusy(false);
    }
  }

  async function saveNotebook(event: FormEvent) {
    event.preventDefault();
    if (!notebookDraft) return;
    setBusy(true);
    onError(null);
    try {
      const saved = await studyLabService.saveNotebook(notebookDraft);
      setNotebookDraft(null);
      await refreshNotebooks();
      setSelectedNotebookId(saved.id);
      if (launch?.mode === "create" && !selectedNote) await createNote(saved, launch);
      else await loadNotes(saved.id);
      await onChanged();
    } catch (reason) {
      onError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setBusy(false);
    }
  }

  async function setNotebookStatus(notebook: StudyNotebook) {
    if (!(await (editorRef.current?.flush() ?? Promise.resolve(true)))) return;
    setBusy(true);
    onError(null);
    try {
      if (notebook.status === "ACTIVE") await studyLabService.archiveNotebook(notebook.id);
      else await studyLabService.restoreNotebook(notebook.id);
      const next = await refreshNotebooks();
      const nextActive = next.find((item) => item.status === "ACTIVE");
      setSelectedNotebookId(nextActive?.id ?? null);
      setSelectedNote(null);
      await loadNotes(nextActive?.id ?? null);
      await onChanged();
    } catch (reason) {
      onError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setBusy(false);
    }
  }

  async function requestNotebookDeletion(notebook: StudyNotebook) {
    if (!(await (editorRef.current?.flush() ?? Promise.resolve(true)))) {
      onError("Não foi possível salvar a nota atual antes de solicitar a exclusão do caderno.");
      return;
    }
    setNotebookDeleteTarget(notebook);
  }

  async function removeNotebook() {
    if (!notebookDeleteTarget) return;
    setBusy(true);
    onError(null);
    try {
      await studyLabService.deleteNotebook(notebookDeleteTarget.id);
      setNotebookDeleteTarget(null);
      setSelectedNote(null);
      contextRef.current = emptyContext;
      setContextFiltered(false);
      const next = await refreshNotebooks();
      const nextNotebook = next.find((item) => item.status === "ACTIVE") ?? (showArchived ? next[0] : null);
      setSelectedNotebookId(nextNotebook?.id ?? null);
      await loadNotes(nextNotebook?.id ?? null);
      await onChanged();
    } catch (reason) {
      onError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setBusy(false);
    }
  }

  async function runSearch(event: FormEvent) {
    event.preventDefault();
    if (!(await (editorRef.current?.flush() ?? Promise.resolve(true)))) return;
    setSelectedNote(null);
    if (!search.trim()) {
      await loadNotes(selectedNotebookId);
      return;
    }
    setBusy(true);
    try {
      setNotes(await studyLabService.searchNotes({ query: search, limit: 50 }));
    } catch (reason) {
      onError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setBusy(false);
    }
  }

  async function refreshCurrentList() {
    await Promise.all([refreshNotebooks(), loadNotes(selectedNotebookId, contextRef.current), onChanged()]);
  }

  async function removeNote() {
    if (!deleteTarget) return;
    setBusy(true);
    try {
      await studyLabService.deleteNote(deleteTarget.id);
      setDeleteTarget(null);
      setSelectedNote(null);
      await refreshCurrentList();
    } catch (reason) {
      onError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setBusy(false);
    }
  }

  async function createCardFromNote(note: StudyNote) {
    if (!(await (editorRef.current?.flush() ?? Promise.resolve(true)))) {
      onError("Salve a nota antes de criar um Study Card vinculado.");
      return;
    }
    onCreateCard(note);
  }

  const visibleNotebooks = notebooks.filter((item) => showArchived || item.status === "ACTIVE");
  const selectedNotebook = notebooks.find((item) => item.id === selectedNotebookId) ?? null;
  if (loading) return <div className="core-empty">CARREGANDO KNOWLEDGE WORKSPACE...</div>;

  return <section className="study-notebooks">
    <aside className="study-notebook-list">
      <header><div><span>CADERNOS</span><b>{visibleNotebooks.length}</b></div><button onClick={() => setNotebookDraft({ id: crypto.randomUUID(), title: "", description: null })}>＋ NOVO</button></header>
      <label className="study-archive-toggle"><input type="checkbox" checked={showArchived} onChange={(event) => setShowArchived(event.target.checked)} /> MOSTRAR ARQUIVADOS</label>
      <div>{visibleNotebooks.map((notebook) => <button key={notebook.id} className={selectedNotebookId === notebook.id ? "active" : ""} onClick={() => void selectNotebook(notebook)}><span><strong>{notebook.title}</strong><small>{notebook.description || "Sem descrição"}</small><time>{new Date(notebook.lastNoteUpdatedAt ?? notebook.updatedAt).toLocaleString("pt-BR")}</time></span><b>{notebook.noteCount}</b><em data-status={notebook.status}>{notebook.status}</em></button>)}{!visibleNotebooks.length && <p>Nenhum caderno cadastrado.</p>}</div>
      {selectedNotebook && <footer><button onClick={() => setNotebookDraft({ id: selectedNotebook.id, title: selectedNotebook.title, description: selectedNotebook.description })}>EDITAR</button><button onClick={() => void setNotebookStatus(selectedNotebook)}>{selectedNotebook.status === "ACTIVE" ? "ARQUIVAR" : "RESTAURAR"}</button><button className="danger-link" onClick={() => void requestNotebookDeletion(selectedNotebook)}>EXCLUIR</button></footer>}
    </aside>
    <section className="study-note-list">
      <header><form onSubmit={runSearch}><input value={search} onChange={(event) => setSearch(event.target.value)} placeholder="Buscar em notas e cadernos..." /><button disabled={busy}>BUSCAR</button></form><button disabled={!selectedNotebook || selectedNotebook.status !== "ACTIVE" || busy} onClick={() => selectedNotebook && void createNote(selectedNotebook, contextRef.current)}>＋ NOVA NOTA</button></header>
      {contextFiltered && <div className="study-note-filter"><span>CONTEXTO ATIVO</span><button onClick={() => { contextRef.current = emptyContext; setContextFiltered(false); void loadNotes(selectedNotebookId); }}>LIMPAR FILTRO</button></div>}
      <div>{notes.map((note) => <button key={note.id} className={selectedNote?.id === note.id ? "active" : ""} onClick={() => void selectNote(note)}><header><strong>{note.title}</strong><time>{new Date(note.updatedAt).toLocaleString("pt-BR")}</time></header><p>{note.preview || "Nota vazia"}</p><footer><span>{note.notebookTitle}</span><small>{note.activityTitle || note.roadmapName || "SEM CONTEXTO"}</small></footer></button>)}{!notes.length && <p className="study-note-list__empty">Nenhuma nota encontrada.</p>}</div>
    </section>
    <main className="study-note-workspace">{selectedNote ? <StudyNoteEditor key={selectedNote.id} ref={editorRef} note={selectedNote} readOnly={selectedNotebook?.status === "ARCHIVED"} onSave={studyLabService.saveNote} onSaved={refreshCurrentList} onDelete={setDeleteTarget} onCreateCard={createCardFromNote} onOpenLibrary={onOpenLibrary} /> : <div className="study-note-workspace__empty"><span>KNOWLEDGE WORKSPACE</span><h2>Selecione ou crie uma nota.</h2><p>Markdown local, contexto acadêmico e autosave confiável.</p></div>}</main>
    {notebookDraft && <div className="study-notebook-form"><form onSubmit={saveNotebook}><header><strong>{notebooks.some((item) => item.id === notebookDraft.id) ? "EDITAR CADERNO" : "NOVO CADERNO"}</strong><button type="button" onClick={() => setNotebookDraft(null)}>×</button></header><label>TÍTULO<input autoFocus maxLength={160} value={notebookDraft.title} onChange={(event) => setNotebookDraft({ ...notebookDraft, title: event.target.value })} /></label><label>DESCRIÇÃO<textarea maxLength={1000} value={notebookDraft.description ?? ""} onChange={(event) => setNotebookDraft({ ...notebookDraft, description: event.target.value || null })} /></label><footer><button type="button" onClick={() => setNotebookDraft(null)}>CANCELAR</button><button disabled={busy || !notebookDraft.title.trim()}>SALVAR</button></footer></form></div>}
    {notebookDeleteTarget && <DeleteConfirmationDialog kind="caderno" title={notebookDeleteTarget.title} busy={busy} description={`Esta ação excluirá definitivamente o caderno e ${notebookDeleteTarget.noteCount} nota${notebookDeleteTarget.noteCount === 1 ? "" : "s"} contida${notebookDeleteTarget.noteCount === 1 ? "" : "s"} nele. Roadmaps, atividades e sessões de estudo permanecerão intactos.`} onCancel={() => setNotebookDeleteTarget(null)} onConfirm={() => void removeNotebook()} />}
    {deleteTarget && <DeleteConfirmationDialog kind="anotação" title={deleteTarget.title} busy={busy} description="A nota de estudo será removida definitivamente. O caderno, a sessão e o roadmap permanecerão intactos." onCancel={() => setDeleteTarget(null)} onConfirm={() => void removeNote()} />}
  </section>;
});
