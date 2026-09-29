import type { LocalAiModelState, LocalAiModelStatus } from "../../types";

export type LocalAiTransition = "LOADING" | "UNLOADING" | null;

export interface LocalAiCardState {
  effectiveState: LocalAiModelState;
  headline: "ONLINE" | "STANDBY" | "OFFLINE" | "ERRO";
  actionLabel: "ATIVAR IA" | "DESATIVAR IA" | "ATIVANDO..." | "DESATIVANDO..." | "VERIFICAR";
  action: "load" | "unload" | "refresh";
  disabled: boolean;
}

export function localAiCardState(status: LocalAiModelStatus | null, transition: LocalAiTransition): LocalAiCardState {
  if (transition === "LOADING") return { effectiveState: "LOADING", headline: "STANDBY", actionLabel: "ATIVANDO...", action: "load", disabled: true };
  if (transition === "UNLOADING") return { effectiveState: "UNLOADING", headline: "ONLINE", actionLabel: "DESATIVANDO...", action: "unload", disabled: true };
  if (!status || status.serverStatus !== "ONLINE") return { effectiveState: status?.modelStatus ?? "UNKNOWN", headline: "OFFLINE", actionLabel: "VERIFICAR", action: "refresh", disabled: false };
  if (status.loaded) return { effectiveState: "LOADED", headline: "ONLINE", actionLabel: "DESATIVAR IA", action: "unload", disabled: false };
  if (status.modelStatus === "ERROR") return { effectiveState: "ERROR", headline: "ERRO", actionLabel: "VERIFICAR", action: "refresh", disabled: false };
  return { effectiveState: status.modelStatus, headline: "STANDBY", actionLabel: "ATIVAR IA", action: "load", disabled: status.modelStatus === "NOT_AVAILABLE" };
}
