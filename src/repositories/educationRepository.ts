import type { EducationInput, EducationItem } from "../types";
import { invokeDatabase } from "./tauri";
export const educationRepository = {
  list: () => invokeDatabase<EducationItem[]>("list_education"),
  save: (input: EducationInput) => invokeDatabase<EducationItem[]>("save_education", { input }),
  remove: (id: string) => invokeDatabase<EducationItem[]>("delete_education", { id }),
};
