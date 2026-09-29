import { useCallback, useEffect, useRef, useState } from "react";
import { DeleteConfirmationDialog } from "../components/daily/DeleteConfirmationDialog";
import { RoadmapEditor } from "../components/stark/RoadmapEditor";
import { RoadmapWorkspace } from "../components/stark/roadmap/RoadmapWorkspace";
import { StudyHistory } from "../components/study/StudyHistory";
import { StudyLibrary, type StudyLibraryLaunch } from "../components/study/StudyLibrary";
import { StudyNotebooks, type StudyNoteLaunch, type StudyNotebooksHandle } from "../components/study/StudyNotebooks";
import { StudyReview, type StudyCardLaunch } from "../components/study/StudyReview";
import { StudySessionPanel } from "../components/study/StudySessionPanel";
import { StudyToday } from "../components/study/StudyToday";
import { useAzrielData } from "../contexts/useAzrielData";
import { starkService } from "../services/starkService";
import { studyLabService } from "../services/studyLabService";
import type { RoadmapActivity, RoadmapActivityStatus, RoadmapStage, RoadmapTopic, StudyMaterialRelationInput, StudyNote, StudyNoteContext, StudyNoteSummary, StudyReviewDashboardSummary, StudyReviewSession, StudySession, StudySettings, StudySettingsInput, StudyTodaySummary, StudyRoadmap, StudyRoadmapInput } from "../types";

type StudyTab = "today" | "roadmaps" | "notebooks" | "library" | "review" | "history";
const HISTORY_PAGE_SIZE = 30;

export function StudiesPage() {
  const { knowledgeAreas, projects } = useAzrielData();
  const [tab, setTab] = useState<StudyTab>("today");
  const [roadmaps, setRoadmaps] = useState<StudyRoadmap[]>([]);
  const [summary, setSummary] = useState<StudyTodaySummary | null>(null);
  const [settings, setSettings] = useState<StudySettings | null>(null);
  const [recentSessions, setRecentSessions] = useState<StudySession[]>([]);
  const [recentNotes, setRecentNotes] = useState<StudyNoteSummary[]>([]);
  const [reviewDashboard, setReviewDashboard] = useState<StudyReviewDashboardSummary | null>(null);
  const [reviewSessions, setReviewSessions] = useState<StudyReviewSession[]>([]);
  const [history, setHistory] = useState<StudySession[]>([]);
  const [historyRoadmapId, setHistoryRoadmapId] = useState("");
  const [historyDateFrom, setHistoryDateFrom] = useState("");
  const [historyDateTo, setHistoryDateTo] = useState("");
  const [historyLoading, setHistoryLoading] = useState(false);
  const [historyHasMore, setHistoryHasMore] = useState(false);
  const [editingRoadmap, setEditingRoadmap] = useState<StudyRoadmap | "new" | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<StudyRoadmap | null>(null);
  const [busyActivityId, setBusyActivityId] = useState<string | null>(null);
  const [sessionBusy, setSessionBusy] = useState(false);
  const [completion, setCompletion] = useState<StudySession | null>(null);
  const [noteLaunch, setNoteLaunch] = useState<StudyNoteLaunch | null>(null);
  const [cardLaunch, setCardLaunch] = useState<StudyCardLaunch | null>(null);
  const [libraryLaunch, setLibraryLaunch] = useState<StudyLibraryLaunch | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const notebooksRef = useRef<StudyNotebooksHandle>(null);
  const noteLaunchToken = useRef(0);

  const refreshStudyState = useCallback(async () => {
    const [nextSummary, nextSettings, nextRecent, nextNotes, nextReview] = await Promise.all([
      studyLabService.todaySummary(),
      studyLabService.settings(),
      studyLabService.sessions({ limit: 10, offset: 0 }),
      studyLabService.recentNotes(5),
      studyLabService.reviewDashboard(),
    ]);
    setSummary(nextSummary);
    setSettings(nextSettings);
    setRecentSessions(nextRecent);
    setRecentNotes(nextNotes);
    setReviewDashboard(nextReview);
  }, []);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const [nextRoadmaps] = await Promise.all([starkService.roadmaps(), refreshStudyState()]);
      setRoadmaps(nextRoadmaps);
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setLoading(false);
    }
  }, [refreshStudyState]);

  useEffect(() => {
    const initialization = window.setTimeout(() => void load(), 0);
    return () => window.clearTimeout(initialization);
  }, [load]);

  async function loadHistory(offset = 0) {
    setHistoryLoading(true);
    setError(null);
    try {
      const [next, nextReviews] = await Promise.all([
        studyLabService.sessions({ roadmapId: historyRoadmapId || null, dateFrom: historyDateFrom || null, dateTo: historyDateTo || null, limit: HISTORY_PAGE_SIZE, offset }),
        offset === 0 ? studyLabService.reviewSessions({ limit: 100, offset: 0 }) : Promise.resolve(reviewSessions),
      ]);
      setHistory((current) => offset ? [...current, ...next] : next);
      if (offset === 0) setReviewSessions(nextReviews);
      setHistoryHasMore(next.length === HISTORY_PAGE_SIZE);
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setHistoryLoading(false);
    }
  }

  async function selectTab(next: StudyTab) {
    if (tab === "notebooks" && next !== "notebooks" && !(await (notebooksRef.current?.flush() ?? Promise.resolve(true)))) {
      setError("A nota possui alterações que ainda não foram persistidas. Corrija o erro de salvamento antes de sair.");
      return;
    }
    setTab(next);
    if (next === "history" && !history.length) void loadHistory();
  }

  const openNotes = useCallback(async (mode: "open" | "create", context: StudyNoteContext, noteId?: string) => {
    if (tab === "notebooks" && !(await (notebooksRef.current?.flush() ?? Promise.resolve(true)))) {
      setError("Não foi possível salvar a nota atual antes de abrir outro contexto.");
      return;
    }
    noteLaunchToken.current += 1;
    setNoteLaunch({ ...context, mode, noteId, token: noteLaunchToken.current });
    setTab("notebooks");
  }, [tab]);

  const openSessionNotes = useCallback((session: StudySession, mode: "open" | "create") => openNotes(mode, {
    roadmapId: session.roadmapId,
    stageId: session.stageId,
    topicId: session.topicId,
    activityId: session.activityId,
    studySessionId: session.id,
  }), [openNotes]);

  const openActivityNotes = useCallback((roadmap: StudyRoadmap, stage: RoadmapStage, topic: RoadmapTopic, activity: RoadmapActivity, mode: "open" | "create") => {
    void openNotes(mode, { roadmapId: roadmap.id, stageId: stage.id, topicId: topic.id, activityId: activity.id, studySessionId: null });
  }, [openNotes]);

  const openCards = useCallback(async (mode: "open" | "create", context: Omit<StudyCardLaunch, "mode" | "token">) => {
    if (tab === "notebooks" && !(await (notebooksRef.current?.flush() ?? Promise.resolve(true)))) {
      setError("Não foi possível salvar a nota atual antes de abrir os Study Cards.");
      return;
    }
    noteLaunchToken.current += 1;
    setCardLaunch({ ...context, mode, token: noteLaunchToken.current });
    setTab("review");
  }, [tab]);

  const openActivityCards = useCallback((roadmap: StudyRoadmap, stage: RoadmapStage, topic: RoadmapTopic, activity: RoadmapActivity, mode: "open" | "create") => {
    void openCards(mode, { roadmapId: roadmap.id, stageId: stage.id, topicId: topic.id, activityId: activity.id, notebookId: null, noteId: null });
  }, [openCards]);

  const openNoteCard = useCallback((note: StudyNote) => {
    void openCards("create", { roadmapId: note.roadmapId, stageId: note.stageId, topicId: note.topicId, activityId: note.activityId, notebookId: note.notebookId, noteId: note.id });
  }, [openCards]);

  const openLibrary = useCallback(async (relation?: StudyMaterialRelationInput, materialId?: string) => {
    if (tab === "notebooks" && !(await (notebooksRef.current?.flush() ?? Promise.resolve(true)))) {
      setError("Não foi possível salvar a nota atual antes de abrir a Biblioteca.");
      return;
    }
    noteLaunchToken.current += 1;
    setLibraryLaunch({ token: noteLaunchToken.current, materialId, relation });
    setTab("library");
  }, [tab]);

  const reportStudyError = useCallback((message: string | null) => setError(message), []);

  async function saveRoadmap(input: StudyRoadmapInput) {
    const result = await starkService.saveRoadmap(input);
    setRoadmaps(result.roadmaps);
    setEditingRoadmap(null);
    await refreshStudyState();
  }

  async function updateRoadmapActivity(activity: RoadmapActivity, status: RoadmapActivityStatus) {
    setBusyActivityId(activity.id);
    setError(null);
    try {
      const result = await starkService.updateActivityStatus({ activityId: activity.id, status });
      setRoadmaps(result.roadmaps);
      await refreshStudyState();
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setBusyActivityId(null);
    }
  }

  async function startSession(roadmap: StudyRoadmap, activity: RoadmapActivity) {
    if (!settings) return;
    setSessionBusy(true);
    setCompletion(null);
    setError(null);
    try {
      await studyLabService.startSession({ id: crypto.randomUUID(), roadmapId: roadmap.id, activityId: activity.id, plannedFocusMinutes: settings.focusMinutes });
      await refreshStudyState();
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setSessionBusy(false);
    }
  }

  async function operateSession(operation: (id: string) => Promise<StudySession>, complete = false) {
    const active = summary?.activeSession;
    if (!active) return;
    setSessionBusy(true);
    setError(null);
    try {
      const result = await operation(active.id);
      if (complete) setCompletion(result);
      await refreshStudyState();
      if (tab === "history") await loadHistory();
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setSessionBusy(false);
    }
  }

  async function saveSettings(input: StudySettingsInput) {
    setSessionBusy(true);
    setError(null);
    try {
      setSettings(await studyLabService.updateSettings(input));
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setSessionBusy(false);
    }
  }

  async function removeRoadmap() {
    if (!deleteTarget) return;
    try {
      setRoadmaps(await starkService.deleteRoadmap(deleteTarget.id));
      setDeleteTarget(null);
      await refreshStudyState();
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    }
  }

  if (loading || !summary || !settings || !reviewDashboard) return <div className="core-empty">INICIALIZANDO STUDY LAB...</div>;

  return <section className="study-lab">
    <header className="study-lab__header"><div><span>STUDY LAB v0.5</span><h1>Estudos</h1><p>Roadmaps, foco, conhecimento, biblioteca local, revisão ativa e ferramentas locais de IA.</p></div><button onClick={() => setEditingRoadmap("new")}>＋ NOVO ROADMAP</button></header>
    <nav className="study-lab__tabs" aria-label="Seções de Estudos">{(["today", "roadmaps", "notebooks", "library", "review", "history"] as const).map((item) => <button key={item} className={tab === item ? "active" : ""} onClick={() => void selectTab(item)}>{item === "today" ? "HOJE" : item === "roadmaps" ? "ROADMAPS" : item === "notebooks" ? "CADERNOS" : item === "library" ? "BIBLIOTECA" : item === "review" ? "REVISÃO" : "HISTÓRICO"}</button>)}</nav>
    {error && <div className="form-error stark-error">{error}<button onClick={() => void load()}>TENTAR NOVAMENTE</button></div>}
    {completion && <div className="study-completion"><strong>SESSÃO CONCLUÍDA</strong><span>{Math.floor(completion.currentFocusSeconds / 60)} min de foco · {completion.activityTitle || completion.roadmapName || "Sessão livre"}{completion.linkedNoteCount ? ` · ${completion.linkedNoteCount} nota(s) vinculada(s)` : ""}</span><button onClick={() => setCompletion(null)}>×</button></div>}
    {summary.activeSession && <StudySessionPanel key={`${summary.activeSession.id}-${summary.activeSession.observedAt}`} session={summary.activeSession} busy={sessionBusy} onPause={() => operateSession(studyLabService.pauseSession)} onResume={() => operateSession(studyLabService.resumeSession)} onComplete={() => operateSession(studyLabService.completeSession, true)} onCancel={() => operateSession(studyLabService.cancelSession)} onOpenNotes={() => openSessionNotes(summary.activeSession!, "open")} onCreateNote={() => openSessionNotes(summary.activeSession!, "create")} />}
    {tab === "today" && <StudyToday roadmaps={roadmaps} summary={summary} recentSessions={recentSessions} recentNotes={recentNotes} reviewSummary={reviewDashboard} settings={settings} busy={sessionBusy} onNewRoadmap={() => setEditingRoadmap("new")} onOpenRoadmaps={() => setTab("roadmaps")} onStartSession={startSession} onSaveSettings={saveSettings} onOpenNote={(note) => void openNotes("open", { roadmapId: note.roadmapId, stageId: null, topicId: null, activityId: note.activityId, studySessionId: note.studySessionId }, note.id)} onOpenReview={() => setTab("review")} />}
    {tab === "roadmaps" && <RoadmapWorkspace roadmaps={roadmaps} busyActivityId={busyActivityId} activeSessionActivityId={summary.activeSession?.activityId ?? null} sessionBusy={sessionBusy} onNew={() => setEditingRoadmap("new")} onEdit={setEditingRoadmap} onDelete={setDeleteTarget} onActivityStatus={updateRoadmapActivity} onStartSession={startSession} onNotes={openActivityNotes} onCards={openActivityCards} onMaterials={(activity) => void openLibrary({ relationType: "ACTIVITY", relationId: activity.id })} />}
    {tab === "notebooks" && <StudyNotebooks key={noteLaunch?.token ?? 0} ref={notebooksRef} launch={noteLaunch} onChanged={refreshStudyState} onError={reportStudyError} onCreateCard={openNoteCard} onOpenLibrary={(note) => void openLibrary({ relationType: "NOTE", relationId: note.id })} />}
    {tab === "library" && <StudyLibrary key={libraryLaunch?.token ?? 0} roadmaps={roadmaps} launch={libraryLaunch} onChanged={refreshStudyState} />}
    {tab === "review" && <StudyReview key={cardLaunch?.token ?? 0} launch={cardLaunch} activeStudySessionId={summary.activeSession?.id ?? null} onChanged={refreshStudyState} onError={reportStudyError} />}
    {tab === "history" && <StudyHistory sessions={history} reviewSessions={reviewSessions} roadmaps={roadmaps} roadmapId={historyRoadmapId} dateFrom={historyDateFrom} dateTo={historyDateTo} loading={historyLoading} hasMore={historyHasMore} onRoadmap={setHistoryRoadmapId} onDateFrom={setHistoryDateFrom} onDateTo={setHistoryDateTo} onApply={() => void loadHistory()} onMore={() => void loadHistory(history.length)} onOpenNotes={(session) => void openSessionNotes(session, "open")} />}
    {editingRoadmap && <RoadmapEditor roadmap={editingRoadmap === "new" ? null : editingRoadmap} knowledge={knowledgeAreas} projects={projects} onCancel={() => setEditingRoadmap(null)} onSave={saveRoadmap} />}
    {deleteTarget && <DeleteConfirmationDialog kind="roadmap" title={deleteTarget.name} description="O roadmap será removido. Sessões anteriores continuarão no histórico com seus nomes preservados." busy={false} onCancel={() => setDeleteTarget(null)} onConfirm={() => void removeRoadmap()} />}
  </section>;
}
