// Web Playback SDK do Spotify: faz o próprio Azriel aparecer como um dispositivo Spotify Connect.
// Aqui ficam só as peças puras e o carregamento do script; o estado vive em useSpotifyWebPlayer.

import { mapPlayer, type SpotifyPlayer } from "../repositories/spotifyRepository";

export type WebPlayerStatus = "off" | "loading" | "ready" | "needs-reauth" | "error";
export type Widevine = "unknown" | "ok" | "no";
export type SdkErrorKind = "initialization_error" | "authentication_error" | "account_error" | "playback_error";

interface SdkPlayer {
  connect(): Promise<boolean>;
  disconnect(): void;
  addListener(event: string, callback: (payload: never) => void): boolean;
}
interface SdkOptions { name: string; volume: number; getOAuthToken(callback: (token: string) => void): void }
declare global {
  interface Window {
    Spotify?: { Player: new (options: SdkOptions) => SdkPlayer };
    onSpotifyWebPlaybackSDKReady?: () => void;
  }
}
export type { SdkPlayer, SdkOptions };

/** Estado que o SDK entrega a cada evento (faixa, pausa, seek); evita consultar a API para saber o que toca no Azriel. */
export interface SdkState {
  paused: boolean; position: number; shuffle: boolean;
  track_window: { current_track: { album?: { images?: Array<{ url: string; width?: number }> } } | null };
}

export function mapSdkState(state: SdkState | null | undefined): SpotifyPlayer | null {
  const track = state?.track_window?.current_track;
  if (!state || !track) return null;
  const images = [...(track.album?.images ?? [])].sort((a, b) => (b.width ?? 0) - (a.width ?? 0));
  return mapPlayer({
    is_playing: !state.paused, progress_ms: state.position, shuffle_state: state.shuffle,
    device: { name: "Azriel", type: "Computer" },
    item: { ...track, album: { ...track.album, images } },
  });
}

/** O SDK só toca com DRM; confirma se este WebView sabe abrir conteúdo protegido (Widevine). */
export async function widevineSupport(): Promise<Widevine> {
  if (typeof navigator === "undefined" || typeof navigator.requestMediaKeySystemAccess !== "function") return "no";
  try {
    await navigator.requestMediaKeySystemAccess("com.widevine.alpha", [{ initDataTypes: ["cenc"], audioCapabilities: [{ contentType: 'audio/mp4;codecs="mp4a.40.2"' }] }]);
    return "ok";
  } catch {
    return "no";
  }
}

/** Traduz os erros do SDK para o estado do painel e uma frase em português. `status: null` = não muda o estado. */
export function describeSdkError(kind: SdkErrorKind, message = ""): { status: WebPlayerStatus | null; text: string } {
  const detail = message ? ` (${message})` : "";
  switch (kind) {
    case "authentication_error":
      return { status: "needs-reauth", text: `O Spotify pediu uma nova autorização para o player embutido${detail}. Reconecte o Spotify.` };
    case "account_error":
      return { status: "error", text: `O player embutido exige Spotify Premium${detail}.` };
    case "initialization_error":
      return { status: "error", text: `Este navegador embutido não conseguiu iniciar o player do Spotify${detail}. O Azriel segue controlando o app do Spotify.` };
    default:
      return { status: null, text: `Erro de reprodução${detail}.` };
  }
}

let sdkLoading: Promise<void> | null = null;

export function loadSdk(): Promise<void> {
  if (typeof window !== "undefined" && window.Spotify) return Promise.resolve();
  sdkLoading ??= new Promise<void>((resolve, reject) => {
    window.onSpotifyWebPlaybackSDKReady = () => resolve();
    const script = document.createElement("script");
    script.src = "https://sdk.scdn.co/spotify-player.js";
    script.async = true;
    script.onerror = () => { sdkLoading = null; reject(new Error("Não foi possível carregar o player do Spotify. Verifique a internet.")); };
    document.head.appendChild(script);
  });
  return sdkLoading;
}
