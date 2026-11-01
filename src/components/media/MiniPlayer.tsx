import { useEffect, useRef, useState, type FormEvent, type PointerEvent } from "react";
import { embedUrl, parseYouTube, type YouTubeTarget } from "./youtube";
import "../../styles/mini-player.css";

type Size = "s" | "m" | "l";
interface Saved { link: string; size: Size; right: number; bottom: number }

const STORAGE_KEY = "azriel.miniPlayer";
const WIDTHS: Record<Size, number> = { s: 320, m: 480, l: 640 };
const defaults: Saved = { link: "", size: "m", right: 24, bottom: 24 };

function load(): Saved {
  try { return { ...defaults, ...JSON.parse(localStorage.getItem(STORAGE_KEY) ?? "{}") }; } catch { return defaults; }
}

/**
 * Vídeo ou aula do YouTube num canto da janela do Azriel. Fica no shell, então continua
 * tocando ao trocar de módulo; recolher só esconde a imagem, o áudio segue.
 */
export function MiniPlayer({ onClose }: { onClose: () => void }) {
  const [saved, setSaved] = useState<Saved>(load);
  const [target, setTarget] = useState<YouTubeTarget | null>(() => parseYouTube(load().link));
  const [draft, setDraft] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [collapsed, setCollapsed] = useState(false);
  const [dragging, setDragging] = useState(false);
  const drag = useRef<{ x: number; y: number; right: number; bottom: number } | null>(null);

  useEffect(() => {
    try { localStorage.setItem(STORAGE_KEY, JSON.stringify(saved)); } catch { /* preferência local, opcional */ }
  }, [saved]);

  function play(event: FormEvent) {
    event.preventDefault();
    const next = parseYouTube(draft);
    if (!next) { setError("Cole um link do YouTube (vídeo, playlist ou aula)."); return; }
    setTarget(next);
    setSaved((current) => ({ ...current, link: draft.trim() }));
    setDraft(""); setError(null); setCollapsed(false);
  }

  function startDrag(event: PointerEvent<HTMLElement>) {
    if ((event.target as HTMLElement).closest("button")) return;
    event.currentTarget.setPointerCapture(event.pointerId);
    drag.current = { x: event.clientX, y: event.clientY, right: saved.right, bottom: saved.bottom };
    setDragging(true);
  }
  function moveDrag(event: PointerEvent<HTMLElement>) {
    const start = drag.current;
    if (!start) return;
    const width = WIDTHS[saved.size];
    const right = Math.min(Math.max(8, start.right - (event.clientX - start.x)), window.innerWidth - width - 8);
    const bottom = Math.min(Math.max(8, start.bottom - (event.clientY - start.y)), window.innerHeight - 60);
    setSaved((current) => ({ ...current, right, bottom }));
  }
  function endDrag() { drag.current = null; setDragging(false); }

  return <aside
    className="mini-player"
    data-size={saved.size}
    data-collapsed={collapsed}
    data-dragging={dragging}
    style={{ right: saved.right, bottom: saved.bottom, width: WIDTHS[saved.size] }}
    aria-label="Mini player do YouTube"
  >
    <header onPointerDown={startDrag} onPointerMove={moveDrag} onPointerUp={endDrag} onPointerCancel={endDrag} title="Arraste para mover">
      <span><i aria-hidden="true" />YOUTUBE</span>
      <div>
        {target && <button type="button" onClick={() => { setTarget(null); setCollapsed(false); }} title="Trocar vídeo">⇄</button>}
        {(["s", "m", "l"] as Size[]).map((size) => <button key={size} type="button" data-active={saved.size === size} onClick={() => setSaved((current) => ({ ...current, size, right: Math.min(current.right, window.innerWidth - WIDTHS[size] - 8) }))} title={size === "s" ? "Pequeno" : size === "m" ? "Médio" : "Grande"}>{size.toUpperCase()}</button>)}
        <button type="button" onClick={() => setCollapsed((current) => !current)} title={collapsed ? "Mostrar vídeo" : "Recolher (o áudio continua)"}>{collapsed ? "▢" : "—"}</button>
        <button type="button" onClick={onClose} title="Fechar e parar">×</button>
      </div>
    </header>
    {target
      ? <div className="mini-player__frame">
        <iframe
          key={embedUrl(target)}
          src={embedUrl(target)}
          title="YouTube"
          allow="autoplay; encrypted-media; picture-in-picture; fullscreen"
          referrerPolicy="strict-origin-when-cross-origin"
          allowFullScreen
        />
      </div>
      : <form className="mini-player__form" onSubmit={play}>
        <input value={draft} onChange={(event) => { setDraft(event.target.value); setError(null); }} placeholder="Cole o link do vídeo, aula ou playlist" autoFocus />
        <button type="submit">TOCAR</button>
        {error && <small>{error}</small>}
      </form>}
  </aside>;
}
