import { describe, expect, it } from "vitest";
import type { SpotifyPlayer } from "../../repositories/spotifyRepository";
import type { NowPlaying } from "../../types";
import { chooseNowPlaying } from "./nowPlaying";

const spotify = (playing: boolean, device = "Azriel"): SpotifyPlayer => ({ isPlaying: playing, progressMs: 61_000, shuffle: false, device: { name: device, type: "Computer", volume: 60 }, item: { uri: "spotify:track:1", name: "Weightless", subtitle: "Marconi Union", image: "capa", durationMs: 488_000 } });
const windows = (playing: boolean): NowPlaying => ({ title: "Aula de Rust", artist: "Canal", album: "", source: "Chrome", playing, positionSeconds: 90, durationSeconds: 600, artwork: null });

describe("widget Tocando agora do Núcleo", () => {
  it("mostra o Spotify pela API, inclusive o player embutido que o Windows não vê", () => {
    const shown = chooseNowPlaying(spotify(true), null);
    expect(shown).toMatchObject({ origin: "spotify", source: "Spotify · Azriel", title: "Weightless", subtitle: "Marconi Union", playing: true, position: "1:01 / 8:08" });
    expect(shown?.progress).toBeCloseTo(12.5, 1);
    expect(chooseNowPlaying(spotify(true, "DESKTOP-PC"), null)?.source).toBe("Spotify");
  });

  it("prefere quem está tocando; no empate, o Spotify", () => {
    expect(chooseNowPlaying(spotify(false), windows(true))?.origin).toBe("windows");
    expect(chooseNowPlaying(spotify(true), windows(true))?.origin).toBe("spotify");
    expect(chooseNowPlaying(spotify(false), windows(false))?.origin).toBe("spotify");
  });

  it("cai no Windows sem Spotify e devolve nulo sem nada", () => {
    expect(chooseNowPlaying(null, windows(true))).toMatchObject({ origin: "windows", source: "Chrome", subtitle: "Canal", progress: 15, position: "1:30 / 10:00" });
    expect(chooseNowPlaying(null, null)).toBeNull();
  });
});
