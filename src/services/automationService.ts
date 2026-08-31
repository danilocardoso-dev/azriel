import { open } from "@tauri-apps/plugin-dialog";
import { automationRepository } from "../repositories/automationRepository";
import type { ActionRequest, RoutineExecutionResult, RunRoutineRequest } from "../types";

export const ROUTINE_RESULT_EVENT = "azriel:routine-result";
const publish = (result: RoutineExecutionResult) => {
  window.dispatchEvent(new CustomEvent<RoutineExecutionResult>(ROUTINE_RESULT_EVENT, { detail: result }));
  return result;
};

export const automationService = {
  ...automationRepository,
  execute: (request: ActionRequest) => automationRepository.execute(request),
  runRoutine: async (request: RunRoutineRequest) => publish(await automationRepository.runRoutine(request)),
  confirmRoutine: async (historyId: number) => publish(await automationRepository.confirmRoutine(historyId)),
  cancelRoutine: (historyId: number) => automationRepository.cancelRoutine(historyId),
  async selectApplication() {
    const selection = await open({ multiple: false, directory: false, filters: [{ name: "Aplicativo Windows", extensions: ["exe"] }] });
    return typeof selection === "string" ? selection : null;
  },
};
