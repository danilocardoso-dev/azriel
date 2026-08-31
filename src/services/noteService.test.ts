import { afterEach, describe, expect, it, vi } from "vitest";
import { noteRepository } from "../repositories/noteRepository";
import { noteService } from "./noteService";

describe("noteService", () => {
  afterEach(() => vi.restoreAllMocks());

  it("cria captura rápida como nota ativa", async () => {
    const save = vi.spyOn(noteRepository, "save").mockImplementation(async (input) => ({ ...input, createdAt: "", updatedAt: "" }));
    const created = await noteService.createQuick("  Ideia sobre MQTT  ");
    expect(created.content).toBe("Ideia sobre MQTT");
    expect(created.status).toBe("active");
    save.mockRestore();
  });

  it("rejeita nota vazia", () => {
    expect(() => noteService.createQuick(" ")).toThrow(/conteúdo/);
  });

  it("restaura uma nota arquivada preservando seus vínculos", async () => {
    vi.spyOn(noteRepository, "get").mockResolvedValue({ id: "note-1", title: "Ideia", content: "Revisar MQTT", status: "archived", projectId: "arc", knowledgeAreaId: "iot", createdAt: "", updatedAt: "" });
    const save = vi.spyOn(noteRepository, "save").mockImplementation(async (input) => ({ ...input, createdAt: "", updatedAt: "" }));
    const restored = await noteService.restore("note-1");
    expect(restored.status).toBe("active");
    expect(save).toHaveBeenCalledWith(expect.objectContaining({ projectId: "arc", knowledgeAreaId: "iot" }));
  });

  it("informa quando a nota a restaurar não existe", async () => {
    vi.spyOn(noteRepository, "get").mockResolvedValue(null);
    await expect(noteService.restore("missing")).rejects.toThrow(/não encontrada/);
  });
});
