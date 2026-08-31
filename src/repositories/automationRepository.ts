import type { ActionHistory, ActionRequest, ActionResult, Application, ApplicationInput, RegisteredAction, RegisteredUrl, RegisteredUrlInput } from "../types";
import { invokeDatabase } from "./tauri";

export const automationRepository = {
  listActions: () => invokeDatabase<RegisteredAction[]>("list_registered_actions"),
  listApplications: () => invokeDatabase<Application[]>("list_applications"),
  saveApplication: (input: ApplicationInput) => invokeDatabase<Application[]>("save_application", { input }),
  deleteApplication: (id: string) => invokeDatabase<Application[]>("delete_application", { id }),
  listUrls: () => invokeDatabase<RegisteredUrl[]>("list_registered_urls"),
  saveUrl: (input: RegisteredUrlInput) => invokeDatabase<RegisteredUrl[]>("save_registered_url", { input }),
  deleteUrl: (id: string) => invokeDatabase<RegisteredUrl[]>("delete_registered_url", { id }),
  listHistory: (limit = 100) => invokeDatabase<ActionHistory[]>("list_action_history", { limit }),
  execute: (request: ActionRequest) => invokeDatabase<ActionResult>("execute_automation_action", { request }),
};
