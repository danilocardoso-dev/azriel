import { describe, expect, it } from "vitest";
import { embedUrl, parseYouTube } from "./youtube";

describe("links do YouTube", () => {
  it("aceita os formatos comuns de vídeo", () => {
    const id = "dQw4w9WgXcQ";
    for (const link of [`https://www.youtube.com/watch?v=${id}`, `youtu.be/${id}`, `https://m.youtube.com/watch?v=${id}&feature=share`, `https://www.youtube.com/shorts/${id}`, `https://www.youtube.com/live/${id}`, `https://www.youtube.com/embed/${id}`, id]) {
      expect(parseYouTube(link)?.videoId, link).toBe(id);
    }
  });

  it("guarda playlist e tempo inicial de uma aula", () => {
    expect(parseYouTube("https://www.youtube.com/watch?v=dQw4w9WgXcQ&list=PLx0sYbCqOb8TBPRdmBHs5Iftvv9TPboYG&t=1h2m3s")).toEqual({ videoId: "dQw4w9WgXcQ", listId: "PLx0sYbCqOb8TBPRdmBHs5Iftvv9TPboYG", start: 3723 });
    expect(parseYouTube("https://youtube.com/playlist?list=PLx0sYbCqOb8TBPRdmBHs5Iftvv9TPboYG")).toEqual({ videoId: null, listId: "PLx0sYbCqOb8TBPRdmBHs5Iftvv9TPboYG", start: 0 });
  });

  it("recusa o que não é YouTube", () => {
    expect(parseYouTube("https://vimeo.com/123")).toBeNull();
    expect(parseYouTube("qualquer coisa")).toBeNull();
    expect(parseYouTube("https://www.youtube.com/")).toBeNull();
  });

  it("monta o embed sem cookies, com autoplay, playlist e início", () => {
    expect(embedUrl({ videoId: "dQw4w9WgXcQ", listId: null, start: 90 })).toBe("https://www.youtube-nocookie.com/embed/dQw4w9WgXcQ?autoplay=1&rel=0&modestbranding=1&playsinline=1&start=90");
    expect(embedUrl({ videoId: null, listId: "PLabcdefghij", start: 0 })).toContain("embed/videoseries?");
  });
});
