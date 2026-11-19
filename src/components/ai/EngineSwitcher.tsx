import { useState } from "react";
import { useAI } from "../../contexts/useAI";
import { modelGroups, optionValue, parseOption } from "./modelGroups";

/** Seletor único: escolher um modelo já troca o motor (Gemini, DeepSeek ou local). */
export function EngineSwitcher({ disabled }: { disabled?: boolean }) {
  const { settings, remoteSettings, remoteModels, deepSeek, deepSeekModels, status, engine, activeModel, chooseModel } = useAI();
  const [busy, setBusy] = useState(false);
  const groups = modelGroups({
    gemini: { configured: Boolean(remoteSettings?.apiKeyConfigured), models: remoteModels, current: remoteSettings?.model ?? "" },
    deepseek: { configured: Boolean(deepSeek?.apiKeyConfigured), models: deepSeekModels, current: deepSeek?.model ?? "" },
    local: { available: Boolean(status?.available), models: status?.models ?? [], current: settings?.model ?? "" },
    engine,
  });
  const value = activeModel ? optionValue(engine, activeModel) : "";

  async function change(next: string) {
    const choice = parseOption(next);
    if (!choice || busy) return;
    setBusy(true);
    try { await chooseModel(choice.engine, choice.model); } catch { /* O contexto exibe a falha. */ }
    finally { setBusy(false); }
  }

  return <select className="ai-model-picker" aria-label="Modelo do Azriel" title="Motor e modelo do Azriel" value={value} disabled={disabled || busy || !groups.length} onChange={(event) => void change(event.target.value)}>
    {!groups.length && <option value="">Configure um motor em Configurações</option>}
    {groups.map((group) => <optgroup key={group.engine} label={group.label}>
      {group.models.map((model) => <option key={model} value={optionValue(group.engine, model)}>{model}</option>)}
    </optgroup>)}
  </select>;
}
