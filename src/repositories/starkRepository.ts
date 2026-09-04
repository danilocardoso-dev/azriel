import type { KnowledgeBaseline, KnowledgeEvent, LearningEngineStatus, LearningMutation, ResearchInput, ResearchItem, RoadmapSaveResult, StarkSummary, StudyRoadmap, StudyRoadmapInput } from "../types";
import { invokeDatabase } from "./tauri";

export const starkRepository = {
  baselines: () => invokeDatabase<KnowledgeBaseline[]>("list_knowledge_baselines"),
  events: () => invokeDatabase<KnowledgeEvent[]>("list_knowledge_events"),
  roadmaps: () => invokeDatabase<StudyRoadmap[]>("list_study_roadmaps"),
  saveRoadmap: (input: StudyRoadmapInput) => invokeDatabase<RoadmapSaveResult>("save_study_roadmap", { input }),
  deleteRoadmap: (id: string) => invokeDatabase<StudyRoadmap[]>("delete_study_roadmap", { id }),
  research: () => invokeDatabase<ResearchItem[]>("list_research_items"),
  saveResearch: (input: ResearchInput) => invokeDatabase<ResearchItem[]>("save_research_item", { input }),
  deleteResearch: (id: string) => invokeDatabase<ResearchItem[]>("delete_research_item", { id }),
  summary: () => invokeDatabase<StarkSummary>("get_stark_summary"),
  learningStatus: () => invokeDatabase<LearningEngineStatus>("get_learning_engine_status"),
  rebuildLearning: () => invokeDatabase<LearningMutation>("rebuild_learning_engine"),
};
