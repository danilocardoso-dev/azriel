import { createContext } from "react";
import type { AgentStep, AIEngine, AISettings, AISettingsInput, AzrielState, ConfirmationRequest, Conversation, ConversationMessage, LocalAiModelStatus, OllamaStatus, RemoteAISettings } from "../types";

export interface AIContextValue {
  settings: AISettings | null;
  remoteSettings: RemoteAISettings | null;
  remoteModels: string[];
  /** Motor ativo e se ele está pronto para responder. */
  engine: AIEngine;
  engineAvailable: boolean;
  setEngine: (engine: AIEngine) => Promise<void>;
  selectModel: (model: string) => Promise<void>;
  saveRemoteKey: (apiKey: string) => Promise<void>;
  removeRemoteKey: () => Promise<void>;
  refreshRemoteModels: () => Promise<string[]>;
  status: OllamaStatus | null;
  modelLifecycle: LocalAiModelStatus | null;
  modelTransition: "LOADING" | "UNLOADING" | null;
  conversations: Conversation[];
  selectedConversation: Conversation | null;
  messages: ConversationMessage[];
  coreState: AzrielState;
  phase: string;
  loading: boolean;
  sending: boolean;
  error: string | null;
  selectConversation: (conversation: Conversation) => Promise<void>;
  newConversation: () => void;
  deleteConversation: (conversation: Conversation) => Promise<void>;
  clearConversationHistory: (conversation: Conversation) => Promise<number>;
  reload: () => Promise<void>;
  /** Devolve o texto para o campo quando o Senhor interrompe antes de qualquer ação. */
  send: (query: string) => Promise<{ restoredText?: string | null }>;
  /** ESC/Parar. */
  stop: () => void;
  /** Passos do agente enquanto ele trabalha. */
  liveSteps: AgentStep[];
  /** Pedido de permissão do Azriel (exclusões) aguardando o Senhor no AI Core. */
  pendingConfirmation: ConfirmationRequest | null;
  answerConfirmation: (allowed: boolean) => void;
  updateSettings: (input: AISettingsInput) => Promise<void>;
  probeOllama: (endpoint: string, timeoutSeconds: number) => Promise<OllamaStatus>;
  refreshStatus: () => Promise<OllamaStatus | null>;
  refreshModelLifecycle: () => Promise<LocalAiModelStatus | null>;
  loadLocalModel: () => Promise<LocalAiModelStatus>;
  unloadLocalModel: () => Promise<LocalAiModelStatus>;
}

export const AIContext = createContext<AIContextValue | null>(null);

