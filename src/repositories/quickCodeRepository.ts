import { Channel } from "@tauri-apps/api/core";
import { invokeDatabase } from "./tauri";

type QuickEvent = { type: "token"; text: string } | { type: "done"; tokens: number; tokensPerSecond: number; seconds: number };

/** Geração em tempo real pelo Ollama local (programação rápida). */
export const quickCodeRepository = {
  stream(endpoint: string, model: string, system: string, prompt: string, onToken: (text: string) => void, onDone?: (stats: { tokens: number; tokensPerSecond: number; seconds: number }) => void) {
    const events = new Channel<QuickEvent>();
    events.onmessage = (event) => {
      if (event.type === "token") onToken(event.text);
      else onDone?.(event);
    };
    return invokeDatabase<string>("quick_code_stream", { endpoint, model, system, prompt, events });
  },
  cancel: () => invokeDatabase<void>("quick_code_cancel"),
};
