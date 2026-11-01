import { useCallback, useEffect, useState } from "react";
import { mediaRepository } from "../../repositories/mediaRepository";
import { spotifyRepository, type SpotifyPlayer } from "../../repositories/spotifyRepository";
import { AZRIEL_DEVICE, spotifyControl } from "../../music/controls";
import type { MediaAction, NowPlaying } from "../../types";
import { chooseNowPlaying } from "./nowPlaying";
import "../../styles/now-playing.css";

const messageOf = (reason: unknown) => reason instanceof Error ? reason.message : String(reason);

/**
 * "Tocando agora": no Azriel o estado vem dos eventos do player embutido; o Spotify de outros aparelhos é lido uma vez
 * (e depois de cada comando), sem consulta periódica. O Windows é local e não passa pela API.
 */
export function NowPlayingBar({ onOpenYouTube, onOpenMusic, azrielDeviceId, azrielState }: { onOpenYouTube: () => void; onOpenMusic: () => void; azrielDeviceId: string | null; azrielState: SpotifyPlayer | null }) {
  const [windows, setWindows] = useState<NowPlaying | null>(null);
  const [fetched, setFetched] = useState<SpotifyPlayer | null>(null);
  const [spotifyOn, setSpotifyOn] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const spotify = azrielState ?? fetched;

  const refreshWindows = useCallback(async () => {
    if (document.hidden) return;
    try { setWindows(await mediaRepository.nowPlaying()); } catch { setWindows(null); }
  }, []);

  const refreshSpotify = useCallback(async () => {
    try { setFetched(await spotifyRepository.player()); setError(null); } catch (reason) { setError(messageOf(reason)); }
  }, []);

  useEffect(() => {
    const first = window.setTimeout(() => void spotifyRepository.status().then((status) => setSpotifyOn(status.connected)).catch(() => undefined), 0);
    return () => window.clearTimeout(first);
  }, []);

  useEffect(() => {
    if (!spotifyOn) return;
    const first = window.setTimeout(() => void refreshSpotify(), 0);
    return () => window.clearTimeout(first);
  }, [spotifyOn, refreshSpotify]);

  useEffect(() => {
    const first = window.setTimeout(() => void refreshWindows(), 0);
    const timer = window.setInterval(() => void refreshWindows(), 3_000);
    return () => { window.clearTimeout(first); window.clearInterval(timer); };
  }, [refreshWindows]);

  const shown = chooseNowPlaying(spotify, windows);
  const viaEvents = azrielState !== null && spotify?.device?.name === AZRIEL_DEVICE;

  async function control(action: MediaAction) {
    try {
      if (shown?.origin === "spotify") await spotifyControl(action === "toggle" ? (shown.playing ? "pause" : "play") : action, undefined, { active: true, activeDevice: spotify?.device?.name, deviceId: azrielDeviceId });
      else if (!shown && spotifyOn) await spotifyControl("play", undefined, { active: false, deviceId: azrielDeviceId });
      else await mediaRepository.control(action);
      setError(null);
    } catch (reason) { setError(messageOf(reason)); }
    window.setTimeout(() => { void refreshWindows(); if (spotifyOn && !viaEvents) void refreshSpotify(); }, 350);
  }

  return <section className="now-playing" data-playing={shown?.playing ?? false} aria-label="Tocando agora">
    {shown ? <>
      <div className="now-playing__art">{shown.artwork ? <img src={shown.artwork} alt="" /> : <i aria-hidden="true">♪</i>}</div>
      <div className="now-playing__info">
        <span>{shown.playing ? "TOCANDO" : "PAUSADO"} · {shown.source.toUpperCase()}</span>
        <strong title={shown.title}>{shown.title}</strong>
        <small title={shown.subtitle}>{shown.subtitle || "—"}</small>
        {shown.progress != null && <div className="now-playing__progress" title={shown.position ?? undefined}><i style={{ width: `${shown.progress}%` }} /></div>}
      </div>
      <div className="now-playing__controls">
        <button type="button" onClick={() => void control("previous")} aria-label="Anterior">⏮</button>
        <button type="button" className="now-playing__toggle" onClick={() => void control("toggle")} aria-label={shown.playing ? "Pausar" : "Tocar"}>{shown.playing ? "❚❚" : "▶"}</button>
        <button type="button" onClick={() => void control("next")} aria-label="Próxima">⏭</button>
      </div>
    </> : <>
      <div className="now-playing__idle"><span>{error ?? "NADA TOCANDO"}</span></div>
      {spotifyOn && <div className="now-playing__controls"><button type="button" className="now-playing__toggle" onClick={() => void control("toggle")} aria-label="Retomar no Spotify">▶</button></div>}
    </>}
    {shown && error && <small className="now-playing__error" title={error}>{error}</small>}
    <div className="now-playing__sources">
      <button type="button" onClick={onOpenMusic} title="Abrir o player do Spotify no Azriel">SPOTIFY</button>
      <button type="button" onClick={onOpenYouTube} title="Abrir o mini player do YouTube no canto da tela">YOUTUBE</button>
    </div>
  </section>;
}
