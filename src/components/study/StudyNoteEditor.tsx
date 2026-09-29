import { forwardRef, useCallback, useEffect, useImperativeHandle, useRef, useState } from "react";
import type { StudyAIContext, StudyCardInput, StudyNote, StudyNoteInput } from "../../types";
import type { StudyAICardDraft } from "../../ai/study/studyAIDrafts";
import { appendStudyAIContent } from "../../ai/study/studyAIText";
import { studyLabService } from "../../services/studyLabService";
import { StudyAIToolPanel } from "./StudyAIToolPanel";
import { StudyMarkdown } from "./StudyMarkdown";
import { StudyRelatedMaterials } from "./StudyRelatedMaterials";
import { reconcileStudyNoteSave, studyNoteFingerprint } from "./studyNoteDraft";

export type StudyNoteEditorHandle = {
  flush: () => Promise<boolean>;
  hasUnsavedChanges: () => boolean;
};

type SaveStatus = "saved" | "dirty" | "saving" | "error";

type Props = {
  note: StudyNote;
  readOnly: boolean;
  onSave: (input: StudyNoteInput) => Promise<StudyNote>;
  onSaved: () => Promise<void>;
  onDelete: (note: StudyNote) => void;
  onCreateCard: (note: StudyNote) => Promise<void>;
  onOpenLibrary: (note: StudyNote) => void;
};

const toInput = (note: StudyNote): StudyNoteInput => ({
  id: note.id,
  notebookId: note.notebookId,
  title: note.title,
  content: note.content,
  roadmapId: note.roadmapId,
  stageId: note.stageId,
  topicId: note.topicId,
  activityId: note.activityId,
  studySessionId: note.studySessionId,
});

export const StudyNoteEditor = forwardRef<StudyNoteEditorHandle, Props>(function StudyNoteEditor(
  { note, readOnly, onSave, onSaved, onDelete, onCreateCard, onOpenLibrary },
  ref,
) {
  const [draft, setDraft] = useState<StudyNoteInput>(() => toInput(note));
  const [status, setStatus] = useState<SaveStatus>("saved");
  const [error, setError] = useState<string | null>(null);
  const [preview, setPreview] = useState(false);
  const [showAI, setShowAI] = useState(false);
  const [selectedText, setSelectedText] = useState("");
  const draftRef = useRef(draft);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const persistedRef = useRef(toInput(note));
  const savePromiseRef = useRef<Promise<boolean> | null>(null);

  const updateDraft = (next: StudyNoteInput) => {
    draftRef.current = next;
    setDraft(next);
    setStatus(studyNoteFingerprint(next) === studyNoteFingerprint(persistedRef.current) ? "saved" : "dirty");
    setError(null);
  };

  const flush = useCallback(async (): Promise<boolean> => {
    if (readOnly) return true;
    if (savePromiseRef.current) {
      const previousSucceeded = await savePromiseRef.current;
      if (!previousSucceeded) return false;
    }
    if (studyNoteFingerprint(draftRef.current) === studyNoteFingerprint(persistedRef.current)) {
      setStatus("saved");
      return true;
    }
    const snapshot = { ...draftRef.current };
    setStatus("saving");
    setError(null);
    const operation = onSave(snapshot)
      .then(async (saved) => {
        const persisted = toInput(saved);
        const reconciliation = reconcileStudyNoteSave(snapshot, draftRef.current, persisted);
        persistedRef.current = reconciliation.persisted;
        if (draftRef.current !== reconciliation.draft) {
          draftRef.current = reconciliation.draft;
          setDraft(reconciliation.draft);
        }
        await onSaved();
        setStatus(reconciliation.dirty ? "dirty" : "saved");
        return true;
      })
      .catch((reason: unknown) => {
        setStatus("error");
        setError(reason instanceof Error ? reason.message : String(reason));
        return false;
      })
      .finally(() => {
        savePromiseRef.current = null;
      });
    savePromiseRef.current = operation;
    const succeeded = await operation;
    if (succeeded && studyNoteFingerprint(draftRef.current) !== studyNoteFingerprint(persistedRef.current)) {
      return flush();
    }
    return succeeded;
  }, [onSave, onSaved, readOnly]);

  useImperativeHandle(ref, () => ({
    flush,
    hasUnsavedChanges: () => studyNoteFingerprint(draftRef.current) !== studyNoteFingerprint(persistedRef.current),
  }), [flush]);

  useEffect(() => {
    if (readOnly || studyNoteFingerprint(draft) === studyNoteFingerprint(persistedRef.current)) return;
    const timer = window.setTimeout(() => void flush(), 700);
    return () => window.clearTimeout(timer);
  }, [draft, flush, readOnly]);

  useEffect(() => {
    const protectDraft = (event: BeforeUnloadEvent) => {
      if (studyNoteFingerprint(draftRef.current) === studyNoteFingerprint(persistedRef.current)) return;
      event.preventDefault();
    };
    window.addEventListener("beforeunload", protectDraft);
    return () => window.removeEventListener("beforeunload", protectDraft);
  }, []);

  const breadcrumb = [note.roadmapName, note.stageName, note.topicName, note.activityTitle].filter(Boolean);
  const aiContext: StudyAIContext = {
    selectedText,
    roadmap: note.roadmapId ? { id: note.roadmapId, name: note.roadmapName } : null,
    stage: note.stageId ? { id: note.stageId, name: note.stageName } : null,
    topic: note.topicId ? { id: note.topicId, name: note.topicName } : null,
    activity: note.activityId ? { id: note.activityId, name: note.activityTitle, description: null, activityType: null } : null,
    note: { id: note.id, title: draft.title, content: draft.content, notebookId: note.notebookId, notebookTitle: note.notebookTitle },
    studySession: note.studySessionId ? { id: note.studySessionId, status: "LINKED", plannedFocusMinutes: null } : null,
  };

  function captureSelection() {
    const target = textareaRef.current;
    if (!target) return;
    setSelectedText(target.value.slice(target.selectionStart, target.selectionEnd).trim());
  }

  function insertAtEnd(content: string) {
    if (readOnly) return;
    const current = draftRef.current;
    updateDraft({ ...current, content: appendStudyAIContent(current.content, content) });
    setPreview(false);
  }

  async function saveGeneratedCards(drafts: StudyAICardDraft[]) {
    const inputs: StudyCardInput[] = drafts.map((card) => ({
      id: card.id,
      front: card.front,
      back: card.back,
      roadmapId: note.roadmapId,
      stageId: note.stageId,
      topicId: note.topicId,
      activityId: note.activityId,
      notebookId: note.notebookId,
      noteId: note.id,
    }));
    await studyLabService.saveCards(inputs);
    await onSaved();
  }
  return <section className="study-note-editor">
    <header>
      <div><span>MARKDOWN NOTE</span><b data-status={status}>{readOnly ? "SOMENTE LEITURA" : status === "saved" ? "SALVO" : status === "dirty" ? "ALTERADO" : status === "saving" ? "SALVANDO" : "ERRO"}</b></div>
      <nav><button className={!preview ? "active" : ""} onClick={() => setPreview(false)}>EDITOR</button><button className={preview ? "active" : ""} onClick={() => setPreview(true)}>PREVIEW</button></nav>
    </header>
    <div className="study-note-editor__meta">
      <input aria-label="Título da nota" disabled={readOnly} maxLength={200} value={draft.title} onChange={(event) => updateDraft({ ...draft, title: event.target.value })} />
      <span>{note.notebookTitle}</span>
    </div>
    {breadcrumb.length > 0 && <div className="study-note-context">{breadcrumb.map((item) => <span key={item}>{item}</span>)}{note.studySessionId && <span>SESSÃO VINCULADA</span>}</div>}
    {preview ? <StudyMarkdown content={draft.content} /> : <textarea ref={textareaRef} aria-label="Conteúdo Markdown" disabled={readOnly} spellCheck value={draft.content} onChange={(event) => { updateDraft({ ...draftRef.current, content: event.target.value }); setSelectedText(""); }} onSelect={captureSelection} onKeyUp={captureSelection} onMouseUp={captureSelection} placeholder="# Título&#10;&#10;Registre conceitos, decisões e exemplos..." />}
    {error && <p className="study-note-editor__error">{error}</p>}
    <StudyRelatedMaterials relationType="NOTE" relationId={note.id} onOpenLibrary={() => onOpenLibrary(note)} />
    {showAI && <StudyAIToolPanel context={aiContext} onInsertAtEnd={readOnly ? undefined : insertAtEnd} onSaveCards={saveGeneratedCards} onClose={() => setShowAI(false)} />}
    <footer><span>{draft.content.length.toLocaleString("pt-BR")} CARACTERES{selectedText ? ` · ${selectedText.length} SELECIONADOS` : ""}</span><button className={showAI ? "active" : ""} onClick={() => setShowAI((current) => !current)}>AZRIEL AI</button><button disabled={status === "saving"} onClick={() => void onCreateCard(note)}>＋ CRIAR STUDY CARD</button><button disabled={readOnly || status === "saving"} onClick={() => void flush()}>{status === "error" ? "TENTAR NOVAMENTE" : "SALVAR AGORA"}</button><button className="danger-link" onClick={() => onDelete(note)}>EXCLUIR NOTA</button></footer>
  </section>;
});
