import type { StudyNoteInput } from "../../types";

export const studyNoteFingerprint = (note: StudyNoteInput) => JSON.stringify(note);

export function reconcileStudyNoteSave(snapshot: StudyNoteInput, current: StudyNoteInput, persisted: StudyNoteInput) {
  const draft = studyNoteFingerprint(current) === studyNoteFingerprint(snapshot) ? persisted : current;
  return {
    draft,
    persisted,
    dirty: studyNoteFingerprint(draft) !== studyNoteFingerprint(persisted),
  };
}
