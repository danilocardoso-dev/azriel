import { describe, expect, it } from "vitest";
import type { RoadmapActivity, RoadmapStage, StudyRoadmap } from "../types";
import { activityProgress, currentStudyContext, evidenceDimension, filterRoadmaps, stageProgress } from "./roadmapExperience";

const activity = (id: string, status: RoadmapActivity["status"]): RoadmapActivity => ({ id, title: id, description: "", activityType: "READING", status, completedAt: null, order: 1 });
const stage = (activities: RoadmapActivity[]): RoadmapStage => ({ id: "stage", name: "Etapa", description: "", order: 1, topics: [{ id: "topic", name: "Tópico", description: "", knowledgeNodeId: null, state: "NOT_STARTED", order: 1, activities }] });
const roadmap = (id: string, status: StudyRoadmap["status"], activities: RoadmapActivity[]): StudyRoadmap => ({ id, name: id, description: "", status, completedActivities: 0, totalActivities: activities.length, progress: 0, stages: [stage(activities)], createdAt: "", updatedAt: "" });

describe("roadmap experience", () => {
  it("calcula progresso somente por atividades concluídas", () => {
    const activities = [activity("a", "completed"), activity("b", "in_progress"), activity("c", "pending")];
    expect(activityProgress(activities)).toBe(33);
    expect(stageProgress(stage(activities))).toBe(33);
  });
  it("continua uma atividade em andamento antes da próxima pendente", () => {
    const result = currentStudyContext([roadmap("principal", "active", [activity("a", "pending"), activity("b", "in_progress")])]);
    expect(result?.activity.id).toBe("b");
  });
  it("respeita o roadmap selecionado e depois usa o ativo", () => {
    const result = currentStudyContext([roadmap("ativo", "active", [activity("a", "pending")]), roadmap("selecionado", "paused", [activity("b", "pending")])], "selecionado");
    expect(result?.roadmap.id).toBe("selecionado");
  });
  it("filtra sem inventar registros", () => expect(filterRoadmaps([roadmap("Eletrônica", "active", [])], "eletr", "active")).toHaveLength(1));
  it("classifica o impacto esperado da evidência", () => {
    expect(evidenceDimension("READING")).toBe("COBERTURA");
    expect(evidenceDimension("EXERCISE")).toBe("PROFUNDIDADE");
    expect(evidenceDimension("PROJECT")).toBe("INTEGRAÇÃO");
  });
  it("encontra deterministicamente a posição em um roadmap grande", () => {
    const stages = Array.from({ length: 8 }, (_, stageIndex) => ({ ...stage(Array.from({ length: 16 }, (_, activityIndex) => activity(`a-${stageIndex}-${activityIndex}`, stageIndex === 5 && activityIndex === 7 ? "in_progress" : stageIndex < 5 ? "completed" : "pending"))), id: `stage-${stageIndex}`, order: stageIndex + 1 }));
    const large = { ...roadmap("grande", "active", []), stages, totalActivities: 128 };
    expect(currentStudyContext([large])?.activity.id).toBe("a-5-7");
  });
});
