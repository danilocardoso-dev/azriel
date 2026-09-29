import type { StudyAIEvaluation, StudyAIQuizQuestion } from "../../types";

export interface StudyAIQuizState {
  questions: StudyAIQuizQuestion[];
  index: number;
  answer: string;
  evaluation: StudyAIEvaluation | null;
  completed: boolean;
}

export const createStudyAIQuizState = (questions: StudyAIQuizQuestion[]): StudyAIQuizState => ({ questions, index: 0, answer: "", evaluation: null, completed: questions.length === 0 });
export const answerStudyAIQuiz = (state: StudyAIQuizState, answer: string): StudyAIQuizState => ({ ...state, answer, evaluation: null });
export const evaluateStudyAIQuiz = (state: StudyAIQuizState, evaluation: StudyAIEvaluation): StudyAIQuizState => ({ ...state, evaluation });
export function advanceStudyAIQuiz(state: StudyAIQuizState): StudyAIQuizState {
  if (!state.evaluation) throw new Error("Avalie a resposta antes de avançar.");
  const nextIndex = state.index + 1;
  return { ...state, index: Math.min(nextIndex, state.questions.length), answer: "", evaluation: null, completed: nextIndex >= state.questions.length };
}
