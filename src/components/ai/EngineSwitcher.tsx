import { useState } from "react";
import { useAI } from "../../contexts/useAI";
import type { AIEngine } from "../../types";

/** Troca o motor (local, Gemini ou DeepSeek) e o modelo sem sair do AI Core. */
export function EngineSwitcher({ disabled }: { disabled?: boolean }) {
  const { settings, remoteSettings, remoteModels, deepSeek, deepSeekModels, status, engine, activeModel, setEngine, selectModel } = useAI();
  const [busy, setBusy] = useState(false);
  // Cada motor mostra os seus modelos (o do Gemini já vem enxuto: Flash, Flash Lite e Pro).
  const models = engine === "deepseek" ? deepSeekModels : engine === "gemini" ? remoteModels : status?.models ?? [];
  const current = activeModel || (engine === "ollama" ? settings?.model ?? "" : "");
  const options = current && !models.includes(current) ? [current, ...models] : models;
  const deepSeekReady = Boolean(deepSeek?.apiKeyConfigured && deepSeek.model);

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
      <button type="button" data-active={engine === "deepseek"} disabled={locked || !deepSeekReady} title={deepSeekReady ? "Usar o DeepSeek" : "Cadastre a chave do DeepSeek e escolha um modelo em Configurações"} onClick={() => toggle("deepseek")}>DEEPSEEK</button>
    </div>
    <select aria-label="Modelo" value={current} disabled={locked || !options.length} onChange={(event) => void run(() => selectModel(event.target.value))}>
      {!options.length && <option value="">SEM MODELOS</option>}
      {options.map((model) => <option value={model} key={model}>{model}</option>)}
    </select>
  </div>;
}
