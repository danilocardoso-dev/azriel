import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import type { RoadmapActivity } from "../../../types";
import { ActivityLearningDetails } from "./ActivityLearningDetails";

const baseActivity = (): RoadmapActivity => ({
  id: "activity-1", title: "Teste", description: "", activityType: "EXPERIMENT", status: "pending", completedAt: null, order: 1,
  primaryKnowledgeNodeId: null, secondaryKnowledgeNodeIds: [], projectId: null, researchId: null,
});

const render = (activity: RoadmapActivity) => renderToStaticMarkup(<ActivityLearningDetails activity={activity} selectedResourceIds={new Set()} onToggleResource={() => undefined} onOpenResource={async () => undefined} onAddToLibrary={async () => undefined} />);

describe("ActivityLearningDetails", () => {
  it("não renderiza seções vazias para roadmaps legados", () => {
    const html = render(baseActivity());
    expect(html).not.toContain("OBJETIVO");
    expect(html).not.toContain("RECURSOS");
    expect(html).not.toContain("ESTIMATIVA");
  });

  it("renderiza o modelo completo na hierarquia educacional", () => {
    const html = render({
      ...baseActivity(), learningObjective: "Objetivo", learningMethod: { type: "HANDS_ON", instructions: "Pratique" }, instructions: "Execute",
      resources: [{ id: "resource-1", type: "VIDEO", title: "Vídeo", url: "https://example.com", provider: "Example", language: "en", required: true, studyMaterialId: null }],
      deliverable: "Relatório", completionCriteria: "Explique", reflectionPrompt: "O que mudou?", estimatedMinutes: 45, isValidation: true,
    });
    for (const label of ["CHECKPOINT", "OBJETIVO", "MÉTODO", "INSTRUÇÕES", "RECURSOS", "ENTREGA", "CRITÉRIO DE CONCLUSÃO", "REFLEXÃO", "ESTIMATIVA"]) expect(html).toContain(label);
    expect(html.indexOf("OBJETIVO")).toBeLessThan(html.indexOf("MÉTODO"));
    expect(html.indexOf("RECURSOS")).toBeLessThan(html.indexOf("ENTREGA"));
  });
});
