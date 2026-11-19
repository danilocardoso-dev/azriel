import { useEffect, useRef, useState } from "react";
import { MarkdownMessage } from "../components/ai/MarkdownMessage";
import { DeleteConfirmationDialog } from "../components/daily/DeleteConfirmationDialog";
import { useAI } from "../contexts/useAI";
import { formatTimestamp } from "../services/dateService";
import type { Conversation } from "../types";
import "../styles/ai-core.css";

/**
 * Esqueleto do AI Core: as conversas salvas continuam à mão, mas o Azriel está sem motor de IA
 * por enquanto. O campo de mensagem fica desligado até um motor voltar.
 */
export function AICorePage() {
  const { conversations, selectedConversation, messages, loading, error, selectConversation, newConversation, deleteConversation } = useAI();
  const [conversationToDelete, setConversationToDelete] = useState<Conversation | null>(null);
  const [deleting, setDeleting] = useState(false);
  const messagesRef = useRef<HTMLDivElement>(null);
  useEffect(() => { const list = messagesRef.current; if (list) list.scrollTo({ top: list.scrollHeight }); }, [messages]);

  async function removeConversation() {
    if (!conversationToDelete || deleting) return;
    setDeleting(true);
    try { await deleteConversation(conversationToDelete); setConversationToDelete(null); }
    catch { /* O contexto exibe a falha. */ }
    finally { setDeleting(false); }
  }

  return <>
    <div className="ai-layout">
      <aside className="ai-conversations">
        <header><span>CONVERSAS</span><strong>{conversations.length}</strong></header>
        <button className="ai-conversations__new" onClick={newConversation}>＋ NOVA CONVERSA</button>
        <nav>{conversations.map((conversation) => <div className={`ai-conversation-item${selectedConversation?.id === conversation.id ? " active" : ""}`} key={conversation.id}>
          <button className="ai-conversation-item__open" onClick={() => void selectConversation(conversation)}><strong>{conversation.title}</strong><small>{formatTimestamp(conversation.updatedAt)}</small></button>
          <button className="ai-conversation-item__delete" aria-label={`Excluir conversa ${conversation.title}`} title="Excluir conversa" onClick={() => setConversationToDelete(conversation)}>×</button>
        </div>)}</nav>
      </aside>
      <section className="ai-terminal">
        <header><div><span>AZRIEL // AI CORE</span><small>{selectedConversation ? selectedConversation.title : "NOVA SESSÃO"}</small></div></header>
        <div className="ai-terminal__messages" aria-live="polite" ref={messagesRef}>
          {!selectedConversation && <div className="ai-welcome"><span>AZRIEL // AI CORE</span><h2>Sem motor de IA por enquanto.</h2><p>O AI Core está em espera. As conversas antigas continuam à esquerda para consulta; quando um motor voltar, a conversa volta a funcionar aqui.</p></div>}
          {messages.map((message) => message.role === "user"
            ? <article className="ai-turn ai-turn--user" key={message.id} title={formatTimestamp(message.createdAt)}><p>{message.content}</p></article>
            : <article className="ai-turn ai-turn--assistant" key={message.id} title={formatTimestamp(message.createdAt)}><MarkdownMessage content={message.content} /></article>)}
          {loading && selectedConversation && <div className="ai-phase"><span /><strong>RECUPERANDO HISTÓRICO</strong></div>}
          {error && <div className="ai-terminal__error"><strong>AI CORE ALERT</strong><p>{error}</p></div>}
        </div>
        <form className="ai-composer" onSubmit={(event) => event.preventDefault()}>
          <div className="ai-composer__card">
            <textarea rows={1} placeholder="Sem motor de IA por enquanto" disabled aria-label="Mensagem para o Azriel (desligado)" />
            <div className="ai-composer__bar">
              <small>AI Core em espera · as conversas salvas ficam disponíveis para consulta</small>
              <button className="ai-composer__send" disabled title="Sem motor de IA" aria-label="Enviar (desligado)">↑</button>
            </div>
          </div>
        </form>
      </section>
    </div>
    {conversationToDelete && <DeleteConfirmationDialog kind="conversa" title={conversationToDelete.title} description="A conversa e todas as mensagens dela serão removidas definitivamente do banco local. Esta ação não poderá ser desfeita." busy={deleting} onCancel={() => setConversationToDelete(null)} onConfirm={() => void removeConversation()} />}
  </>;
}
