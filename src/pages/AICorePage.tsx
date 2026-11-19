import { useEffect, useRef, useState, type FormEvent, type KeyboardEvent } from "react";
import { AIConversationHistoryDialog } from "../components/ai/AIConversationHistoryDialog";
import { EngineSwitcher } from "../components/ai/EngineSwitcher";
import { AgentSteps } from "../components/ai/AgentSteps";
import { MarkdownMessage } from "../components/ai/MarkdownMessage";
import { DeleteConfirmationDialog } from "../components/daily/DeleteConfirmationDialog";
import { useAI } from "../contexts/useAI";
import { formatTimestamp } from "../services/dateService";
import type { Conversation } from "../types";
import "../styles/ai-core.css";

const suggestions = [
  "Azriel, relatório do sistema.",
  "O que tenho agendado para hoje?",
  "Quais são as tarefas prioritárias?",
  "Resuma minhas notas recentes.",
];

export function AICorePage() {
  const { settings, status, engine, engineAvailable, conversations, selectedConversation, messages, phase, loading, sending, error, selectConversation, newConversation, deleteConversation, clearConversationHistory, send, stop, liveSteps, pendingConfirmation, answerConfirmation } = useAI();
  const [input, setInput] = useState("");
  const [conversationToDelete, setConversationToDelete] = useState<Conversation | null>(null);
  const [deleting, setDeleting] = useState(false);
  const [historyOpen, setHistoryOpen] = useState(false);
  const [historyClearPending, setHistoryClearPending] = useState(false);
  const [clearingHistory, setClearingHistory] = useState(false);
  const messagesRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);
  // Rola só a lista de mensagens (scrollIntoView também arrastava a página inteira).
  useEffect(() => { const list = messagesRef.current; if (list) list.scrollTo({ top: list.scrollHeight, behavior: "smooth" }); }, [messages, phase, pendingConfirmation, liveSteps]);
  // Campo cresce com o texto, como no Claude (até 40% da altura da tela; depois rola).
  useEffect(() => {
    const field = inputRef.current;
    if (!field) return;
    field.style.height = "auto";
    field.style.height = `${Math.min(field.scrollHeight, Math.round(window.innerHeight * 0.4))}px`;
  }, [input]);
  // O campo fica sempre pronto para a próxima mensagem.
  useEffect(() => { if (!sending) inputRef.current?.focus(); }, [sending, selectedConversation?.id]);

  // ESC para o Azriel de qualquer ponto da tela (antes do atalho global, que sairia da tela cheia).
  useEffect(() => {
    if (!sending) return;
    const onKey = (event: globalThis.KeyboardEvent) => {
      if (event.key !== "Escape") return;
      event.preventDefault();
      event.stopPropagation();
      stop();
    };
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
  }, [sending, stop]);

  async function submit(event: FormEvent) {
    event.preventDefault(); const query = input.trim(); if (!query || sending) return; setInput("");
    try {
      const { restoredText } = await send(query);
      // Interrompido antes de qualquer ação: o texto volta para corrigir (sem apagar o que já foi digitado).
      if (restoredText) setInput((current) => current.trim() ? `${restoredText}\n${current}` : restoredText);
    } catch { /* O contexto exibe a falha sem rejeição solta. */ }
    finally { inputRef.current?.focus(); }
  }
  const lastUserMessage = [...messages].reverse().find((message) => message.role === "user")?.content ?? "";
  function handleKeyDown(event: KeyboardEvent<HTMLTextAreaElement>) {
    if (event.key === "Enter" && !event.shiftKey) { event.preventDefault(); event.currentTarget.form?.requestSubmit(); return; }
    // ↑ com o campo vazio: traz a última mensagem para reaproveitar.
    if (event.key === "ArrowUp" && !input && lastUserMessage) {
      event.preventDefault();
      setInput(lastUserMessage);
      const field = event.currentTarget;
      window.requestAnimationFrame(() => field.setSelectionRange(field.value.length, field.value.length));
    }
  }
  async function removeConversation() {
    if (!conversationToDelete || deleting) return;
    setDeleting(true);
    try {
      await deleteConversation(conversationToDelete);
      setConversationToDelete(null);
    } catch { /* O contexto exibe a falha. */ }
    finally { setDeleting(false); }
  }
  async function clearHistory() {
    if (!selectedConversation || clearingHistory || sending) return;
    setClearingHistory(true);
    try {
      await clearConversationHistory(selectedConversation);
      setHistoryClearPending(false);
      setHistoryOpen(false);
    } catch { /* O contexto exibe a falha. */ }
    finally { setClearingHistory(false); }
  }

  return <>
    <div className="ai-layout">
      <aside className="ai-conversations">
        <header><span>CONVERSAS</span><strong>{conversations.length}</strong></header>
        <button className="ai-conversations__new" onClick={newConversation}>＋ NOVA CONVERSA</button>
        <nav>{conversations.map((conversation) => <div className={`ai-conversation-item${selectedConversation?.id === conversation.id ? " active" : ""}`} key={conversation.id}><button className="ai-conversation-item__open" onClick={() => void selectConversation(conversation)}><strong>{conversation.title}</strong><small>{formatTimestamp(conversation.updatedAt)}</small></button><button className="ai-conversation-item__delete" aria-label={`Excluir conversa ${conversation.title}`} title="Excluir conversa" onClick={() => setConversationToDelete(conversation)}>×</button></div>)}</nav>
      </aside>
      <section className="ai-terminal">
        <header><div><span>AZRIEL // AI CORE</span><small>{selectedConversation ? selectedConversation.title : "NOVA SESSÃO"}</small></div><div className="ai-terminal__header-actions"><EngineSwitcher disabled={sending} /><button type="button" onClick={() => setHistoryOpen(true)} disabled={!selectedConversation || loading || sending}>CONTEXTO {messages.length}</button><i data-state={engineAvailable ? "online" : "offline"}>{engineAvailable ? "ONLINE" : "OFFLINE"}</i></div></header>
        <div className="ai-terminal__messages" aria-live="polite" ref={messagesRef}>
          {!selectedConversation && messages.length === 0 && <div className="ai-welcome"><span>AZRIEL // AI CORE</span><h2>Em que posso ajudar, Senhor?</h2><p>Converso, lembro, organizo agenda, notas e finanças e programo no seu PC. Excluir, só com a sua permissão.</p><div>{suggestions.map((suggestion) => <button onClick={() => { setInput(suggestion); inputRef.current?.focus(); }} key={suggestion}>{suggestion}</button>)}</div></div>}
          {selectedConversation && messages.length === 0 && !loading && <div className="ai-welcome"><span>CONTEXTO LIMPO</span><h2>Esta conversa está pronta para recomeçar.</h2><p>A conversa foi preservada, mas nenhuma mensagem anterior será enviada ao modelo.</p></div>}
          {messages.map((message) => message.role === "user"
            ? <article className="ai-turn ai-turn--user" key={message.id} title={formatTimestamp(message.createdAt)}><p>{message.content}</p></article>
            : <article className="ai-turn ai-turn--assistant" key={message.id} title={formatTimestamp(message.createdAt)}>
              {message.steps && <AgentSteps steps={message.steps} />}
              <MarkdownMessage content={message.content} />
            </article>)}
          {sending && liveSteps.length > 0 && <article className="ai-turn ai-turn--assistant"><AgentSteps steps={liveSteps} live /></article>}
          {pendingConfirmation && <div className="ai-confirm" role="alertdialog" aria-labelledby="ai-confirm-title">
            <span>PERMISSÃO NECESSÁRIA</span>
            <strong id="ai-confirm-title">O Azriel quer: {pendingConfirmation.title}</strong>
            <p>{pendingConfirmation.detail}</p>
            <div><button type="button" className="ai-confirm__allow" onClick={() => answerConfirmation(true)}>PERMITIR</button><button type="button" className="ai-confirm__deny" onClick={() => answerConfirmation(false)} autoFocus>NEGAR</button></div>
          </div>}
          {sending && !pendingConfirmation && <div className="ai-phase"><span /><strong>{phase}</strong><small>ESC para parar</small></div>}
          {loading && selectedConversation && <div className="ai-phase"><span /><strong>RECUPERANDO HISTÓRICO</strong></div>}
          {error && <div className="ai-terminal__error"><strong>AI CORE ALERT</strong><p>{error}</p></div>}
          {!engineAvailable && !loading && <div className="ai-terminal__offline"><strong>AI CORE OFFLINE</strong><p>{engine === "deepseek" ? "DeepSeek sem chave ou sem modelo selecionado." : engine === "gemini" ? "Gemini sem chave ou sem modelo selecionado." : status?.error || "Ollama não encontrado no endpoint configurado."}</p><small>Os demais módulos permanecem operacionais. Troque o motor acima ou verifique em Configurações.</small></div>}
        </div>
        <form className="ai-composer" onSubmit={submit} onClick={() => inputRef.current?.focus()}>
          <div className="ai-composer__card">
            <textarea ref={inputRef} autoFocus rows={1} placeholder={sending ? "O Azriel está trabalhando… (ESC para parar)" : "Pergunte ou peça algo ao Azriel"} value={input} onChange={(event) => setInput(event.target.value)} onKeyDown={handleKeyDown} disabled={!engineAvailable} aria-busy={sending} aria-label="Mensagem para o Azriel" />
            <div className="ai-composer__bar">
              <small>{sending ? "ESC para parar" : "Enter envia · Shift+Enter quebra linha · ↑ última mensagem"}</small>
              {sending
                ? <button type="button" className="ai-composer__send ai-composer__send--stop" onClick={stop} title="Parar (ESC)" aria-label="Parar">■</button>
                : <button className="ai-composer__send" disabled={!engineAvailable || !input.trim()} title="Enviar (Enter)" aria-label="Enviar">↑</button>}
            </div>
          </div>
        </form>
      </section>
    </div>
    {historyOpen && selectedConversation && settings && <AIConversationHistoryDialog conversation={selectedConversation} messages={messages} contextMessageLimit={settings.contextMessageLimit} busy={clearingHistory} onClose={() => setHistoryOpen(false)} onRequestClear={() => { setHistoryOpen(false); setHistoryClearPending(true); }} />}
    {historyClearPending && selectedConversation && <DeleteConfirmationDialog kind="histórico da conversa" title={selectedConversation.title} description="Todas as mensagens desta conversa serão removidas do banco local. A conversa e as configurações do AI Core serão preservadas." busy={clearingHistory} onCancel={() => { setHistoryClearPending(false); setHistoryOpen(true); }} onConfirm={() => void clearHistory()} />}
    {conversationToDelete && <DeleteConfirmationDialog kind="conversa" title={conversationToDelete.title} description="A conversa e todas as mensagens dela serão removidas definitivamente do banco local. Esta ação não poderá ser desfeita." busy={deleting} onCancel={() => setConversationToDelete(null)} onConfirm={() => void removeConversation()} />}
  </>;
}

