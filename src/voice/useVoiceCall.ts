import { useCallback, useEffect, useRef, useState } from "react";
import { Channel } from "@tauri-apps/api/core";
import { invokeDatabase } from "../repositories/tauri";
import { localDateKey } from "../services/dateService";
import { Microphone, Speaker } from "./audio";

export type CallPhase = "idle" | "connecting" | "listening" | "speaking" | "tool" | "ended";
export interface Caption { id: number; who: "you" | "az"; text: string }

type VoiceEvent =
  | { type: "ready" }
  | { type: "audio"; data: string }
  | { type: "inputText"; text: string }
  | { type: "outputText"; text: string }
  | { type: "interrupted" }
  | { type: "turnComplete" }
  | { type: "toolCall"; calls: Array<{ id: string; name: string; args: unknown }> }
  | { type: "closed"; reason: string }
  | { type: "error"; message: string };

export const SESSION_LIMIT_SECONDS = 15 * 60;
const messageOf = (error: unknown) => error instanceof Error ? error.message : String(error);

/** Junta pedaços de legenda na fala atual; troca de quem fala abre uma linha nova. */
export function appendCaption(captions: Caption[], who: Caption["who"], text: string, nextId: () => number): Caption[] {
  if (!text) return captions;
  const last = captions.at(-1);
  if (last && last.who === who) return [...captions.slice(0, -1), { ...last, text: `${last.text}${text}` }];
  return [...captions, { id: nextId(), who, text: text.trimStart() }].slice(-4);
}

export function useVoiceCall() {
  const [phase, setPhase] = useState<CallPhase>("idle");
  const [captions, setCaptions] = useState<Caption[]>([]);
  const [tool, setTool] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [startedAt, setStartedAt] = useState<number | null>(null);
  const [muted, setMuted] = useState(false);
  const mic = useRef<Microphone | null>(null);
  const speaker = useRef<Speaker | null>(null);
  const userLevel = useRef(0);
  const captionId = useRef(0);
  const phaseRef = useRef<CallPhase>("idle");
  const live = useRef(false);

  const changePhase = useCallback((next: CallPhase) => { phaseRef.current = next; setPhase(next); }, []);

  const cleanup = useCallback(() => {
    live.current = false;
    mic.current?.stop();
    speaker.current?.close();
    mic.current = null;
    speaker.current = null;
    userLevel.current = 0;
  }, []);

  // A voz roda sem ferramentas por enquanto; se o modelo pedir alguma, responde que não há.
  const runTools = useCallback(async (calls: Array<{ id: string; name: string; args?: unknown }>) => {
    for (const call of calls) {
      await invokeDatabase("voice_tool_response", { id: call.id, name: call.name, result: { error: "Sem ferramentas na voz por enquanto." } }).catch(() => undefined);
    }
  }, []);

  const handle = useCallback((event: VoiceEvent) => {
    switch (event.type) {
      case "ready": {
        const microphone = new Microphone();
        mic.current = microphone;
        void microphone.start(
          (data) => { if (live.current) void invokeDatabase("voice_send_audio", { data }).catch(() => undefined); },
          (value) => { userLevel.current = value; },
          (label) => setNotice(`O microfone "${label}" não está captando som. Confira se ele está ligado e escolhido como entrada no Windows.`),
        ).then((info) => {
          if (info.outputId) void speaker.current?.useOutput(info.outputId);
          setStartedAt(Date.now()); changePhase("listening");
        }).catch((error) => {
          setNotice(`Microfone indisponível: ${messageOf(error)}`);
          void invokeDatabase("voice_stop").catch(() => undefined);
        });
        break;
      }
      case "audio": speaker.current?.play(event.data); break;
      case "inputText": setCaptions((current) => appendCaption(current, "you", event.text, () => ++captionId.current)); break;
      case "outputText": setCaptions((current) => appendCaption(current, "az", event.text, () => ++captionId.current)); break;
      case "interrupted": speaker.current?.interrupt(); break;
      case "toolCall": void runTools(event.calls); break;
      case "error": setNotice(event.message); break;
      case "closed": cleanup(); setNotice((current) => current ?? event.reason); changePhase("ended"); break;
      case "turnComplete": break;
    }
  }, [changePhase, cleanup, runTools]);

  const start = useCallback(async () => {
    if (live.current) return;
    live.current = true;
    setCaptions([]); setNotice(null); setTool(null); setStartedAt(null); setMuted(false);
    changePhase("connecting");
    speaker.current = new Speaker();
    const events = new Channel<VoiceEvent>();
    events.onmessage = handle;
    try {
      await invokeDatabase("voice_start", { tools: [], date: localDateKey(), persona: null, events });
    } catch (error) {
      cleanup();
      setNotice(messageOf(error));
      changePhase("ended");
    }
  }, [changePhase, cleanup, handle]);

  const stop = useCallback(() => {
    void invokeDatabase("voice_stop").catch(() => undefined);
    cleanup();
    changePhase("idle");
  }, [changePhase, cleanup]);

  const toggleMute = useCallback(() => {
    setMuted((current) => { const next = !current; if (mic.current) mic.current.muted = next; return next; });
  }, []);

  // Níveis lidos a cada quadro pela animação, sem re-renderizar o React.
  const levels = useCallback(() => {
    const az = speaker.current?.level() ?? 0;
    const speaking = speaker.current?.speaking ?? false;
    const current = phaseRef.current;
    if (current === "listening" && speaking) changePhase("speaking");
    else if (current === "speaking" && !speaking) changePhase("listening");
    return { az, you: userLevel.current };
  }, [changePhase]);

  useEffect(() => () => { if (live.current) { void invokeDatabase("voice_stop").catch(() => undefined); cleanup(); } }, [cleanup]);

  return { phase, captions, tool, notice, startedAt, muted, start, stop, toggleMute, levels };
}
