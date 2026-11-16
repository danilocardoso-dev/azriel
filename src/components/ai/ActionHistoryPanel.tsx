import { useCallback, useEffect, useState } from "react";
import { workspaceRepository, type ActionRecord } from "../../repositories/workspaceRepository";
import { formatTimestamp } from "../../services/dateService";
import "../../styles/mind.css";

/** O que o Azriel fez no PC: arquivos gravados e editados (com backup), comandos e exclusões. */
export function ActionHistoryPanel() {
  const [actions, setActions] = useState<ActionRecord[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const load = useCallback(() => {
    void workspaceRepository.history(100).then((list) => { setActions(list); setError(null); }).catch((reason) => setError(reason instanceof Error ? reason.message : String(reason)));
  }, []);
  useEffect(() => { const first = window.setTimeout(load, 0); return () => window.clearTimeout(first); }, [load]);

  return <section className="ai-settings mind-panel">
    <header><span>HISTÓRICO DE AÇÕES // O QUE O AZRIEL FEZ NO PC</span><i data-state="online">{actions ? `${actions.length} REGISTRO${actions.length === 1 ? "" : "S"}` : "CARREGANDO"}</i></header>
    <div className="mind-panel__body">
      <div className="mind-panel__actions">
        <p className="mind-panel__hint">Arquivos editados ou sobrescritos têm backup na pasta de dados do Azriel; exclusões vão para a Lixeira do Windows.</p>
        <button type="button" className="mind-button--ghost" onClick={load}>ATUALIZAR</button>
      </div>
      {error && <p className="mind-panel__status" role="alert" data-kind="error">{error}</p>}
      <ul className="mind-list">
        {actions && !actions.length && <li className="mind-list__empty">O Azriel ainda não mexeu em arquivos nem rodou comandos.</li>}
        {actions?.map((action) => <li key={action.id} className="mind-memory mind-action" data-ok={action.ok}>
          <header><b>{action.kind}</b><small>{action.ok ? "ok" : "falhou"} · {formatTimestamp(action.createdAt)}</small></header>
          <code>{action.target}</code>
          {action.detail && <small className="mind-action__detail">{action.detail}</small>}
        </li>)}
      </ul>
    </div>
  </section>;
}
