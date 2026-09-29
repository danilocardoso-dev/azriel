import { describe, expect, it } from "vitest";
import { FakeAIProvider } from "../providers/FakeAIProvider";
import type { AISettings, StudyAIContext } from "../../types";
import { StudyAIService } from "./StudyAIService";

const settings: AISettings = { provider: "ollama", endpoint: "http://localhost:11434", model: "configured:model", contextMessageLimit: 6, timeoutSeconds: 30, updatedAt: "" };
const context: StudyAIContext = { note: { id: "note-1", title: "Lei de Ohm", content: "V = R × I", notebookId: null, notebookTitle: null } };

describe("StudyAIService", () => {
  it("usa prompt e modelo configurados para explicar e resumir", async () => {
    const provider = new FakeAIProvider(["Explicação", "Resumo"]);
    const service = new StudyAIService(provider, settings);
    expect((await service.run("EXPLAIN", context)).promptVersion).toBe("STUDY_AI_EXPLAIN_V2");
    expect((await service.run("SUMMARIZE", context)).status).toBe("SUCCESS");
    expect(provider.requests.every((request) => request.model === "configured:model")).toBe(true);
    expect(provider.requests.every((request) => request.requestMetadata?.domain === "study")).toBe(true);
  });

  it("propaga somente os IDs dos materiais explicitamente usados", async () => {
    const provider = new FakeAIProvider("Explicação da fonte");
    const result = await new StudyAIService(provider, settings).run("EXPLAIN", { materials: [{ id: "material-1", title: "Fonte", materialType: "PDF", selectedText: "Trecho", pageRange: null }] });
    expect(result.materialIds).toEqual(["material-1"]);
    expect(provider.requests[0].messages.at(-1)?.content).toContain("material-1");
    expect(provider.requests[0].requestMetadata?.sourceMaterialCount).toBe(1);
  });

  it("valida quiz estruturado com 3, 5 ou 10 perguntas", async () => {
    for (const count of [3, 5, 10] as const) {
      const questions = Array.from({ length: count }, (_, index) => ({ question: `Q${index}`, expected_answer: `A${index}` }));
      const provider = new FakeAIProvider(JSON.stringify({ questions }));
      const result = await new StudyAIService(provider, settings).run("QUIZ", context, { count });
      expect(result.status).toBe("SUCCESS");
      expect(result.quiz).toHaveLength(count);
      expect(provider.requests[0].structuredOutputSchema).toBeTruthy();
    }
  });

  it("faz somente um retry e rejeita output inválido sem persistir nada", async () => {
    const provider = new FakeAIProvider(["not-json", "still-not-json"]);
    const result = await new StudyAIService(provider, settings).run("GENERATE_CARDS", context, { count: 3 });
    expect(result.status).toBe("INVALID_OUTPUT");
    expect(result.cards).toBeNull();
    expect(provider.requests).toHaveLength(2);
  });

  it("valida avaliação sem pontuação numérica", async () => {
    const provider = new FakeAIProvider(JSON.stringify({ assessment: "PARTIALLY_CORRECT", explanation: "Quase.", missing_points: ["Unidade"], strengths: ["Fórmula"], suggested_answer: "Use volts." }));
    const result = await new StudyAIService(provider, settings).run("EVALUATE_ANSWER", { review: { question: "Q", expectedAnswer: "A", userAnswer: "B" } });
    expect(result.status).toBe("SUCCESS");
    expect(result.evaluation?.assessment).toBe("PARTIALLY_CORRECT");
    expect(JSON.stringify(result.evaluation)).not.toMatch(/score/i);
  });

  it("classifica timeout, offline e cancelamento", async () => {
    const timeoutProvider = new FakeAIProvider();
    timeoutProvider.chat = async () => { throw new Error("O Ollama excedeu o tempo limite da solicitação"); };
    expect((await new StudyAIService(timeoutProvider, settings).run("EXPLAIN", context)).status).toBe("TIMEOUT");
    const offlineProvider = new FakeAIProvider();
    offlineProvider.chat = async () => { throw new Error("Ollama não encontrado no endpoint configurado"); };
    expect((await new StudyAIService(offlineProvider, settings).run("EXPLAIN", context)).status).toBe("PROVIDER_ERROR");
    expect((await new StudyAIService(new FakeAIProvider(), settings).run("EXPLAIN", context, { isCancelled: () => true })).status).toBe("CANCELLED");
  });
});
