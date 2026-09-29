import systemPrompt from "../prompts/system.txt?raw";
import generalPrompt from "../prompts/general.txt?raw";
import type { AIRequest, AISettings, AIToolResult, AzrielState, Conversation, ConversationMessage, Task } from "../types";
import { routeIntent } from "./router/toolRouter";
import { ContextBuilder } from "./context/ContextBuilder";
import type { AIProvider } from "./providers/AIProvider";

export interface ConversationGateway {
  create(title: string): Promise<Conversation>;
  messages(conversationId: string): Promise<ConversationMessage[]>;
  addMessage(input: { conversationId: string; role: "user" | "assistant"; content: string }): Promise<ConversationMessage>;
  saveTaskReferences(conversationId: string, sourceMessageId: string, taskIds: string[]): Promise<number>;
}

const taskFromToolData = (value: unknown): value is Task => {
  if (!value || typeof value !== "object") return false;
  const task = value as Partial<Task>;
  return typeof task.id === "string" && typeof task.title === "string" && typeof task.priority === "string" && typeof task.status === "string";
};

export function formatOperationalAnswer(intent: string, results: AIToolResult[]): string | null {
  if (intent === "complete_task_reference") {
    const result = results[0]?.data;
    if (!result || typeof result !== "object") return "Não consegui concluir a demanda com segurança.";
    const operation = result as { success?: boolean; task?: unknown; error?: unknown };
    if (!operation.success || !taskFromToolData(operation.task)) return typeof operation.error === "string" ? operation.error : "Não consegui concluir a demanda com segurança.";
    return `Demanda concluída:\n\n${operation.task.title}`;
  }
  if (!["critical_tasks", "priority_tasks"].includes(intent)) return null;
  const tasks = results.flatMap((result) => Array.isArray(result.data) ? result.data.filter(taskFromToolData) : []);
  if (!tasks.length) return null;
  const subject = intent === "critical_tasks"
    ? (tasks.length === 1 ? "demanda crítica" : "demandas críticas")
    : (tasks.length === 1 ? "demanda prioritária" : "demandas prioritárias");
  const rows = tasks.map((task, index) => {
    const priority = task.priority === "critical" ? "CRÍTICA" : "ALTA";
    return `${index + 1}. ${task.title} — ${priority} · ${task.dueDate ? `prazo: ${task.dueDate}` : "sem prazo"}`;
  });
  return `Você possui ${tasks.length} ${subject} ${tasks.length === 1 ? "ativa" : "ativas"}:\n\n${rows.join("\n")}`;
}

export interface AIExchange {
  conversation: Conversation;
  userMessage: ConversationMessage;
  assistantMessage: ConversationMessage;
}

const repetitionFallback = "Não consegui formular uma resposta confiável sem repetição. Tente reformular a pergunta ou selecione um modelo maior nas Configurações do AI Core.";
export const AI_HISTORY_CHARACTER_BUDGET = 1_600;

export function hasPathologicalRepetition(value: string): boolean {
  const words = value.toLocaleLowerCase("pt-BR").match(/[\p{L}\p{N}]+/gu) ?? [];
  if (words.length < 40) return false;
  const trigrams = new Map<string, number>();
  for (let index = 0; index <= words.length - 3; index += 1) {
    const key = words.slice(index, index + 3).join(" ");
    const count = (trigrams.get(key) ?? 0) + 1;
    if (count >= 4) return true;
    trigrams.set(key, count);
  }
  return words.length >= 60 && new Set(words).size / words.length < 0.28;
}

export function selectConversationHistory(messages: ConversationMessage[], limit: number, characterBudget = AI_HISTORY_CHARACTER_BUDGET) {
  const eligible = messages
    .filter((message) => message.role !== "assistant" || !hasPathologicalRepetition(message.content))
    .filter((message, index, collection) => {
      const previous = collection[index - 1];
      return !previous || previous.role !== message.role || previous.content.trim() !== message.content.trim();
    })
    .slice(-Math.max(1, limit));
  if (eligible.length === 0) return [];

  const selected: ConversationMessage[] = [];
  let usedCharacters = 0;
  for (let index = eligible.length - 1; index >= 0; index -= 1) {
    const message = eligible[index];
    const size = message.content.length;
    if (selected.length > 0 && usedCharacters + size > characterBudget) break;
    selected.unshift(message);
    usedCharacters += size;
  }
  return selected;
}

export class AICoreService {
  constructor(
    private readonly provider: AIProvider,
    private readonly contextBuilder: ContextBuilder,
    private readonly conversations: ConversationGateway,
    private readonly settings: AISettings,
  ) {}

  async send(query: string, currentConversation: Conversation | null, onPhase?: (state: AzrielState, detail?: string) => void, onMessage?: (message: ConversationMessage) => void): Promise<AIExchange> {
    const cleanQuery = query.trim();
    if (!cleanQuery) throw new Error("Digite uma pergunta para o Azriel.");
    if (cleanQuery.length > 4_000) throw new Error("A pergunta excede o limite de 4.000 caracteres.");
    const conversation = currentConversation ?? await this.conversations.create(cleanQuery.slice(0, 54));
    const userMessage = await this.conversations.addMessage({ conversationId: conversation.id, role: "user", content: cleanQuery });
    onMessage?.(userMessage);
    const intent = routeIntent(cleanQuery);
    let answer: string;
    let systemMessages: Array<{ role: "system"; content: string }>;
    if (intent.scope === "azriel") {
      onPhase?.("tool", "CONSULTANDO NÚCLEOS DO AZRIEL");
      const context = await this.contextBuilder.build(cleanQuery, intent, (domain, permission) => onPhase?.(
        permission === "confirm_write" ? "routine" : permission === "safe_write" ? "executing" : permission === "visual_action" ? "engineering" : "tool",
        permission === "confirm_write" ? "VALIDANDO ROTINA / CONFIRMAÇÃO NECESSÁRIA" : permission === "safe_write" ? "EXECUTANDO AÇÃO AUTORIZADA" : permission === "visual_action" ? "MANIPULANDO MODELO" : `CONSULTANDO ${domain.toUpperCase()}`,
      ), conversation.id);
      if (context.empty) {
        answer = intent.intent === "unsupported_internal"
          ? "Reconheci uma solicitação relacionada aos seus dados internos, mas não encontrei uma referência segura para executá-la. Liste as demandas e use o número apresentado."
          : "Não encontrei essa informação registrada no Azriel.";
        const assistantMessage = await this.conversations.addMessage({ conversationId: conversation.id, role: "assistant", content: answer });
        onMessage?.(assistantMessage);
        onPhase?.("idle", "RESPOSTA CONCLUÍDA");
        return { conversation, userMessage, assistantMessage };
      }
      const operationalAnswer = formatOperationalAnswer(intent.intent, context.results);
      if (operationalAnswer) {
        const assistantMessage = await this.conversations.addMessage({ conversationId: conversation.id, role: "assistant", content: operationalAnswer });
        if (["critical_tasks", "priority_tasks"].includes(intent.intent)) {
          const taskIds = context.results.flatMap((result) => Array.isArray(result.data) ? result.data.filter(taskFromToolData).map((task) => task.id) : []);
          await this.conversations.saveTaskReferences(conversation.id, assistantMessage.id, taskIds);
        }
        onMessage?.(assistantMessage);
        onPhase?.("idle", "RESPOSTA CONCLUÍDA");
        return { conversation, userMessage, assistantMessage };
      }
      systemMessages = [
        { role: "system", content: systemPrompt },
        { role: "system", content: `DADOS ESTRUTURADOS DA CONSULTA ATUAL:\n${context.text}` },
      ];
    } else {
      systemMessages = [{ role: "system", content: generalPrompt }];
    }
    onPhase?.("processing", "FORMULANDO RESPOSTA");
    const history = selectConversationHistory(await this.conversations.messages(conversation.id), this.settings.contextMessageLimit)
      .map(({ role, content }) => ({ role, content }));
    const request: AIRequest = {
      model: this.settings.model,
      timeoutSeconds: this.settings.timeoutSeconds,
      messages: [...systemMessages, ...history],
    };
    let response = await this.provider.chat(request);
    answer = response.content.trim();
    if (hasPathologicalRepetition(answer) || response.truncated) {
      const retryReason = response.truncated
        ? "A resposta anterior atingiu o limite. Responda novamente de forma completa e concisa, usando no máximo três parágrafos curtos."
        : "A resposta anterior entrou em repetição. Responda novamente de forma curta, factual e sem repetir frases ou termos.";
      onPhase?.("processing", response.truncated ? "REGENERANDO RESPOSTA COMPLETA" : "REGENERANDO RESPOSTA SEM REPETIÇÕES");
      response = await this.provider.chat({
        ...request,
        generationProfile: "repetition-retry",
        messages: [
          ...systemMessages,
          { role: "system", content: retryReason },
          ...history,
        ],
      });
      answer = response.content.trim();
      if (hasPathologicalRepetition(answer) || response.truncated) answer = repetitionFallback;
    }
    const assistantMessage = await this.conversations.addMessage({ conversationId: conversation.id, role: "assistant", content: answer });
    onMessage?.(assistantMessage);
    onPhase?.("idle", "RESPOSTA CONCLUÍDA");
    return { conversation, userMessage, assistantMessage };
  }
}
