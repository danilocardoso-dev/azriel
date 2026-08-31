import { invoke } from "@tauri-apps/api/core";

export async function invokeDatabase<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try { return await invoke<T>(command, args); }
  catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    throw new Error(message.includes("__TAURI_INTERNALS__")
      ? "O banco local está disponível apenas no aplicativo Azriel. Execute npm run tauri dev."
      : message, { cause: error });
  }
}
