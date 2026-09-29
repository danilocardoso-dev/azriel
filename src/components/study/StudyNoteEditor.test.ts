import { describe, expect, it } from "vitest";
import type { StudyNoteInput } from "../../types";
import { reconcileStudyNoteSave, studyNoteFingerprint } from "./studyNoteDraft";

const note: StudyNoteInput = { id: "note", notebookId: "book", title: "Título", content: "Texto", roadmapId: null, stageId: null, topicId: null, activityId: null, studySessionId: null };

describe("StudyNoteEditor autosave identity", () => {
  it("distingue conteúdo local alterado do último conteúdo persistido", () => {
    expect(studyNoteFingerprint(note)).toBe(studyNoteFingerprint({ ...note }));
    expect(studyNoteFingerprint(note)).not.toBe(studyNoteFingerprint({ ...note, content: "Novo texto" }));
  });

  it("inclui vínculos contextuais na identidade persistida", () => {
    expect(studyNoteFingerprint(note)).not.toBe(studyNoteFingerprint({ ...note, studySessionId: "session" }));
  });

  it("aceita a versão normalizada do backend quando não houve nova edição", () => {
    const snapshot = { ...note, title: " Título " };
    const result = reconcileStudyNoteSave(snapshot, snapshot, note);
    expect(result.draft.title).toBe("Título");
    expect(result.dirty).toBe(false);
  });

  it("preserva a edição feita enquanto a gravação anterior estava em andamento", () => {
    const snapshot = { ...note, content: "Primeira edição" };
    const current = { ...note, content: "Segunda edição" };
    const result = reconcileStudyNoteSave(snapshot, current, snapshot);
    expect(result.draft.content).toBe("Segunda edição");
    expect(result.dirty).toBe(true);
  });
});
