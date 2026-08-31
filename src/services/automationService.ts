import { open } from "@tauri-apps/plugin-dialog";
import { automationRepository } from "../repositories/automationRepository";
import type { ActionRequest } from "../types";

export const automationService = {
  ...automationRepository,
  execute: (request: ActionRequest) => automationRepository.execute(request),
  async selectApplication() {
    const selection = await open({ multiple: false, directory: false, filters: [{ name: "Aplicativo Windows", extensions: ["exe"] }] });
    return typeof selection === "string" ? selection : null;
  },
};
