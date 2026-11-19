import { useState, type FormEvent } from "react";
import { useAI } from "../../contexts/useAI";

const messageOf = (reason: unknown) => reason instanceof Error ? reason.message : String(reason);

/** DeepSeek pela API oficial, com a chave do Senhor (platform.deepseek.com). */
export function DeepSeekSettingsPanel() {
  const { deepSeek, deepSeekModels, saveDeepSeek, refreshDeepSeekModels } = useAI();
  const [apiKey, setApiKey] = useState("");
  const [busy, setBusy] = useState<"save" | "remove" | "models" | "model" | null>(null);
  const [message, setMessage] = useState<{ kind: "success" | "error"; text: string } | null>(null);
  const configured = Boolean(deepSeek?.apiKeyConfigured);

  async function run(kind: NonNullable<typeof busy>, action: () => Promise<string>) {
    setBusy(kind); setMessage(null);
    try { setMessage({ kind: "success", text: await action() }); }
    catch (reason) { setMessage({ kind: "error", text: messageOf(reason) }); }
    finally { setBusy(null); }
  }

  function submit(event: FormEvent) {
    event.preventDefault();
    const key = apiKey.trim();
    if (!key || !deepSeek) return;
    void run("save", async () => {
      await saveDeepSeek({ enabled: false, model: deepSeek.model, apiKey: key });
      setApiKey("");
      const models = await refreshDeepSeekModels();
      // Sem modelo escolhido: começa pelo de conversa (o "chat"), que aceita ferramentas.
      if (!deepSeek.model && models.length) await saveDeepSeek({ enabled: false, model: models.find((model) => model.includes("chat")) ?? models[0] });
      return "Chave salva. Ligue o DeepSeek pelo seletor no topo do AI Core.";
    });
  }

  const options = deepSeek?.model && !deepSeekModels.includes(deepSeek.model) ? [deepSeek.model, ...deepSeekModels] : deepSeekModels;

  return <section className="ai-settings"><header><span>MOTOR REMOTO // DEEPSEEK</span><i data-state={configured ? "online" : "offline"}>{!deepSeek ? "CARREGANDO" : deepSeek.enabled ? "LIGADO" : configured ? "CHAVE CONFIGURADA" : "SEM CHAVE"}</i></header>
    <form onSubmit={submit}>
      <label><span><strong>Chave da API</strong><small>platform.deepseek.com. Fica só no banco local e nunca é exibida de volta.</small></span><input type="password" autoComplete="off" spellCheck={false} value={apiKey} placeholder={configured ? "•••••••• (substituir)" : "Cole a chave aqui (sk-...)"} onChange={(event) => setApiKey(event.target.value)} /></label>
      <label><span><strong>Modelo</strong><small>O "chat" é o indicado para o Azriel agir e programar.</small></span>
        <select value={deepSeek?.model ?? ""} disabled={!configured || busy !== null || !options.length} onChange={(event) => { const model = event.target.value; if (deepSeek) void run("model", async () => { await saveDeepSeek({ enabled: deepSeek.enabled, model }); return `Modelo do DeepSeek: ${model}.`; }); }}>
          {!options.length && <option value="">{configured ? "Atualize os modelos" : "Cadastre a chave"}</option>}
          {options.map((model) => <option key={model} value={model}>{model}</option>)}
        </select>
      </label>
      <div className="ai-settings__actions"><button disabled={busy !== null || !apiKey.trim()}>{busy === "save" ? "SALVANDO..." : "SALVAR CHAVE"}</button><small>O DeepSeek cobra por uso, dos créditos da sua conta. Com ele ligado, suas perguntas e os dados consultados vão para os servidores da DeepSeek.</small></div>
      {configured && <div className="ai-settings__discovery">
        <div><strong>MODELOS DISPONÍVEIS</strong><small>{deepSeekModels.length ? `${deepSeekModels.length} modelo(s). Troque aqui ou no seletor do AI Core.` : "Nenhum modelo carregado ainda."}</small></div>
        <button type="button" disabled={busy !== null} onClick={() => void run("models", async () => `${(await refreshDeepSeekModels()).length} modelo(s) encontrado(s).`)}>{busy === "models" ? "BUSCANDO..." : "ATUALIZAR MODELOS"}</button>
        <button type="button" disabled={busy !== null} onClick={() => void run("remove", async () => { await saveDeepSeek({ enabled: false, model: "", apiKey: "" }); return "Chave removida. O DeepSeek foi desligado."; })}>{busy === "remove" ? "REMOVENDO..." : "REMOVER CHAVE"}</button>
      </div>}
      {message && <p role="status" data-kind={message.kind}>{message.text}</p>}
    </form>
  </section>;
}
