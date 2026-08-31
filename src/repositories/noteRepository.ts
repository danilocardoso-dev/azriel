import type { Note, NoteInput } from "../types";
import { invokeDatabase } from "./tauri";

export const noteRepository = {
  list: (includeArchived = false) => invokeDatabase<Note[]>("list_notes", { includeArchived }),
  get: (id: string) => invokeDatabase<Note | null>("get_note", { id }),
  save: (input: NoteInput) => invokeDatabase<Note>("save_note", { input }),
  archive: (id: string) => invokeDatabase<Note>("archive_note", { id }),
  remove: (id: string) => invokeDatabase<void>("delete_note", { id }),
};
