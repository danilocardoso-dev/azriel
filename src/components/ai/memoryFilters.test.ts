import { describe, expect, it } from "vitest";
import type { Memory } from "../../types";
import { filterMemories } from "./memoryFilters";

const memory = (id: string, content: string, category: Memory["category"]): Memory => ({ id, content, category, source: "azriel", conversationId: null, createdAt: "", updatedAt: "" });
const all = [memory("1", "O Senhor prefere café", "preferencia"), memory("2", "Projeto Azriel em Tauri", "projeto"), memory("3", "Aniversário em 25/12", "operador")];

describe("filtro de memórias", () => {
  it("filtra por categoria e por texto, sem ligar para acento ou maiúscula", () => {
    expect(filterMemories(all, "all", "")).toHaveLength(3);
    expect(filterMemories(all, "projeto", "").map((m) => m.id)).toEqual(["2"]);
    expect(filterMemories(all, "all", "CAFE").map((m) => m.id)).toEqual(["1"]);
    expect(filterMemories(all, "operador", "café")).toHaveLength(0);
  });
});
