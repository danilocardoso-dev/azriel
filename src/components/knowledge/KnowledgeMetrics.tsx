import { useEffect, useState, type FormEvent } from "react";
import { Meter } from "../hud/Meter";
import { useAzrielData } from "../../contexts/useAzrielData";
import type { KnowledgeArea, KnowledgeHistory } from "../../types";

export function KnowledgeMetrics({ area, onUpdated }: { area: KnowledgeArea; onUpdated: (area: KnowledgeArea) => void }) {
  const { updateMetrics, loadHistory } = useAzrielData();
  const [coverage, setCoverage] = useState(area.coverage);
  const [depth, setDepth] = useState(area.depth);
  const [reason, setReason] = useState("");
  const [history, setHistory] = useState<KnowledgeHistory[]>([]);
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string | null>(null);

  useEffect(() => {
    void loadHistory(area.id).then(setHistory).catch((error) => setMessage(String(error)));
  }, [area.id, loadHistory]);

  async function submit(event: FormEvent) {
    event.preventDefault(); setBusy(true); setMessage(null);
    try {
      const updated = await updateMetrics({ knowledgeId: area.id, coverage, depth, reason });
      onUpdated(updated); setReason(""); setHistory(await loadHistory(area.id)); setMessage("Métricas persistidas com sucesso.");
    } catch (error) { setMessage(error instanceof Error ? error.message : String(error)); }
    finally { setBusy(false); }
  }

  return <>
    <Meter label="Cobertura" value={area.coverage} />
    <Meter label="Profundidade" value={area.depth} tone="amber" />
    <form className="metric-editor" onSubmit={submit}>
      <h3>Atualizar métricas</h3>
      <label>Cobertura<input type="number" min="0" max="100" value={coverage} onChange={(event) => setCoverage(Number(event.target.value))} /></label>
      <label>Profundidade<input type="number" min="0" max="100" value={depth} onChange={(event) => setDepth(Number(event.target.value))} /></label>
      <label className="metric-editor__reason">Motivo<input value={reason} maxLength={180} onChange={(event) => setReason(event.target.value)} placeholder="Ex.: módulo concluído" /></label>
      <button disabled={busy}>{busy ? "SALVANDO..." : "REGISTRAR EVOLUÇÃO"}</button>
      {message && <p className="metric-editor__message" role="status">{message}</p>}
    </form>
    <h3>Histórico</h3>
    <div className="history-list">
      {history.map((entry) => <div key={entry.id}><span>{new Date(`${entry.recordedAt.replace(" ", "T")}Z`).toLocaleString("pt-BR")}</span><strong>C {entry.coverage} / P {entry.depth}</strong><p>{entry.reason}</p></div>)}
      {!history.length && <p>Nenhum histórico registrado.</p>}
    </div>
  </>;
}
