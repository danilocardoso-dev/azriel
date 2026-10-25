import type { AgentMessage, AgentResponse, AIToolPermission } from "../../types";
import type { AIProvider } from "../providers/AIProvider";
import type { ToolRegistry } from "../tools/toolRegistry";

export const AGENT_MAX_STEPS = 4;
export const AGENT_MAX_CALLS_PER_STEP = 4;
export const AGENT_TOOL_RESULT_CHARACTERS = 6_000;

export interface AgentRunInput {
  provider: AIProvider;
  registry: ToolRegistry;
  model: string;
  timeoutSeconds: number;
  messages: AgentMessage[];
  query: string;
  conversationId?: string;
  onTool?: (domain: string, permission: AIToolPermission) => void;
}

export interface AgentRunResult { content: string; truncated: boolean; toolsUsed: string[] }

const serialize = (value: unknown) => {
  const text = JSON.stringify(value ?? null);
  return text.length > AGENT_TOOL_RESULT_CHARACTERS ? `${text.slice(0, AGENT_TOOL_RESULT_CHARACTERS)} [RESULTADO LIMITADO PELO SISTEMA]` : text;
};

/**
 * Loop de agente: o modelo escolhe as ferramentas, o Azriel executa e devolve o resultado.
 * Só ferramentas somente leitura são executadas; a última rodada vai sem ferramentas
 * para obrigar uma resposta final.
 */
export async function runAgent(input: AgentRunInput): Promise<AgentRunResult> {
  if (!input.provider.chatWithTools) throw new Error("O provider atual não suporta ferramentas.");
  const tools = input.registry.agentTools();
  const messages = [...input.messages];
  const toolsUsed: string[] = [];
  let response: AgentResponse | null = null;
  for (let step = 0; step < AGENT_MAX_STEPS; step += 1) {
    const last = step === AGENT_MAX_STEPS - 1;
    response = await input.provider.chatWithTools({ model: input.model, timeoutSeconds: input.timeoutSeconds, messages, tools: last ? [] : tools });
    if (!response.toolCalls.length) break;
    messages.push({ role: "assistant", content: response.content, tool_calls: response.toolCalls });
    for (const call of response.toolCalls.slice(0, AGENT_MAX_CALLS_PER_STEP)) {
      const name = call.function.name;
      if (!input.registry.has(name) || !input.registry.get(name).readonly) {
        messages.push({ role: "tool", tool_name: name, content: serialize({ error: `Ferramenta indisponível: ${name}` }) });
        continue;
      }
      const tool = input.registry.get(name);
      input.onTool?.(tool.domain, tool.permission ?? "read");
      try {
        const result = await input.registry.execute(name, { query: input.query, conversationId: input.conversationId });
        toolsUsed.push(name);
        messages.push({ role: "tool", tool_name: name, content: serialize(result.empty ? { empty: true, data: result.data } : result.data) });
      } catch (error) {
        messages.push({ role: "tool", tool_name: name, content: serialize({ error: error instanceof Error ? error.message : String(error) }) });
      }
    }
  }
  return { content: response?.content.trim() ?? "", truncated: response?.truncated ?? false, toolsUsed };
}
