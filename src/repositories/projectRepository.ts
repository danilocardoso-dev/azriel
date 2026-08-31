import type { Project, ProjectInput } from "../types";
import { invokeDatabase } from "./tauri";
export const projectRepository = {
  list: () => invokeDatabase<Project[]>("list_projects"),
  get: (id: string) => invokeDatabase<Project | null>("get_project", { id }),
  save: (input: ProjectInput) => invokeDatabase<Project[]>("save_project", { input }),
  remove: (id: string) => invokeDatabase<Project[]>("delete_project", { id }),
};
