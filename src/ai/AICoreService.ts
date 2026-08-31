import systemPrompt from "../prompts/system.txt?raw";
import generalPrompt from "../prompts/general.txt?raw";
import type { AIRequest, AISettings, AzrielState, Conversation, ConversationMessage } from "../types";
import { routeIntent } from "./router/toolRouter";
import { ContextBuilder } from "./context/ContextBuilder";
import type { AIProvider } from "./providers/AIProvider";

export interface ConversationGateway {
  create(title: string): Promise<Conversation>;
  messages(conversationId: string): Promise<ConversationMessage[]>;
  addMessage(input: { conversationId: string; role: "user" | "assistant"; content: string }): Promise<ConversationMessage>;
}

export interface AIExchange {
  conversation: Conversation;
  userMessage: ConversationMessage;
  assistantMessage: ConversationMessage;
}

const repetitionFallback = "Não consegui formular uma resposta confiável sem repetição. Tente reformular a pergunta ou selecione um modelo maior nas Configurações do AI Core.";

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

function usefulHistory(messages: ConversationMessage[], limit: number) {
  return messages
    .filter((message) => message.role !== "assistant" || !hasPathologicalRepetition(message.content))
    .slice(-limit)
    .map(({ role, content }) => ({ role, content }));
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
      const context = await this.contextBuilder.build(cleanQuery, intent, (domain) => onPhase?.("tool", `CONSULTANDO ${domain.toUpperCase()}`));
      if (context.empty) {
        answer = "Não encontrei essa informação registrada no Azriel.";
        const assistantMessage = await this.conversations.addMessage({ conversationId: conversation.id, role: "assistant", content: answer });
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
    const history = usefulHistory(await this.conversations.messages(conversation.id), this.settings.contextMessageLimit);
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
