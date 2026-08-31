import { createContext } from "react";
import type { ProcessSnapshot, SystemSnapshot, Workspace, WorkspaceInput, WorkspaceStatus } from "../types";

export interface SystemContextValue {
  snapshot: SystemSnapshot | null;
  processes: ProcessSnapshot[];
  workspaces: Workspace[];
  selectedWorkspace: WorkspaceStatus | null;
  loading: boolean;
  processLoading: boolean;
  error: string | null;
  refreshSnapshot: () => Promise<void>;
  refreshProcesses: () => Promise<void>;
  refreshWorkspaces: () => Promise<void>;
  inspectWorkspace: (id: string) => Promise<void>;
  saveWorkspace: (input: WorkspaceInput) => Promise<void>;
  deleteWorkspace: (id: string) => Promise<void>;
  selectDirectory: () => Promise<string | null>;
}

export const SystemContext = createContext<SystemContextValue | null>(null);
