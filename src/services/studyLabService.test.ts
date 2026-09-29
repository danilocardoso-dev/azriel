import { describe, expect, it } from "vitest";
import { formatReviewInterval, formatStudyDuration, formatStudyTimer, remainingFocusSeconds, studyCardSuccessRate } from "./studyLabService";
import type { StudySession } from "../types";

const session = (currentFocusSeconds: number): StudySession => ({
  id: "session", roadmapId: "roadmap", stageId: "stage", topicId: "topic", activityId: "activity",
  roadmapName: "Roadmap", stageName: "Etapa", topicName: "Tópico", activityTitle: "Atividade",
  plannedFocusMinutes: 25, actualFocusSeconds: currentFocusSeconds, currentFocusSeconds, linkedNoteCount: 0, observedAt: "2026-09-25T12:00:00Z", breakSeconds: 0,
  status: "ACTIVE", startedAt: "2026-09-25T12:00:00Z", runningSince: "2026-09-25T12:00:00Z",
  pausedAt: null, endedAt: null, createdAt: "2026-09-25T12:00:00Z", updatedAt: "2026-09-25T12:00:00Z",
});

describe("studyLabService", () => {
  it("formata duração e timer sem valores negativos", () => {
    expect(formatStudyDuration(3720)).toBe("1h 02min");
    expect(formatStudyDuration(-1)).toBe("0min");
    expect(formatStudyTimer(65)).toBe("01:05");
    expect(formatStudyTimer(3661)).toBe("01:01:01");
  });

  it("calcula o tempo restante a partir do valor efetivo do backend", () => {
    expect(remainingFocusSeconds(session(300))).toBe(1200);
    expect(remainingFocusSeconds(session(1800))).toBe(0);
  });

  it("formata intervalos e calcula taxa histórica sem inventar mastery", () => {
    expect(formatReviewInterval(600)).toBe("10 min");
    expect(formatReviewInterval(86_400)).toBe("1 d");
    expect(studyCardSuccessRate(3, 4)).toBe(75);
    expect(studyCardSuccessRate(0, 0)).toBeNull();
  });
});
