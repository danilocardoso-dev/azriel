import { describe, expect, it } from "vitest";
import type { LocalAiModelStatus } from "../../types";
import { localAiCardState } from "./localAiLifecycle";

const status = (patch: Partial<LocalAiModelStatus> = {}): LocalAiModelStatus => ({
  provider: "OLLAMA", model: "qwen3.5:4b", serverStatus: "ONLINE", modelStatus: "UNLOADED",
  loaded: false, expiresAt: null, lastCheckedAt: "2026-09-26T00:00:00Z", error: null, ...patch,
});

describe("localAiCardState", () => {
  it("mapeia unloaded, loaded e servidor offline para as acoes corretas", () => {
    expect(localAiCardState(status(), null).action).toBe("load");
    expect(localAiCardState(status({ loaded: true, modelStatus: "LOADED" }), null).action).toBe("unload");
    expect(localAiCardState(status({ serverStatus: "OFFLINE", modelStatus: "ERROR", error: "offline" }), null).action).toBe("refresh");
  });

  it("bloqueia acoes durante loading e unloading", () => {
    expect(localAiCardState(status(), "LOADING")).toMatchObject({ effectiveState: "LOADING", disabled: true });
    expect(localAiCardState(status({ loaded: true, modelStatus: "LOADED" }), "UNLOADING")).toMatchObject({ effectiveState: "UNLOADING", disabled: true });
  });

  it("nao tenta carregar automaticamente um modelo indisponivel", () => {
    expect(localAiCardState(status({ modelStatus: "NOT_AVAILABLE" }), null)).toMatchObject({ action: "load", disabled: true });
  });
});
