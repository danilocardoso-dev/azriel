import { useCallback, useEffect, useRef, useState } from "react";
import { spotifyRepository, type SpotifyPlayer } from "../repositories/spotifyRepository";
import { describeSdkError, loadSdk, mapSdkState, widevineSupport, type SdkErrorKind, type SdkPlayer, type SdkState, type WebPlayerStatus, type Widevine } from "./webPlayer";

const READY_TIMEOUT_MS = 20_000;
const messageOf = (error: unknown) => error instanceof Error ? error.message : String(error);

/**
 * Player embutido do Spotify. Mora no shell (App) para continuar tocando ao trocar de módulo;
 * só sobe quando a tela Música pede (`start`).
 */
export function useSpotifyWebPlayer() {
  const [status, setStatus] = useState<WebPlayerStatus>("off");
  const [deviceId, setDeviceId] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [state, setState] = useState<SpotifyPlayer | null>(null);
  const [widevine, setWidevine] = useState<Widevine>("unknown");
  const player = useRef<SdkPlayer | null>(null);
  const started = useRef(false);
  const timer = useRef<number | null>(null);

  const clearTimer = () => { if (timer.current !== null) { window.clearTimeout(timer.current); timer.current = null; } };
  const fail = useCallback((next: WebPlayerStatus, text: string) => { clearTimer(); setStatus(next); setMessage(text); started.current = false; }, []);

  const stop = useCallback(() => {
    clearTimer();
    player.current?.disconnect();
    player.current = null;
    started.current = false;
    setDeviceId(null); setState(null); setStatus("off"); setMessage(null);
  }, []);

  const start = useCallback(async () => {
    if (started.current) return;
    started.current = true;
    setStatus("loading"); setMessage(null);

    const support = await widevineSupport();
    setWidevine(support);
    if (support === "no") { fail("error", "Este navegador embutido não tem o DRM (Widevine) que o Spotify exige. O Azriel segue controlando o app do Spotify."); return; }
    try { await loadSdk(); } catch (reason) { fail("error", messageOf(reason)); return; }
    if (!window.Spotify) { fail("error", "O player do Spotify não carregou."); return; }

    const instance = new window.Spotify.Player({
      name: "Azriel",
      volume: 0.6,
      getOAuthToken: (callback) => { void spotifyRepository.token().then(callback).catch((reason) => fail("error", messageOf(reason))); },
    });
    player.current = instance;
    instance.addListener("ready", (({ device_id }: { device_id: string }) => { clearTimer(); setDeviceId(device_id); setStatus("ready"); setMessage(null); }) as never);
    instance.addListener("not_ready", (() => { setDeviceId(null); setState(null); setStatus("loading"); }) as never);
    // O que toca no Azriel chega por evento: nada de consultar a API de tempos em tempos.
    instance.addListener("player_state_changed", ((payload: SdkState | null) => setState(mapSdkState(payload))) as never);
    for (const kind of ["initialization_error", "authentication_error", "account_error", "playback_error"] as SdkErrorKind[]) {
      instance.addListener(kind, (({ message: detail }: { message: string }) => {
        const { status: next, text } = describeSdkError(kind, detail);
        if (next) fail(next, text); else setMessage(text);
      }) as never);
    }
    timer.current = window.setTimeout(() => fail("error", "O player embutido não respondeu em 20 segundos. O Azriel segue controlando o app do Spotify."), READY_TIMEOUT_MS);
    try {
      if (!(await instance.connect())) fail("error", "O Spotify não aceitou a conexão do player embutido.");
    } catch (reason) { fail("error", messageOf(reason)); }
  }, [fail]);

  const restart = useCallback(async () => { stop(); await start(); }, [start, stop]);

  useEffect(() => () => { clearTimer(); player.current?.disconnect(); }, []);

  return { status, deviceId, state, message, widevine, start, stop, restart };
}

export type SpotifyWebPlayer = ReturnType<typeof useSpotifyWebPlayer>;
