import { describe, expect, it } from "vitest";
import { diagnosticAvailability, qualityLabel } from "./holdDiagnosticsView";

describe("market hold diagnostics UI rules", () => {
  it("only enables completed AI Intraday 15M experiments", () => {
    expect(diagnosticAvailability(["ai-intraday-v1"], "15M", "completed")).toBe(true);
    expect(diagnosticAvailability(["ai-intraday-v1"], "1D", "completed")).toBe(false);
    expect(diagnosticAvailability(["simple-trend"], "15M", "completed")).toBe(false);
  });

  it("distinguishes a flat good hold as good avoidance", () => {
    expect(qualityLabel("GOOD_HOLD", "FLAT")).toBe("GOOD AVOIDANCE");
    expect(qualityLabel("GOOD_HOLD", "LONG")).toBe("GOOD HOLD");
  });
});
