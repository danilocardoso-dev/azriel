import { invokeDatabase } from "./tauri";

export interface DirEntry { name: string; path: string; kind: "pasta" | "arquivo"; size: number | null }
export interface Listing { path: string; entries: DirEntry[]; truncated: boolean }
export interface FileText { path: string; content: string; startLine: number; endLine: number; totalLines: number; truncated: boolean }
export interface SearchHit { path: string; line: number; text: string }
export interface WriteResult { path: string; created: boolean; backup: string | null; replacements: number }
export interface CommandResult { exitCode: number | null; stdout: string; stderr: string; timedOut: boolean; cancelled?: boolean; durationMs: number; cwd: string }
export interface ActionRecord { id: number; createdAt: string; kind: string; target: string; detail: string; ok: boolean }

/** Arquivos e comandos no disco C, executados pelo backend (com backup, Lixeira e histórico). */
export const workspaceRepository = {
  list: (path: string) => invokeDatabase<Listing>("fs_list", { path }),
  read: (path: string, startLine?: number | null, maxLines?: number | null) => invokeDatabase<FileText>("fs_read", { path, startLine: startLine ?? null, maxLines: maxLines ?? null }),
  search: (root: string, query: string, extension?: string | null) => invokeDatabase<SearchHit[]>("fs_search", { root, query, extension: extension ?? null }),
  write: (path: string, content: string, overwrite: boolean) => invokeDatabase<WriteResult>("fs_write", { path, content, overwrite }),
  edit: (path: string, oldText: string, newText: string, replaceAll: boolean) => invokeDatabase<WriteResult>("fs_edit", { path, oldText, newText, replaceAll }),
  mkdir: (path: string) => invokeDatabase<string>("fs_mkdir", { path }),
  remove: (path: string) => invokeDatabase<string>("fs_delete", { path }),
  run: (command: string, cwd: string, timeoutSeconds?: number | null) => invokeDatabase<CommandResult>("run_command", { command, cwd, timeoutSeconds: timeoutSeconds ?? null }),
  /** ESC: para os comandos em andamento (e tudo o que eles abriram). */
  cancel: () => invokeDatabase<number>("cancel_commands"),
  history: (limit = 100) => invokeDatabase<ActionRecord[]>("list_actions", { limit }),
};
