import { useState } from "react";
import { useAI } from "../../contexts/useAI";
import type { AIEngine } from "../../types";

/** Troca o motor (local, Gemini ou OpenRouter) e o modelo sem sair do AI Core. */
export function EngineSwitcher({ disabled }: { disabled?: boolean }) {
  const { settings, remoteSettings, remoteModels, openRouter, status, engine, activeModel, setEngine, selectModel } = useAI();
  const [busy, setBusy] = useState(false);
  // No OpenRouter, o seletor mostra só os favoritos escolhidos em Configurações (o catálogo tem centenas).
  const models = engine === "openrouter" ? openRouter?.favorites ?? [] : engine === "gemini" ? remoteModels : status?.models ?? [];
  const current = activeModel || (engine === "ollama" ? settings?.model ?? "" : "");
  const options = current && !models.includes(current) ? [current, ...models] : models;
  const routerReady = Boolean(openRouter?.apiKeyConfigured && openRouter.model);

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
      <button type="button" data-active={engine === "openrouter"} disabled={locked || !routerReady} title={routerReady ? "Usar o OpenRouter (DeepSeek, Kimi e outros)" : "Cadastre a chave do OpenRouter e escolha um modelo em Configurações"} onClick={() => toggle("openrouter")}>OPENROUTER</button>
    </div>
    <select aria-label="Modelo" value={current} disabled={locked || !options.length} onChange={(event) => void run(() => selectModel(event.target.value))}>
      {!options.length && <option value="">SEM MODELOS</option>}
      {options.map((model) => <option value={model} key={model}>{model}</option>)}
    </select>
  </div>;
}
