import { formatStudyDuration } from "../../services/studyLabService";
import type { StudyReviewSession, StudySession, StudyRoadmap } from "../../types";

type Props = {
  sessions: StudySession[];
  reviewSessions: StudyReviewSession[];
  roadmaps: StudyRoadmap[];
  roadmapId: string;
  dateFrom: string;
  dateTo: string;
  loading: boolean;
  hasMore: boolean;
  onRoadmap: (value: string) => void;
  onDateFrom: (value: string) => void;
  onDateTo: (value: string) => void;
  onApply: () => void;
  onMore: () => void;
  onOpenNotes: (session: StudySession) => void;
};

const statusLabel = {
  ACTIVE: "ATIVA",
  PAUSED: "PAUSADA",
  COMPLETED: "CONCLUÍDA",
  CANCELLED: "CANCELADA",
};

function sessionDay(session: StudySession) {
  return new Date(session.startedAt).toLocaleDateString("pt-BR", {
    weekday: "long",
    day: "2-digit",
    month: "long",
    year: "numeric",
  });
}

export function StudyHistory({
  sessions,
  reviewSessions,
  roadmaps,
  roadmapId,
  dateFrom,
  dateTo,
  loading,
  hasMore,
  onRoadmap,
  onDateFrom,
  onDateTo,
  onApply,
  onMore,
  onOpenNotes,
}: Props) {
  return (
    <section className="study-history">
      <header>
        <div>
          <span>STUDY LAB // HISTORY</span>
          <h2>Histórico de estudo</h2>
          <p>Sessões reais, tempos de foco e contexto preservado.</p>
        </div>
      </header>
      <div className="study-history__filters">
        <label>
          ROADMAP
          <select value={roadmapId} onChange={(event) => onRoadmap(event.target.value)}>
            <option value="">TODOS</option>
            {roadmaps.map((roadmap) => <option key={roadmap.id} value={roadmap.id}>{roadmap.name}</option>)}
          </select>
        </label>
        <label>DE<input type="date" value={dateFrom} onChange={(event) => onDateFrom(event.target.value)} /></label>
        <label>ATÉ<input type="date" value={dateTo} onChange={(event) => onDateTo(event.target.value)} /></label>
        <button disabled={loading} onClick={onApply}>APLICAR</button>
      </div>
      <div className="study-history__list">
        {sessions.map((session, index) => {
          const day = sessionDay(session);
          const showDay = index === 0 || day !== sessionDay(sessions[index - 1]);
          return (
            <div key={session.id}>
              {showDay && <h3>{day}</h3>}
              <article>
                <time>{new Date(session.startedAt).toLocaleTimeString("pt-BR", { hour: "2-digit", minute: "2-digit" })}</time>
                <span>
                  <strong>{session.activityTitle || session.topicName || session.roadmapName || "Sessão livre"}</strong>
                  <small>{[session.roadmapName, session.topicName].filter(Boolean).join(" · ") || "SEM ROADMAP"}</small>
                </span>
                <b>{formatStudyDuration(session.currentFocusSeconds)}</b>
                <em data-status={session.status}>{statusLabel[session.status]}</em>
                <button disabled={!session.linkedNoteCount} onClick={() => onOpenNotes(session)}>{session.linkedNoteCount} NOTA{session.linkedNoteCount === 1 ? "" : "S"}</button>
              </article>
            </div>
          );
        })}
        {!sessions.length && !loading && <p className="study-history__empty">Nenhuma sessão encontrada para este período.</p>}
      </div>
      <section className="study-review-history"><header><strong>SESSÕES DE REVISÃO</strong><span>{reviewSessions.length} REGISTROS</span></header><div>{reviewSessions.map((session) => <article key={session.id}><time>{new Date(session.startedAt).toLocaleString("pt-BR")}</time><span><strong>REVIEW · {session.contextLabel}</strong><small>{session.reviewedCards} DE {session.plannedCards} CARDS</small></span><b>{session.endedAt ? formatStudyDuration(Math.max(0, Math.floor((new Date(session.endedAt).getTime() - new Date(session.startedAt).getTime()) / 1000))) : "EM CURSO"}</b><em data-status={session.status}>{statusLabel[session.status]}</em></article>)}</div>{!reviewSessions.length && <p className="study-history__empty">Nenhuma sessão de revisão registrada.</p>}</section>
      {hasMore && <button className="study-history__more" disabled={loading} onClick={onMore}>{loading ? "CARREGANDO..." : "CARREGAR MAIS"}</button>}
    </section>
  );
}
