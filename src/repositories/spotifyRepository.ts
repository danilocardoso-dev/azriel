import { invokeDatabase } from "./tauri";

export interface SpotifyStatus { clientIdConfigured: boolean; connected: boolean; redirectUri: string }
export interface SpotifyItem { uri: string; name: string; subtitle: string; image: string | null; durationMs?: number; albumUri?: string }
export interface SpotifyPlayer {
  isPlaying: boolean; progressMs: number; shuffle: boolean;
  device: { name: string; type: string; volume: number | null } | null;
  item: SpotifyItem & { durationMs: number } | null;
}
export interface SpotifySearch { tracks: SpotifyItem[]; playlists: SpotifyItem[]; albums: SpotifyItem[]; artists: SpotifyItem[]; hasMore: boolean }
export type SearchKind = "all" | "track" | "playlist" | "album" | "artist";
export type SpotifyAction = "play" | "pause" | "next" | "previous" | "shuffle" | "volume" | "seek";

type Raw = Record<string, unknown> | null | undefined;
const get = (value: unknown, ...path: Array<string | number>): unknown => path.reduce<unknown>((current, key) => (current && typeof current === "object" ? (current as Record<string | number, unknown>)[key] : undefined), value);
const text = (value: unknown) => typeof value === "string" ? value : "";
/** A menor imagem que ainda tenha pelo menos 160px (o Spotify manda do maior para o menor). */
function image(images: unknown): string | null {
  const list = Array.isArray(images) ? images as Array<{ url?: string; width?: number | null }> : [];
  const good = [...list].reverse().find((entry) => (entry.width ?? 640) >= 160) ?? list[0];
  return good?.url ?? null;
}
const names = (artists: unknown) => Array.isArray(artists) ? artists.map((artist) => text(get(artist, "name"))).filter(Boolean).join(", ") : "";

export function mapTrack(raw: unknown): SpotifyItem & { durationMs: number } {
  return { uri: text(get(raw, "uri")), name: text(get(raw, "name")), subtitle: names(get(raw, "artists")) || text(get(raw, "show", "name")), image: image(get(raw, "album", "images") ?? get(raw, "images")), durationMs: Number(get(raw, "duration_ms") ?? 0), albumUri: text(get(raw, "album", "uri")) || undefined };
}

export function mapPlayer(raw: Raw): SpotifyPlayer | null {
  if (!raw || !get(raw, "item")) return null;
  const device = get(raw, "device");
  return {
    isPlaying: Boolean(get(raw, "is_playing")),
    progressMs: Number(get(raw, "progress_ms") ?? 0),
    shuffle: Boolean(get(raw, "shuffle_state")),
    device: device ? { name: text(get(device, "name")), type: text(get(device, "type")), volume: typeof get(device, "volume_percent") === "number" ? Number(get(device, "volume_percent")) : null } : null,
    item: mapTrack(get(raw, "item")),
  };
}

function mapList(raw: unknown, subtitle: (item: unknown) => string): SpotifyItem[] {
  const items = get(raw, "items");
  return (Array.isArray(items) ? items : []).filter(Boolean).map((item) => ({ uri: text(get(item, "uri")), name: text(get(item, "name")), subtitle: subtitle(item), image: image(get(item, "images")) })).filter((item) => item.uri);
}

export function mapSearch(raw: Raw, kind: SearchKind = "all"): SpotifySearch {
  const tracks = get(raw, "tracks", "items");
  return {
    tracks: (Array.isArray(tracks) ? tracks : []).filter(Boolean).map(mapTrack).filter((item) => item.uri),
    playlists: mapList(get(raw, "playlists"), (item) => `Playlist · ${text(get(item, "owner", "display_name"))}`),
    albums: mapList(get(raw, "albums"), (item) => `Álbum · ${names(get(item, "artists"))}`),
    artists: mapList(get(raw, "artists"), () => "Artista"),
    // Só as buscas filtradas paginam; na busca geral o "ver todas" leva para o filtro.
    hasMore: kind !== "all" && Boolean(get(raw, `${kind}s`, "next")),
  };
}

export interface SpotifyTrackPage { tracks: SpotifyItem[]; hasMore: boolean; /** Quantos itens vieram, para pedir a próxima página. */ count: number }
export const uriKind = (uri: string) => uri.split(":")[1] ?? "";

/**
 * Faixas de uma playlist, álbum ou artista. A playlist traz a faixa em `item` (formato atual) ou `track`
 * (antigo); o álbum traz faixas simplificadas, sem capa, então herdam a capa e o álbum do item aberto;
 * o artista vem de uma busca de faixas.
 */
export function mapTrackPage(raw: Raw, kind: string, fallbackImage: string | null, albumUri?: string): SpotifyTrackPage {
  const container = kind === "artist" ? get(raw, "tracks") : raw;
  const items = get(container, "items");
  const list = Array.isArray(items) ? items : [];
  const tracks = list
    .map((entry) => get(entry, "item") ?? get(entry, "track") ?? entry)
    .filter((entry) => text(get(entry, "uri")).startsWith("spotify:track:"))
    .map((entry) => { const track = mapTrack(entry); return { ...track, image: track.image ?? fallbackImage, albumUri: track.albumUri ?? albumUri }; });
  return { tracks, hasMore: Boolean(get(container, "next")), count: list.length };
}

export const mapPlaylists = (raw: Raw) => mapList(raw, (item) => `${Number(get(item, "tracks", "total") ?? get(item, "items", "total") ?? 0)} músicas`);

export const spotifyRepository = {
  status: () => invokeDatabase<SpotifyStatus>("spotify_status"),
  setClientId: (clientId: string) => invokeDatabase<SpotifyStatus>("spotify_set_client_id", { clientId }),
  connect: () => invokeDatabase<SpotifyStatus>("spotify_connect"),
  disconnect: () => invokeDatabase<SpotifyStatus>("spotify_disconnect"),
  player: async () => mapPlayer(await invokeDatabase<Raw>("spotify_player")),
  control: (action: SpotifyAction, value?: number) => invokeDatabase<void>("spotify_control", { action, value: value ?? null }),
  search: async (query: string, kind: SearchKind = "all", offset = 0) => mapSearch(await invokeDatabase<Raw>("spotify_search", { query, kind, offset }), kind),
  tracks: async (item: SpotifyItem, offset = 0) => {
    const kind = uriKind(item.uri);
    return mapTrackPage(await invokeDatabase<Raw>("spotify_tracks", { uri: item.uri, name: item.name, offset }), kind, item.image, kind === "album" ? item.uri : undefined);
  },
  queue: (uri: string, deviceId?: string | null) => invokeDatabase<void>("spotify_queue", { uri, deviceId: deviceId ?? null }),
  playlists: async () => mapPlaylists(await invokeDatabase<Raw>("spotify_playlists")),
  play: (uri: string, deviceId?: string | null, context?: string | null) => invokeDatabase<void>("spotify_play", { uri, deviceId: deviceId ?? null, context: context ?? null }),
  token: () => invokeDatabase<string>("spotify_token"),
  transfer: (deviceId: string, play: boolean) => invokeDatabase<void>("spotify_transfer", { deviceId, play }),
};
