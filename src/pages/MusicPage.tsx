import { useCallback, useEffect, useRef, useState, type FormEvent } from "react";
import { spotifyRepository, uriKind, type SearchKind, type SpotifyAction, type SpotifyItem, type SpotifyPlayer, type SpotifySearch, type SpotifyStatus } from "../repositories/spotifyRepository";
import { shouldAdoptAzriel, spotifyControl, AZRIEL_DEVICE } from "../music/controls";
import { loadSearches, rememberSearch, saveSearches } from "../music/searchHistory";
import type { SpotifyWebPlayer } from "../music/useSpotifyWebPlayer";
import "../styles/music.css";

const messageOf = (error: unknown) => error instanceof Error ? error.message : String(error);
const clock = (ms: number) => `${Math.floor(ms / 60000)}:${String(Math.floor((ms % 60000) / 1000)).padStart(2, "0")}`;

export function MusicPage({ webPlayer }: { webPlayer: SpotifyWebPlayer }) {
  const [status, setStatus] = useState<SpotifyStatus | null>(null);
  const [error, setError] = useState<string | null>(null);

  const refreshStatus = useCallback(async () => {
    try { setStatus(await spotifyRepository.status()); } catch (reason) { setError(messageOf(reason)); }
  }, []);
  useEffect(() => { const first = window.setTimeout(() => void refreshStatus(), 0); return () => window.clearTimeout(first); }, [refreshStatus]);

  return <section className="module-page music-page">
    <header className="music-page__head">
      <div><span>MÓDULO // MUS-01</span><h1>Música</h1></div>
      {status?.connected && <button type="button" onClick={() => void spotifyRepository.disconnect().then(setStatus).catch((reason) => setError(messageOf(reason)))}>DESCONECTAR SPOTIFY</button>}
    </header>
    {error && <p className="music-error" role="alert">{error} <button type="button" onClick={() => setError(null)}>×</button></p>}
    {!status ? <p className="music-muted">Carregando...</p>
      : !status.clientIdConfigured ? <SpotifySetup status={status} onSaved={setStatus} onError={setError} />
      : !status.connected ? <SpotifyConnect onConnected={setStatus} onError={setError} />
      : <SpotifyPlayerView onError={setError} webPlayer={webPlayer} onReconnected={setStatus} />}
  </section>;
}

function SpotifySetup({ status, onSaved, onError }: { status: SpotifyStatus; onSaved: (status: SpotifyStatus) => void; onError: (message: string) => void }) {
  const [clientId, setClientId] = useState("");
  const [busy, setBusy] = useState(false);
  async function submit(event: FormEvent) {
    event.preventDefault(); setBusy(true);
    try { onSaved(await spotifyRepository.setClientId(clientId)); } catch (reason) { onError(messageOf(reason)); } finally { setBusy(false); }
  }
  const copy = (value: string) => void navigator.clipboard?.writeText(value).catch(() => undefined);
  return <div className="music-card music-setup">
    <h2>Conectar o Spotify</h2>
    <p>Configuração única, de uns 5 minutos. Usa a API oficial do Spotify com o seu Premium; nenhuma senha passa pelo Azriel.</p>
    <ol>
      <li>Abra <code>developer.spotify.com/dashboard</code> no navegador e entre com a sua conta do Spotify. <button type="button" onClick={() => copy("https://developer.spotify.com/dashboard")}>COPIAR LINK</button></li>
      <li>Clique em <b>Create app</b>. Nome: <code>Azriel</code>; descrição: qualquer uma.</li>
      <li>Em <b>Redirect URIs</b>, adicione exatamente <code>{status.redirectUri}</code> <button type="button" onClick={() => copy(status.redirectUri)}>COPIAR</button></li>
      <li>Em <b>Which API/SDKs</b>, marque <b>Web API</b> e salve.</li>
      <li>Em <b>User Management</b>, adicione o e-mail da sua conta do Spotify (o modo de desenvolvimento só libera usuários da lista).</li>
      <li>Abra <b>Settings</b> do app, copie o <b>Client ID</b> e cole aqui embaixo.</li>
    </ol>
    <form onSubmit={submit}>
      <input value={clientId} onChange={(event) => setClientId(event.target.value)} placeholder="Client ID (32 caracteres)" spellCheck={false} autoComplete="off" />
      <button disabled={busy || !clientId.trim()}>{busy ? "SALVANDO..." : "SALVAR CLIENT ID"}</button>
    </form>
  </div>;
}

function SpotifyConnect({ onConnected, onError }: { onConnected: (status: SpotifyStatus) => void; onError: (message: string) => void }) {
  const [busy, setBusy] = useState(false);
  async function connect() {
    setBusy(true);
    try { onConnected(await spotifyRepository.connect()); } catch (reason) { onError(messageOf(reason)); } finally { setBusy(false); }
  }
  return <div className="music-card music-connect">
    <h2>Autorizar o Azriel</h2>
    <p>O navegador vai abrir a página do Spotify. Faça login, autorize o Azriel e volte para cá.</p>
    <button type="button" className="music-primary" onClick={() => void connect()} disabled={busy}>{busy ? "AGUARDANDO A AUTORIZAÇÃO NO NAVEGADOR..." : "CONECTAR AO SPOTIFY"}</button>
  </div>;
}

const KINDS: Array<[SearchKind, string]> = [["all", "TUDO"], ["track", "MÚSICAS"], ["playlist", "PLAYLISTS"], ["album", "ÁLBUNS"], ["artist", "ARTISTAS"]];
const LIST_OF = { track: "tracks", playlist: "playlists", album: "albums", artist: "artists" } as const;

function SpotifyPlayerView({ onError, webPlayer, onReconnected }: { onError: (message: string) => void; webPlayer: SpotifyWebPlayer; onReconnected: (status: SpotifyStatus) => void }) {
  const [fetched, setFetched] = useState<SpotifyPlayer | null>(null);
  const [loaded, setLoaded] = useState(false);
  // Tocando no Azriel, o estado vem dos eventos do player embutido (sem chamar a API); a consulta só vale para outros aparelhos.
  const player = webPlayer.state ?? fetched;
  const viaEvents = player?.device?.name === AZRIEL_DEVICE && webPlayer.state !== null;
  const [progress, setProgress] = useState(0);
  const [playlists, setPlaylists] = useState<SpotifyItem[]>([]);
  const [query, setQuery] = useState("");
  const [kind, setKind] = useState<SearchKind>("all");
  const [results, setResults] = useState<SpotifySearch | null>(null);
  const [searching, setSearching] = useState(false);
  const [history, setHistory] = useState<string[]>(loadSearches);
  const [notice, setNotice] = useState<string | null>(null);
  const adopted = useRef<string | null>(null);
  const [opened, setOpened] = useState<SpotifyItem | null>(null);
  const [detail, setDetail] = useState<DetailState>(emptyDetail);

  const refresh = useCallback(async () => {
    if (document.hidden) return;
    try { setFetched(await spotifyRepository.player()); }
    catch (reason) { onError(messageOf(reason)); }
    finally { setLoaded(true); }
  }, [onError]);

  // Sem consulta periódica: lê o estado uma vez ao abrir; depois só reage a eventos e a comandos.
  useEffect(() => {
    const first = window.setTimeout(() => { void refresh(); void spotifyRepository.playlists().then(setPlaylists).catch((reason) => onError(messageOf(reason))); }, 0);
    return () => window.clearTimeout(first);
  }, [refresh, onError]);

  // Depois de um comando, só consulta a API se o Azriel não for o player (nesse caso o evento já avisa).
  const refreshSoon = useCallback((delay: number) => { if (!viaEvents) window.setTimeout(() => void refresh(), delay); }, [viaEvents, refresh]);

  useEffect(() => {
    const sync = window.setTimeout(() => setProgress(player?.progressMs ?? 0), 0);
    return () => window.clearTimeout(sync);
  }, [player]);

  const { start: startWebPlayer } = webPlayer;
  useEffect(() => {
    const first = window.setTimeout(() => void startWebPlayer(), 0);
    return () => window.clearTimeout(first);
  }, [startWebPlayer]);

  const embedded = webPlayer.status === "ready" ? webPlayer.deviceId : null;

  // O Azriel vira a saída padrão sozinho: sem ninguém tocando em outro aparelho, assume o player.
  useEffect(() => {
    if (!embedded || !shouldAdoptAzriel(player, loaded, embedded)) return;
    const timer = window.setTimeout(() => {
      if (adopted.current === embedded) return;
      adopted.current = embedded;
      void spotifyRepository.transfer(embedded, false).then(() => refresh()).catch(() => { adopted.current = null; });
    }, 0);
    return () => window.clearTimeout(timer);
  }, [player, loaded, embedded, refresh]);

  async function reauthorize() {
    try {
      await spotifyRepository.disconnect();
      onReconnected(await spotifyRepository.connect());
      await webPlayer.restart();
    } catch (reason) { onError(messageOf(reason)); }
  }
  async function moveToAzriel() {
    if (!embedded) return;
    try { await spotifyRepository.transfer(embedded, player?.isPlaying ?? false); refreshSoon(700); } catch (reason) { onError(messageOf(reason)); }
  }

  // Avança a barra localmente entre os eventos, para a barra andar sozinha.
  useEffect(() => {
    if (!player?.isPlaying) return;
    const tick = window.setInterval(() => setProgress((current) => Math.min(current + 1000, player.item?.durationMs ?? current)), 1000);
    return () => window.clearInterval(tick);
  }, [player]);

  // Busca ao digitar: espera a pessoa parar, descarta respostas atrasadas e guarda no histórico quando acha algo.
  useEffect(() => {
    const text = query.trim();
    let active = true;
    const timer = window.setTimeout(async () => {
      if (!text) { setResults(null); setSearching(false); return; }
      setSearching(true);
      try {
        const found = await spotifyRepository.search(text, kind);
        if (!active) return;
        setResults(found);
        if (found.tracks.length + found.playlists.length + found.albums.length + found.artists.length > 0) {
          setHistory((current) => { const next = rememberSearch(current, text); saveSearches(next); return next; });
        }
      } catch (reason) { if (active) onError(messageOf(reason)); }
      finally { if (active) setSearching(false); }
    }, text ? 350 : 0);
    return () => { active = false; window.clearTimeout(timer); };
  }, [query, kind, onError]);

  async function loadMore() {
    if (kind === "all" || !results) return;
    const key = LIST_OF[kind];
    setSearching(true);
    try {
      const next = await spotifyRepository.search(query.trim(), kind, results[key].length);
      setResults((current) => current ? { ...current, [key]: [...current[key], ...next[key]], hasMore: next.hasMore } : next);
    } catch (reason) { onError(messageOf(reason)); } finally { setSearching(false); }
  }

  // Abrir uma playlist, álbum ou artista: carrega a primeira página de faixas.
  useEffect(() => {
    if (!opened) return;
    let active = true;
    const timer = window.setTimeout(async () => {
      setDetail({ ...emptyDetail, loading: true });
      try {
        const page = await spotifyRepository.tracks(opened, 0);
        if (active) setDetail({ tracks: page.tracks, hasMore: page.hasMore, next: page.count, error: null, loading: false });
      } catch (reason) { if (active) setDetail({ ...emptyDetail, error: messageOf(reason) }); }
    }, 0);
    return () => { active = false; window.clearTimeout(timer); };
  }, [opened]);

  async function loadMoreDetail() {
    if (!opened) return;
    setDetail((current) => ({ ...current, loading: true }));
    try {
      const page = await spotifyRepository.tracks(opened, detail.next);
      setDetail((current) => ({ tracks: [...current.tracks, ...page.tracks], hasMore: page.hasMore, next: current.next + page.count, error: null, loading: false }));
    } catch (reason) { setDetail((current) => ({ ...current, loading: false, error: messageOf(reason) })); }
  }
  const open = (entry: SpotifyItem) => { if (["playlist", "album", "artist"].includes(uriKind(entry.uri))) setOpened(entry); else void play(entry); };

  async function control(action: SpotifyAction, value?: number) {
    try { await spotifyControl(action, value, { active: Boolean(player), activeDevice: player?.device?.name, deviceId: embedded }); refreshSoon(350); } catch (reason) { onError(messageOf(reason)); }
  }
  async function play(item: SpotifyItem, context?: string) {
    try { await spotifyRepository.play(item.uri, embedded, context); refreshSoon(700); } catch (reason) { onError(messageOf(reason)); }
  }
  async function queue(track: SpotifyItem) {
    try {
      await spotifyRepository.queue(track.uri, embedded);
      setNotice(`Na fila: ${track.name}`);
      window.setTimeout(() => setNotice(null), 2_500);
    } catch (reason) { onError(messageOf(reason)); }
  }

  const item = player?.item ?? null;
  const duration = item?.durationMs ?? 0;
  const text = query.trim();
  const hasAny = results ? results.tracks.length + results.playlists.length + results.albums.length + results.artists.length > 0 : false;
  const onAll = (target: SearchKind) => kind === "all" ? () => setKind(target) : undefined;
  const output = webPlayer.status === "ready"
    ? (player?.device?.name === AZRIEL_DEVICE ? "Azriel (player embutido)" : player?.isPlaying ? `Tocando em ${player.device?.name ?? "outro aparelho"}` : "Azriel é a saída padrão")
    : webPlayer.status === "loading" ? "Preparando o player embutido…" : webPlayer.status === "needs-reauth" ? "Precisa de nova autorização" : webPlayer.status === "error" ? "Usando o app do Spotify" : "App do Spotify";

  return <div className="music-layout">
    <section className="music-now">
      <div className="music-now__art">{item?.image ? <img src={item.image} alt="" /> : <i>♪</i>}</div>
      <div className="music-now__info">
        <span>{player ? `${player.isPlaying ? "TOCANDO" : "PAUSADO"}${player.device ? ` · ${player.device.name.toUpperCase()}` : ""}` : "NADA TOCANDO NO SPOTIFY"}</span>
        <strong>{item?.name ?? "Escolha uma música ou playlist"}</strong>
        <small>{item?.subtitle ?? "Busque ao lado ou toque uma das suas playlists."}</small>
      </div>
      {item && <div className="music-now__progress">
        <small>{clock(progress)}</small>
        <input type="range" min={0} max={duration} step={1000} value={Math.min(progress, duration)} onChange={(event) => setProgress(Number(event.target.value))} onMouseUp={(event) => void control("seek", Number(event.currentTarget.value))} aria-label="Posição da música" />
        <small>{clock(duration)}</small>
      </div>}
      <div className="music-now__controls">
        <button type="button" data-active={player?.shuffle} onClick={() => void control("shuffle", player?.shuffle ? 0 : 1)} title="Aleatório">⇄</button>
        <button type="button" onClick={() => void control("previous")} aria-label="Anterior">⏮</button>
        <button type="button" className="music-now__toggle" onClick={() => void control(player?.isPlaying ? "pause" : "play")} aria-label={player?.isPlaying ? "Pausar" : "Tocar"}>{player?.isPlaying ? "❚❚" : "▶"}</button>
        <button type="button" onClick={() => void control("next")} aria-label="Próxima">⏭</button>
        <label className="music-now__volume" title="Volume">🔊<input type="range" min={0} max={100} defaultValue={player?.device?.volume ?? 50} key={player?.device?.name} onMouseUp={(event) => void control("volume", Number(event.currentTarget.value))} disabled={player?.device?.volume == null} /></label>
      </div>
      <div className="music-output" data-status={webPlayer.status}>
        <span>SAÍDA DO SOM</span>
        <strong>{output}</strong>
        {webPlayer.message && <small>{webPlayer.message}</small>}
        <small className="music-output__diag">DRM: {webPlayer.widevine === "ok" ? "ok" : webPlayer.widevine === "no" ? "indisponível" : "…"} · Player: {webPlayer.status === "ready" ? "pronto" : webPlayer.status}</small>
        {webPlayer.status === "ready" && player?.device?.name !== AZRIEL_DEVICE && player?.isPlaying && <button type="button" onClick={() => void moveToAzriel()}>TRAZER PARA O AZRIEL</button>}
        {webPlayer.status === "needs-reauth" && <button type="button" onClick={() => void reauthorize()}>RECONECTAR SPOTIFY</button>}
        {webPlayer.status === "error" && <button type="button" onClick={() => void webPlayer.restart()}>TENTAR DE NOVO</button>}
      </div>
    </section>

    <section className="music-browse">
      <form className="music-search" onSubmit={(event: FormEvent) => event.preventDefault()}>
        <span aria-hidden="true">⌕</span>
        <input value={query} onChange={(event) => { setOpened(null); setQuery(event.target.value); }} onKeyDown={(event) => { if (event.key === "Escape") setQuery(""); }} placeholder="Buscar músicas, artistas, álbuns ou playlists" spellCheck={false} />
        {searching ? <em aria-label="Buscando">…</em> : query && <button type="button" onClick={() => setQuery("")} aria-label="Limpar busca">×</button>}
      </form>
      {text && <nav className="music-kinds" aria-label="Tipo de resultado">{KINDS.map(([id, label]) => <button type="button" key={id} className={kind === id ? "active" : ""} onClick={() => { setOpened(null); setKind(id); }}>{label}</button>)}</nav>}
      {notice && <p className="music-notice" role="status">{notice}</p>}
      {opened ? <Detail item={opened} state={detail} currentUri={item?.uri} onBack={() => setOpened(null)} onPlayAll={() => void play(opened)} onPlayTrack={(track, context) => void play(track, context)} onQueue={queue} onMore={() => void loadMoreDetail()} /> : results && text ? (hasAny ? <>
        {results.tracks.length > 0 && <Shelf title="Músicas" onAll={onAll("track")}><TrackList tracks={results.tracks} currentUri={item?.uri} onPlay={play} onQueue={queue} /></Shelf>}
        <Cards title="Playlists" items={results.playlists} onPlay={play} onOpen={open} onAll={onAll("playlist")} />
        <Cards title="Álbuns" items={results.albums} onPlay={play} onOpen={open} onAll={onAll("album")} />
        <Cards title="Artistas" items={results.artists} onPlay={play} onOpen={open} onAll={onAll("artist")} round />
        {kind !== "all" && results.hasMore && <button type="button" className="music-more" onClick={() => void loadMore()} disabled={searching}>{searching ? "CARREGANDO..." : "MOSTRAR MAIS"}</button>}
      </> : searching ? null : <p className="music-muted">Nada encontrado para “{text}”.</p>) : <>
        {history.length > 0 && <div className="music-history"><span>RECENTES</span>{history.map((entry) => <button type="button" key={entry} onClick={() => { setOpened(null); setQuery(entry); }}>{entry}</button>)}</div>}
        <Cards title="Suas playlists" items={playlists} onPlay={play} onOpen={open} empty="Nenhuma playlist encontrada na sua conta." />
      </>}
    </section>
  </div>;
}

interface DetailState { tracks: SpotifyItem[]; hasMore: boolean; next: number; error: string | null; loading: boolean }
const emptyDetail: DetailState = { tracks: [], hasMore: false, next: 0, error: null, loading: false };

function Detail({ item, state, currentUri, onBack, onPlayAll, onPlayTrack, onQueue, onMore }: { item: SpotifyItem; state: DetailState; currentUri?: string; onBack: () => void; onPlayAll: () => void; onPlayTrack: (track: SpotifyItem, context?: string) => void; onQueue: (track: SpotifyItem) => void; onMore: () => void }) {
  const kind = uriKind(item.uri);
  const label = kind === "playlist" ? "PLAYLIST" : kind === "album" ? "ÁLBUM" : "ARTISTA";
  return <div className="music-detail">
    <button type="button" className="music-detail__back" onClick={onBack}>← VOLTAR</button>
    <header className="music-detail__head">
      <div className="music-detail__art" data-round={kind === "artist"}>{item.image ? <img src={item.image} alt="" /> : <i>♪</i>}</div>
      <div>
        <span>{label}</span>
        <h2>{item.name}</h2>
        <small>{item.subtitle}</small>
        <button type="button" className="music-primary" onClick={onPlayAll}>▶ TOCAR</button>
      </div>
    </header>
    {state.loading && state.tracks.length === 0 && <p className="music-muted">Carregando faixas…</p>}
    {state.error && <p className="music-muted">{state.error}{kind === "playlist" ? " O Spotify só libera as faixas de playlists suas ou colaborativas (regra do modo de desenvolvimento). Dá para tocar mesmo assim." : ""}</p>}
    {state.tracks.length > 0 && <TrackList tracks={state.tracks} currentUri={currentUri} context={kind === "artist" ? undefined : item.uri} onPlay={onPlayTrack} onQueue={onQueue} />}
    {state.hasMore && <button type="button" className="music-more" onClick={onMore} disabled={state.loading}>{state.loading ? "CARREGANDO..." : "MOSTRAR MAIS"}</button>}
  </div>;
}

function TrackList({ tracks, currentUri, context, onPlay, onQueue }: { tracks: SpotifyItem[]; currentUri?: string; context?: string; onPlay: (item: SpotifyItem, context?: string) => void; onQueue: (item: SpotifyItem) => void }) {
  return <ol className="music-tracks">{tracks.map((track) => <li key={track.uri} data-current={track.uri === currentUri}>
    <button type="button" className="music-tracks__main" onClick={() => onPlay(track, context ?? track.albumUri)} title="Tocar (continua pelo álbum ou playlist)">
      {track.image ? <img src={track.image} alt="" /> : <i aria-hidden="true">♪</i>}
      <span><b>{track.name}</b><small>{track.subtitle}</small></span>
      <em>{track.uri === currentUri ? "♪" : clock(track.durationMs ?? 0)}</em>
    </button>
    <button type="button" className="music-tracks__queue" onClick={() => onQueue(track)} title="Adicionar à fila" aria-label={`Adicionar ${track.name} à fila`}>+</button>
  </li>)}</ol>;
}

function Shelf({ title, onAll, children }: { title: string; onAll?: () => void; children: React.ReactNode }) {
  return <div className="music-shelf"><h3>{title}{onAll && <button type="button" onClick={onAll}>VER TODAS →</button>}</h3>{children}</div>;
}

function Cards({ title, items, onPlay, onOpen, onAll, round, empty }: { title: string; items: SpotifyItem[]; onPlay: (item: SpotifyItem) => void; onOpen: (item: SpotifyItem) => void; onAll?: () => void; round?: boolean; empty?: string }) {
  if (!items.length && !empty) return null;
  return <Shelf title={title} onAll={onAll}>{items.length ? <div className="music-cards">{items.map((item) => <div className="music-cards__item" key={item.uri}>
    <button type="button" className="music-cards__open" data-round={round} onClick={() => onOpen(item)} title="Ver as faixas">
      <span className="music-cards__art">{item.image ? <img src={item.image} alt="" /> : <i>♪</i>}</span>
      <b>{item.name}</b><small>{item.subtitle}</small>
    </button>
    <button type="button" className="music-cards__play" onClick={() => onPlay(item)} aria-label={`Tocar ${item.name}`} title="Tocar agora">▶</button>
  </div>)}</div> : <p className="music-muted">{empty}</p>}</Shelf>;
}
