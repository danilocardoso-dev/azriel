import type { MediaAction, NowPlaying } from "../types";
import { invokeDatabase } from "./tauri";

export const mediaRepository = {
  nowPlaying: () => invokeDatabase<NowPlaying | null>("media_now_playing"),
  control: (action: MediaAction) => invokeDatabase<void>("media_control", { action }),
};
