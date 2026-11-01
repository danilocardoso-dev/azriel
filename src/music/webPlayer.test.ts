import { afterEach, describe, expect, it, vi } from "vitest";
import { describeSdkError, widevineSupport } from "./webPlayer";

afterEach(() => vi.unstubAllGlobals());

describe("player embutido do Spotify", () => {
  it("detecta o suporte a DRM pelo Widevine", async () => {
    vi.stubGlobal("navigator", { requestMediaKeySystemAccess: vi.fn().mockResolvedValue({}) });
    expect(await widevineSupport()).toBe("ok");
    vi.stubGlobal("navigator", { requestMediaKeySystemAccess: vi.fn().mockRejectedValue(new Error("sem CDM")) });
    expect(await widevineSupport()).toBe("no");
    vi.stubGlobal("navigator", {});
    expect(await widevineSupport()).toBe("no");
  });

  it("explica cada erro do SDK e só muda o estado quando o erro é fatal", () => {
    expect(describeSdkError("authentication_error", "Invalid token scopes.")).toMatchObject({ status: "needs-reauth" });
    expect(describeSdkError("authentication_error").text).toContain("Reconecte");
    expect(describeSdkError("account_error")).toMatchObject({ status: "error" });
    expect(describeSdkError("account_error").text).toContain("Premium");
    expect(describeSdkError("initialization_error", "EME")).toMatchObject({ status: "error" });
    expect(describeSdkError("playback_error", "falhou")).toEqual({ status: null, text: "Erro de reprodução (falhou)." });
  });
});
