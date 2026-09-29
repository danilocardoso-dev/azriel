import { useCallback, useEffect, useState, type FormEvent } from "react";
import { DeleteConfirmationDialog } from "../daily/DeleteConfirmationDialog";
import { studyCardSuccessRate } from "../../services/studyLabService";
import { studyLabService } from "../../services/studyLabService";
import type { StudyCard, StudyCardContext, StudyCardFilter, StudyCardInput, StudyReviewDashboardSummary, StudyReviewQueueMode, StudyReviewResultValue, StudyReviewSession, StudyReviewSummary } from "../../types";
import { StudyCardEditor } from "./StudyCardEditor";
import { StudyMarkdown } from "./StudyMarkdown";
import { StudyReviewPlayer } from "./StudyReviewPlayer";

const emptyContext: StudyCardContext = { roadmapId: null, stageId: null, topicId: null, activityId: null, notebookId: null, noteId: null };
export type StudyCardLaunch = StudyCardContext & { mode: "open" | "create"; token: number };

type Props = {
  launch: StudyCardLaunch | null;
  activeStudySessionId: string | null;
  onChanged: () => Promise<void>;
  onError: (message: string | null) => void;
};

export function StudyReview({ launch, activeStudySessionId, onChanged, onError }: Props) {
  const [dashboard, setDashboard] = useState<StudyReviewDashboardSummary | null>(null);
  const [cards, setCards] = useState<StudyCard[]>([]);
  const [activeSession, setActiveSession] = useState<StudyReviewSession | null>(null);
  const [summary, setSummary] = useState<StudyReviewSummary | null>(null);
  const [editor, setEditor] = useState<{ card: StudyCard | null; context: StudyCardContext } | null>(null);
  const [selectedCard, setSelectedCard] = useState<StudyCard | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<StudyCard | null>(null);
  const [filter, setFilter] = useState<StudyCardFilter>("ALL");
  const [query, setQuery] = useState("");
  const [busy, setBusy] = useState(false);
  const [loading, setLoading] = useState(true);

  const loadCards = useCallback(async (nextFilter: StudyCardFilter = "ALL", nextQuery = "", context: StudyCardContext = emptyContext) => {
    const result = await studyLabService.cards({
      query: nextQuery.trim() || null,
      filter: nextFilter,
      roadmapId: context.roadmapId,
      activityId: context.activityId,
      noteId: context.noteId,
      limit: 100,
      offset: 0,
    });
    setCards(result);
    setSelectedCard((current) => result.find((item) => item.id === current?.id) ?? result[0] ?? null);
    return result;
  }, []);

  useEffect(() => {
    const initialize = window.setTimeout(() => {
      setLoading(true);
      void Promise.all([studyLabService.reviewDashboard(), studyLabService.activeReviewSession(), loadCards("ALL", "")])
        .then(([nextDashboard, nextSession]) => { setDashboard(nextDashboard); setActiveSession(nextSession); })
        .catch((reason: unknown) => onError(reason instanceof Error ? reason.message : String(reason)))
        .finally(() => setLoading(false));
    }, 0);
    return () => window.clearTimeout(initialize);
  }, [onError, loadCards]);

  useEffect(() => {
    if (!launch) return;
    const context: StudyCardContext = { roadmapId: launch.roadmapId, stageId: launch.stageId, topicId: launch.topicId, activityId: launch.activityId, notebookId: launch.notebookId, noteId: launch.noteId };
    const open = window.setTimeout(() => {
      if (launch.mode === "create") setEditor({ card: null, context });
      void loadCards("ALL", "", context);
    }, 0);
    return () => window.clearTimeout(open);
  }, [launch, loadCards]);

  async function saveCard(input: StudyCardInput) {
    setBusy(true);
    onError(null);
    try {
      const saved = await studyLabService.saveCard(input);
      setEditor(null);
      setSelectedCard(saved);
      await Promise.all([loadCards(filter, query), studyLabService.reviewDashboard().then(setDashboard), onChanged()]);
    } catch (reason) {
      onError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setBusy(false);
    }
  }

  async function updateStatus(card: StudyCard, status: StudyCard["status"]) {
    setBusy(true);
    onError(null);
    try {
      await studyLabService.setCardStatus(card.id, status);
      await Promise.all([loadCards(filter, query), studyLabService.reviewDashboard().then(setDashboard), onChanged()]);
    } catch (reason) {
      onError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setBusy(false);
    }
  }

  async function removeCard() {
    if (!deleteTarget) return;
    setBusy(true);
    onError(null);
    try {
      await studyLabService.deleteCard(deleteTarget.id);
      setDeleteTarget(null);
      setSelectedCard(null);
      await Promise.all([loadCards(filter, query), studyLabService.reviewDashboard().then(setDashboard), onChanged()]);
    } catch (reason) {
      onError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setBusy(false);
    }
  }

  async function startReview(mode: StudyReviewQueueMode) {
    setBusy(true);
    setSummary(null);
    onError(null);
    try {
      setActiveSession(await studyLabService.startReviewSession({ id: crypto.randomUUID(), mode, studySessionId: activeStudySessionId }));
    } catch (reason) {
      onError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setBusy(false);
    }
  }

  async function rate(result: StudyReviewResultValue, responseTimeMs: number) {
    const itemId = activeSession?.currentItem?.itemId;
    if (!activeSession || !itemId || busy) return;
    setBusy(true);
    onError(null);
    try {
      const submitted = await studyLabService.submitReviewResult({ id: crypto.randomUUID(), reviewSessionId: activeSession.id, sessionItemId: itemId, result, responseTimeMs });
      setActiveSession(submitted.session);
      if (submitted.session.status === "COMPLETED") setSummary(await studyLabService.reviewSummary(submitted.session.id));
      await Promise.all([loadCards(filter, query), studyLabService.reviewDashboard().then(setDashboard), onChanged()]);
    } catch (reason) {
      onError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setBusy(false);
    }
  }

  async function cancelReview() {
    if (!activeSession) return;
    setBusy(true);
    try {
      const cancelled = await studyLabService.cancelReviewSession(activeSession.id);
      setActiveSession(cancelled);
      setSummary(await studyLabService.reviewSummary(cancelled.id));
      setDashboard(await studyLabService.reviewDashboard());
      await onChanged();
    } catch (reason) {
      onError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setBusy(false);
    }
  }

  function search(event: FormEvent) {
    event.preventDefault();
    void loadCards(filter, query);
  }

  if (loading || !dashboard) return <div className="core-empty">CARREGANDO REVIEW & ACTIVE RECALL...</div>;
  if (activeSession?.status === "ACTIVE") return <StudyReviewPlayer key={activeSession.currentItem?.itemId ?? activeSession.id} session={activeSession} busy={busy} onRate={rate} onCancel={cancelReview} />;
  if (summary) return <section className="study-review-summary"><span>REVIEW SUMMARY</span><h2>{summary.status === "COMPLETED" ? "Revisão concluída" : "Revisão cancelada"}</h2><div><article><small>CARDS REVISADOS</small><strong>{summary.reviewedCards}</strong></article><article><small>ERREI / DIFÍCIL</small><strong>{summary.again} / {summary.hard}</strong></article><article><small>BOM / FÁCIL</small><strong>{summary.good} / {summary.easy}</strong></article><article><small>DURAÇÃO</small><strong>{Math.floor(summary.durationSeconds / 60)} min</strong></article></div><p>PRÓXIMA REVISÃO · {summary.nextReviewAt ? new Date(summary.nextReviewAt).toLocaleString("pt-BR") : "SEM AGENDAMENTO"}</p><button onClick={() => { setSummary(null); setActiveSession(null); }}>VOLTAR AO DASHBOARD</button></section>;

  return <section className="study-review">
    <header className="study-review__hero"><div><span>REVIEW & ACTIVE RECALL</span><h2>Revisão</h2><p>Recupere o conhecimento sem consultar a resposta e avalie honestamente a dificuldade.</p></div><button onClick={() => setEditor({ card: null, context: emptyContext })}>＋ NOVO CARD</button></header>
    <section className="study-review-dashboard">
      <article><span>VENCIDOS</span><strong>{dashboard.overdue}</strong></article><article><span>HOJE</span><strong>{dashboard.dueToday}</strong></article><article><span>NOVOS</span><strong>{dashboard.newCards}</strong></article><article><span>REVISADOS HOJE</span><strong>{dashboard.reviewedToday}</strong><small>{dashboard.lastReviewedAt ? `ÚLTIMA · ${new Date(dashboard.lastReviewedAt).toLocaleString("pt-BR")}` : "NENHUMA REVISÃO"}</small></article>
    </section>
    <section className="study-review-start"><div><strong>INICIAR REVISÃO</strong><span>Fila estável: vencidos, hoje e depois novos.</span></div><nav><button disabled={busy || dashboard.overdue + dashboard.dueToday + dashboard.newCards === 0} onClick={() => void startReview("TEN")}>10 CARDS</button><button disabled={busy || dashboard.overdue + dashboard.dueToday + dashboard.newCards === 0} onClick={() => void startReview("TWENTY")}>20 CARDS</button><button disabled={busy || dashboard.overdue === 0} onClick={() => void startReview("ALL_OVERDUE")}>TODOS VENCIDOS</button></nav></section>
    <div className="study-card-browser">
      <aside><header><strong>STUDY CARDS</strong><b>{cards.length}</b></header><form onSubmit={search}><input value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Buscar pergunta ou resposta..." /><button>BUSCAR</button></form><div className="study-card-filters">{(["ALL","DUE","NEW","SUSPENDED","ARCHIVED"] as const).map((item) => <button key={item} className={filter === item ? "active" : ""} onClick={() => { setFilter(item); void loadCards(item, query); }}>{item === "ALL" ? "TODOS" : item === "DUE" ? "A REVISAR" : item === "NEW" ? "NOVOS" : item === "SUSPENDED" ? "SUSPENSOS" : "ARQUIVADOS"}</button>)}</div><div className="study-card-list">{cards.map((card) => <button key={card.id} className={selectedCard?.id === card.id ? "active" : ""} onClick={() => setSelectedCard(card)}><strong>{card.front}</strong><small>{card.activityTitle || card.noteTitle || card.roadmapName || "SEM CONTEXTO"}</small><span>{card.reviewCount ? `${card.reviewCount} REVIEWS` : "NOVO"} · {card.status}</span></button>)}{!cards.length && <p>Nenhum card corresponde ao filtro.</p>}</div></aside>
      <main>{selectedCard ? <article className="study-card-inspector"><header><span>{selectedCard.status}</span><nav><button onClick={() => setEditor({ card: selectedCard, context: selectedCard })}>EDITAR</button>{selectedCard.status === "ACTIVE" && <button onClick={() => void updateStatus(selectedCard,"SUSPENDED")}>SUSPENDER</button>}{selectedCard.status !== "ACTIVE" && <button onClick={() => void updateStatus(selectedCard,"ACTIVE")}>REATIVAR</button>}<button onClick={() => void updateStatus(selectedCard,"ARCHIVED")}>ARQUIVAR</button><button className="danger-link" disabled={selectedCard.reviewCount > 0} title={selectedCard.reviewCount > 0 ? "Arquive cards que já possuem histórico" : "Excluir card"} onClick={() => setDeleteTarget(selectedCard)}>EXCLUIR</button></nav></header><section><small>PERGUNTA</small><StudyMarkdown content={selectedCard.front} /></section><section><small>RESPOSTA</small><StudyMarkdown content={selectedCard.back} /></section><dl><div><dt>REVIEWS</dt><dd>{selectedCard.reviewCount}</dd></div><div><dt>CORRETAS / INCORRETAS</dt><dd>{selectedCard.correctCount} / {selectedCard.incorrectCount}</dd></div><div><dt>SUCESSO HISTÓRICO</dt><dd>{studyCardSuccessRate(selectedCard.correctCount,selectedCard.reviewCount) ?? "—"}{selectedCard.reviewCount ? "%" : ""}</dd></div><div><dt>PRÓXIMA REVISÃO</dt><dd>{new Date(selectedCard.dueAt).toLocaleString("pt-BR")}</dd></div><div><dt>ÚLTIMA REVISÃO</dt><dd>{selectedCard.lastReviewedAt ? new Date(selectedCard.lastReviewedAt).toLocaleString("pt-BR") : "—"}</dd></div></dl></article> : <div className="study-note-workspace__empty"><span>STUDY CARD BROWSER</span><h2>Selecione ou crie um card.</h2><p>Pergunta, resposta, contexto e histórico real de revisão.</p></div>}</main>
    </div>
    {editor && <StudyCardEditor card={editor.card} context={editor.context} busy={busy} onSave={saveCard} onCancel={() => setEditor(null)} />}
    {deleteTarget && <DeleteConfirmationDialog kind="anotação" title={deleteTarget.front} busy={busy} description={deleteTarget.reviewCount ? "Cards com histórico não podem ser apagados. A operação será recusada; arquive o card para preservar a auditoria." : "O Study Card ainda não revisado será removido definitivamente."} onCancel={() => setDeleteTarget(null)} onConfirm={() => void removeCard()} />}
  </section>;
}
