import { useEffect, useState, type FormEvent } from "react";
import { aiRepository } from "../../repositories/aiRepository";
import type { RemoteAISettings } from "../../types";

const messageOf = (reason: unknown) => reason instanceof Error ? reason.message : String(reason);

/** A chamada de voz usa o Gemini Live do Google; este painel guarda só a chave dela. */
export function VoiceKeyPanel() {
  const [settings, setSettings] = useState<RemoteAISettings | null>(null);
  const [apiKey, setApiKey] = useState("");
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<{ kind: "success" | "error"; text: string } | null>(null);
  const configured = Boolean(settings?.apiKeyConfigured);

  useEffect(() => { void aiRepository.getRemoteSettings().then(setSettings).catch((reason) => setMessage({ kind: "error", text: messageOf(reason) })); }, []);

  async function save(key: string, done: string) {
    if (!settings) return;
    setBusy(true); setMessage(null);
    try { setSettings(await aiRepository.updateRemoteSettings({ enabled: false, model: settings.model, apiKey: key })); setApiKey(""); setMessage({ kind: "success", text: done }); }
    catch (reason) { setMessage({ kind: "error", text: messageOf(reason) }); }
    finally { setBusy(false); }
  }

  function submit(event: FormEvent) {
    event.preventDefault();
    if (apiKey.trim()) void save(apiKey.trim(), "Chave salva. A chamada de voz já pode usar.");
  }

  return <section className="ai-settings"><header><span>CHAMADA DE VOZ // CHAVE DO GOOGLE</span><i data-state={configured ? "online" : "offline"}>{!settings ? "CARREGANDO" : configured ? "CHAVE CONFIGURADA" : "SEM CHAVE"}</i></header>
    <form onSubmit={submit}>
      <label><span><strong>Chave da API</strong><small>Google AI Studio. Usada só pela chamada de voz (Ctrl+Espaço). Fica no banco local e nunca é exibida de volta.</small></span><input type="password" autoComplete="off" spellCheck={false} value={apiKey} placeholder={configured ? "•••••••• (substituir)" : "Cole a chave aqui"} onChange={(event) => setApiKey(event.target.value)} /></label>
      <div className="ai-settings__actions">
        <button disabled={busy || !apiKey.trim()}>{busy ? "SALVANDO..." : "SALVAR CHAVE"}</button>
        {configured && <button type="button" disabled={busy} onClick={() => void save("", "Chave removida. A chamada de voz fica desligada.")}>REMOVER CHAVE</button>}
      </div>
      {message && <p role="status" data-kind={message.kind}>{message.text}</p>}
    </form>
  </section>;
}
