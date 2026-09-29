import type { StudyAIGeneratedCard } from "../../types";

export interface StudyAICardDraft extends StudyAIGeneratedCard { id: string; selected: boolean; saved: boolean }

export const createStudyAICardDrafts = (cards: StudyAIGeneratedCard[], idFactory: () => string = () => crypto.randomUUID()): StudyAICardDraft[] => cards.map((card) => ({ ...card, id: idFactory(), selected: true, saved: false }));
export const updateStudyAICardDraft = (drafts: StudyAICardDraft[], id: string, patch: Partial<Pick<StudyAICardDraft, "front" | "back" | "selected">>) => drafts.map((draft) => draft.id === id && !draft.saved ? { ...draft, ...patch } : draft);
export const discardStudyAICardDraft = (drafts: StudyAICardDraft[], id: string) => drafts.filter((draft) => draft.id !== id || draft.saved);
export const selectedStudyAICardDrafts = (drafts: StudyAICardDraft[]) => drafts.filter((draft) => draft.selected && !draft.saved && draft.front.trim() && draft.back.trim());
export const markStudyAICardDraftsSaved = (drafts: StudyAICardDraft[], ids: Set<string>) => drafts.map((draft) => ids.has(draft.id) ? { ...draft, saved: true, selected: false } : draft);
