import { describe, expect, it } from "vitest";
import { canSaveAiSettings, modelAfterProbe, normalizeOllamaEndpoint, preferredGeminiModel } from "./aiSettingsFlow";

describe("AI settings connection flow", () => {
  it("normaliza o endpoint sem alterar host ou porta", () => {
    expect(normalizeOllamaEndpoint("  http://azriel-ai.ts.net:11434/// ")).toBe("http://azriel-ai.ts.net:11434");
  });

  it("preserva somente um modelo realmente retornado pelo endpoint consultado", () => {
    expect(modelAfterProbe("qwen3.5:4b", ["qwen3.5:4b", "qwen3.5:8b"])).toBe("qwen3.5:4b");
    expect(modelAfterProbe("qwen2.5:3b", ["qwen3.5:4b"])).toBe("");
  });

  it("escolhe um Gemini flash estável como padrão", () => {
    expect(preferredGeminiModel(["gemini-2.5-flash-lite", "gemini-2.5-flash-preview-09", "gemini-2.5-pro", "gemini-2.5-flash"])).toBe("gemini-2.5-flash");
    expect(preferredGeminiModel(["gemini-3-flash-preview", "gemini-2.5-pro"])).toBe("gemini-3-flash-preview");
    expect(preferredGeminiModel([])).toBe("");
  });

  it("exige endpoint verificado e modelo selecionado antes de salvar", () => {
    expect(canSaveAiSettings("http://server:11434/", "http://server:11434", "qwen3.5:4b")).toBe(true);
    expect(canSaveAiSettings("http://outro:11434", "http://server:11434", "qwen3.5:4b")).toBe(false);
    expect(canSaveAiSettings("http://server:11434", "http://server:11434", "")).toBe(false);
  });
});
