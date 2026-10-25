import { useState, type FormEvent } from "react";
import { useAI } from "../../contexts/useAI";

export function GeminiSettingsPanel() {
  const { remoteSettings, remoteModels, saveRemoteKey, removeRemoteKey, refreshRemoteModels } = useAI();
  const [apiKey, setApiKey] = useState("");
  const [busy, setBusy] = useState<"save" | "remove" | "models" | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [messageKind, setMessageKind] = useState<"success" | "error">("success");
  const configured = Boolean(remoteSettings?.apiKeyConfigured);

  async function run(kind: "save" | "remove" | "models", action: () => Promise<string>) {
    setBusy(kind);
    setMessage(null);
    try { setMessage(await action()); setMessageKind("success"); }
    catch (reason) { setMessage(reason instanceof Error ? reason.message : String(reason)); setMessageKind("error"); }
    finally { setBusy(null); }
  }

  function submit(event: FormEvent) {
    event.preventDefault();
    const key = apiKey.trim();
    if (!key) return;
    void run("save", async () => { await saveRemoteKey(key); setApiKey(""); return "Chave salva. Ligue o Gemini pelo seletor no topo do AI Core."; });
  }

  return <section className="ai-settings"><header><span>MOTOR REMOTO // GEMINI</span><i data-state={configured ? "online" : "offline"}>{configured ? "CHAVE CONFIGURADA" : "SEM CHAVE"}</i></header>
    <form onSubmit={submit}>
      <label><span><strong>Chave da API</strong><small>Google AI Studio. Fica só no banco local e nunca é exibida de volta.</small></span><input type="password" autoComplete="off" spellCheck={false} value={apiKey} placeholder={configured ? "•••••••• (substituir)" : "Cole a chave aqui"} onChange={(event) => setApiKey(event.target.value)} /></label>
      <div className="ai-settings__actions"><button disabled={busy !== null || !apiKey.trim()}>{busy === "save" ? "SALVANDO..." : "SALVAR CHAVE"}</button><small>Com o Gemini ligado, suas perguntas e os dados consultados pelas ferramentas vão para o Google. No plano gratuito, o Google pode usá-los para melhorar os modelos.</small></div>
      {configured && <div className="ai-settings__discovery">
        <div><strong>MODELOS DISPONÍVEIS</strong><small>{remoteModels.length ? `${remoteModels.length} modelo(s). Troque pelo seletor no AI Core.` : "Nenhum modelo carregado ainda."}</small></div>
        <button type="button" disabled={busy !== null} onClick={() => void run("models", async () => `${(await refreshRemoteModels()).length} modelo(s) encontrado(s).`)}>{busy === "models" ? "BUSCANDO..." : "ATUALIZAR MODELOS"}</button>
        <button type="button" disabled={busy !== null} onClick={() => void run("remove", async () => { await removeRemoteKey(); return "Chave removida. O Azriel voltou ao motor local."; })}>{busy === "remove" ? "REMOVENDO..." : "REMOVER CHAVE"}</button>
      </div>}
      {message && <p role="status" data-kind={messageKind}>{message}</p>}
    </form>
  </section>;
}
