import { describe, expect, it } from "vitest";
import { appendStudyAIContent } from "./studyAIText";

describe("explicit Study AI note insertion", () => {
  it("só altera o conteúdo quando a operação explícita é chamada", () => {
    const original = "Conteúdo humano";
    const generated = "Resumo proposto";
    expect(original).toBe("Conteúdo humano");
    expect(appendStudyAIContent(original, generated)).toBe("Conteúdo humano\n\nResumo proposto\n");
  });
});
