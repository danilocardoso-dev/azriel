export function normalizeOllamaEndpoint(endpoint: string) {
  return endpoint.trim().replace(/\/+$/, "");
}

export function modelAfterProbe(currentModel: string, availableModels: string[]) {
  return availableModels.includes(currentModel) ? currentModel : "";
}

/** Ao ligar o Gemini sem modelo escolhido: prefere um "flash" estável (rápido e com cota gratuita maior). */
export function preferredGeminiModel(models: string[]) {
  const stable = models.filter((model) => !/preview|exp|lite/.test(model));
  return stable.find((model) => model.includes("flash")) ?? models.find((model) => model.includes("flash")) ?? models[0] ?? "";
}

export function canSaveAiSettings(endpoint: string, verifiedEndpoint: string | null, model: string) {
  return Boolean(model.trim()) && normalizeOllamaEndpoint(endpoint) === verifiedEndpoint;
}
