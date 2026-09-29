import type { StudyAIAction, StudyAIBuiltContext, StudyAIContext } from "../../types";

const TRUNCATION_MARKER = "\n[CONTEXTO TRUNCADO PELO STUDY AI]";
const DEFAULT_CONTEXT_LIMIT = 12_000;

type ContextSource = StudyAIBuiltContext["source"];
type Payload = {
  contextVersion: "STUDY_AI_CONTEXT_V2";
  action: StudyAIAction;
  source: ContextSource;
  selectedText: string | null;
  roadmap: { id: string | null; name: string | null } | null;
  stage: { id: string | null; name: string | null } | null;
  topic: { id: string | null; name: string | null } | null;
  activity: { id: string | null; name: string | null; description: string | null; activityType: string | null } | null;
  note: { id: string; title: string; content: string | null; notebookId: string | null; notebookTitle: string | null } | null;
  studySession: { id: string; status: string; plannedFocusMinutes: number | null } | null;
  relatedCard: { id: string | null; front: string; back: string } | null;
  review: { question: string; expectedAnswer: string; userAnswer: string | null } | null;
  sourceMaterials: Array<{ id: string; title: string; materialType: string; selectedText: string; pageRange: { from: number; to: number } | null }>;
};

const clean = (value?: string | null) => value?.trim() || null;

function sourceOf(context: StudyAIContext): ContextSource {
  if (clean(context.selectedText)) return "SELECTED_TEXT";
  if (context.materials?.some((material) => clean(material.selectedText))) return "MATERIAL";
  if (context.review && clean(context.review.question) && clean(context.review.expectedAnswer)) return "REVIEW";
  if (context.note && clean(context.note.content)) return "NOTE";
  if (context.activity && (clean(context.activity.description) || clean(context.activity.name))) return "ACTIVITY";
  if (context.topic && clean(context.topic.name)) return "TOPIC";
  if (context.relatedCard && clean(context.relatedCard.front) && clean(context.relatedCard.back)) return "CARD";
  throw new Error("Não há contexto de estudo suficiente para esta ação.");
}

function payloadOf(action: StudyAIAction, context: StudyAIContext, source: ContextSource): Payload {
  const selectedText = clean(context.selectedText);
  return {
    contextVersion: "STUDY_AI_CONTEXT_V2",
    action,
    source,
    selectedText,
    roadmap: context.roadmap ? { id: context.roadmap.id, name: clean(context.roadmap.name) } : null,
    stage: context.stage ? { id: context.stage.id, name: clean(context.stage.name) } : null,
    topic: context.topic ? { id: context.topic.id, name: clean(context.topic.name) } : null,
    activity: context.activity ? {
      id: context.activity.id,
      name: clean(context.activity.name),
      description: source === "SELECTED_TEXT" ? null : clean(context.activity.description),
      activityType: clean(context.activity.activityType),
    } : null,
    note: context.note ? {
      id: context.note.id,
      title: context.note.title.trim(),
      content: source === "NOTE" ? clean(context.note.content) : null,
      notebookId: context.note.notebookId,
      notebookTitle: clean(context.note.notebookTitle),
    } : null,
    studySession: context.studySession ? {
      id: context.studySession.id,
      status: context.studySession.status,
      plannedFocusMinutes: context.studySession.plannedFocusMinutes,
    } : null,
    relatedCard: context.relatedCard ? {
      id: context.relatedCard.id,
      front: context.relatedCard.front.trim(),
      back: context.relatedCard.back.trim(),
    } : null,
    review: context.review ? {
      question: context.review.question.trim(),
      expectedAnswer: context.review.expectedAnswer.trim(),
      userAnswer: clean(context.review.userAnswer),
    } : null,
    sourceMaterials: (context.materials ?? []).map((material) => ({
      id: material.id,
      title: material.title.trim(),
      materialType: material.materialType,
      selectedText: material.selectedText.trim(),
      pageRange: material.pageRange,
    })),
  };
}

function truncatePayload(payload: Payload, maximumCharacters: number): { serialized: string; truncated: boolean } {
  let serialized = JSON.stringify(payload, null, 2);
  let truncated = false;
  const fields: Array<{ get: () => string | null; set: (value: string) => void }> = [
    ...payload.sourceMaterials.map((material) => ({ get: () => material.selectedText, set: (value: string) => { material.selectedText = value; } })),
    { get: () => payload.note?.content ?? null, set: (value) => { if (payload.note) payload.note.content = value; } },
    { get: () => payload.activity?.description ?? null, set: (value) => { if (payload.activity) payload.activity.description = value; } },
    { get: () => payload.review?.userAnswer ?? null, set: (value) => { if (payload.review) payload.review.userAnswer = value; } },
    { get: () => payload.relatedCard?.back ?? null, set: (value) => { if (payload.relatedCard) payload.relatedCard.back = value; } },
    { get: () => payload.review?.expectedAnswer ?? null, set: (value) => { if (payload.review) payload.review.expectedAnswer = value; } },
    { get: () => payload.selectedText, set: (value) => { payload.selectedText = value; } },
  ];
  while (serialized.length > maximumCharacters) {
    const candidate = fields
      .map((field) => ({ field, value: field.get() }))
      .filter((item): item is { field: (typeof fields)[number]; value: string } => item.value !== null && item.value.length > 180)
      .sort((left, right) => right.value.length - left.value.length)[0];
    if (!candidate) throw new Error(`O contexto excede o limite seguro de ${maximumCharacters.toLocaleString("pt-BR")} caracteres.`);
    const excess = serialized.length - maximumCharacters;
    const nextLength = Math.max(140, candidate.value.length - excess - TRUNCATION_MARKER.length - 16);
    candidate.field.set(`${candidate.value.slice(0, nextLength).trimEnd()}${TRUNCATION_MARKER}`);
    truncated = true;
    serialized = JSON.stringify(payload, null, 2);
  }
  return { serialized, truncated };
}

export class StudyAIContextBuilder {
  constructor(private readonly maximumCharacters = DEFAULT_CONTEXT_LIMIT) {
    if (maximumCharacters < 1_000 || maximumCharacters > 30_000) throw new Error("Limite de contexto inválido para o Study AI.");
  }

  build(action: StudyAIAction, context: StudyAIContext): StudyAIBuiltContext {
    const materials = context.materials ?? [];
    if (materials.length > 5) throw new Error("Selecione no máximo 5 materiais por ação do Study AI.");
    if (materials.some((material) => !material.id.trim() || !clean(material.selectedText))) {
      throw new Error("Cada material usado pela IA precisa de um trecho selecionado explicitamente.");
    }
    const source = sourceOf(context);
    if (action === "EVALUATE_ANSWER" && (!context.review || !clean(context.review.userAnswer))) {
      throw new Error("A avaliação exige pergunta, resposta esperada e resposta do usuário.");
    }
    const { serialized, truncated } = truncatePayload(payloadOf(action, context, source), this.maximumCharacters);
    return { serialized, contextTruncated: truncated, source, materialIds: [...new Set(materials.map((material) => material.id))] };
  }
}
