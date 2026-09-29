import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import type { RoadmapContext } from "../../../services/roadmapExperience";
import type { RoadmapActivity } from "../../../types";
import { ActivityPreview } from "./ActivityPreview";
import { StudyFocusView } from "./StudyFocusView";

const activity: RoadmapActivity = {
  id: "activity-focus", title: "Investigar comportamento", description: "Preview compacto", activityType: "RESEARCH", status: "pending", completedAt: null, order: 1,
  learningObjective: "Identificar padrões verificáveis.", instructions: "Colete evidências antes de concluir.", completionCriteria: "Evidência reproduzível.", deliverable: "Relatório curto.", estimatedMinutes: 90,
  learningMethod: { type: "PROBLEM_SOLVING", instructions: "Formule hipóteses." }, isValidation: true, reflectionPrompt: "O que mudou?",
  resources: [{ id: "internal-resource-id", type: "BOOK", title: "Testing Business Ideas", provider: "Strategyzer", language: "en", required: true, url: "https://example.com/book", studyMaterialId: null }],
};
const context: RoadmapContext = {
  roadmap: { id: "roadmap-1", name: "Empreendedorismo Tecnológico", description: "", status: "active", completedActivities: 0, totalActivities: 1, progress: 0, createdAt: "", updatedAt: "", stages: [] },
  stage: { id: "stage-1", name: "Baseline", description: "", order: 1, topics: [] },
  topic: { id: "topic-1", name: "Decisão por evidência", description: "", knowledgeNodeId: "technology-business", state: "NOT_STARTED", order: 1, activities: [activity] },
  activity,
};

const noop = async () => undefined;

describe("Study Lab focus experience", () => {
  it("mantém o preview compacto sem expor instruções e critérios", () => {
    const html = renderToStaticMarkup(<ActivityPreview stage={context.stage} topic={context.topic} selectedActivityId={activity.id} activeSessionActivityId={null} onSelect={() => undefined} onStartStudy={() => undefined} />);
    expect(html).toContain("INICIAR ESTUDO");
    expect(html).toContain("Identificar padrões verificáveis.");
    expect(html).not.toContain("Colete evidências antes de concluir.");
    expect(html).not.toContain("Evidência reproduzível.");
  });

  it("prioriza objetivo, instruções e recursos no foco sem mostrar IDs internos", () => {
    const html = renderToStaticMarkup(<StudyFocusView context={context} activeSession={null} sessionBusy={false} busyActivityId={null} onBack={() => undefined} onStartSession={noop} onPauseSession={noop} onResumeSession={noop} onCompleteSession={noop} onCancelSession={noop} onOpenSessionNotes={noop} onCreateSessionNote={noop} onActivityStatus={noop} onNotes={() => undefined} onCards={() => undefined} onOpenLibrary={() => undefined} onOpenResource={noop} onAddResourceToLibrary={noop} />);
    expect(html).toContain("OBJETIVO");
    expect(html).toContain("O QUE FAZER");
    expect(html).toContain("Testing Business Ideas");
    expect(html).toContain("ENTREGA E CRITÉRIO");
    expect(html).toContain("AZRIEL AI");
    expect(html).not.toContain("internal-resource-id");
  });
});
