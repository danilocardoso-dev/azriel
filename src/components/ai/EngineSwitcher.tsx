import { useState } from "react";
import { useAI } from "../../contexts/useAI";
import type { AIEngine } from "../../types";

/** Liga/desliga o Gemini e troca o modelo do motor ativo sem sair do AI Core. */
export function EngineSwitcher({ disabled }: { disabled?: boolean }) {
  const { settings, remoteSettings, remoteModels, status, engine, setEngine, selectModel } = useAI();
  const [busy, setBusy] = useState(false);
  const models = engine === "gemini" ? remoteModels : status?.models ?? [];
  const current = engine === "gemini" ? remoteSettings?.model ?? "" : settings?.model ?? "";
  const options = current && !models.includes(current) ? [current, ...models] : models;

  async function run(action: () => Promise<void>) {
    if (busy) return;
    setBusy(true);
    try { await action(); } catch { /* O contexto exibe a falha. */ }
    finally { setBusy(false); }
  }

  const toggle = (next: AIEngine) => { if (next !== engine) void run(() => setEngine(next)); };
  const locked = disabled || busy;
  return <div className="ai-engine" role="group" aria-label="Motor do Azriel">
    <div className="ai-engine__toggle">
      <button type="button" data-active={engine === "ollama"} disabled={locked} onClick={() => toggle("ollama")}>LOCAL</button>
      <button type="button" data-active={engine === "gemini"} disabled={locked || !remoteSettings?.apiKeyConfigured} title={remoteSettings?.apiKeyConfigured ? "Usar o Gemini" : "Cadastre a chave do Gemini em Configurações"} onClick={() => toggle("gemini")}>GEMINI</button>
    </div>
    <select aria-label="Modelo" value={current} disabled={locked || !options.length} onChange={(event) => void run(() => selectModel(event.target.value))}>
      {!options.length && <option value="">SEM MODELOS</option>}
      {options.map((model) => <option value={model} key={model}>{model}</option>)}
    </select>
  </div>;
}
