import { useState, type FormEvent } from "react";
import { useAI } from "../../contexts/useAI";
import type { AISettings, AISettingsInput } from "../../types";

export function AISettingsPanel() {
  const { settings, status, updateSettings, refreshStatus } = useAI();
  if (!settings) return <section className="ai-settings"><header><span>AI CORE</span><i>CARREGANDO</i></header><p>Recuperando configurações locais...</p></section>;
  return <AISettingsForm key={`${settings.updatedAt}-${settings.model}-${settings.endpoint}`} initial={settings} status={status} updateSettings={updateSettings} refreshStatus={refreshStatus} />;
}

function AISettingsForm({ initial, status, updateSettings, refreshStatus }: {
  initial: AISettings;
  status: ReturnType<typeof useAI>["status"];
  updateSettings: ReturnType<typeof useAI>["updateSettings"];
  refreshStatus: ReturnType<typeof useAI>["refreshStatus"];
}) {
  const [form, setForm] = useState<AISettingsInput>({ endpoint: initial.endpoint, model: initial.model, contextMessageLimit: initial.contextMessageLimit, timeoutSeconds: initial.timeoutSeconds });
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string | null>(null);

  async function submit(event: FormEvent) {
    event.preventDefault(); setBusy(true); setMessage(null);
    try { await updateSettings(form); setMessage("Configuração salva e conexão verificada."); }
    catch (reason) { setMessage(reason instanceof Error ? reason.message : String(reason)); }
    finally { setBusy(false); }
  }

  async function testConnection() { setBusy(true); setMessage(null); const result = await refreshStatus(); setMessage(result?.available ? `Ollama online. ${result.models.length} modelo(s) encontrado(s).` : result?.error || "Ollama indisponível."); setBusy(false); }
  const models = status?.models ?? [];
  return <section className="ai-settings"><header><span>AI CORE</span><i data-state={status?.available ? "online" : "offline"}>{status?.available ? "ONLINE" : "OFFLINE"}</i></header>
    <form onSubmit={submit}>
      <label><span><strong>Provedor</strong><small>Provider local desacoplado</small></span><input value="OLLAMA LOCAL" disabled /></label>
      <label><span><strong>Endpoint</strong><small>Apenas localhost é permitido</small></span><input value={form.endpoint} onChange={(event) => setForm({ ...form, endpoint: event.target.value })} /></label>
      <label><span><strong>Modelo</strong><small>Sem substituição silenciosa</small></span>{models.length ? <select value={form.model} onChange={(event) => setForm({ ...form, model: event.target.value })}>{!models.includes(form.model) && <option value={form.model}>{form.model}</option>}{models.map((model) => <option value={model} key={model}>{model}</option>)}</select> : <input value={form.model} onChange={(event) => setForm({ ...form, model: event.target.value })} />}</label>
      <label><span><strong>Contexto</strong><small>Últimas mensagens enviadas ao modelo</small></span><input type="number" min="1" max="20" value={form.contextMessageLimit} onChange={(event) => setForm({ ...form, contextMessageLimit: Number(event.target.value) })} /></label>
      <label><span><strong>Timeout</strong><small>Limite entre 5 e 180 segundos</small></span><input type="number" min="5" max="180" value={form.timeoutSeconds} onChange={(event) => setForm({ ...form, timeoutSeconds: Number(event.target.value) })} /></label>
      <div className="ai-settings__actions"><button disabled={busy}>SALVAR CONFIGURAÇÃO</button><button type="button" onClick={() => void testConnection()} disabled={busy}>TESTAR CONEXÃO</button></div>
      {message && <p role="status">{message}</p>}
    </form>
  </section>;
}
