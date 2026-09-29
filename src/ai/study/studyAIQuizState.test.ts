import { describe, expect, it } from "vitest";
import { advanceStudyAIQuiz, answerStudyAIQuiz, createStudyAIQuizState, evaluateStudyAIQuiz } from "./studyAIQuizState";

describe("study AI temporary quiz state", () => {
  it("navega até a conclusão sem qualquer estado de scheduler", () => {
    let state = createStudyAIQuizState([{ question: "Q1", expectedAnswer: "A1" }, { question: "Q2", expectedAnswer: "A2" }]);
    state = answerStudyAIQuiz(state, "R1");
    state = evaluateStudyAIQuiz(state, { assessment: "CORRECT", explanation: "Ok", missingPoints: [], strengths: ["Certo"], suggestedAnswer: "A1" });
    state = advanceStudyAIQuiz(state);
    expect(state.index).toBe(1);
    expect(state.completed).toBe(false);
    state = evaluateStudyAIQuiz(answerStudyAIQuiz(state, "R2"), { assessment: "INCORRECT", explanation: "Não", missingPoints: ["A2"], strengths: [], suggestedAnswer: "A2" });
    state = advanceStudyAIQuiz(state);
    expect(state.completed).toBe(true);
    expect(state).not.toHaveProperty("reviewInterval");
  });

  it("não avança sem avaliação", () => expect(() => advanceStudyAIQuiz(createStudyAIQuizState([{ question: "Q", expectedAnswer: "A" }]))).toThrow());
});
