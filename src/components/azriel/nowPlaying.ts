import type { SpotifyPlayer } from "../../repositories/spotifyRepository";
import type { NowPlaying } from "../../types";

export interface ShownMedia {
  origin: "spotify" | "windows";
  source: string;
  title: string;
  subtitle: string;
  playing: boolean;
  artwork: string | null;
  /** 0–100, ou null quando a duração é desconhecida. */
  progress: number | null;
  position: string | null;
}

const clock = (seconds: number) => `${Math.floor(seconds / 60)}:${String(Math.floor(seconds % 60)).padStart(2, "0")}`;

/**
 * O widget do Núcleo mostra o Spotify pela API (inclusive o player embutido do Azriel, que o Windows
 * não enxerga) e cai nos controles de mídia do Windows para o resto (YouTube, outros players).
 * Prioridade: o que está tocando; empate/pausado: o Spotify, depois o Windows.
 */
export function chooseNowPlaying(spotify: SpotifyPlayer | null, windows: NowPlaying | null): ShownMedia | null {
  const fromSpotify: ShownMedia | null = spotify?.item ? {
    origin: "spotify",
    source: spotify.device?.name === "Azriel" ? "Spotify · Azriel" : "Spotify",
    title: spotify.item.name,
    subtitle: spotify.item.subtitle,
    playing: spotify.isPlaying,
    artwork: spotify.item.image,
    progress: spotify.item.durationMs > 0 ? Math.min(100, (spotify.progressMs / spotify.item.durationMs) * 100) : null,
    position: spotify.item.durationMs > 0 ? `${clock(spotify.progressMs / 1000)} / ${clock(spotify.item.durationMs / 1000)}` : null,
  } : null;
  const fromWindows: ShownMedia | null = windows ? {
    origin: "windows",
    source: windows.source,
    title: windows.title,
    subtitle: windows.artist || windows.album,
    playing: windows.playing,
    artwork: windows.artwork,
    progress: windows.durationSeconds && windows.positionSeconds != null ? Math.min(100, (windows.positionSeconds / windows.durationSeconds) * 100) : null,
    position: windows.durationSeconds && windows.positionSeconds != null ? `${clock(windows.positionSeconds)} / ${clock(windows.durationSeconds)}` : null,
  } : null;
  if (fromSpotify?.playing) return fromSpotify;
  if (fromWindows?.playing) return fromWindows;
  return fromSpotify ?? fromWindows;
}
