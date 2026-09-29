import { useEffect, useState } from "react";
import { formatStudyTimer } from "../../services/studyLabService";
import type { StudySession } from "../../types";

type Props = {
  session: StudySession;
  busy: boolean;
  onPause: () => Promise<void>;
  onResume: () => Promise<void>;
  onComplete: () => Promise<void>;
  onCancel: () => Promise<void>;
  onOpenNotes: () => Promise<void>;
  onCreateNote: () => Promise<void>;
};

export function StudySessionPanel({ session, busy, onPause, onResume, onComplete, onCancel, onOpenNotes, onCreateNote }: Props) {
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    if (session.status !== "ACTIVE") return;
    const timer = window.setInterval(() => setNow(Date.now()), 1000);
    return () => window.clearInterval(timer);
  }, [session.status]);
  const liveIncrement = session.status === "ACTIVE" ? Math.max(0, Math.floor((now - Date.parse(session.observedAt)) / 1000)) : 0;
  const focused = session.currentFocusSeconds + liveIncrement;
  const target = session.plannedFocusMinutes * 60;
  const remaining = Math.max(0, target - focused);
  const progress = Math.min(100, Math.round(focused / target * 100));
  return <section className="study-session-panel" data-status={session.status}>
    <header><div><span>STUDY SESSION</span><strong>{session.status === "ACTIVE" ? "FOCO ATIVO" : "SESSÃO PAUSADA"}</strong></div><b>{session.status}</b></header>
    <div className="study-session-panel__body">
      <div className="study-session-context"><span>{session.roadmapName || "SESSÃO LIVRE"}</span><strong>{session.activityTitle || session.topicName || "Estudo sem atividade vinculada"}</strong><small>{[session.stageName, session.topicName].filter(Boolean).join(" · ") || "SEM CONTEXTO DE ROADMAP"}</small></div>
      <div className="study-session-clock"><span>{remaining > 0 ? "TEMPO RESTANTE" : "FOCO ADICIONAL"}</span><strong>{formatStudyTimer(remaining > 0 ? remaining : focused - target)}</strong><div><i style={{ width: `${progress}%` }} /></div><small>{formatStudyTimer(focused)} DE FOCO · {session.plannedFocusMinutes} MIN PLANEJADOS</small></div>
    </div>
    <footer><button disabled={busy} onClick={() => void onOpenNotes()}>NOTAS ({session.linkedNoteCount})</button><button disabled={busy} onClick={() => void onCreateNote()}>＋ CRIAR NOTA</button>{session.status === "ACTIVE" ? <button disabled={busy} onClick={() => void onPause()}>PAUSAR</button> : <button disabled={busy} onClick={() => void onResume()}>RETOMAR</button>}<button className="primary" disabled={busy} onClick={() => void onComplete()}>CONCLUIR SESSÃO</button><button className="danger-link" disabled={busy} onClick={() => void onCancel()}>CANCELAR</button></footer>
  </section>;
}
