import { describe, expect, it } from "vitest";
import type { StudyAIContext } from "../../types";
import { StudyAIContextBuilder } from "./StudyAIContextBuilder";

const fullContext = (): StudyAIContext => ({
  selectedText: null,
  roadmap: { id: "roadmap-1", name: "Eletrônica" },
  stage: { id: "stage-1", name: "Fundamentos" },
  topic: { id: "topic-1", name: "Lei de Ohm" },
  activity: { id: "activity-1", name: "Exercício", description: "Calcular tensão e corrente.", activityType: "exercise" },
  note: { id: "note-1", title: "Anotações", content: "V = R × I", notebookId: "notebook-1", notebookTitle: "Circuitos" },
  studySession: { id: "session-1", status: "RUNNING", plannedFocusMinutes: 25 },
});

describe("StudyAIContextBuilder", () => {
  it("prioriza seleção e omite o corpo completo da nota", () => {
    const result = new StudyAIContextBuilder().build("EXPLAIN", { ...fullContext(), selectedText: "corrente elétrica" });
    expect(result.source).toBe("SELECTED_TEXT");
    expect(result.serialized).toContain("corrente elétrica");
    expect(result.serialized).not.toContain("V = R × I");
  });

  it("serializa Note, Activity e Topic deterministicamente sem dump adicional", () => {
    const builder = new StudyAIContextBuilder();
    const first = builder.build("SUMMARIZE", fullContext());
    const second = builder.build("SUMMARIZE", { ...fullContext(), unexpectedDatabaseDump: "SECRET" } as StudyAIContext);
    expect(second.serialized).toBe(first.serialized);
    expect(first.serialized).toContain("V = R × I");
    expect(first.serialized).toContain("Lei de Ohm");
    expect(first.serialized).not.toContain("SECRET");
  });

  it("inclui somente o contexto de review necessário para avaliação", () => {
    const result = new StudyAIContextBuilder().build("EVALUATE_ANSWER", {
      topic: { id: "topic-1", name: "Circuitos" },
      review: { question: "O que é tensão?", expectedAnswer: "Diferença de potencial.", userAnswer: "Uma diferença de potencial elétrico." },
    });
    expect(result.source).toBe("REVIEW");
    expect(result.serialized).not.toContain("unexpectedDatabaseDump");
    expect(result.serialized).toContain("diferença de potencial elétrico");
  });

  it("trunca explicitamente conteúdo longo dentro do limite", () => {
    const result = new StudyAIContextBuilder(1_200).build("SUMMARIZE", {
      note: { id: "note-1", title: "Longa", content: "conteúdo ".repeat(600), notebookId: null, notebookTitle: null },
    });
    expect(result.contextTruncated).toBe(true);
    expect(result.serialized.length).toBeLessThanOrEqual(1_200);
    expect(result.serialized).toContain("CONTEXTO TRUNCADO");
  });

  it("bloqueia contexto ausente e avaliação sem resposta do usuário", () => {
    expect(() => new StudyAIContextBuilder().build("EXPLAIN", {})).toThrow("contexto de estudo suficiente");
    expect(() => new StudyAIContextBuilder().build("EVALUATE_ANSWER", { review: { question: "Q", expectedAnswer: "A", userAnswer: "" } })).toThrow("resposta do usuário");
  });

  it("inclui somente material explicitamente selecionado e preserva source ids", () => {
    const result = new StudyAIContextBuilder().build("EXPLAIN", {
      materials: [{ id: "material-1", title: "Circuitos", materialType: "PDF", selectedText: "Trecho escolhido pelo usuário", pageRange: { from: 2, to: 3 } }],
    });
    expect(result.source).toBe("MATERIAL");
    expect(result.materialIds).toEqual(["material-1"]);
    expect(result.serialized).toContain("Trecho escolhido pelo usuário");
    expect(result.serialized).toContain('"from": 2');
    expect(() => new StudyAIContextBuilder().build("EXPLAIN", { materials: [{ id: "material-2", title: "Não selecionado", materialType: "TEXT", selectedText: "", pageRange: null }] })).toThrow("selecionado explicitamente");
  });

  it("inclui o modelo de aprendizagem e somente recursos escolhidos pela interface", () => {
    const result = new StudyAIContextBuilder().build("EXPLAIN", {
      activity: {
        id: "activity-1", name: "Handshake", description: "Captura TCP", activityType: "EXPERIMENT",
        learningObjective: "Compreender a conexão", instructions: "Capture no Wireshark", completionCriteria: "Explique sem roteiro",
        deliverable: "PCAP", estimatedMinutes: 60, learningMethod: { type: "HANDS_ON", instructions: "Faça primeiro" },
        isValidation: true, reflectionPrompt: "O que não viu?",
        resources: [{ id: "selected", type: "VIDEO", title: "TCP", url: "https://example.com", provider: "Example", language: "en", required: true, studyMaterialId: null }],
      },
    });
    expect(result.serialized).toContain("Compreender a conexão");
    expect(result.serialized).toContain('"id": "selected"');
    expect(result.serialized).not.toContain("conteúdo da página");
  });
});
