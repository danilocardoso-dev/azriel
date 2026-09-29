import { useEffect, useMemo, useRef } from "react";
import { createPortal } from "react-dom";
import { selectConversationHistory } from "../../ai/AICoreService";
import { formatTimestamp } from "../../services/dateService";
import type { Conversation, ConversationMessage } from "../../types";

type AIConversationHistoryDialogProps = {
  conversation: Conversation;
  messages: ConversationMessage[];
  contextMessageLimit: number;
  busy: boolean;
  onClose: () => void;
  onRequestClear: () => void;
};

export function AIConversationHistoryDialog({
  conversation,
  messages,
  contextMessageLimit,
  busy,
  onClose,
  onRequestClear,
}: AIConversationHistoryDialogProps) {
  const closeButtonRef = useRef<HTMLButtonElement>(null);
  const selected = useMemo(
    () => new Set(selectConversationHistory(messages, contextMessageLimit).map((message) => message.id)),
    [contextMessageLimit, messages],
  );
  const storedCharacters = messages.reduce((total, message) => total + message.content.length, 0);
  const contextCharacters = messages.reduce(
    (total, message) => total + (selected.has(message.id) ? message.content.length : 0),
    0,
  );

  useEffect(() => {
    closeButtonRef.current?.focus();
    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape" && !busy) onClose();
    }
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [busy, onClose]);

  return createPortal(
    <div
      className="ai-history-dialog__backdrop"
      onMouseDown={(event) => {
        if (event.target === event.currentTarget && !busy) onClose();
      }}
    >
      <section className="ai-history-dialog" role="dialog" aria-modal="true" aria-labelledby="ai-history-title">
        <header>
          <div>
            <span>AI CORE // CONTEXTO LOCAL</span>
            <h2 id="ai-history-title">Histórico de envio</h2>
            <small>{conversation.title}</small>
          </div>
          <button ref={closeButtonRef} type="button" onClick={onClose} disabled={busy} aria-label="Fechar histórico">×</button>
        </header>

        <div className="ai-history-dialog__summary">
          <span><small>ARMAZENADAS</small><strong>{messages.length}</strong></span>
          <span><small>NO LIMITE ATUAL</small><strong>{selected.size}</strong></span>
          <span><small>CARACTERES</small><strong>{contextCharacters} / {storedCharacters}</strong></span>
          <span><small>LIMITE CONFIGURADO</small><strong>{contextMessageLimit} mensagens</strong></span>
        </div>

        <p className="ai-history-dialog__notice">
          O Azriel prioriza as mensagens mais recentes e limita automaticamente o tamanho enviado ao Ollama. A limpeza afeta somente esta conversa.
        </p>

        <div className="ai-history-dialog__messages">
          {messages.length === 0 && <div className="ai-history-dialog__empty">NENHUMA MENSAGEM ARMAZENADA</div>}
          {messages.map((message) => (
            <article key={message.id} data-in-context={selected.has(message.id)}>
              <header>
                <strong>{message.role === "user" ? "OPERADOR" : message.role.toUpperCase()}</strong>
                <span>{selected.has(message.id) ? "NO CONTEXTO" : "FORA DO LIMITE"}</span>
                <time>{formatTimestamp(message.createdAt)}</time>
              </header>
              <p>{message.content}</p>
              <small>{message.content.length} caracteres</small>
            </article>
          ))}
        </div>

        <footer>
          <button type="button" onClick={onClose} disabled={busy}>FECHAR</button>
          <button type="button" className="danger" onClick={onRequestClear} disabled={busy || messages.length === 0}>
            LIMPAR HISTÓRICO
          </button>
        </footer>
      </section>
    </div>,
    document.body,
  );
}
