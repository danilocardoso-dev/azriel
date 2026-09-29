export function normalizeOllamaEndpoint(endpoint: string) {
  return endpoint.trim().replace(/\/+$/, "");
}

export function modelAfterProbe(currentModel: string, availableModels: string[]) {
  return availableModels.includes(currentModel) ? currentModel : "";
}

export function canSaveAiSettings(endpoint: string, verifiedEndpoint: string | null, model: string) {
  return Boolean(model.trim()) && normalizeOllamaEndpoint(endpoint) === verifiedEndpoint;
}
