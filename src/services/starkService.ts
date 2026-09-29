import { starkRepository } from "../repositories/starkRepository";
import type { StudyRoadmap, StudyRoadmapInput } from "../types";

export const calculateRoadmapProgress = (roadmap: Pick<StudyRoadmap, "stages">) => {
  const activities = roadmap.stages.flatMap((stage) => stage.topics).flatMap((topic) => topic.activities);
  const completed = activities.filter((activity) => activity.status === "completed").length;
  return { completed, total: activities.length, progress: activities.length ? Math.floor(completed * 100 / activities.length) : 0 };
};

export const normalizeRoadmapOrder = (input: StudyRoadmapInput): StudyRoadmapInput => ({
  ...input,
  name: input.name.trim(),
  description: input.description.trim(),
  stages: input.stages.map((stage, stageIndex) => ({
    ...stage, order: stageIndex + 1,
    topics: stage.topics.map((topic, topicIndex) => ({
      ...topic, order: topicIndex + 1,
      prerequisiteTopicIds: topic.prerequisiteTopicIds ?? [],
      activities: topic.activities.map((activity, activityIndex) => ({ ...activity, order: activityIndex + 1, primaryKnowledgeNodeId: activity.primaryKnowledgeNodeId ?? null, secondaryKnowledgeNodeIds: activity.secondaryKnowledgeNodeIds ?? [], projectId: activity.projectId ?? null, researchId: activity.researchId ?? null, learningObjective: activity.learningObjective?.trim() || null, instructions: activity.instructions?.trim() || null, completionCriteria: activity.completionCriteria?.trim() || null, deliverable: activity.deliverable?.trim() || null, estimatedMinutes: activity.estimatedMinutes ?? null, learningMethod: activity.learningMethod ? { type: activity.learningMethod.type, instructions: activity.learningMethod.instructions?.trim() || null } : null, resources: activity.resources ?? [], isValidation: activity.isValidation ?? false, reflectionPrompt: activity.reflectionPrompt?.trim() || null })),
    })),
  })),
});

export const starkService = {
  ...starkRepository,
  saveRoadmap: (input: StudyRoadmapInput) => starkRepository.saveRoadmap(normalizeRoadmapOrder(input)),
};
