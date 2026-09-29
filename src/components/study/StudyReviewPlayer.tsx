import { useEffect, useRef, useState } from "react";
import type { StudyReviewResultValue, StudyReviewSession } from "../../types";
import { StudyMarkdown } from "./StudyMarkdown";
import { StudyAIToolPanel } from "./StudyAIToolPanel";

type Props = {
  session: StudyReviewSession;
  busy: boolean;
  onRate: (result: StudyReviewResultValue, responseTimeMs: number) => Promise<void>;
  onCancel: () => Promise<void>;
};

const ratings: Array<{ result: StudyReviewResultValue; label: string; interval: string }> = [
  { result: "AGAIN", label: "ERREI", interval: "10 MIN" },
  { result: "HARD", label: "DIFÍCIL", interval: "CURTO" },
  { result: "GOOD", label: "BOM", interval: "CRESCE" },
  { result: "EASY", label: "FÁCIL", interval: "MAIOR" },
];

export function StudyReviewPlayer({ session, busy, onRate, onCancel }: Props) {
  const [revealed, setRevealed] = useState(false);
  const startedAt = useRef(0);
  const item = session.currentItem;

  useEffect(() => {
    startedAt.current = performance.now();
  }, []);

  if (!item) return null;
  const progress = session.plannedCards ? Math.round((session.reviewedCards / session.plannedCards) * 100) : 0;
  return <section className="study-review-player">
    <header><div><span>REVIEW SESSION · {session.contextLabel}</span><strong>{session.reviewedCards + 1} / {session.plannedCards}</strong></div><div className="study-review-player__progress"><i style={{ width: `${progress}%` }} /></div><button disabled={busy} onClick={() => void onCancel()}>CANCELAR SESSÃO</button></header>
    <main>
      <span className="study-review-player__category" data-category={item.category}>{item.category === "OVERDUE" ? "VENCIDO" : item.category === "TODAY" ? "PARA HOJE" : "NOVO"}</span>
      <div className="study-review-player__face"><small>PERGUNTA</small><StudyMarkdown content={item.card.front} /></div>
      {!revealed ? <button className="study-review-player__reveal" disabled={busy} onClick={() => setRevealed(true)}>MOSTRAR RESPOSTA</button> : <>
        <div className="study-review-player__face answer"><small>RESPOSTA</small><StudyMarkdown content={item.card.back} /></div>
        <StudyAIToolPanel actions={["EXPLAIN"]} context={{
          roadmap: item.card.roadmapId ? { id: item.card.roadmapId, name: item.card.roadmapName } : null,
          stage: item.card.stageId ? { id: item.card.stageId, name: item.card.stageName } : null,
          topic: item.card.topicId ? { id: item.card.topicId, name: item.card.topicName } : null,
          activity: item.card.activityId ? { id: item.card.activityId, name: item.card.activityTitle, description: null, activityType: null } : null,
          relatedCard: { id: item.card.id, front: item.card.front, back: item.card.back },
        }} />
        <div className="study-review-player__ratings">{ratings.map((rating) => <button key={rating.result} data-result={rating.result} disabled={busy} onClick={() => void onRate(rating.result, Math.max(0, Math.round(performance.now() - startedAt.current)))}><strong>{rating.label}</strong><small>{rating.interval}</small></button>)}</div>
      </>}
    </main>
  </section>;
}
