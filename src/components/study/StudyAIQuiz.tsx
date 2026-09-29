import { useState } from "react";
import { advanceStudyAIQuiz, answerStudyAIQuiz, createStudyAIQuizState, evaluateStudyAIQuiz } from "../../ai/study/studyAIQuizState";
import type { StudyAIEvaluation, StudyAIQuizQuestion } from "../../types";

type Props = {
  questions: StudyAIQuizQuestion[];
  busy: boolean;
  onEvaluate: (question: StudyAIQuizQuestion, answer: string) => Promise<StudyAIEvaluation>;
  onClose: () => void;
};

const assessmentLabel = {
  CORRECT: "CORRETA",
  PARTIALLY_CORRECT: "PARCIALMENTE CORRETA",
  INCORRECT: "INCORRETA",
  INSUFFICIENT_CONTEXT: "CONTEXTO INSUFICIENTE",
};

export function StudyAIQuiz({ questions, busy, onEvaluate, onClose }: Props) {
  const [state, setState] = useState(() => createStudyAIQuizState(questions));
  const [error, setError] = useState<string | null>(null);
  const question = state.questions[state.index];

  async function evaluate() {
    if (!question || !state.answer.trim()) return;
    setError(null);
    try {
      const evaluation = await onEvaluate(question, state.answer);
      setState((current) => evaluateStudyAIQuiz(current, evaluation));
    }
    catch (reason) { setError(reason instanceof Error ? reason.message : String(reason)); }
  }

  if (state.completed) return <section className="study-ai-quiz study-ai-quiz--complete"><span>QUIZ TEMPORÁRIO</span><h3>Teste concluído</h3><p>{questions.length} perguntas respondidas. Nenhum card ou agendamento foi alterado.</p><button onClick={onClose}>FECHAR</button></section>;
  if (!question) return null;
  return <section className="study-ai-quiz">
    <header><span>QUIZ TEMPORÁRIO</span><strong>{state.index + 1} / {state.questions.length}</strong></header>
    <h3>{question.question}</h3>
    <textarea rows={5} value={state.answer} disabled={busy || Boolean(state.evaluation)} onChange={(event) => setState((current) => answerStudyAIQuiz(current, event.target.value))} placeholder="Digite sua resposta..." />
    {state.evaluation ? <article className="study-ai-evaluation" data-assessment={state.evaluation.assessment}>
      <header><strong>{assessmentLabel[state.evaluation.assessment]}</strong><span>AVALIAÇÃO ASSISTIVA</span></header>
      <p>{state.evaluation.explanation}</p>
      {state.evaluation.strengths.length > 0 && <div><small>PONTOS FORTES</small><ul>{state.evaluation.strengths.map((item) => <li key={item}>{item}</li>)}</ul></div>}
      {state.evaluation.missingPoints.length > 0 && <div><small>PONTOS AUSENTES</small><ul>{state.evaluation.missingPoints.map((item) => <li key={item}>{item}</li>)}</ul></div>}
      <div><small>RESPOSTA SUGERIDA</small><p>{state.evaluation.suggestedAnswer}</p></div>
      <button onClick={() => setState((current) => advanceStudyAIQuiz(current))}>{state.index + 1 === state.questions.length ? "CONCLUIR QUIZ" : "PRÓXIMA PERGUNTA"}</button>
    </article> : <button disabled={busy || !state.answer.trim()} onClick={() => void evaluate()}>{busy ? "AVALIANDO..." : "AVALIAR RESPOSTA"}</button>}
    {error && <p className="study-ai-error">{error}</p>}
  </section>;
}
