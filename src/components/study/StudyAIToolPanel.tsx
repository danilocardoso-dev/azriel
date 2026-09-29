import { useMemo, useRef, useState } from "react";
import { createStudyAICardDrafts, markStudyAICardDraftsSaved, selectedStudyAICardDrafts, type StudyAICardDraft } from "../../ai/study/studyAIDrafts";
import { StudyAIService } from "../../ai/study/StudyAIService";
import { OllamaProvider } from "../../ai/providers/OllamaProvider";
import { useAI } from "../../contexts/useAI";
import type { StudyAIAction, StudyAIContext, StudyAIEvaluation, StudyAIQuizQuestion } from "../../types";
import { StudyMarkdown } from "./StudyMarkdown";
import { StudyAICardDrafts } from "./StudyAICardDrafts";
import { StudyAIQuiz } from "./StudyAIQuiz";

type Props = {
  context: StudyAIContext;
  actions?: Array<Exclude<StudyAIAction, "EVALUATE_ANSWER">>;
  onInsertAtEnd?: (content: string) => void;
  onSaveCards?: (drafts: StudyAICardDraft[]) => Promise<void>;
  onClose?: () => void;
};

const labels = { EXPLAIN: "EXPLICAR", SUMMARIZE: "RESUMIR", QUIZ: "ME TESTAR", GENERATE_CARDS: "GERAR CARDS" };

export function StudyAIToolPanel({ context, actions = ["EXPLAIN", "SUMMARIZE", "QUIZ", "GENERATE_CARDS"], onInsertAtEnd, onSaveCards, onClose }: Props) {
  const { settings, status } = useAI();
  const [activeAction, setActiveAction] = useState<Exclude<StudyAIAction, "EVALUATE_ANSWER"> | null>(null);
  const [count, setCount] = useState<3 | 5 | 10>(5);
  const [instruction, setInstruction] = useState("");
  const [result, setResult] = useState<Awaited<ReturnType<StudyAIService["run"]>> | null>(null);
  const [drafts, setDrafts] = useState<StudyAICardDraft[]>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const requestToken = useRef(0);
  const service = useMemo(() => settings ? new StudyAIService(new OllamaProvider(settings), settings) : null, [settings]);

  async function execute(action: Exclude<StudyAIAction, "EVALUATE_ANSWER">) {
    if (!service || !status?.available) { setError(status?.error || "AI Core offline. Verifique o Ollama nas Configurações."); return; }
    const token = ++requestToken.current;
    setActiveAction(action); setBusy(true); setError(null); setResult(null); setDrafts([]);
    const next = await service.run(action, context, { count, userInstruction: instruction || null, isCancelled: () => token !== requestToken.current });
    if (token !== requestToken.current) return;
    setBusy(false);
    if (next.status !== "SUCCESS") { setError(next.error || `A solicitação terminou com status ${next.status}.`); setResult(next); return; }
    setResult(next);
    if (next.cards) setDrafts(createStudyAICardDrafts(next.cards));
  }

  function discard() {
    requestToken.current += 1;
    setBusy(false); setResult(null); setDrafts([]); setError(null); setActiveAction(null);
  }

  async function copy(content: string) {
    try { await navigator.clipboard.writeText(content); }
    catch { setError("Não foi possível copiar o resultado para a área de transferência."); }
  }

  async function evaluate(question: StudyAIQuizQuestion, answer: string): Promise<StudyAIEvaluation> {
    if (!service) throw new Error("AI Core ainda não foi inicializado.");
    setBusy(true); setError(null);
    try {
      const evaluation = await service.run("EVALUATE_ANSWER", { ...context, review: { question: question.question, expectedAnswer: question.expectedAnswer, userAnswer: answer } });
      if (evaluation.status !== "SUCCESS" || !evaluation.evaluation) throw new Error(evaluation.error || "A avaliação retornou uma estrutura inválida.");
      return evaluation.evaluation;
    } finally { setBusy(false); }
  }

  async function saveDrafts() {
    if (!onSaveCards) return;
    const selected = selectedStudyAICardDrafts(drafts);
    if (!selected.length) return;
    setBusy(true); setError(null);
    try {
      await onSaveCards(selected);
      setDrafts((current) => markStudyAICardDraftsSaved(current, new Set(selected.map((item) => item.id))));
    } catch (reason) { setError(reason instanceof Error ? reason.message : String(reason)); }
    finally { setBusy(false); }
  }

  return <section className="study-ai-panel">
    <header><div><span>AZRIEL AI</span><strong>FERRAMENTAS CONTEXTUAIS</strong></div><div><i data-state={status?.available ? "online" : "offline"}>{status?.available ? "LOCAL / ONLINE" : "OFFLINE"}</i>{onClose && <button onClick={onClose}>×</button>}</div></header>
    <nav>{actions.map((action) => <button key={action} className={activeAction === action ? "active" : ""} disabled={busy || !status?.available} onClick={() => void execute(action)}>{labels[action]}</button>)}</nav>
    {actions.some((action) => action === "QUIZ" || action === "GENERATE_CARDS") && <div className="study-ai-options"><label>QUANTIDADE<select value={count} disabled={busy} onChange={(event) => setCount(Number(event.target.value) as 3 | 5 | 10)}><option value={3}>3</option><option value={5}>5</option><option value={10}>10</option></select></label><label>INSTRUÇÃO OPCIONAL<input maxLength={1_000} value={instruction} disabled={busy} onChange={(event) => setInstruction(event.target.value)} placeholder="Ex.: use exemplos práticos" /></label></div>}
    {busy && <div className="study-ai-loading"><span /><strong>PROCESSANDO LOCALMENTE...</strong><button onClick={discard}>CANCELAR EXIBIÇÃO</button></div>}
    {result?.contextTruncated && <p className="study-ai-warning">CONTEXTO TRUNCADO — o resultado foi produzido com material parcial.</p>}
    {result && result.materialIds.length > 0 && <div className="study-ai-sources"><strong>FONTES DO CONTEXTO</strong>{result.materialIds.map((id) => <span key={id}>{context.materials?.find((material) => material.id === id)?.title || id}</span>)}</div>}
    {error && <p className="study-ai-error">{error}</p>}
    {result?.content && <article className="study-ai-result"><header><span>{result.promptVersion}</span><small>{result.model} · {result.latencyMs} ms</small></header><StudyMarkdown content={result.content} /><footer><button onClick={() => void copy(result.content!)}>COPIAR</button>{onInsertAtEnd && <button onClick={() => onInsertAtEnd(result.content!)}>INSERIR NO FINAL DA NOTA</button>}<button onClick={discard}>DESCARTAR</button></footer></article>}
    {result?.quiz && <StudyAIQuiz questions={result.quiz} busy={busy} onEvaluate={evaluate} onClose={discard} />}
    {drafts.length > 0 && <StudyAICardDrafts drafts={drafts} busy={busy} onChange={setDrafts} onSave={saveDrafts} onDiscard={discard} />}
  </section>;
}
