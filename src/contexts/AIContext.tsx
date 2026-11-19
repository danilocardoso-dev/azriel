import { useCallback, useEffect, useMemo, useState, type ReactNode } from "react";
import { conversationService } from "../services/conversationService";
import type { AzrielState, Conversation, ConversationMessage } from "../types";
import { AIContext, type AIContextValue } from "./ai-context";

const messageOf = (error: unknown) => error instanceof Error ? error.message : String(error);

/** Esqueleto do AI Core: lista e mostra as conversas salvas. Sem motor de IA por enquanto. */
export function AIProvider({ children }: { children: ReactNode }) {
  const [conversations, setConversations] = useState<Conversation[]>([]);
  const [selectedConversation, setSelectedConversation] = useState<Conversation | null>(null);
  const [messages, setMessages] = useState<ConversationMessage[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  // Sem motor, o núcleo fica em repouso (não "offline"): os outros módulos funcionam normalmente.
  const coreState: AzrielState = "idle";

  const reload = useCallback(async () => {
    setLoading(true); setError(null);
    try { setConversations(await conversationService.list()); }
    catch (reason) { setError(`As conversas não puderam ser carregadas: ${messageOf(reason)}`); }
    finally { setLoading(false); }
  }, []);

  useEffect(() => {
    const first = window.setTimeout(() => void reload(), 0);
    return () => window.clearTimeout(first);
  }, [reload]);

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
      if (selectedConversation?.id === conversation.id) { setSelectedConversation(null); setMessages([]); }
    } catch (reason) { setError(messageOf(reason)); throw reason; }
  }, [selectedConversation]);

  const value = useMemo<AIContextValue>(() => ({
    conversations, selectedConversation, messages, coreState, loading, error,
    engineAvailable: false, engineLabel: "SEM IA",
    selectConversation, newConversation, deleteConversation, reload,
  }), [conversations, selectedConversation, messages, coreState, loading, error, selectConversation, newConversation, deleteConversation, reload]);
  return <AIContext.Provider value={value}>{children}</AIContext.Provider>;
}
