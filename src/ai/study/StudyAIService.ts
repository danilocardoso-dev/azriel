import explainPrompt from "../../prompts/study/explain-v2.txt?raw";
import summarizePrompt from "../../prompts/study/summarize-v2.txt?raw";
import quizPrompt from "../../prompts/study/quiz-v2.txt?raw";
import cardsPrompt from "../../prompts/study/generate-cards-v2.txt?raw";
import evaluatePrompt from "../../prompts/study/evaluate-answer-v2.txt?raw";
import type { AIRequest, AISettings, StudyAIAction, StudyAIContext, StudyAIEvaluation, StudyAIGeneratedCard, StudyAIQuizQuestion, StudyAIResult } from "../../types";
import type { AIProvider } from "../providers/AIProvider";
import { StudyAIContextBuilder } from "./StudyAIContextBuilder";

const promptByAction = {
  EXPLAIN: { version: "STUDY_AI_EXPLAIN_V2", prompt: explainPrompt },
  SUMMARIZE: { version: "STUDY_AI_SUMMARIZE_V2", prompt: summarizePrompt },
  QUIZ: { version: "STUDY_AI_QUIZ_V2", prompt: quizPrompt },
  GENERATE_CARDS: { version: "STUDY_AI_GENERATE_CARDS_V2", prompt: cardsPrompt },
  EVALUATE_ANSWER: { version: "STUDY_AI_EVALUATE_ANSWER_V2", prompt: evaluatePrompt },
} satisfies Record<StudyAIAction, { version: string; prompt: string }>;

const structuredActions = new Set<StudyAIAction>(["QUIZ", "GENERATE_CARDS", "EVALUATE_ANSWER"]);
const allowedCounts = new Set([3, 5, 10]);
const text = (value: unknown, maximum = 12_000) => typeof value === "string" && value.trim() && value.trim().length <= maximum ? value.trim() : null;

function schemaFor(action: StudyAIAction, count: number): Record<string, unknown> | null {
  const pair = { type: "object", additionalProperties: false, required: ["question", "expected_answer"], properties: { question: { type: "string" }, expected_answer: { type: "string" } } };
  if (action === "QUIZ") return { type: "object", additionalProperties: false, required: ["questions"], properties: { questions: { type: "array", minItems: count, maxItems: count, items: pair } } };
  if (action === "GENERATE_CARDS") return {
    type: "object",
    additionalProperties: false,
    required: ["cards"],
    properties: {
      cards: {
        type: "array",
        minItems: count,
        maxItems: count,
        items: {
          type: "object",
          additionalProperties: false,
          required: ["front", "back"],
          properties: { front: { type: "string" }, back: { type: "string" } },
        },
      },
    },
  };
  if (action === "EVALUATE_ANSWER") return { type: "object", additionalProperties: false, required: ["assessment", "explanation", "missing_points", "strengths", "suggested_answer"], properties: { assessment: { type: "string", enum: ["CORRECT", "PARTIALLY_CORRECT", "INCORRECT", "INSUFFICIENT_CONTEXT"] }, explanation: { type: "string" }, missing_points: { type: "array", items: { type: "string" } }, strengths: { type: "array", items: { type: "string" } }, suggested_answer: { type: "string" } } };
  return null;
}

function parseQuiz(value: unknown, count: number): StudyAIQuizQuestion[] | null {
  if (!value || typeof value !== "object" || !Array.isArray((value as { questions?: unknown }).questions)) return null;
  const items = (value as { questions: unknown[] }).questions;
  if (items.length !== count) return null;
  const parsed = items.map((item) => {
    if (!item || typeof item !== "object") return null;
    const question = text((item as Record<string, unknown>).question, 4_000);
    const expectedAnswer = text((item as Record<string, unknown>).expected_answer, 12_000);
    return question && expectedAnswer ? { question, expectedAnswer } : null;
  });
  return parsed.every(Boolean) ? parsed as StudyAIQuizQuestion[] : null;
}

function parseCards(value: unknown, count: number): StudyAIGeneratedCard[] | null {
  if (!value || typeof value !== "object" || !Array.isArray((value as { cards?: unknown }).cards)) return null;
  const items = (value as { cards: unknown[] }).cards;
  if (items.length !== count) return null;
  const parsed = items.map((item) => {
    if (!item || typeof item !== "object") return null;
    const front = text((item as Record<string, unknown>).front, 4_000);
    const back = text((item as Record<string, unknown>).back, 12_000);
    return front && back ? { front, back } : null;
  });
  return parsed.every(Boolean) ? parsed as StudyAIGeneratedCard[] : null;
}

function parseEvaluation(value: unknown): StudyAIEvaluation | null {
  if (!value || typeof value !== "object") return null;
  const record = value as Record<string, unknown>;
  const assessment = typeof record.assessment === "string" && ["CORRECT", "PARTIALLY_CORRECT", "INCORRECT", "INSUFFICIENT_CONTEXT"].includes(record.assessment) ? record.assessment as StudyAIEvaluation["assessment"] : null;
  const explanation = text(record.explanation, 12_000);
  const suggestedAnswer = text(record.suggested_answer, 12_000);
  const strings = (candidate: unknown) => Array.isArray(candidate) && candidate.length <= 30 && candidate.every((item) => Boolean(text(item, 2_000))) ? candidate.map((item) => String(item).trim()) : null;
  const missingPoints = strings(record.missing_points);
  const strengths = strings(record.strengths);
  return assessment && explanation && suggestedAnswer && missingPoints && strengths ? { assessment, explanation, missingPoints, strengths, suggestedAnswer } : null;
}

function parseStructured(action: StudyAIAction, content: string, count: number) {
  let value: unknown;
  try { value = JSON.parse(content); } catch { return null; }
  if (action === "QUIZ") return { quiz: parseQuiz(value, count), cards: null, evaluation: null };
  if (action === "GENERATE_CARDS") return { quiz: null, cards: parseCards(value, count), evaluation: null };
  if (action === "EVALUATE_ANSWER") return { quiz: null, cards: null, evaluation: parseEvaluation(value) };
  return null;
}

function cancelled(action: StudyAIAction, promptVersion: string, contextTruncated: boolean, materialIds: string[], latencyMs: number): StudyAIResult {
  return { action, status: "CANCELLED", content: null, quiz: null, cards: null, evaluation: null, model: "", promptVersion, latencyMs, contextTruncated, materialIds, error: "Solicitação cancelada localmente." };
}

function logResult(result: StudyAIResult): StudyAIResult {
  console.info("[STUDY_AI]", {
    action: result.action,
    model: result.model,
    promptVersion: result.promptVersion,
    latencyMs: result.latencyMs,
    status: result.status,
    contextTruncated: result.contextTruncated,
    error: result.error ? result.status.toLowerCase() : null,
  });
  return result;
}

export interface StudyAIRunOptions { count?: 3 | 5 | 10; userInstruction?: string | null; isCancelled?: () => boolean }

export class StudyAIService {
  constructor(private readonly provider: AIProvider, private readonly settings: AISettings, private readonly contextBuilder = new StudyAIContextBuilder()) {}

  async run(action: StudyAIAction, context: StudyAIContext, options: StudyAIRunOptions = {}): Promise<StudyAIResult> {
    const started = performance.now();
    const prompt = promptByAction[action];
    let built;
    try { built = this.contextBuilder.build(action, context); }
    catch (reason) { return logResult({ action, status: "INVALID_OUTPUT", content: null, quiz: null, cards: null, evaluation: null, model: this.settings.model, promptVersion: prompt.version, latencyMs: Math.round(performance.now() - started), contextTruncated: false, materialIds: [], error: reason instanceof Error ? reason.message : String(reason) }); }
    if (options.isCancelled?.()) return logResult(cancelled(action, prompt.version, built.contextTruncated, built.materialIds, Math.round(performance.now() - started)));
    const count = options.count ?? 5;
    if (structuredActions.has(action) && action !== "EVALUATE_ANSWER" && !allowedCounts.has(count)) {
      return logResult({ action, status: "INVALID_OUTPUT", content: null, quiz: null, cards: null, evaluation: null, model: this.settings.model, promptVersion: prompt.version, latencyMs: Math.round(performance.now() - started), contextTruncated: built.contextTruncated, materialIds: built.materialIds, error: "Use 3, 5 ou 10 itens." });
    }
    const instruction = options.userInstruction?.trim();
    const messages = [
      { role: "system" as const, content: prompt.prompt.trim() },
      { role: "user" as const, content: `${built.serialized}${action === "QUIZ" || action === "GENERATE_CARDS" ? `\n\nQUANTIDADE SOLICITADA: ${count}` : ""}${instruction ? `\nINSTRUÇÃO ADICIONAL DO USUÁRIO: ${instruction.slice(0, 1_000)}` : ""}` },
    ];
    const schema = schemaFor(action, count);
    const request: AIRequest = {
      model: this.settings.model,
      timeoutSeconds: this.settings.timeoutSeconds,
      generationProfile: schema ? "study-structured" : "study",
      structuredOutputSchema: schema,
      requestMetadata: { domain: "study", action, promptVersion: prompt.version, contextTruncated: built.contextTruncated, sourceMaterialCount: built.materialIds.length },
      messages,
    };
    try {
      let response = await this.provider.chat(request);
      if (options.isCancelled?.()) return logResult(cancelled(action, prompt.version, built.contextTruncated, built.materialIds, Math.round(performance.now() - started)));
      if (!schema) {
        const content = response.content.trim();
        return logResult({ action, status: content ? "SUCCESS" : "INVALID_OUTPUT", content: content || null, quiz: null, cards: null, evaluation: null, model: response.model, promptVersion: prompt.version, latencyMs: Math.round(performance.now() - started), contextTruncated: built.contextTruncated || response.truncated, materialIds: built.materialIds, error: content ? null : "O modelo retornou conteúdo vazio." });
      }
      let parsed = parseStructured(action, response.content, count);
      const valid = parsed && (parsed.quiz || parsed.cards || parsed.evaluation);
      if (!valid) {
        response = await this.provider.chat({ ...request, messages: [{ role: "system", content: prompt.prompt.trim() }, { role: "system", content: "A saída anterior foi inválida. Retorne somente JSON compatível com o schema, sem comentários." }, messages[1]] });
        if (options.isCancelled?.()) return logResult(cancelled(action, prompt.version, built.contextTruncated, built.materialIds, Math.round(performance.now() - started)));
        parsed = parseStructured(action, response.content, count);
      }
      if (!parsed || (!parsed.quiz && !parsed.cards && !parsed.evaluation)) return logResult({ action, status: "INVALID_OUTPUT", content: null, quiz: null, cards: null, evaluation: null, model: response.model, promptVersion: prompt.version, latencyMs: Math.round(performance.now() - started), contextTruncated: built.contextTruncated || response.truncated, materialIds: built.materialIds, error: "O modelo não retornou a estrutura esperada." });
      return logResult({ action, status: "SUCCESS", content: null, quiz: parsed.quiz, cards: parsed.cards, evaluation: parsed.evaluation, model: response.model, promptVersion: prompt.version, latencyMs: Math.round(performance.now() - started), contextTruncated: built.contextTruncated || response.truncated, materialIds: built.materialIds, error: null });
    } catch (reason) {
      const error = reason instanceof Error ? reason.message : String(reason);
      const status = /timeout|tempo limite|excedeu o tempo/i.test(error) ? "TIMEOUT" : "PROVIDER_ERROR";
      return logResult({ action, status, content: null, quiz: null, cards: null, evaluation: null, model: this.settings.model, promptVersion: prompt.version, latencyMs: Math.round(performance.now() - started), contextTruncated: built.contextTruncated, materialIds: built.materialIds, error });
    }
  }
}
