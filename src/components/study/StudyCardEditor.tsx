import { useState, type FormEvent } from "react";
import type { StudyCard, StudyCardContext, StudyCardInput } from "../../types";
import { StudyMarkdown } from "./StudyMarkdown";

type Props = {
  card: StudyCard | null;
  context: StudyCardContext;
  busy: boolean;
  onSave: (input: StudyCardInput) => Promise<void>;
  onCancel: () => void;
};

export function StudyCardEditor({ card, context, busy, onSave, onCancel }: Props) {
  const [draft, setDraft] = useState<StudyCardInput>(() => card ? {
    id: card.id,
    front: card.front,
    back: card.back,
    roadmapId: card.roadmapId,
    stageId: card.stageId,
    topicId: card.topicId,
    activityId: card.activityId,
    notebookId: card.notebookId,
    noteId: card.noteId,
  } : { id: crypto.randomUUID(), front: "", back: "", ...context });
  const [preview, setPreview] = useState<"none" | "front" | "back">("none");

  function submit(event: FormEvent) {
    event.preventDefault();
    if (!draft.front.trim() || !draft.back.trim()) return;
    void onSave(draft);
  }

  const contextLabels = card
    ? [card.roadmapName, card.stageName, card.topicName, card.activityTitle, card.notebookTitle, card.noteTitle].filter(Boolean)
    : [context.noteId ? "NOTA VINCULADA" : null, context.activityId ? "ATIVIDADE VINCULADA" : null, context.roadmapId ? "ROADMAP VINCULADO" : null].filter(Boolean);

  return <div className="study-card-modal" role="dialog" aria-modal="true" aria-label={card ? "Editar Study Card" : "Criar Study Card"}>
    <form onSubmit={submit}>
      <header><div><span>ACTIVE RECALL</span><h2>{card ? "Editar Study Card" : "Novo Study Card"}</h2></div><button type="button" onClick={onCancel}>×</button></header>
      {contextLabels.length > 0 && <div className="study-card-context">{contextLabels.map((label) => <span key={label}>{label}</span>)}</div>}
      <label>PERGUNTA / PROMPT
        <textarea autoFocus disabled={busy} maxLength={4000} value={draft.front} onChange={(event) => setDraft({ ...draft, front: event.target.value })} placeholder="Qual conceito você quer recuperar sem consultar a resposta?" />
      </label>
      <div className="study-card-preview-toggle"><button type="button" className={preview === "front" ? "active" : ""} onClick={() => setPreview(preview === "front" ? "none" : "front")}>PREVIEW DA PERGUNTA</button></div>
      {preview === "front" && <StudyMarkdown content={draft.front} />}
      <label>RESPOSTA / REFERÊNCIA
        <textarea disabled={busy} maxLength={12000} value={draft.back} onChange={(event) => setDraft({ ...draft, back: event.target.value })} placeholder="Registre a resposta, explicação ou referência em Markdown." />
      </label>
      <div className="study-card-preview-toggle"><button type="button" className={preview === "back" ? "active" : ""} onClick={() => setPreview(preview === "back" ? "none" : "back")}>PREVIEW DA RESPOSTA</button></div>
      {preview === "back" && <StudyMarkdown content={draft.back} />}
      <footer><span>MARKDOWN LOCAL · CONTEÚDO NÃO EXECUTÁVEL</span><button type="button" onClick={onCancel}>CANCELAR</button><button className="primary" disabled={busy || !draft.front.trim() || !draft.back.trim()}>SALVAR CARD</button></footer>
    </form>
  </div>;
}
