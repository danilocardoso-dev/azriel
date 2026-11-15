import type { AzrielIdentity, Memory, MemoryInput } from "../types";
import { invokeDatabase } from "./tauri";

export const MEMORY_CHANGED = "azriel:memory-changed";
const changed = <T,>(value: T) => { window.dispatchEvent(new Event(MEMORY_CHANGED)); return value; };

export const memoryRepository = {
  identity: () => invokeDatabase<AzrielIdentity>("get_azriel_identity"),
  saveIdentity: (content: string) => invokeDatabase<AzrielIdentity>("save_azriel_identity", { content }),
  resetIdentity: () => invokeDatabase<AzrielIdentity>("reset_azriel_identity"),
  list: () => invokeDatabase<Memory[]>("list_memories"),
  search: (query: string) => invokeDatabase<Memory[]>("search_memories", { query }),
  save: async (input: MemoryInput) => changed(await invokeDatabase<Memory>("save_memory", { input: { ...input, conversationId: input.conversationId ?? null } })),
  remove: async (id: string) => changed(await invokeDatabase<void>("delete_memory", { id })),
};
