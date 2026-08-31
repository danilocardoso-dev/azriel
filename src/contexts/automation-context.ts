import { createContext } from "react";
import type { ActionHistory, ActionRequest, ActionResult, Application, ApplicationInput, AutomationState, RegisteredAction, RegisteredUrl, RegisteredUrlInput } from "../types";

export interface AutomationContextValue {
  actions: RegisteredAction[]; applications: Application[]; urls: RegisteredUrl[]; history: ActionHistory[];
  state: AutomationState; loading: boolean; error: string | null; lastResult: ActionResult | null;
  refresh: () => Promise<void>; execute: (request: ActionRequest) => Promise<ActionResult>;
  saveApplication: (input: ApplicationInput) => Promise<void>; deleteApplication: (id: string) => Promise<void>;
  saveUrl: (input: RegisteredUrlInput) => Promise<void>; deleteUrl: (id: string) => Promise<void>;
  selectApplication: () => Promise<string | null>;
}

export const AutomationContext = createContext<AutomationContextValue | null>(null);
