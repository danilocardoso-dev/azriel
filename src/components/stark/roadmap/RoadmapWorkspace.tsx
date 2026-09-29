import { useEffect, useState } from "react";
import { currentStudyContext, findRoadmapContext } from "../../../services/roadmapExperience";
import type { RoadmapActivity, RoadmapActivityResource, RoadmapActivityStatus, RoadmapStage, RoadmapTopic, StudyRoadmap, StudySession } from "../../../types";
import { StudySessionPanel } from "../../study/StudySessionPanel";
import { ActivityPreview } from "./ActivityPreview";
import { RoadmapManagementMenu } from "./RoadmapManagementMenu";
import { RoadmapSelector } from "./RoadmapSelector";
import { RoadmapStructure } from "./RoadmapStructure";
import { StudyFocusView } from "./StudyFocusView";

const STORAGE_KEY = "azriel.stark.roadmap-workspace.v2";
type WorkspaceView = "roadmap" | "focus";
type PersistedState = { roadmapId?: string; topicId?: string; activityId?: string; expandedStageIds?: string[]; view?: WorkspaceView };
const readState = (): PersistedState => { try { return JSON.parse(localStorage.getItem(STORAGE_KEY) ?? "{}") as PersistedState; } catch { return {}; } };

type Props = {
  roadmaps: StudyRoadmap[];
  busyActivityId: string | null;
  activeSession: StudySession | null;
  sessionBusy: boolean;
  onNew: () => void;
  onImport: () => void;
  onExport: (roadmap: StudyRoadmap) => void;
  onEdit: (roadmap: StudyRoadmap) => void;
  onDelete: (roadmap: StudyRoadmap) => void;
  onActivityStatus: (activity: RoadmapActivity, status: RoadmapActivityStatus) => Promise<void>;
  onStartSession: (roadmap: StudyRoadmap, activity: RoadmapActivity) => Promise<void>;
  onPauseSession: () => Promise<void>;
  onResumeSession: () => Promise<void>;
  onCompleteSession: () => Promise<void>;
  onCancelSession: () => Promise<void>;
  onOpenSessionNotes: () => Promise<void>;
  onCreateSessionNote: () => Promise<void>;
  onNotes: (roadmap: StudyRoadmap, stage: RoadmapStage, topic: RoadmapTopic, activity: RoadmapActivity, mode: "open" | "create") => void;
  onCards: (roadmap: StudyRoadmap, stage: RoadmapStage, topic: RoadmapTopic, activity: RoadmapActivity, mode: "open" | "create") => void;
  onMaterials: (activity: RoadmapActivity, materialId?: string) => void;
  onOpenResource: (resource: RoadmapActivityResource) => Promise<void>;
  onAddResourceToLibrary: (resource: RoadmapActivityResource) => Promise<void>;
  onFocusChange: (active: boolean) => void;
};

export function RoadmapWorkspace(props: Props) {
  const { roadmaps, busyActivityId, activeSession, sessionBusy, onNew, onImport, onExport, onEdit, onDelete, onActivityStatus, onStartSession, onPauseSession, onResumeSession, onCompleteSession, onCancelSession, onOpenSessionNotes, onCreateSessionNote, onNotes, onCards, onMaterials, onOpenResource, onAddResourceToLibrary, onFocusChange } = props;
  const [stored] = useState(() => readState());
  const [view, setView] = useState<WorkspaceView>(() => stored.view ?? "roadmap");
  const [selectedRoadmapId, setSelectedRoadmapId] = useState<string | null>(() => stored.roadmapId ?? null);
  const [selectedTopicId, setSelectedTopicId] = useState<string | null>(() => stored.topicId ?? null);
  const [selectedActivityId, setSelectedActivityId] = useState<string | null>(() => stored.activityId ?? null);
  const [expandedStages, setExpandedStages] = useState<Set<string>>(() => {
    if (Array.isArray(stored.expandedStageIds)) return new Set(stored.expandedStageIds);
    const initialRoadmap = roadmaps.find((item) => item.id === stored.roadmapId) ?? roadmaps.find((item) => item.status === "active") ?? roadmaps[0];
    const initialStage = currentStudyContext(initialRoadmap ? [initialRoadmap] : [])?.stage ?? initialRoadmap?.stages[0];
    return new Set(initialStage ? [initialStage.id] : []);
  });
  const selectedRoadmap = roadmaps.find((item) => item.id === selectedRoadmapId) ?? roadmaps.find((item) => item.status === "active") ?? roadmaps[0] ?? null;
  const topicPairs = selectedRoadmap?.stages.flatMap((stage) => stage.topics.map((topic) => ({ stage, topic }))) ?? [];
  const selectedPair = topicPairs.find((item) => item.topic.id === selectedTopicId) ?? topicPairs[0] ?? null;
  const current = selectedRoadmap ? currentStudyContext([selectedRoadmap]) : null;
  const selectedActivity = selectedPair?.topic.activities.find((item) => item.id === selectedActivityId)
    ?? selectedPair?.topic.activities.find((item) => item.id === current?.activity.id)
    ?? selectedPair?.topic.activities[0] ?? null;
  const focusContext = selectedRoadmap && selectedActivityId ? findRoadmapContext([selectedRoadmap], selectedActivityId) : null;
  const focusActive = view === "focus" && Boolean(focusContext);

  useEffect(() => {
    try { localStorage.setItem(STORAGE_KEY, JSON.stringify({ roadmapId: selectedRoadmap?.id, topicId: selectedPair?.topic.id, activityId: selectedActivity?.id, expandedStageIds: [...expandedStages], view: focusActive ? "focus" : "roadmap" })); } catch { /* estado local indisponível não bloqueia o módulo */ }
  }, [expandedStages, focusActive, selectedActivity?.id, selectedPair?.topic.id, selectedRoadmap?.id]);
  useEffect(() => { onFocusChange(focusActive); return () => onFocusChange(false); }, [focusActive, onFocusChange]);

  const selectRoadmap = (roadmap: StudyRoadmap) => {
    setSelectedRoadmapId(roadmap.id);
    const context = currentStudyContext([roadmap]);
    const stage = context?.stage ?? roadmap.stages[0];
    const topic = context?.topic ?? stage?.topics[0];
    const activity = context?.activity ?? topic?.activities[0];
    setSelectedTopicId(topic?.id ?? null);
    setSelectedActivityId(activity?.id ?? null);
    setExpandedStages(stage ? new Set([stage.id]) : new Set());
    setView("roadmap");
  };
  const selectTopic = (stage: RoadmapStage, topic: RoadmapTopic) => {
    setSelectedTopicId(topic.id);
    const activity = topic.activities.find((item) => item.status === "in_progress") ?? topic.activities.find((item) => item.status === "pending") ?? topic.activities[0];
    setSelectedActivityId(activity?.id ?? null);
    setExpandedStages((value) => new Set(value).add(stage.id));
  };
  const enterFocus = (activity: RoadmapActivity) => { setSelectedActivityId(activity.id); setView("focus"); };
  const continueStudy = () => {
    if (!selectedRoadmap) return;
    const context = currentStudyContext([selectedRoadmap]);
    if (!context) return;
    setSelectedTopicId(context.topic.id);
    setSelectedActivityId(context.activity.id);
    setExpandedStages(new Set([context.stage.id]));
    setView("focus");
  };

  if (!roadmaps.length) return <div className="study-empty-state"><span>STUDY LAB // ROADMAPS</span><h2>Nenhum roadmap ativo.</h2><p>Crie ou importe um roadmap para iniciar seu caminho de estudo.</p><div><button onClick={onNew}>＋ NOVO ROADMAP</button><button onClick={onImport}>IMPORTAR JSON</button></div></div>;
  if (focusActive && focusContext) return <StudyFocusView context={focusContext} activeSession={activeSession} sessionBusy={sessionBusy} busyActivityId={busyActivityId} onBack={() => setView("roadmap")} onStartSession={() => onStartSession(focusContext.roadmap, focusContext.activity)} onPauseSession={onPauseSession} onResumeSession={onResumeSession} onCompleteSession={onCompleteSession} onCancelSession={onCancelSession} onOpenSessionNotes={onOpenSessionNotes} onCreateSessionNote={onCreateSessionNote} onActivityStatus={(status) => onActivityStatus(focusContext.activity, status)} onNotes={(mode) => onNotes(focusContext.roadmap, focusContext.stage, focusContext.topic, focusContext.activity, mode)} onCards={(mode) => onCards(focusContext.roadmap, focusContext.stage, focusContext.topic, focusContext.activity, mode)} onOpenLibrary={(materialId) => onMaterials(focusContext.activity, materialId)} onOpenResource={onOpenResource} onAddResourceToLibrary={onAddResourceToLibrary} />;

  return <section className="roadmap-workspace-shell">
    <header className="roadmap-workspace-toolbar"><RoadmapSelector roadmaps={roadmaps} selectedId={selectedRoadmap?.id ?? null} onSelect={selectRoadmap} /><RoadmapManagementMenu hasRoadmap={Boolean(selectedRoadmap)} onNew={onNew} onImport={onImport} onExport={() => selectedRoadmap && onExport(selectedRoadmap)} onEdit={() => selectedRoadmap && onEdit(selectedRoadmap)} onDelete={() => selectedRoadmap && onDelete(selectedRoadmap)} /></header>
    {activeSession && <StudySessionPanel compact session={activeSession} busy={sessionBusy} onPause={onPauseSession} onResume={onResumeSession} onComplete={onCompleteSession} onCancel={onCancelSession} onOpenNotes={onOpenSessionNotes} onCreateNote={onCreateSessionNote} />}
    <div className="roadmap-workspace roadmap-workspace--refined">
      {selectedRoadmap && <RoadmapStructure roadmap={selectedRoadmap} expandedStages={expandedStages} selectedTopicId={selectedPair?.topic.id ?? null} currentActivityId={current?.activity.id ?? null} onToggleStage={(id) => setExpandedStages((value) => { const next = new Set(value); if (next.has(id)) next.delete(id); else next.add(id); return next; })} onSelectTopic={selectTopic} onContinue={continueStudy} />}
      <ActivityPreview stage={selectedPair?.stage ?? null} topic={selectedPair?.topic ?? null} selectedActivityId={selectedActivity?.id ?? null} activeSessionActivityId={activeSession?.activityId ?? null} onSelect={(activity) => setSelectedActivityId(activity.id)} onStartStudy={enterFocus} />
    </div>
  </section>;
}
