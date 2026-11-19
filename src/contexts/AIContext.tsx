import { useCallback, useEffect, useMemo, useRef, useState, type ReactNode } from "react";
import { AICoreService } from "../ai/AICoreService";
import { ContextBuilder } from "../ai/context/ContextBuilder";
import { isInterrupted } from "../ai/interrupt";
import { loadPersona } from "../ai/persona";
import { GeminiProvider } from "../ai/providers/GeminiProvider";
import { OllamaProvider } from "../ai/providers/OllamaProvider";
import { OpenRouterProvider } from "../ai/providers/OpenRouterProvider";
import { ToolRegistry } from "../ai/tools/toolRegistry";
import { routeIntent } from "../ai/router/toolRouter";
import { preferredGeminiModel } from "../components/ai/aiSettingsFlow";
import { aiRepository } from "../repositories/aiRepository";
import { workspaceRepository } from "../repositories/workspaceRepository";
import { conversationService } from "../services/conversationService";
import type { AgentStep, AIEngine, AISettings, AISettingsInput, AzrielState, ConfirmationRequest, Conversation, ConversationMessage, LocalAiModelStatus, OllamaStatus, OpenRouterSettings, OpenRouterSettingsInput, RemoteAISettings } from "../types";
import { AIContext, type AIContextValue } from "./ai-context";

const messageOf = (error: unknown) => error instanceof Error ? error.message : String(error);
// Sem resposta do Senhor em 5 minutos, a exclusão é negada (o Azriel não fica esperando para sempre).
const CONFIRMATION_TIMEOUT_MS = 5 * 60_000;

export function AIProvider({ children }: { children: ReactNode }) {
  const [settings, setSettings] = useState<AISettings | null>(null);
  const [remoteSettings, setRemoteSettings] = useState<RemoteAISettings | null>(null);
  const [remoteModels, setRemoteModels] = useState<string[]>([]);
  const [openRouter, setOpenRouter] = useState<OpenRouterSettings | null>(null);
  const [status, setStatus] = useState<OllamaStatus | null>(null);
  const [modelLifecycle, setModelLifecycle] = useState<LocalAiModelStatus | null>(null);
  const [modelTransition, setModelTransition] = useState<"LOADING" | "UNLOADING" | null>(null);
  const [conversations, setConversations] = useState<Conversation[]>([]);
  const [selectedConversation, setSelectedConversation] = useState<Conversation | null>(null);
  const [messages, setMessages] = useState<ConversationMessage[]>([]);
  const [coreState, setCoreState] = useState<AzrielState>("offline");
  const [phase, setPhase] = useState("INICIALIZANDO AI CORE");
  const [loading, setLoading] = useState(true);
  const [sending, setSending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [pendingConfirmation, setPendingConfirmation] = useState<ConfirmationRequest | null>(null);
  // Passos do agente ao vivo (o que ele está fazendo agora), como as linhas de ferramenta do Claude.
  const [liveSteps, setLiveSteps] = useState<AgentStep[]>([]);
  const confirmationResolver = useRef<((allowed: boolean) => void) | null>(null);
  const abortRef = useRef<AbortController | null>(null);
  const answerConfirmation = useCallback((allowed: boolean) => {
    const resolve = confirmationResolver.current;
    confirmationResolver.current = null;
    setPendingConfirmation(null);
    resolve?.(allowed);
  }, []);
  const requestConfirmation = useCallback((request: ConfirmationRequest) => new Promise<boolean>((resolve) => {
    confirmationResolver.current?.(false);
    const timer = window.setTimeout(() => answerConfirmation(false), CONFIRMATION_TIMEOUT_MS);
    confirmationResolver.current = (allowed) => { window.clearTimeout(timer); resolve(allowed); };
    setPendingConfirmation(request);
  }), [answerConfirmation]);

  const checkStatus = useCallback(async (target: AISettings): Promise<OllamaStatus> => {
    const next = await aiRepository.status(target.endpoint, Math.min(target.timeoutSeconds, 8));
    setStatus(next); setCoreState(next.available ? "idle" : "offline");
    setPhase(next.available ? "OLLAMA LOCAL DISPONÍVEL" : "AI CORE OFFLINE");
    return next;
  }, []);

  const initialize = useCallback(async () => {
    setLoading(true); setError(null);
    try {
      const nextSettings = await aiRepository.getSettings();
      setSettings(nextSettings);
      const nextRemote = await aiRepository.getRemoteSettings().catch(() => null);
      setRemoteSettings(nextRemote);
      setOpenRouter(await aiRepository.getOpenRouterSettings().catch(() => null));
      if (nextRemote?.apiKeyConfigured) void aiRepository.listRemoteModels().then(setRemoteModels).catch(() => setRemoteModels([]));
      try {
        setConversations(await conversationService.list());
      } catch (reason) {
        setConversations([]);
        setError(`Configurações recuperadas, mas o histórico de conversas não pôde ser carregado: ${messageOf(reason)}`);
      }
      await checkStatus(nextSettings);
      return true;
    } catch (reason) {
      setError(messageOf(reason)); setCoreState("offline");
      return false;
    } finally { setLoading(false); }
  }, [checkStatus]);

  useEffect(() => {
    let active = true;
    let retryTask: number | null = null;
    const task = window.setTimeout(() => {
      void initialize().then((initialized) => {
        if (active && !initialized) retryTask = window.setTimeout(() => void initialize(), 1_500);
      });
    }, 0);
    return () => {
      active = false;
      window.clearTimeout(task);
      if (retryTask !== null) window.clearTimeout(retryTask);
    };
  }, [initialize]);

  const reload = useCallback(async () => { await initialize(); }, [initialize]);

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

  const clearConversationHistory = useCallback(async (conversation: Conversation) => {
    setError(null);
    try {
      const removed = await conversationService.clearMessages(conversation.id);
      if (selectedConversation?.id === conversation.id) setMessages([]);
      setConversations(await conversationService.list());
      return removed;
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

  const probeOllama = useCallback((endpoint: string, timeoutSeconds: number) => (
    aiRepository.status(endpoint, Math.min(timeoutSeconds, 8))
  ), []);

  const updateSettings = useCallback(async (input: AISettingsInput) => {
    setError(null);
    try {
      const updated = await aiRepository.updateSettings(input);
      setSettings(updated);
      const [nextStatus, nextLifecycle] = await Promise.all([
        checkStatus(updated),
        aiRepository.modelLifecycleStatus(),
      ]);
      setStatus(nextStatus);
      setModelLifecycle(nextLifecycle);
    }
    catch (reason) { setError(messageOf(reason)); throw reason; }
  }, [checkStatus]);

  const refreshModelLifecycle = useCallback(async () => {
    try {
      const next = await aiRepository.modelLifecycleStatus();
      setModelLifecycle(next);
      return next;
    } catch {
      return null;
    }
  }, []);

  const loadLocalModel = useCallback(async () => {
    if (modelTransition) throw new Error("Ja existe uma operacao de lifecycle em andamento.");
    setModelTransition("LOADING");
    try { const next = await aiRepository.loadModel(); setModelLifecycle(next); return next; }
    finally { setModelTransition(null); }
  }, [modelTransition]);

  const unloadLocalModel = useCallback(async () => {
    if (modelTransition) throw new Error("Ja existe uma operacao de lifecycle em andamento.");
    setModelTransition("UNLOADING");
    try { const next = await aiRepository.unloadModel(); setModelLifecycle(next); return next; }
    finally { setModelTransition(null); }
  }, [modelTransition]);

  // Só um motor fica ligado por vez (o backend garante); OpenRouter tem prioridade sobre o Gemini.
  const engine: AIEngine = openRouter?.enabled ? "openrouter" : remoteSettings?.enabled ? "gemini" : "ollama";
  const engineAvailable = engine === "openrouter" ? Boolean(openRouter?.apiKeyConfigured && openRouter.model)
    : engine === "gemini" ? Boolean(remoteSettings?.apiKeyConfigured && remoteSettings.model)
    : Boolean(status?.available);
  const engineLabel = engine === "openrouter" ? "OPENROUTER" : engine === "gemini" ? "GEMINI" : "LOCAL";
  const activeModel = (engine === "openrouter" ? openRouter?.model : engine === "gemini" ? remoteSettings?.model : settings?.model) ?? "";

  const saveOpenRouter = useCallback(async (input: OpenRouterSettingsInput) => {
    setError(null);
    try {
      const next = await aiRepository.updateOpenRouterSettings(input);
      setOpenRouter(next);
      // Ligar um motor remoto desliga o outro no backend: relê o Gemini para a tela acompanhar.
      if (input.enabled) setRemoteSettings(await aiRepository.getRemoteSettings());
      return next;
    } catch (reason) { setError(messageOf(reason)); throw reason; }
  }, []);

  const updateRemote = useCallback(async (input: Parameters<typeof aiRepository.updateRemoteSettings>[0]) => {
    setError(null);
    try {
      const next = await aiRepository.updateRemoteSettings(input);
      setRemoteSettings(next);
      if (input.enabled) setOpenRouter(await aiRepository.getOpenRouterSettings().catch(() => null));
      return next;
    }
    catch (reason) { setError(messageOf(reason)); throw reason; }
  }, []);

  const refreshRemoteModels = useCallback(async () => {
    const models = await aiRepository.listRemoteModels();
    setRemoteModels(models);
    return models;
  }, []);

  const setEngine = useCallback(async (next: AIEngine) => {
    if (!remoteSettings) throw new Error("Configurações do Gemini ainda não foram carregadas.");
    if (next === "openrouter") {
      if (!openRouter?.apiKeyConfigured || !openRouter.model) { const detail = "Cadastre a chave do OpenRouter e escolha um modelo em Configurações."; setError(detail); throw new Error(detail); }
      await saveOpenRouter({ enabled: true, model: openRouter.model, favorites: openRouter.favorites });
      return;
    }
    // Saindo do OpenRouter para outro motor: desliga ele primeiro.
    if (openRouter?.enabled) await saveOpenRouter({ enabled: false, model: openRouter.model, favorites: openRouter.favorites });
    if (next === "ollama") { await updateRemote({ enabled: false, model: remoteSettings.model }); return; }
    if (!remoteSettings.apiKeyConfigured) { const detail = "Cadastre a chave do Gemini em Configurações antes de ligá-lo."; setError(detail); throw new Error(detail); }
    let model = remoteSettings.model;
    if (!model) {
      try { model = preferredGeminiModel(remoteModels.length ? remoteModels : await refreshRemoteModels()); }
      catch (reason) { setError(messageOf(reason)); throw reason; }
    }
    await updateRemote({ enabled: true, model });
  }, [openRouter, refreshRemoteModels, remoteModels, remoteSettings, saveOpenRouter, updateRemote]);

  const saveRemoteKey = useCallback(async (apiKey: string) => {
    const current = await updateRemote({ enabled: false, model: remoteSettings?.model ?? "", apiKey });
    try { const models = await refreshRemoteModels(); if (!current.model) await updateRemote({ enabled: false, model: preferredGeminiModel(models) }); }
    catch (reason) { setError(`Chave salva, mas a lista de modelos falhou: ${messageOf(reason)}`); }
  }, [refreshRemoteModels, remoteSettings, updateRemote]);

  const removeRemoteKey = useCallback(async () => {
    await updateRemote({ enabled: false, model: "", apiKey: "" });
    setRemoteModels([]);
  }, [updateRemote]);

  const send = useCallback(async (query: string) => {
    if (!settings) throw new Error("Configurações do AI Core ainda não foram carregadas.");
    const localIntent = routeIntent(query).intent;
    const worksWithoutModel = ["critical_tasks", "priority_tasks", "complete_task_reference", "unsupported_internal"].includes(localIntent);
    if (!engineAvailable && !worksWithoutModel) throw new Error(engine === "openrouter" ? "OpenRouter sem chave ou modelo. Verifique em Configurações." : engine === "gemini" ? "Gemini sem chave ou modelo. Verifique em Configurações." : status?.error || "AI Core offline. Verifique o Ollama nas Configurações.");
    setSending(true); setError(null); setLiveSteps([]);
    let conversation = selectedConversation;
    const createdNow = !conversation;
    const controller = new AbortController();
    abortRef.current = controller;
    try {
      if (!conversation) {
        conversation = await conversationService.create(query);
        setSelectedConversation(conversation);
        setConversations((current) => [conversation!, ...current]);
      }
      const provider = engine === "openrouter" && openRouter ? new OpenRouterProvider(openRouter)
        : engine === "gemini" && remoteSettings ? new GeminiProvider(remoteSettings)
        : new OllamaProvider(settings);
      const effective = engine === "ollama" ? settings : { ...settings, model: activeModel };
      const registry = new ToolRegistry();
      const service = new AICoreService(provider, new ContextBuilder(registry), conversationService, effective, () => loadPersona(registry));
      await service.send(query, conversation, (state, detail) => { if (controller.signal.aborted) return; setCoreState(state); if (detail) setPhase(detail); }, (message) => setMessages((current) => [...current, message]), requestConfirmation, controller.signal, (step) => setLiveSteps((current) => current.some((item) => item.id === step.id) ? current.map((item) => item.id === step.id ? step : item) : [...current, step]));
      setConversations(await conversationService.list());
      return {};
    } catch (reason) {
      if (isInterrupted(reason) && conversation) {
        // ESC/Parar: sem alerta. A conversa mostra só o que de fato ficou no banco.
        const remaining = await conversationService.messages(conversation.id).catch(() => [] as ConversationMessage[]);
        if (createdNow && remaining.length === 0) {
          // Conversa criada só para este pedido, que foi desfeito: some junto.
          await conversationService.remove(conversation.id).catch(() => undefined);
          setSelectedConversation(null);
          setConversations((current) => current.filter((item) => item.id !== conversation!.id));
        }
        setMessages(remaining);
        setCoreState("idle"); setPhase("INTERROMPIDO PELO SENHOR");
        return { restoredText: reason.restoredText };
      }
      const detail = messageOf(reason); setError(detail); setCoreState(/ollama|gemini|internet/i.test(detail) ? "offline" : "alert"); setPhase("FALHA NO AI CORE");
      if (conversation) setMessages(await conversationService.messages(conversation.id).catch(() => []));
      throw reason;
    } finally {
      if (abortRef.current === controller) abortRef.current = null;
      setLiveSteps([]);
      setSending(false);
    }
  }, [activeModel, engine, engineAvailable, openRouter, remoteSettings, requestConfirmation, selectedConversation, settings, status]);

  /** ESC/Parar: corta a espera pelo modelo, nega um pedido de permissão aberto e mata comandos em andamento. */
  const stop = useCallback(() => {
    const controller = abortRef.current;
    if (!controller || controller.signal.aborted) return;
    setPhase("INTERROMPENDO...");
    controller.abort();
    answerConfirmation(false);
    void workspaceRepository.cancel().catch(() => undefined);
  }, [answerConfirmation]);

  const selectModel = useCallback(async (model: string) => {
    if (engine === "openrouter" && openRouter) { await saveOpenRouter({ enabled: true, model, favorites: openRouter.favorites }); return; }
    if (engine === "gemini") { await updateRemote({ enabled: true, model }); return; }
    if (!settings) throw new Error("Configurações do AI Core ainda não foram carregadas.");
    await updateSettings({ endpoint: settings.endpoint, model, contextMessageLimit: settings.contextMessageLimit, timeoutSeconds: settings.timeoutSeconds });
  }, [engine, openRouter, saveOpenRouter, settings, updateRemote, updateSettings]);

  const value = useMemo<AIContextValue>(() => ({ settings, remoteSettings, remoteModels, openRouter, saveOpenRouter, engineLabel, activeModel, engine, engineAvailable, setEngine, selectModel, saveRemoteKey, removeRemoteKey, refreshRemoteModels, status, modelLifecycle, modelTransition, conversations, selectedConversation, messages, coreState, phase, loading, sending, error, selectConversation, newConversation, deleteConversation, clearConversationHistory, reload, send, stop, liveSteps, pendingConfirmation, answerConfirmation, updateSettings, probeOllama, refreshStatus, refreshModelLifecycle, loadLocalModel, unloadLocalModel }), [openRouter, saveOpenRouter, engineLabel, activeModel, stop, liveSteps, pendingConfirmation, answerConfirmation, settings, remoteSettings, remoteModels, engine, engineAvailable, setEngine, selectModel, saveRemoteKey, removeRemoteKey, refreshRemoteModels, status, modelLifecycle, modelTransition, conversations, selectedConversation, messages, coreState, phase, loading, sending, error, selectConversation, newConversation, deleteConversation, clearConversationHistory, reload, send, updateSettings, probeOllama, refreshStatus, refreshModelLifecycle, loadLocalModel, unloadLocalModel]);
  return <AIContext.Provider value={value}>{children}</AIContext.Provider>;
}

