import { describe, expect, it } from "vitest";
import type { StudyRoadmap } from "../../types";
import { knowledgeReferenceCount } from "./knowledgeCatalog";

const roadmap: StudyRoadmap = {
  id: "roadmap-1",
  name: "Roadmap",
  description: "",
  status: "active",
  completedActivities: 0,
  totalActivities: 2,
  progress: 0,
  createdAt: "2026-09-29",
  updatedAt: "2026-09-29",
  stages: [{
    id: "stage-1",
    name: "Etapa",
    description: "",
    order: 1,
    topics: [{
      id: "topic-1",
      name: "Tópico",
      description: "",
      knowledgeNodeId: "software-engineering",
      state: "NOT_STARTED",
      order: 1,
      activities: [
        { id: "activity-1", title: "Principal", description: "", activityType: "READING", status: "pending", completedAt: null, order: 1, primaryKnowledgeNodeId: "software-engineering", secondaryKnowledgeNodeIds: [] },
        { id: "activity-2", title: "Secundária", description: "", activityType: "EXERCISE", status: "pending", completedAt: null, order: 2, primaryKnowledgeNodeId: "automation", secondaryKnowledgeNodeIds: ["software-engineering"] },
      ],
    }],
  }],
};

describe("StudyKnowledgeCatalog", () => {
  it("conta referências de tópico, conhecimento primário e secundário", () => {
    expect(knowledgeReferenceCount("software-engineering", [roadmap])).toEqual({ topics: 1, activities: 2 });
    expect(knowledgeReferenceCount("automation", [roadmap])).toEqual({ topics: 0, activities: 1 });
    expect(knowledgeReferenceCount("english", [roadmap])).toEqual({ topics: 0, activities: 0 });
  });
});
