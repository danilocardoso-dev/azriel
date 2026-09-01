import { useEffect, useRef, useState, type FormEvent, type KeyboardEvent } from "react";
import { DeleteConfirmationDialog } from "../components/daily/DeleteConfirmationDialog";
import { ModuleIntro } from "../components/layout/ModuleIntro";
import { useAI } from "../contexts/useAI";
import { formatTimestamp } from "../services/dateService";
import type { Conversation } from "../types";

const suggestions = ["Azriel, situação.", "O que tenho para hoje?", "Qual minha maior lacuna?", "Como está minha formação?"];

export function AICorePage() {
  const { status, conversations, selectedConversation, messages, phase, loading, sending, error, selectConversation, newConversation, deleteConversation, send } = useAI();
  const [input, setInput] = useState("");
  const [conversationToDelete, setConversationToDelete] = useState<Conversation | null>(null);
  const [deleting, setDeleting] = useState(false);
  const endRef = useRef<HTMLDivElement>(null);
  useEffect(() => { endRef.current?.scrollIntoView({ behavior: "smooth" }); }, [messages, phase]);

  async function submit(event: FormEvent) {
    event.preventDefault(); const query = input.trim(); if (!query || sending) return; setInput("");
    try { await send(query); } catch { /* O contexto exibe a falha sem rejeição solta. */ }
  }
  function handleKeyDown(event: KeyboardEvent<HTMLTextAreaElement>) { if (event.key === "Enter" && !event.shiftKey) { event.preventDefault(); event.currentTarget.form?.requestSubmit(); } }
  async function removeConversation() {
    if (!conversationToDelete || deleting) return;
    setDeleting(true);
    try {
      await deleteConversation(conversationToDelete);
      setConversationToDelete(null);
    } catch { /* O contexto exibe a falha. */ }
    finally { setDeleting(false); }
  }

  return <>
    <ModuleIntro code="AI-10" title="AI Core" description="Consulta local e somente leitura aos dados reais do Azriel." metric={`OLLAMA // ${status?.available ? "ONLINE" : "OFFLINE"}`} />
    <div className="ai-layout">
      <aside className="ai-conversations">
        <header><span>CONVERSAS</span><strong>{conversations.length}</strong></header>
        <button className="ai-conversations__new" onClick={newConversation}>＋ NOVA CONVERSA</button>
        <nav>{conversations.map((conversation) => <div className={`ai-conversation-item${selectedConversation?.id === conversation.id ? " active" : ""}`} key={conversation.id}><button className="ai-conversation-item__open" onClick={() => void selectConversation(conversation)}><strong>{conversation.title}</strong><small>{formatTimestamp(conversation.updatedAt)}</small></button><button className="ai-conversation-item__delete" aria-label={`Excluir conversa ${conversation.title}`} title="Excluir conversa" onClick={() => setConversationToDelete(conversation)}>×</button></div>)}</nav>
      </aside>
      <section className="ai-terminal">
        <header><div><span>AZRIEL // AI CORE</span><small>{selectedConversation ? selectedConversation.title : "NOVA SESSÃO"}</small></div><i data-state={status?.available ? "online" : "offline"}>{status?.available ? "ONLINE" : "OFFLINE"}</i></header>
        <div className="ai-terminal__messages" aria-live="polite">
          {!selectedConversation && messages.length === 0 && <div className="ai-welcome"><span>AI CORE // READ ONLY</span><h2>Como posso consultar o sistema?</h2><p>Azriel utiliza ferramentas controladas para recuperar dados. Nenhuma alteração pode ser executada pela IA.</p><div>{suggestions.map((suggestion) => <button onClick={() => setInput(suggestion)} key={suggestion}>{suggestion}</button>)}</div></div>}
          {messages.map((message) => <article className={`ai-message ai-message--${message.role}`} key={message.id}><header><span>{message.role === "user" ? "AZ" : "AZRIEL"}</span><time>{formatTimestamp(message.createdAt)}</time></header><p>{message.content}</p></article>)}
          {sending && <div className="ai-phase"><span /><strong>{phase}</strong></div>}
          {loading && selectedConversation && <div className="ai-phase"><span /><strong>RECUPERANDO HISTÓRICO</strong></div>}
          {error && <div className="ai-terminal__error"><strong>AI CORE ALERT</strong><p>{error}</p></div>}
          {!status?.available && !loading && <div className="ai-terminal__offline"><strong>AI CORE OFFLINE</strong><p>{status?.error || "Ollama não encontrado em localhost:11434."}</p><small>Os demais módulos permanecem operacionais. Verifique a conexão em Configurações.</small></div>}
          <div ref={endRef} />
        </div>
        <form className="ai-composer" onSubmit={submit}><span>&gt;</span><textarea rows={5} placeholder="Digite um comando para o Azriel..." value={input} onChange={(event) => setInput(event.target.value)} onKeyDown={handleKeyDown} disabled={!status?.available || sending} /><button disabled={!status?.available || sending || !input.trim()}>{sending ? "PROCESSANDO" : "ENVIAR"}</button></form>
      </section>
    </div>
    {conversationToDelete && <DeleteConfirmationDialog kind="conversa" title={conversationToDelete.title} description="A conversa e todas as mensagens dela serão removidas definitivamente do banco local. Esta ação não poderá ser desfeita." busy={deleting} onCancel={() => setConversationToDelete(null)} onConfirm={() => void removeConversation()} />}
  </>;
}
