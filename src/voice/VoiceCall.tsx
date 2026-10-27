import { useEffect, useState } from "react";
import { Reactor } from "../components/azriel/Reactor";
import { SESSION_LIMIT_SECONDS, type useVoiceCall } from "./useVoiceCall";
import "../styles/voice.css";

const statusLabel = { idle: "", connecting: "ESTABELECENDO LINK", listening: "OUVINDO", speaking: "AZRIEL FALANDO", tool: "CONSULTANDO", ended: "CHAMADA ENCERRADA" } as const;
const clock = (seconds: number) => `${String(Math.floor(seconds / 60)).padStart(2, "0")}:${String(Math.floor(seconds % 60)).padStart(2, "0")}`;

/** Tela da chamada: núcleo que reage às duas vozes, legendas ao vivo e controles mínimos. */
export function VoiceCall({ call, onClose }: { call: ReturnType<typeof useVoiceCall>; onClose: () => void }) {
  const { phase, captions, tool, notice, startedAt, muted, stop, toggleMute, levels, start } = call;
  const [now, setNow] = useState(() => Date.now());

  useEffect(() => {
    const timer = window.setInterval(() => setNow(Date.now()), 1000);
    return () => window.clearInterval(timer);
  }, []);

  useEffect(() => {
    const keys = (event: KeyboardEvent) => { if (event.key === "Escape") { stop(); onClose(); } };
    window.addEventListener("keydown", keys);
    return () => window.removeEventListener("keydown", keys);
  }, [onClose, stop]);

  const elapsed = startedAt ? Math.max(0, (now - startedAt) / 1000) : 0;
  const ended = phase === "ended";
  return <div className="voice-call" data-phase={phase} role="dialog" aria-label="Chamada de voz com o Azriel">
    <header className="voice-call__top">
      <span>AZRIEL // LINK DE VOZ</span>
      <span title="O Google limita cada sessão de áudio a 15 minutos">{startedAt ? `${clock(elapsed)} / ${clock(SESSION_LIMIT_SECONDS)}` : "—"}</span>
    </header>

    <Reactor levels={levels} className="voice-stage">
      <div className="voice-stage__status">
        <strong>{phase === "tool" && tool ? `CONSULTANDO ${tool.toUpperCase()}` : statusLabel[phase]}</strong>
        {muted && !ended && <small>MICROFONE MUDO</small>}
      </div>
    </Reactor>

    <div className="voice-captions" aria-live="polite">
      {captions.map((caption) => <p key={caption.id} data-who={caption.who}><b>{caption.who === "you" ? "VOCÊ" : "AZRIEL"}</b>{caption.text}</p>)}
      {!captions.length && !notice && phase === "listening" && <p className="voice-captions__hint">Pode falar. Interrompa quando quiser.</p>}
      {notice && <p className="voice-captions__notice">{notice}</p>}
    </div>

    <footer className="voice-call__controls">
      {ended
        ? <>
          <button type="button" className="voice-btn" onClick={() => void start()}>LIGAR DE NOVO</button>
          <button type="button" className="voice-btn" onClick={onClose}>FECHAR</button>
        </>
        : <>
          <button type="button" className="voice-btn" data-active={muted} onClick={toggleMute} disabled={phase === "connecting"}>{muted ? "ATIVAR MICROFONE" : "MUTAR"}</button>
          <button type="button" className="voice-btn voice-btn--end" onClick={() => { stop(); onClose(); }} aria-label="Encerrar chamada">ENCERRAR</button>
        </>}
    </footer>
    <small className="voice-call__privacy">Gemini Live · o áudio é processado pelo Google · Esc encerra</small>
  </div>;
}
