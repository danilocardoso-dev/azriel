import { describe, expect, it } from "vitest";
import { mapPlayer, mapPlaylists, mapSearch, mapTrackPage, uriKind } from "./spotifyRepository";

const images = [{ url: "big", width: 640 }, { url: "mid", width: 300 }, { url: "tiny", width: 64 }];

describe("Spotify: leitura das respostas da API", () => {
  it("monta o tocando agora a partir de /me/player", () => {
    const player = mapPlayer({ is_playing: true, progress_ms: 61000, shuffle_state: true, device: { name: "DESKTOP-AZ", type: "Computer", volume_percent: 70 }, item: { uri: "spotify:track:1", name: "Weightless", duration_ms: 488000, artists: [{ name: "Marconi Union" }], album: { images } } });
    expect(player).toEqual({ isPlaying: true, progressMs: 61000, shuffle: true, device: { name: "DESKTOP-AZ", type: "Computer", volume: 70 }, item: { uri: "spotify:track:1", name: "Weightless", subtitle: "Marconi Union", image: "mid", durationMs: 488000 } });
    expect(mapPlayer(null)).toBeNull();
    expect(mapPlayer({ is_playing: false })).toBeNull();
  });

  it("separa faixas, playlists, álbuns e artistas da busca, ignorando itens nulos", () => {
    const result = mapSearch({
      tracks: { items: [{ uri: "spotify:track:2", name: "Clocks", duration_ms: 1, artists: [{ name: "Coldplay" }], album: { images } }] },
      playlists: { items: [null, { uri: "spotify:playlist:9", name: "Foco", owner: { display_name: "Spotify" }, images }] },
      albums: { items: [{ uri: "spotify:album:3", name: "A Rush of Blood", artists: [{ name: "Coldplay" }], images: [] }] },
      artists: { items: [{ uri: "spotify:artist:4", name: "Coldplay", images }] },
    });
    expect(result.tracks[0]).toMatchObject({ name: "Clocks", subtitle: "Coldplay", image: "mid" });
    expect(result.playlists).toEqual([{ uri: "spotify:playlist:9", name: "Foco", subtitle: "Playlist · Spotify", image: "mid" }]);
    expect(result.albums[0].image).toBeNull();
    expect(result.artists[0].subtitle).toBe("Artista");
  });

  it("guarda o álbum da faixa e sabe se há mais resultados nas buscas filtradas", () => {
    const raw = { tracks: { items: [{ uri: "spotify:track:2", name: "Clocks", duration_ms: 1, artists: [{ name: "Coldplay" }], album: { uri: "spotify:album:9", images } }], next: "https://api.spotify.com/v1/search?offset=10" } };
    expect(mapSearch(raw, "track").tracks[0].albumUri).toBe("spotify:album:9");
    expect(mapSearch(raw, "track").hasMore).toBe(true);
    expect(mapSearch(raw, "all").hasMore).toBe(false);
    expect(mapSearch({ tracks: { items: [], next: null } }, "track").hasMore).toBe(false);
  });

  it("lê as faixas de uma playlist nos dois formatos e ignora episódios e itens vazios", () => {
    const track = (n: number) => ({ uri: `spotify:track:${n}`, name: `Faixa ${n}`, duration_ms: 1000, artists: [{ name: "Banda" }], album: { uri: "spotify:album:9", images } });
    const page = mapTrackPage({ items: [{ item: track(1) }, { track: track(2) }, null, { item: { uri: "spotify:episode:5", name: "Podcast" } }], next: "x" }, "playlist", "capa-da-playlist");
    expect(page.tracks.map((entry) => entry.name)).toEqual(["Faixa 1", "Faixa 2"]);
    expect(page.tracks[0]).toMatchObject({ subtitle: "Banda", image: "mid", albumUri: "spotify:album:9" });
    expect(page.hasMore).toBe(true);
    expect(page.count).toBe(4);
  });

  it("faixas de álbum herdam a capa e o álbum aberto; as do artista vêm da busca", () => {
    const simplified = { items: [{ uri: "spotify:track:7", name: "Clocks", duration_ms: 1000, artists: [{ name: "Coldplay" }] }], next: null };
    expect(mapTrackPage(simplified, "album", "capa-do-album", "spotify:album:1").tracks[0]).toMatchObject({ image: "capa-do-album", albumUri: "spotify:album:1" });
    const artist = mapTrackPage({ tracks: { items: [{ uri: "spotify:track:8", name: "Yellow", duration_ms: 1, artists: [{ name: "Coldplay" }], album: { uri: "spotify:album:2", images } }], next: "y" } }, "artist", null);
    expect(artist.tracks[0].name).toBe("Yellow");
    expect(artist.hasMore).toBe(true);
    expect(uriKind("spotify:playlist:abc")).toBe("playlist");
  });

  it("lista as playlists do usuário com a contagem de músicas", () => {
    expect(mapPlaylists({ items: [{ uri: "spotify:playlist:1", name: "Estudos", tracks: { total: 42 }, images }] })[0].subtitle).toBe("42 músicas");
  });
});
