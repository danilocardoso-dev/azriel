import { describe, expect, it } from "vitest";
import { createStudyAICardDrafts, discardStudyAICardDraft, markStudyAICardDraftsSaved, selectedStudyAICardDrafts, updateStudyAICardDraft } from "./studyAIDrafts";

describe("study AI card drafts", () => {
  it("permite editar, desmarcar, descartar e congelar drafts salvos", () => {
    let sequence = 0;
    let drafts = createStudyAICardDrafts([{ front: "F1", back: "B1" }, { front: "F2", back: "B2" }], () => `draft-${++sequence}`);
    drafts = updateStudyAICardDraft(drafts, "draft-1", { front: "F1 editada" });
    drafts = updateStudyAICardDraft(drafts, "draft-2", { selected: false });
    expect(selectedStudyAICardDrafts(drafts).map((item) => item.id)).toEqual(["draft-1"]);
    drafts = markStudyAICardDraftsSaved(drafts, new Set(["draft-1"]));
    expect(updateStudyAICardDraft(drafts, "draft-1", { front: "não altera" })[0].front).toBe("F1 editada");
    expect(discardStudyAICardDraft(drafts, "draft-1")).toHaveLength(2);
    expect(discardStudyAICardDraft(drafts, "draft-2")).toHaveLength(1);
  });
});
