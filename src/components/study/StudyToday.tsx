import { useState, type FormEvent } from "react";
import { currentStudyContext } from "../../services/roadmapExperience";
import { formatStudyDuration } from "../../services/studyLabService";
import type { RoadmapActivity, StudyNoteSummary, StudyReviewDashboardSummary, StudySession, StudySettings, StudySettingsInput, StudyTodaySummary, StudyRoadmap } from "../../types";

type Props = {
  roadmaps: StudyRoadmap[];
  summary: StudyTodaySummary;
  recentSessions: StudySession[];
  recentNotes: StudyNoteSummary[];
  reviewSummary: StudyReviewDashboardSummary;
  settings: StudySettings;
  busy: boolean;
  onNewRoadmap: () => void;
  onOpenRoadmaps: () => void;
  onStartSession: (roadmap: StudyRoadmap, activity: RoadmapActivity) => Promise<void>;
  onSaveSettings: (input: StudySettingsInput) => Promise<void>;
  onOpenNote: (note: StudyNoteSummary) => void;
  onOpenReview: () => void;
};

function StudyTimerSettings({ settings, busy, onSave }: { settings: StudySettings; busy: boolean; onSave: (input: StudySettingsInput) => Promise<void> }) {
  const [focusMinutes, setFocusMinutes] = useState(settings.focusMinutes);
  const [shortBreakMinutes, setShortBreakMinutes] = useState(settings.shortBreakMinutes);
  const submit = (event: FormEvent) => { event.preventDefault(); void onSave({ focusMinutes, shortBreakMinutes }); };
  return <form className="study-timer-settings" onSubmit={submit}><header><strong>CONFIGURAÇÃO DO POMODORO</strong><span>LOCAL</span></header><div><label>FOCO<input type="number" min={1} max={240} value={focusMinutes} onChange={(event) => setFocusMinutes(Number(event.target.value))} /></label><label>PAUSA CURTA<input type="number" min={1} max={60} value={shortBreakMinutes} onChange={(event) => setShortBreakMinutes(Number(event.target.value))} /></label><button disabled={busy}>SALVAR</button></div></form>;
}

export function StudyToday({ roadmaps, summary, recentSessions, recentNotes, reviewSummary, settings, busy, onNewRoadmap, onOpenRoadmaps, onStartSession, onSaveSettings, onOpenNote, onOpenReview }: Props) {
  const current = currentStudyContext(roadmaps);
  const activeRoadmaps = roadmaps.filter((roadmap) => roadmap.status === "active").slice(0, 3);
  if (!roadmaps.length) return <section className="study-empty-state"><span>STUDY LAB // FOUNDATION</span><h2>Nenhum roadmap ativo.</h2><p>Crie um roadmap para iniciar seu primeiro caminho de estudo.</p><button onClick={onNewRoadmap}>＋ NOVO ROADMAP</button></section>;
  return <div className="study-today">
    <section className="study-focus-card"><header><span>FOCO ATUAL</span><b>{summary.activeSession ? summary.activeSession.status : "READY"}</b></header>{current ? <><div><small>{current.roadmap.name} · {current.stage.name}</small><h2>{current.activity.title}</h2><p>{current.topic.name} · {current.activity.activityType}</p></div><footer><button disabled={busy || Boolean(summary.activeSession)} onClick={() => void onStartSession(current.roadmap, current.activity)}>▶ CONTINUAR ESTUDANDO</button><button onClick={onOpenRoadmaps}>ABRIR ROADMAP</button></footer></> : <div className="study-focus-card__empty"><h2>Roadmaps sem atividades pendentes.</h2><button onClick={onOpenRoadmaps}>REVISAR ROADMAPS</button></div>}</section>
    <section className="study-today-metrics"><article><span>FOCO HOJE</span><strong>{formatStudyDuration(summary.focusSeconds)}</strong></article><article><span>SESSÕES CONCLUÍDAS</span><strong>{summary.completedSessions}</strong></article><article><span>ATIVIDADES CONCLUÍDAS</span><strong>{summary.completedActivities}</strong></article><article><span>MAIS ESTUDADO</span><strong>{summary.mostStudiedRoadmap?.roadmapName || "—"}</strong><small>{summary.mostStudiedRoadmap ? formatStudyDuration(summary.mostStudiedRoadmap.focusSeconds) : "SEM SESSÕES"}</small></article></section>
    <section className="study-active-roadmaps"><header><strong>ROADMAPS ATIVOS</strong><button onClick={onOpenRoadmaps}>VER TODOS</button></header><div>{activeRoadmaps.map((roadmap) => { const next = currentStudyContext([roadmap]); const last = recentSessions.find((session) => session.roadmapId === roadmap.id); return <article key={roadmap.id}><span>{roadmap.progress}%</span><h3>{roadmap.name}</h3><div><i style={{ width: `${roadmap.progress}%` }} /></div><p>{next?.activity.title ?? "Nenhuma atividade pendente"}</p><small>{last ? `ÚLTIMO ESTUDO · ${new Date(last.startedAt).toLocaleString("pt-BR")}` : "AINDA NÃO ESTUDADO"}</small>{next && <button disabled={busy || Boolean(summary.activeSession)} onClick={() => void onStartSession(roadmap, next.activity)}>INICIAR SESSÃO</button>}</article>; })}{!activeRoadmaps.length && <p className="roadmap-empty-compact">Nenhum roadmap com status ativo.</p>}</div></section>
    <section className="study-recent"><header><strong>ATIVIDADE RECENTE</strong><span>{recentSessions.length} REGISTROS</span></header><div>{recentSessions.slice(0, 5).map((session) => <article key={session.id}><time>{new Date(session.startedAt).toLocaleString("pt-BR")}</time><span><strong>{session.activityTitle || session.roadmapName || "Sessão livre"}</strong><small>{session.roadmapName || "SEM ROADMAP"}</small></span><b>{formatStudyDuration(session.currentFocusSeconds)}</b><em data-status={session.status}>{session.status}</em></article>)}{!recentSessions.length && <p className="roadmap-empty-compact">Nenhuma sessão registrada.</p>}</div></section>
    <section className="study-recent-notes"><header><strong>NOTAS RECENTES</strong><span>{recentNotes.length} ITENS</span></header><div>{recentNotes.slice(0, 5).map((note) => <button key={note.id} onClick={() => onOpenNote(note)}><span><strong>{note.title}</strong><small>{note.notebookTitle} · {note.activityTitle || note.roadmapName || "SEM CONTEXTO"}</small></span><time>{new Date(note.updatedAt).toLocaleString("pt-BR")}</time></button>)}{!recentNotes.length && <p className="roadmap-empty-compact">Nenhuma nota de estudo registrada.</p>}</div></section>
    <section className="study-today-review"><header><div><strong>REVISÃO</strong><span>ACTIVE RECALL</span></div><button onClick={onOpenReview}>REVISAR AGORA</button></header><div><article><small>VENCIDOS</small><strong>{reviewSummary.overdue}</strong></article><article><small>HOJE</small><strong>{reviewSummary.dueToday}</strong></article><article><small>REVISADOS</small><strong>{reviewSummary.reviewedToday}</strong></article></div><p>{reviewSummary.lastReviewedAt ? `ÚLTIMA REVISÃO · ${new Date(reviewSummary.lastReviewedAt).toLocaleString("pt-BR")}` : "Nenhuma revisão realizada ainda."}</p></section>
    <StudyTimerSettings key={settings.updatedAt} settings={settings} busy={busy} onSave={onSaveSettings} />
  </div>;
}
