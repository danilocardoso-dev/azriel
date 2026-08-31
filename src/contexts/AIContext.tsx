import { useCallback, useEffect, useMemo, useState, type ReactNode } from "react";
import { AICoreService } from "../ai/AICoreService";
import { ContextBuilder } from "../ai/context/ContextBuilder";
import { OllamaProvider } from "../ai/providers/OllamaProvider";
import { ToolRegistry } from "../ai/tools/toolRegistry";
import { aiRepository } from "../repositories/aiRepository";
import { conversationService } from "../services/conversationService";
import type { AISettings, AISettingsInput, AzrielState, Conversation, ConversationMessage, OllamaStatus } from "../types";
import { AIContext, type AIContextValue } from "./ai-context";

const messageOf = (error: unknown) => error instanceof Error ? error.message : String(error);

export function AIProvider({ children }: { children: ReactNode }) {
  const [settings, setSettings] = useState<AISettings | null>(null);
  const [status, setStatus] = useState<OllamaStatus | null>(null);
  const [conversations, setConversations] = useState<Conversation[]>([]);
  const [selectedConversation, setSelectedConversation] = useState<Conversation | null>(null);
  const [messages, setMessages] = useState<ConversationMessage[]>([]);
  const [coreState, setCoreState] = useState<AzrielState>("offline");
  const [phase, setPhase] = useState("INICIALIZANDO AI CORE");
  const [loading, setLoading] = useState(true);
  const [sending, setSending] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const checkStatus = useCallback(async (target: AISettings): Promise<OllamaStatus> => {
    const next = await aiRepository.status(target.endpoint, Math.min(target.timeoutSeconds, 8));
    setStatus(next); setCoreState(next.available ? "idle" : "offline");
    setPhase(next.available ? "OLLAMA LOCAL DISPONÍVEL" : "AI CORE OFFLINE");
    return next;
  }, []);

  const initialize = useCallback(async () => {
    setLoading(true); setError(null);
    try {
      const [nextSettings, nextConversations] = await Promise.all([aiRepository.getSettings(), conversationService.list()]);
      setSettings(nextSettings); setConversations(nextConversations);
      await checkStatus(nextSettings);
    } catch (reason) { setError(messageOf(reason)); setCoreState("offline"); }
    finally { setLoading(false); }
  }, [checkStatus]);

  useEffect(() => { const task = window.setTimeout(() => void initialize(), 0); return () => window.clearTimeout(task); }, [initialize]);

  const selectConversation = useCallback(async (conversation: Conversation) => {
    setSelectedConversation(conversation); setError(null); setLoading(true);
    try { setMessages(await conversationService.messages(conversation.id)); }
    catch (reason) { setError(messageOf(reason)); }
    finally { setLoading(false); }
  }, []);

  const newConversation = useCallback(() => { setSelectedConversation(null); setMessages([]); setError(null); }, []);

  const deleteConversation = useCallback(async (conversation: Conversation) => {
    setError(null);
    try {
      await conversationService.remove(conversation.id);
      setConversations((current) => current.filter((item) => item.id !== conversation.id));
      if (selectedConversation?.id === conversation.id) {
        setSelectedConversation(null);
        setMessages([]);
      }
    } catch (reason) {
      setError(messageOf(reason));
      throw reason;
    }
  }, [selectedConversation]);

  const refreshStatus = useCallback(async () => {
    if (!settings) return null;
    try { setError(null); return await checkStatus(settings); }
    catch (reason) { setError(messageOf(reason)); setCoreState("offline"); return null; }
  }, [checkStatus, settings]);

  const updateSettings = useCallback(async (input: AISettingsInput) => {
    setError(null);
    try { const updated = await aiRepository.updateSettings(input); setSettings(updated); await checkStatus(updated); }
    catch (reason) { setError(messageOf(reason)); throw reason; }
  }, [checkStatus]);

  const send = useCallback(async (query: string) => {
    if (!settings) throw new Error("Configurações do AI Core ainda não foram carregadas.");
    if (!status?.available) throw new Error(status?.error || "AI Core offline. Verifique o Ollama nas Configurações.");
    setSending(true); setError(null);
    let conversation = selectedConversation;
    try {
      if (!conversation) {
        conversation = await conversationService.create(query);
        setSelectedConversation(conversation);
        setConversations((current) => [conversation!, ...current]);
      }
      const service = new AICoreService(new OllamaProvider(settings), new ContextBuilder(new ToolRegistry()), conversationService, settings);
      await service.send(query, conversation, (state, detail) => { setCoreState(state); if (detail) setPhase(detail); }, (message) => setMessages((current) => [...current, message]));
      setConversations(await conversationService.list());
    } catch (reason) {
      const detail = messageOf(reason); setError(detail); setCoreState(detail.toLowerCase().includes("ollama") ? "offline" : "alert"); setPhase("FALHA NO AI CORE");
      if (conversation) setMessages(await conversationService.messages(conversation.id).catch(() => []));
      throw reason;
    } finally { setSending(false); }
  }, [selectedConversation, settings, status]);

  const value = useMemo<AIContextValue>(() => ({ settings, status, conversations, selectedConversation, messages, coreState, phase, loading, sending, error, selectConversation, newConversation, deleteConversation, send, updateSettings, refreshStatus }), [settings, status, conversations, selectedConversation, messages, coreState, phase, loading, sending, error, selectConversation, newConversation, deleteConversation, send, updateSettings, refreshStatus]);
  return <AIContext.Provider value={value}>{children}</AIContext.Provider>;
}
