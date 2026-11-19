import { describe, expect, it } from "vitest";
import { modelGroups, optionValue, parseOption, type ModelSources } from "./modelGroups";

const sources = (patch: Partial<ModelSources> = {}): ModelSources => ({
  gemini: { configured: true, models: ["gemini-3.8-flash", "gemini-3.8-flash-lite", "gemini-3-pro"], current: "gemini-3.8-flash" },
  deepseek: { configured: true, models: ["deepseek-chat", "deepseek-reasoner"], current: "deepseek-chat" },
  local: { available: false, models: [], current: "qwen3:8b" },
  engine: "gemini",
  ...patch,
});

describe("seletor único de motor e modelo", () => {
  it("agrupa por provedor e esconde o local desligado", () => {
    expect(modelGroups(sources()).map((group) => group.label)).toEqual(["Gemini", "DeepSeek"]);
  });

  it("mostra só provedores configurados", () => {
    const groups = modelGroups(sources({ deepseek: { configured: false, models: [], current: "" } }));
    expect(groups.map((group) => group.engine)).toEqual(["gemini"]);
  });

  it("o motor em uso sempre aparece, mesmo desligado, com o modelo atual", () => {
    const groups = modelGroups(sources({ engine: "ollama" }));
    expect(groups.at(-1)).toEqual({ engine: "ollama", label: "Local (Ollama desligado)", models: ["qwen3:8b"] });
  });

  it("modelo atual que não está na lista entra no topo do grupo", () => {
    const groups = modelGroups(sources({ gemini: { configured: true, models: ["gemini-3.8-flash"], current: "gemini-2.5-pro" } }));
    expect(groups[0].models).toEqual(["gemini-2.5-pro", "gemini-3.8-flash"]);
  });

  it("cada opção guarda motor e modelo", () => {
    expect(parseOption(optionValue("deepseek", "deepseek-chat"))).toEqual({ engine: "deepseek", model: "deepseek-chat" });
    expect(parseOption(optionValue("ollama", "qwen3:8b"))).toEqual({ engine: "ollama", model: "qwen3:8b" });
    expect(parseOption("hacker::x")).toBeNull();
    expect(parseOption("gemini::")).toBeNull();
  });
});
