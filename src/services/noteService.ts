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
  remove: noteRepository.remove,
};
