import { useState } from "react";
import { useAutomation } from "../../contexts/useAutomation";

export function RoutineConfirmationDialog() {
  const { pendingRoutine, confirmRoutine, cancelRoutine } = useAutomation();
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [hiddenHistoryId, setHiddenHistoryId] = useState<number | null>(null);
  if (!pendingRoutine?.confirmation) return null;
  if (hiddenHistoryId === pendingRoutine.historyId) return null;
  const confirmation = pendingRoutine.confirmation;
  const execute = async () => {
    setBusy(true); setError(null);
    try { await confirmRoutine(); }
    catch (reason) { setError(reason instanceof Error ? reason.message : String(reason)); setBusy(false); }
  };
  const cancel = async () => {
    setBusy(true); setError(null);
    try { await cancelRoutine(); }
    catch (reason) { setError(reason instanceof Error ? reason.message : String(reason)); setBusy(false); }
  };
  return <div className="routine-confirmation__backdrop" role="presentation">
    <section className="routine-confirmation" role="dialog" aria-modal="true" aria-labelledby="routine-confirmation-title">
      <header><span>AZRIEL // CONFIRMAÇÃO DE ROTINA</span><strong>POLICY GATE</strong></header>
      <div className="routine-confirmation__body">
        <small>ROTINA REGISTRADA</small>
        <h2 id="routine-confirmation-title">{confirmation.routineName}</h2>
        <p>Confira a sequência autorizada antes de executar. Nenhum comando livre será usado.</p>
        <ol>{confirmation.actions.map((action) => <li key={`${action.order}-${action.actionId}`}>
          <span>{String(action.order).padStart(2, "0")}</span>
          <div><strong>{action.actionName}</strong><small>{action.targetName}{action.delayMs > 0 ? ` // intervalo ${action.delayMs} ms` : ""}</small></div>
        </li>)}</ol>
        {error && <div className="form-error">{error}</div>}
      </div>
      <footer><button onClick={() => busy ? setHiddenHistoryId(pendingRoutine.historyId) : void cancel()}>{busy ? "OCULTAR E CONTINUAR" : "CANCELAR"}</button><button className="confirm" disabled={busy} onClick={() => void execute()}>{busy ? "PROCESSANDO..." : "EXECUTAR ROTINA"}</button></footer>
    </section>
  </div>;
}
