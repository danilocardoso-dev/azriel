import type { ProcessSnapshot, SystemSnapshot, Workspace, WorkspaceInput, WorkspaceStatus } from "../types";
import { invokeDatabase } from "./tauri";

export const systemRepository = {
  snapshot: () => invokeDatabase<SystemSnapshot>("system_snapshot"),
  processes: () => invokeDatabase<ProcessSnapshot[]>("list_processes"),
  listWorkspaces: () => invokeDatabase<Workspace[]>("list_workspaces"),
  saveWorkspace: (input: WorkspaceInput) => invokeDatabase<Workspace[]>("save_workspace", { input }),
  deleteWorkspace: (id: string) => invokeDatabase<Workspace[]>("delete_workspace", { id }),
  workspaceStatus: (workspaceId: string) => invokeDatabase<WorkspaceStatus>("get_workspace_status", { workspaceId }),
};
