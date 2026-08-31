import { noteRepository } from "../repositories/noteRepository";
import type { NoteInput } from "../types";

function validate(input: NoteInput) {
  if (!input.content.trim()) throw new Error("Informe o conteúdo da nota.");
  const title = input.title?.trim() || null;
  return { ...input, title, content: input.content.trim() };
}

export const noteService = {
  list: noteRepository.list,
  get: noteRepository.get,
  save: (input: NoteInput) => noteRepository.save(validate(input)),
  createQuick: (content: string) => noteRepository.save(validate({ id: crypto.randomUUID(), title: null, content, status: "active", projectId: null, knowledgeAreaId: null })),
  archive: noteRepository.archive,
  restore: async (id: string) => {
    const note = await noteRepository.get(id);
    if (!note) throw new Error("Nota não encontrada.");
    return noteRepository.save(validate({ id: note.id, title: note.title, content: note.content, status: "active", projectId: note.projectId, knowledgeAreaId: note.knowledgeAreaId }));
  },
  remove: noteRepository.remove,
};
