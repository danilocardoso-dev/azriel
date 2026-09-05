import { useEffect, useState } from "react";
import { currentStudyContext, type RoadmapFilter } from "../../../services/roadmapExperience";
import type { KnowledgeArea, KnowledgeEvent, LearningMutation, Project, RoadmapActivity, RoadmapActivityStatus, RoadmapStage, RoadmapTopic, StudyRoadmap } from "../../../types";
import { RoadmapNavigator } from "./RoadmapNavigator";
import { RoadmapStructure } from "./RoadmapStructure";
import { TopicInspector } from "./TopicInspector";

const STORAGE_KEY = "azriel.stark.roadmap-workspace.v1";
type PersistedState = { roadmapId?: string; topicId?: string; expandedStageIds?: string[]; navigatorCollapsed?: boolean };
const readState = (): PersistedState => { try { return JSON.parse(localStorage.getItem(STORAGE_KEY) ?? "{}") as PersistedState; } catch { return {}; } };

type Props = {
  roadmaps: StudyRoadmap[];
  knowledge: KnowledgeArea[];
  projects: Project[];
  events: KnowledgeEvent[];
  busyActivityId: string | null;
  lastMutation: LearningMutation | null;
  onNew: () => void;
  onEdit: (roadmap: StudyRoadmap) => void;
  onDelete: (roadmap: StudyRoadmap) => void;
  onActivityStatus: (activity: RoadmapActivity, status: RoadmapActivityStatus) => Promise<void>;
  onKnowledge: (knowledge: KnowledgeArea) => void;
};

export function RoadmapWorkspace({ roadmaps, knowledge, projects, events, busyActivityId, lastMutation, onNew, onEdit, onDelete, onActivityStatus, onKnowledge }: Props) {
  const [stored] = useState(() => readState());
  const [query, setQuery] = useState("");
  const [filter, setFilter] = useState<RoadmapFilter>("all");
  const [selectedRoadmapId, setSelectedRoadmapId] = useState<string | null>(() => stored.roadmapId ?? null);
  const [selectedTopicId, setSelectedTopicId] = useState<string | null>(() => stored.topicId ?? null);
  const [navigatorCollapsed, setNavigatorCollapsed] = useState(() => stored.navigatorCollapsed ?? false);
  const [expandedStages, setExpandedStages] = useState<Set<string>>(() => {
    if (Array.isArray(stored.expandedStageIds)) return new Set(stored.expandedStageIds);
    const initialRoadmap = roadmaps.find((item) => item.id === stored.roadmapId) ?? roadmaps.find((item) => item.status === "active") ?? roadmaps[0];
    const initialStage = currentStudyContext(initialRoadmap ? [initialRoadmap] : [])?.stage ?? initialRoadmap?.stages[0];
    return new Set(initialStage ? [initialStage.id] : []);
  });
  const [revealActivitiesToken, setRevealActivitiesToken] = useState(0);
  const [revealTopicId, setRevealTopicId] = useState<string | null>(null);
  const selectedRoadmap = roadmaps.find((item) => item.id === selectedRoadmapId) ?? roadmaps.find((item) => item.status === "active") ?? roadmaps[0] ?? null;
  const selectedPair = selectedRoadmap?.stages.flatMap((stage) => stage.topics.map((topic) => ({ stage, topic }))).find((item) => item.topic.id === selectedTopicId)
    ?? selectedRoadmap?.stages.flatMap((stage) => stage.topics.map((topic) => ({ stage, topic })))[0] ?? null;
  const current = selectedRoadmap ? currentStudyContext([selectedRoadmap]) : null;

  useEffect(() => {
    try { localStorage.setItem(STORAGE_KEY, JSON.stringify({ roadmapId: selectedRoadmap?.id, topicId: selectedPair?.topic.id, expandedStageIds: [...expandedStages], navigatorCollapsed })); } catch { /* armazenamento indisponível não bloqueia o módulo */ }
  }, [expandedStages, navigatorCollapsed, selectedPair?.topic.id, selectedRoadmap?.id]);

  const selectRoadmap = (roadmap: StudyRoadmap) => {
    setSelectedRoadmapId(roadmap.id);
    const context = currentStudyContext([roadmap]);
    const stage = context?.stage ?? roadmap.stages[0]; const topic = context?.topic ?? stage?.topics[0];
    setSelectedTopicId(topic?.id ?? null); setExpandedStages(stage ? new Set([stage.id]) : new Set()); setRevealTopicId(null);
  };
  const selectTopic = (stage: RoadmapStage, topic: RoadmapTopic) => { setSelectedTopicId(topic.id); setExpandedStages((value) => new Set(value).add(stage.id)); setRevealTopicId(null); };
  const continueStudy = () => {
    if (!selectedRoadmap) return;
    const context = currentStudyContext([selectedRoadmap]);
    if (context) { setSelectedTopicId(context.topic.id); setExpandedStages((value) => new Set(value).add(context.stage.id)); setRevealTopicId(context.topic.id); setRevealActivitiesToken((value) => value + 1); }
  };

  return <section className="roadmap-workspace-shell">
    <header className="roadmap-workspace-title"><div><span>MAPA STARK // ROADMAP EXPERIENCE</span><h1>Roadmaps</h1><p>Estudo estruturado por etapas, tópicos e evidências reais.</p></div><button onClick={onNew}>＋ NOVO ROADMAP</button></header>
    {roadmaps.length ? <div className={`roadmap-workspace ${navigatorCollapsed ? "navigator-collapsed" : ""}`}>
      <RoadmapNavigator roadmaps={roadmaps} selectedId={selectedRoadmap?.id ?? null} query={query} filter={filter} collapsed={navigatorCollapsed} onQuery={setQuery} onFilter={setFilter} onSelect={selectRoadmap} onToggleCollapsed={() => setNavigatorCollapsed((value) => !value)} />
      {selectedRoadmap && <RoadmapStructure roadmap={selectedRoadmap} events={events} expandedStages={expandedStages} selectedTopicId={selectedPair?.topic.id ?? null} currentActivityId={current?.activity.id ?? null} onToggleStage={(id) => setExpandedStages((value) => { const next = new Set(value); if (next.has(id)) next.delete(id); else next.add(id); return next; })} onSelectTopic={selectTopic} onContinue={continueStudy} onEdit={() => onEdit(selectedRoadmap)} onDelete={() => onDelete(selectedRoadmap)} />}
      {selectedRoadmap && <TopicInspector key={`${selectedRoadmap.id}-${selectedPair?.topic.id ?? "empty"}-${revealActivitiesToken}`} roadmap={selectedRoadmap} stage={selectedPair?.stage ?? null} topic={selectedPair?.topic ?? null} currentActivityId={current?.activity.id ?? null} knowledge={knowledge} projects={projects} events={events} busyActivityId={busyActivityId} lastMutation={lastMutation} startInActivities={revealTopicId === selectedPair?.topic.id} onActivityStatus={onActivityStatus} onKnowledge={onKnowledge} />}
    </div> : <div className="core-empty">Nenhum roadmap cadastrado. Crie o primeiro caminho de estudo.</div>}
  </section>;
}
