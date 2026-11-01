import { describe, expect, it } from "vitest";
import { rememberSearch } from "./searchHistory";

describe("histórico de buscas", () => {
  it("põe a mais recente no topo, sem repetir e sem passar do limite", () => {
    expect(rememberSearch(["coldplay", "lofi"], "Lofi")).toEqual(["Lofi", "coldplay"]);
    expect(rememberSearch(["a", "b", "c"], "d", 3)).toEqual(["d", "a", "b"]);
    expect(rememberSearch(["a"], "   ")).toEqual(["a"]);
  });
});
