import { describe, expect, it } from "vitest";
import { mapSdkState } from "./webPlayer";

describe("estado do player embutido", () => {
  const state = { paused: false, position: 5000, shuffle: true, track_window: { current_track: { uri: "spotify:track:1", name: "Clocks", duration_ms: 300000, artists: [{ name: "Coldplay" }], album: { uri: "spotify:album:9", images: [{ url: "tiny", width: 64 }, { url: "mid", width: 300 }, { url: "big", width: 640 }] } } } };

  it("vira o mesmo formato da API, marcado como o dispositivo Azriel", () => {
    expect(mapSdkState(state as never)).toMatchObject({ isPlaying: true, progressMs: 5000, shuffle: true, device: { name: "Azriel" }, item: { uri: "spotify:track:1", name: "Clocks", subtitle: "Coldplay", image: "mid", durationMs: 300000 } });
  });

  it("sem faixa ou sem estado, não há nada tocando", () => {
    expect(mapSdkState(null)).toBeNull();
    expect(mapSdkState({ ...state, track_window: { current_track: null } } as never)).toBeNull();
  });
});
