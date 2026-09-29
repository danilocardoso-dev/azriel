import { useState, type FormEvent } from "react";
import { useAI } from "../../contexts/useAI";
import type { AISettings, AISettingsInput, OllamaStatus } from "../../types";
import { canSaveAiSettings, modelAfterProbe, normalizeOllamaEndpoint } from "./aiSettingsFlow";

export function AISettingsPanel() {
  const { settings, status, loading, error, reload, updateSettings, probeOllama } = useAI();
  if (!settings) return <section className="ai-settings"><header><span>AI CORE</span><i data-state={error ? "offline" : "online"}>{loading ? "CARREGANDO" : "FALHA LOCAL"}</i></header><p>{error || "Recuperando configurações locais..."}</p>{!loading && <div className="ai-settings__recovery"><button type="button" onClick={() => void reload()}>RECARREGAR CONFIGURAÇÕES</button></div>}</section>;
  return <AISettingsForm initial={settings} status={status} updateSettings={updateSettings} probeOllama={probeOllama} />;
}

function AISettingsForm({ initial, status, updateSettings, probeOllama }: {
  initial: AISettings;
  status: OllamaStatus | null;
  updateSettings: ReturnType<typeof useAI>["updateSettings"];
  probeOllama: ReturnType<typeof useAI>["probeOllama"];
}) {
  const initialModels = status?.available ? status.models : [];
  const [form, setForm] = useState<AISettingsInput>({ endpoint: initial.endpoint, model: status?.available ? modelAfterProbe(initial.model, initialModels) : initial.model, contextMessageLimit: initial.contextMessageLimit, timeoutSeconds: initial.timeoutSeconds });
  const [availableModels, setAvailableModels] = useState<string[]>(initialModels);
  const [verifiedEndpoint, setVerifiedEndpoint] = useState<string | null>(status?.available ? normalizeOllamaEndpoint(initial.endpoint) : null);
  const [busy, setBusy] = useState<"probe" | "save" | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [messageKind, setMessageKind] = useState<"success" | "error" | "info">("info");

  function changeEndpoint(endpoint: string) {
    setForm((current) => ({ ...current, endpoint }));
    setAvailableModels([]);
    setVerifiedEndpoint(null);
    setMessage(null);
  }

  async function discoverModels() {
    setBusy("probe");
    setMessage(null);
    const endpoint = normalizeOllamaEndpoint(form.endpoint);
    try {
      const result = await probeOllama(endpoint, form.timeoutSeconds);
      if (!result.available) throw new Error(result.error || "O servidor Ollama não respondeu.");
      if (!result.models.length) {
        setAvailableModels([]);
        setVerifiedEndpoint(endpoint);
        setForm((current) => ({ ...current, model: "" }));
        setMessageKind("error");
        setMessage("Ollama conectado, mas nenhum modelo instalado foi encontrado.");
        return;
      }
      setAvailableModels(result.models);
      setVerifiedEndpoint(endpoint);
      setForm((current) => ({ ...current, model: modelAfterProbe(current.model, result.models) }));
      setMessageKind("success");
      setMessage(`${result.models.length} modelo(s) encontrado(s). Selecione o modelo que o Azriel deve usar.`);
    } catch (reason) {
      setAvailableModels([]);
      setVerifiedEndpoint(null);
      setMessageKind("error");
      setMessage(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setBusy(null);
    }
  }

  async function submit(event: FormEvent) {
    event.preventDefault();
    if (!canSaveAiSettings(form.endpoint, verifiedEndpoint, form.model)) return;
    setBusy("save");
    setMessage(null);
    try {
      await updateSettings({ ...form, endpoint: normalizeOllamaEndpoint(form.endpoint) });
      setMessageKind("success");
      setMessage("Configuração salva. AI Core e Command Center foram atualizados.");
    } catch (reason) {
      setMessageKind("error");
      setMessage(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setBusy(null);
    }
  }

  const saveEnabled = canSaveAiSettings(form.endpoint, verifiedEndpoint, form.model);
  return <section className="ai-settings"><header><span>CONEXÃO DO AI CORE</span><i data-state={verifiedEndpoint ? "online" : "offline"}>{verifiedEndpoint ? "ENDPOINT VERIFICADO" : "CONFIGURAÇÃO NECESSÁRIA"}</i></header>
    <form onSubmit={submit}>
      <label><span><strong>Provedor</strong><small>API Ollama protegida pelo backend local</small></span><input value="OLLAMA" disabled /></label>
      <label><span><strong>Endpoint</strong><small>Servidor acessível pela rede privada ou Tailscale</small></span><input value={form.endpoint} placeholder="http://servidor.ts.net:11434" onChange={(event) => changeEndpoint(event.target.value)} /></label>
      <div className="ai-settings__discovery">
        <div><strong>DESCOBERTA DE MODELOS</strong><small>Consulta o endpoint digitado sem salvar alterações.</small></div>
        <button type="button" onClick={() => void discoverModels()} disabled={busy !== null || !form.endpoint.trim()}>{busy === "probe" ? "BUSCANDO..." : "CONECTAR E BUSCAR MODELOS"}</button>
      </div>
      <label><span><strong>Modelo</strong><small>Seleção explícita entre os modelos encontrados</small></span><select value={form.model} disabled={!availableModels.length || busy !== null} onChange={(event) => setForm({ ...form, model: event.target.value })}><option value="">{availableModels.length ? "SELECIONE UM MODELO" : "CONECTE AO OLLAMA PRIMEIRO"}</option>{availableModels.map((model) => <option value={model} key={model}>{model}</option>)}</select></label>
      <label><span><strong>Contexto</strong><small>Últimas mensagens enviadas ao modelo</small></span><input type="number" min="1" max="20" value={form.contextMessageLimit} onChange={(event) => setForm({ ...form, contextMessageLimit: Number(event.target.value) })} /></label>
      <label><span><strong>Timeout</strong><small>Limite entre 5 e 180 segundos</small></span><input type="number" min="5" max="180" value={form.timeoutSeconds} onChange={(event) => setForm({ ...form, timeoutSeconds: Number(event.target.value) })} /></label>
      <div className="ai-settings__actions"><button disabled={busy !== null || !saveEnabled}>{busy === "save" ? "SALVANDO..." : "SALVAR CONFIGURAÇÃO"}</button><small>{saveEnabled ? "Endpoint verificado e modelo selecionado." : "Verifique o endpoint e selecione um modelo para salvar."}</small></div>
      {message && <p role="status" data-kind={messageKind}>{message}</p>}
    </form>
  </section>;
}
