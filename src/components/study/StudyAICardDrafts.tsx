import type { StudyAICardDraft } from "../../ai/study/studyAIDrafts";

type Props = {
  drafts: StudyAICardDraft[];
  busy: boolean;
  onChange: (drafts: StudyAICardDraft[]) => void;
  onSave: () => Promise<void>;
  onDiscard: () => void;
};

export function StudyAICardDrafts({ drafts, busy, onChange, onSave, onDiscard }: Props) {
  const selected = drafts.filter((draft) => draft.selected && !draft.saved && draft.front.trim() && draft.back.trim()).length;
  const patch = (id: string, values: Partial<StudyAICardDraft>) => onChange(drafts.map((draft) => draft.id === id && !draft.saved ? { ...draft, ...values } : draft));
  const remove = (id: string) => onChange(drafts.filter((draft) => draft.id !== id || draft.saved));
  return <section className="study-ai-drafts">
    <header><div><strong>AI GENERATED CARD DRAFTS</strong><span>{selected} SELECIONADOS</span></div><button disabled={busy} onClick={onDiscard}>DESCARTAR TODOS</button></header>
    <div>{drafts.map((draft, index) => <article key={draft.id} data-saved={draft.saved}>
      <header><label><input type="checkbox" checked={draft.selected} disabled={busy || draft.saved} onChange={(event) => patch(draft.id, { selected: event.target.checked })} /> DRAFT {String(index + 1).padStart(2, "0")}</label><span>{draft.saved ? "SALVO" : "NÃO PERSISTIDO"}</span><button disabled={busy || draft.saved} onClick={() => remove(draft.id)}>DESCARTAR</button></header>
      <label>PERGUNTA<textarea rows={3} maxLength={4_000} disabled={busy || draft.saved} value={draft.front} onChange={(event) => patch(draft.id, { front: event.target.value })} /></label>
      <label>RESPOSTA<textarea rows={4} maxLength={12_000} disabled={busy || draft.saved} value={draft.back} onChange={(event) => patch(draft.id, { back: event.target.value })} /></label>
    </article>)}</div>
    <footer><small>Nenhum card entra na fila antes da confirmação.</small><button disabled={busy || selected === 0} onClick={() => void onSave()}>{busy ? "SALVANDO..." : `SALVAR SELECIONADOS (${selected})`}</button></footer>
  </section>;
}
