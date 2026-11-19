import systemPrompt from "../prompts/system.txt?raw";
import generalPrompt from "../prompts/general.txt?raw";
import agentPrompt from "../prompts/agent.txt?raw";
import { localDateKey } from "../services/dateService";
import type { AgentStep, AIRequest, AISettings, AIToolResult, AzrielState, ConfirmationRequest, Conversation, ConversationMessage, Task } from "../types";
import { runAgent } from "./agent/agentLoop";
import { abortable, InterruptedError, isInterrupted } from "./interrupt";
import { routeIntent } from "./router/toolRouter";
import { ContextBuilder } from "./context/ContextBuilder";
import type { AIProvider } from "./providers/AIProvider";

export interface ConversationGateway {
  create(title: string): Promise<Conversation>;
  messages(conversationId: string): Promise<ConversationMessage[]>;
  addMessage(input: { conversationId: string; role: "user" | "assistant"; content: string; steps?: AgentStep[] }): Promise<ConversationMessage>;
  saveTaskReferences(conversationId: string, sourceMessageId: string, taskIds: string[]): Promise<number>;
  deleteMessage(id: string): Promise<void>;
}

// O que cada ferramenta de escrita faz, para contar ao Senhor o que já tinha começado antes do ESC.
const ACTION_LABELS: Record<string, string> = {
  remember: "registrar memória", update_memory: "corrigir memória", create_task: "criar tarefa", update_task: "alterar tarefa", delete_task: "apagar tarefa",
  create_note: "criar nota", update_note: "alterar nota", archive_note: "arquivar nota", delete_note: "apagar nota",
  save_money_item: "salvar lançamento", set_money_paid: "marcar pagamento", delete_money_item: "apagar lançamento",
  play_music: "tocar música", music_control: "controlar a música",
  write_file: "gravar arquivo", edit_file: "editar arquivo", create_folder: "criar pasta", delete_path: "apagar arquivo ou pasta", run_command: "rodar comando",
};

/** Ações com efeito (não leituras) entre as ferramentas usadas, já sem repetição. */
export function writesIn(toolsUsed: string[], registry: { has(name: string): boolean; get(name: never): { readonly: boolean } }): string[] {
  const written = toolsUsed.filter((name) => registry.has(name) && !registry.get(name as never).readonly);
  return [...new Set(written.map((name) => ACTION_LABELS[name] ?? name))];
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
export const AGENT_HISTORY_CHARACTER_BUDGET = 6_000;
// Intenções que dependem de referências numeradas salvas pelo fluxo determinístico.
const DETERMINISTIC_INTENTS = ["critical_tasks", "priority_tasks", "complete_task_reference", "unsupported_internal"];
const toolsUnsupported = (error: unknown) => (error instanceof Error ? error.message : String(error)).includes("não suporta ferramentas");

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

/**
 * Junta todas as mensagens de sistema numa só, no topo. A API do Gemini compatível com OpenAI
 * aproveita apenas uma mensagem de sistema: com várias, a identidade era descartada.
 */
export function withSingleSystem<T extends { role: string; content: string }>(messages: T[]): T[] {
  const system = messages.filter((message) => message.role === "system");
  if (system.length <= 1 && (system.length === 0 || messages[0].role === "system")) return messages;
  const merged = { ...system[0], content: system.map((message) => message.content.trim()).filter(Boolean).join("\n\n") };
  return [merged, ...messages.filter((message) => message.role !== "system")];
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
    /** Identidade + memórias do Azriel; abre toda conversa. */
    private readonly persona: () => Promise<string> = async () => "",
  ) {}

  private async personaMessages(): Promise<Array<{ role: "system"; content: string }>> {
    const persona = (await this.persona().catch(() => "")).trim();
    return persona ? [{ role: "system", content: persona }] : [];
  }

  async send(query: string, currentConversation: Conversation | null, onPhase?: (state: AzrielState, detail?: string) => void, onMessage?: (message: ConversationMessage) => void, confirm?: (request: ConfirmationRequest) => Promise<boolean>, signal?: AbortSignal, onStep?: (step: AgentStep) => void): Promise<AIExchange> {
    const cleanQuery = query.trim();
    if (!cleanQuery) throw new Error("Digite uma pergunta para o Azriel.");
    if (cleanQuery.length > 4_000) throw new Error("A pergunta excede o limite de 4.000 caracteres.");
    const conversation = currentConversation ?? await this.conversations.create(cleanQuery.slice(0, 54));
    const userMessage = await this.conversations.addMessage({ conversationId: conversation.id, role: "user", content: cleanQuery });
    onMessage?.(userMessage);
    try {
      return await this.respond(cleanQuery, conversation, userMessage, onPhase, onMessage, confirm, signal, onStep);
    } catch (error) {
      if (!isInterrupted(error)) throw error;
      const acted = writesIn(error.toolsUsed, this.contextBuilder.registry);
      if (!acted.length) {
        // Nada foi feito: como se o pedido nunca tivesse existido; o texto volta para o campo.
        await this.conversations.deleteMessage(userMessage.id).catch(() => undefined);
        throw new InterruptedError(cleanQuery);
      }
      const note = await this.conversations.addMessage({ conversationId: conversation.id, role: "assistant", content: `⏹ **Interrompido pelo Senhor.** Antes de parar, eu já tinha começado a: ${acted.join(", ")}. Convém conferir o resultado antes de repetir o pedido.`, steps: error.steps });
      onMessage?.(note);
      throw new InterruptedError(null, error.toolsUsed);
    }
  }

  private async respond(cleanQuery: string, conversation: Conversation, userMessage: ConversationMessage, onPhase?: (state: AzrielState, detail?: string) => void, onMessage?: (message: ConversationMessage) => void, confirm?: (request: ConfirmationRequest) => Promise<boolean>, signal?: AbortSignal, onStep?: (step: AgentStep) => void): Promise<AIExchange> {
    const intent = routeIntent(cleanQuery);
    if (this.provider.chatWithTools && !DETERMINISTIC_INTENTS.includes(intent.intent)) {
      try {
        const agent = await this.answerWithAgent(cleanQuery, conversation.id, onPhase, confirm, signal, onStep);
        const assistantMessage = await this.conversations.addMessage({ conversationId: conversation.id, role: "assistant", content: agent.content, steps: agent.steps });
        onMessage?.(assistantMessage);
        onPhase?.("idle", "RESPOSTA CONCLUÍDA");
        return { conversation, userMessage, assistantMessage };
      } catch (error) {
        // Modelo sem tool calling: segue pelo roteador por palavras-chave.
        if (!toolsUnsupported(error)) throw error;
      }
    }
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
        ...await this.personaMessages(),
        { role: "system", content: systemPrompt },
        { role: "system", content: `DADOS ESTRUTURADOS DA CONSULTA ATUAL:\n${context.text}` },
      ];
    } else {
      systemMessages = [...await this.personaMessages(), { role: "system", content: generalPrompt }];
    }
    onPhase?.("processing", "FORMULANDO RESPOSTA");
    const history = selectConversationHistory(await this.conversations.messages(conversation.id), this.settings.contextMessageLimit)
      .map(({ role, content }) => ({ role, content }));
    const request: AIRequest = {
      model: this.settings.model,
      timeoutSeconds: this.settings.timeoutSeconds,
      messages: withSingleSystem([...systemMessages, ...history]),
    };
    let response = await abortable(this.provider.chat(request), signal);
    answer = response.content.trim();
    if (hasPathologicalRepetition(answer) || response.truncated) {
      const retryReason = response.truncated
        ? "A resposta anterior atingiu o limite. Responda novamente de forma completa e concisa, usando no máximo três parágrafos curtos."
        : "A resposta anterior entrou em repetição. Responda novamente de forma curta, factual e sem repetir frases ou termos.";
      onPhase?.("processing", response.truncated ? "REGENERANDO RESPOSTA COMPLETA" : "REGENERANDO RESPOSTA SEM REPETIÇÕES");
      response = await abortable(this.provider.chat({
        ...request,
        generationProfile: "repetition-retry",
        messages: withSingleSystem([
          ...systemMessages,
          { role: "system", content: retryReason },
          ...history,
        ]),
      }), signal);
      answer = response.content.trim();
      if (hasPathologicalRepetition(answer) || response.truncated) answer = repetitionFallback;
    }
    const assistantMessage = await this.conversations.addMessage({ conversationId: conversation.id, role: "assistant", content: answer });
    onMessage?.(assistantMessage);
    onPhase?.("idle", "RESPOSTA CONCLUÍDA");
    return { conversation, userMessage, assistantMessage };
  }

  private async answerWithAgent(query: string, conversationId: string, onPhase?: (state: AzrielState, detail?: string) => void, confirm?: (request: ConfirmationRequest) => Promise<boolean>, signal?: AbortSignal, onStep?: (step: AgentStep) => void): Promise<{ content: string; steps: AgentStep[] }> {
    onPhase?.("processing", "ANALISANDO SOLICITAÇÃO");
    const history = selectConversationHistory(await this.conversations.messages(conversationId), this.settings.contextMessageLimit, AGENT_HISTORY_CHARACTER_BUDGET)
      .map(({ role, content }) => ({ role, content }));
    const result = await runAgent({
      provider: this.provider,
      registry: this.contextBuilder.registry,
      model: this.settings.model,
      timeoutSeconds: this.settings.timeoutSeconds,
      messages: withSingleSystem([...await this.personaMessages(), { role: "system" as const, content: `${agentPrompt}\nData local atual: ${localDateKey()}.` }, ...history]),
      query,
      conversationId,
      onTool: (domain, permission) => onPhase?.(
        permission === "read" ? "tool" : "executing",
        permission === "confirm_write" ? "AGUARDANDO A PERMISSÃO DO SENHOR" : permission === "safe_write" ? `EXECUTANDO EM ${domain.toUpperCase()}` : `CONSULTANDO ${domain.toUpperCase()}`,
      ),
      confirm,
      signal,
      onStep,
    });
    if (!result.content || hasPathologicalRepetition(result.content)) return { content: repetitionFallback, steps: result.steps };
    return { content: result.content, steps: result.steps };
  }
}
