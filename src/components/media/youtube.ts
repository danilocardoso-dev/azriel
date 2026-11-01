/** Lê um link do YouTube (vídeo, playlist, shorts, live, youtu.be ou só o ID) e monta a URL de embed. */
export interface YouTubeTarget { videoId: string | null; listId: string | null; start: number }

const VIDEO_ID = /^[A-Za-z0-9_-]{11}$/;
const LIST_ID = /^[A-Za-z0-9_-]{10,64}$/;

/** "1h2m3s", "90s" ou "90" → segundos. */
function seconds(value: string | null) {
  if (!value) return 0;
  if (/^\d+$/.test(value)) return Number(value);
  const match = /^(?:(\d+)h)?(?:(\d+)m)?(?:(\d+)s)?$/.exec(value);
  return match ? Number(match[1] ?? 0) * 3600 + Number(match[2] ?? 0) * 60 + Number(match[3] ?? 0) : 0;
}

export function parseYouTube(input: string): YouTubeTarget | null {
  const text = input.trim();
  if (VIDEO_ID.test(text)) return { videoId: text, listId: null, start: 0 };
  let url: URL;
  try { url = new URL(/^https?:\/\//i.test(text) ? text : `https://${text}`); } catch { return null; }
  const host = url.hostname.replace(/^(www|m|music)\./, "");
  let videoId: string | null;
  if (host === "youtu.be") videoId = url.pathname.slice(1).split("/")[0];
  else if (host === "youtube.com" || host === "youtube-nocookie.com") {
    const [, kind, id] = url.pathname.split("/");
    videoId = url.searchParams.get("v") ?? (["shorts", "embed", "live", "v"].includes(kind) ? id ?? null : null);
  } else return null;
  if (videoId && !VIDEO_ID.test(videoId)) videoId = null;
  const list = url.searchParams.get("list");
  const listId = list && LIST_ID.test(list) ? list : null;
  if (!videoId && !listId) return null;
  return { videoId, listId, start: seconds(url.searchParams.get("t") ?? url.searchParams.get("start")) };
}

export function embedUrl(target: YouTubeTarget) {
  const params = new URLSearchParams({ autoplay: "1", rel: "0", modestbranding: "1", playsinline: "1" });
  if (target.listId) params.set("list", target.listId);
  if (target.start > 0) params.set("start", String(target.start));
  const path = target.videoId ? `embed/${target.videoId}` : "embed/videoseries";
  return `https://www.youtube-nocookie.com/${path}?${params}`;
}
