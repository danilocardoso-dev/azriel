import { describe, expect, it } from "vitest";
import { modules } from "./system";

describe("system modules", () => {
  it("expõe Estudos e retira os módulos descontinuados da navegação", () => {
    expect(modules.find((module) => module.id === "studies")?.label).toBe("Estudos");
    expect(modules.map((module) => module.label)).not.toEqual(
      expect.arrayContaining(["Engineering View", "Mapa Stark", "Formação", "Sistema"]),
    );
  });
});
