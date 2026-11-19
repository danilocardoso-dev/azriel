import type { AgentMessage, AgentResponse, AgentStep, AIToolPermission, ConfirmationRequest } from "../../types";
import type { AIProvider } from "../providers/AIProvider";
import type { ToolRegistry } from "../tools/toolRegistry";
import { abortable, InterruptedError, isInterrupted, throwIfAborted } from "../interrupt";
import { finishedStep, startedStep } from "./steps";

// Programar pede várias rodadas (ler, editar, testar, corrigir).
export const AGENT_MAX_STEPS = 12;
export const AGENT_MAX_CALLS_PER_STEP = 6;
export const AGENT_TOOL_RESULT_CHARACTERS = 20_000;

export interface AgentRunInput {
  provider: AIProvider;
  registry: ToolRegistry;
  model: string;
  timeoutSeconds: number;
  messages: AgentMessage[];
  query: string;
  conversationId?: string;
  onTool?: (domain: string, permission: AIToolPermission) => void;
  /** Pergunta ao Senhor antes de excluir; sem ela (voz), exclusões são recusadas. */
  confirm?: (request: ConfirmationRequest) => Promise<boolean>;
  /** ESC/Parar: interrompe entre rodadas, entre ferramentas e durante a espera pelo modelo. */
  signal?: AbortSignal;
  /** Cada passo ao começar e ao terminar (mesmo id), para a tela mostrar ao vivo. */
  onStep?: (step: AgentStep) => void;
}

export interface AgentRunResult { content: string; truncated: boolean; toolsUsed: string[]; steps: AgentStep[] }

const serialize = (value: unknown) => {
  const text = JSON.stringify(value ?? null);
  return text.length > AGENT_TOOL_RESULT_CHARACTERS ? `${text.slice(0, AGENT_TOOL_RESULT_CHARACTERS)} [RESULTADO LIMITADO PELO SISTEMA]` : text;
};

/**
 * Loop de agente: o modelo escolhe as ferramentas, o Azriel executa e devolve o resultado.
 * Só leituras e escritas autônomas (memória) são executadas; a última rodada vai sem ferramentas
 * para obrigar uma resposta final.
 */
export async function runAgent(input: AgentRunInput): Promise<AgentRunResult> {
  if (!input.provider.chatWithTools) throw new Error("O provider atual não suporta ferramentas.");
  const tools = input.registry.agentTools();
  const messages = [...input.messages];
  const toolsUsed: string[] = [];
  const steps: AgentStep[] = [];
  const report = (step: AgentStep) => {
    const index = steps.findIndex((item) => item.id === step.id);
    if (index >= 0) steps[index] = step; else steps.push(step);
    input.onStep?.(step);
  };
  let response: AgentResponse | null = null;
  try {
  for (let step = 0; step < AGENT_MAX_STEPS; step += 1) {
    const last = step === AGENT_MAX_STEPS - 1;
    throwIfAborted(input.signal, toolsUsed);
    response = await abortable(input.provider.chatWithTools({ model: input.model, timeoutSeconds: input.timeoutSeconds, messages, tools: last ? [] : tools }), input.signal, toolsUsed);
    if (!response.toolCalls.length) break;
    messages.push({ role: "assistant", content: response.content, tool_calls: response.toolCalls });
    for (const call of response.toolCalls.slice(0, AGENT_MAX_CALLS_PER_STEP)) {
      const name = call.function.name;
      if (!input.registry.agentCanRun(name)) {
        messages.push({ role: "tool", tool_name: name, content: serialize({ error: `Ferramenta indisponível: ${name}` }) });
        continue;
      }
      const tool = input.registry.get(name);
      throwIfAborted(input.signal, toolsUsed);
      input.onTool?.(tool.domain, tool.permission ?? "read");
      // Conta como usada assim que começa: se o Senhor parar no meio, a ação pode já ter acontecido.
      toolsUsed.push(name);
      const id = steps.length + 1;
      report(startedStep(id, name, tool.domain, call.function.arguments));
      try {
        const result = await abortable(input.registry.execute(name, { query: input.query, conversationId: input.conversationId, args: call.function.arguments, confirm: input.confirm }), input.signal, toolsUsed);
        report(finishedStep(id, name, tool.domain, call.function.arguments, result.data));
        messages.push({ role: "tool", tool_name: name, content: serialize(result.empty ? { empty: true, data: result.data } : result.data) });
      } catch (error) {
        if (isInterrupted(error)) throw error;
        const message = error instanceof Error ? error.message : String(error);
        report(finishedStep(id, name, tool.domain, call.function.arguments, null, message));
        messages.push({ role: "tool", tool_name: name, content: serialize({ error: message }) });
      }
    }
  }
  } catch (error) {
    if (!isInterrupted(error)) throw error;
    // O passo que estava rodando fica marcado como interrompido; os passos seguem junto do erro.
    for (const step of steps.filter((item) => item.status === "running")) report({ ...step, label: "Interrompido", status: "error" });
    throw new InterruptedError(null, toolsUsed, steps);
  }
  return { content: response?.content.trim() ?? "", truncated: response?.truncated ?? false, toolsUsed, steps };
}
