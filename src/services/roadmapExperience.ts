import type { CurrentStudyPosition, KnowledgeEvent, RoadmapActivity, RoadmapStage, RoadmapTopic, StudyRoadmap } from "../types";

export type RoadmapFilter = "all" | "active" | "paused" | "completed";

export interface RoadmapContext {
  roadmap: StudyRoadmap;
  stage: RoadmapStage;
  topic: RoadmapTopic;
  activity: RoadmapActivity;
}

export const activityProgress = (activities: RoadmapActivity[]) => {
  if (!activities.length) return 0;
  return Math.round(activities.filter((item) => item.status === "completed").length / activities.length * 100);
};

export const stageActivities = (stage: RoadmapStage) => stage.topics.flatMap((topic) => topic.activities);
export const stageProgress = (stage: RoadmapStage) => activityProgress(stageActivities(stage));

export function filterRoadmaps(roadmaps: StudyRoadmap[], query: string, filter: RoadmapFilter) {
  const normalized = query.trim().toLocaleLowerCase("pt-BR");
  return roadmaps.filter((roadmap) => (filter === "all" || roadmap.status === filter)
    && (!normalized || `${roadmap.name} ${roadmap.description}`.toLocaleLowerCase("pt-BR").includes(normalized)));
}

export function findRoadmapContext(roadmaps: StudyRoadmap[], activityId: string): RoadmapContext | null {
  for (const roadmap of roadmaps) for (const stage of roadmap.stages) for (const topic of stage.topics) {
    const activity = topic.activities.find((item) => item.id === activityId);
    if (activity) return { roadmap, stage, topic, activity };
  }
  return null;
}

export function currentStudyContext(roadmaps: StudyRoadmap[], preferredRoadmapId?: string | null): RoadmapContext | null {
  const preferred = preferredRoadmapId ? roadmaps.find((item) => item.id === preferredRoadmapId) : null;
  const candidates = [preferred, ...roadmaps.filter((item) => item.id !== preferred?.id && item.status === "active"), ...roadmaps.filter((item) => item.id !== preferred?.id && item.status !== "completed")].filter(Boolean) as StudyRoadmap[];
  for (const roadmap of candidates) {
    const contexts = roadmap.stages.flatMap((stage) => stage.topics.flatMap((topic) => topic.activities.map((activity) => ({ roadmap, stage, topic, activity }))));
    const next = contexts.find((item) => item.activity.status === "in_progress") ?? contexts.find((item) => item.activity.status === "pending");
    if (next) return next;
  }
  return null;
}

export function currentStudyPosition(roadmaps: StudyRoadmap[], preferredRoadmapId?: string | null): CurrentStudyPosition | null {
  const context = currentStudyContext(roadmaps, preferredRoadmapId);
  return context ? { roadmapId: context.roadmap.id, stageId: context.stage.id, topicId: context.topic.id, activityId: context.activity.id } : null;
}

export function roadmapImpact(events: KnowledgeEvent[], roadmapId: string) {
  return events.filter((event) => event.roadmapId === roadmapId && !event.reversalOfEventId).reduce((total, event) => ({
    coverage: total.coverage + event.coverageImpact,
    depth: total.depth + event.depthImpact,
    integration: total.integration + event.integrationImpact,
  }), { coverage: 0, depth: 0, integration: 0 });
}

export function evidenceDimension(type: RoadmapActivity["activityType"]) {
  if (["READING", "LESSON", "RESEARCH"].includes(type)) return "COBERTURA";
  if (["PROJECT", "DOCUMENTATION"].includes(type)) return "INTEGRAÇÃO";
  return "PROFUNDIDADE";
}

export function cleanTopicDescription(description: string) {
  return description.replace(/(?:^|\n)\s*Pr[eé]-requisitos?:[^\n]*/gi, "").trim();
}
