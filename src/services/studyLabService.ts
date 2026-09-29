import { studyLabRepository } from "../repositories/studyLabRepository";
import { open } from "@tauri-apps/plugin-dialog";
import type { StudySession } from "../types";

export function formatStudyDuration(totalSeconds: number): string {
  const seconds = Math.max(0, Math.floor(totalSeconds));
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  if (hours) return `${hours}h ${minutes.toString().padStart(2, "0")}min`;
  return `${minutes}min`;
}

export function formatStudyTimer(totalSeconds: number): string {
  const seconds = Math.max(0, Math.floor(totalSeconds));
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const remainder = seconds % 60;
  return hours
    ? `${hours.toString().padStart(2, "0")}:${minutes.toString().padStart(2, "0")}:${remainder.toString().padStart(2, "0")}`
    : `${minutes.toString().padStart(2, "0")}:${remainder.toString().padStart(2, "0")}`;
}

export function remainingFocusSeconds(session: StudySession): number {
  return Math.max(0, session.plannedFocusMinutes * 60 - session.currentFocusSeconds);
}

export function studyCardSuccessRate(correct: number, reviews: number): number | null {
  if (reviews <= 0) return null;
  return Math.round((Math.max(0, correct) / reviews) * 100);
}

export function formatReviewInterval(totalSeconds: number): string {
  const seconds = Math.max(0, Math.floor(totalSeconds));
  if (seconds < 3600) return `${Math.max(1, Math.round(seconds / 60))} min`;
  if (seconds < 86400) return `${Math.max(1, Math.round(seconds / 3600))} h`;
  return `${Math.max(1, Math.round(seconds / 86400))} d`;
}

export const studyLabService = {
  ...studyLabRepository,
  async selectMaterialFile() {
    const selection = await open({
      multiple: false,
      directory: false,
      title: "Adicionar material ao Study Lab",
      filters: [{ name: "Materiais de estudo", extensions: ["pdf", "png", "jpg", "jpeg", "gif", "webp", "txt", "md", "markdown"] }],
    });
    return typeof selection === "string" ? selection : null;
  },
};
