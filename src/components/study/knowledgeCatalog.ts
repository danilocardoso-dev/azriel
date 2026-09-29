import type { StudyRoadmap } from "../../types";

export function knowledgeReferenceCount(areaId: string, roadmaps: StudyRoadmap[]) {
  let topics = 0;
  let activities = 0;
  for (const roadmap of roadmaps) for (const stage of roadmap.stages) for (const topic of stage.topics) {
    if (topic.knowledgeNodeId === areaId) topics += 1;
    for (const activity of topic.activities) {
      if (activity.primaryKnowledgeNodeId === areaId || (activity.secondaryKnowledgeNodeIds ?? []).includes(areaId)) activities += 1;
    }
  }
  return { topics, activities };
}
