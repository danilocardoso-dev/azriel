import { useMemo, useState } from "react";
import type { MarketAiDecisionLog, MarketAiExperimentComparison, MarketAiRuntimeMetric, MarketAiStatus, MarketSignalDiagnostics } from "../../types";

type DecisionFilter = "ALL" | "BUY" | "SELL" | "HOLD" | "NO_LLM_CALL";

const valueAt = (source: Record<string, unknown> | null, path: string[]): unknown => {
  let current: unknown = source;
  for (const key of path) {
    if (!current || typeof current !== "object" || Array.isArray(current)) return null;
    const object = current as Record<string, unknown>;
    if (key === "portfolio" && object[key] == null && object.position != null) {
      current = object.position;
    } else if ((key === "position_pct" || key === "exposure_pct") && object[key] == null) {
      current = object.current_exposure_pct ?? object.exposurePct;
    } else if (key === "holding_period" && object[key] == null) {
      current = object.holding_candles ?? object.holdingCandles;
    } else {
      current = object[key];
    }
  }
  return current;
};

const printable = (value: unknown, suffix = "") => typeof value === "number" ? `${value.toFixed(4)}${suffix}` : typeof value === "string" || typeof value === "boolean" ? `${String(value)}${suffix}` : "N/A";
const percentage = (value: number | null) => value == null ? "N/A" : `${value >= 0 ? "+" : ""}${value.toFixed(2)}%`;
const average = (values: number[]) => values.length === 0 ? null : values.reduce((sum, value) => sum + value, 0) / values.length;
const optionalRate = (value: number | null) => value == null ? "N/A" : `${value.toFixed(1)}%`;

function ReliabilityPanel({ runtime, decisions }: { runtime: MarketAiRuntimeMetric; decisions: MarketAiDecisionLog[] }) {
  if (!["MARKET_AI_AGENT_V4_1", "MARKET_AI_AGENT_V4_2"].includes(runtime.promptVersion)) return null;
  const invalidTypes = decisions.flatMap((decision) => decision.outputAttempts).reduce<Record<string, number>>((counts, attempt) => {
    if (attempt.errorType) counts[attempt.errorType] = (counts[attempt.errorType] ?? 0) + 1;
    return counts;
  }, {});
  return <section className="market-ai-reliability">
    <div className="panel-heading"><strong>AI OUTPUT RELIABILITY</strong><span>{runtime.promptVersion === "MARKET_AI_AGENT_V4_2" ? "V4.2 · INTENT CONTRACT" : "V4.1 · STRUCTURED OUTPUT CONTRACT"}</span></div>
    <div className="market-ai-runtime__metrics">
      <div><span>RAW CALLS</span><strong>{runtime.callCount}</strong></div>
      <div><span>FIRST-PASS VALID</span><strong>{runtime.firstPassValidCount} · {runtime.firstPassValidRatePct.toFixed(1)}%</strong></div>
      <div><span>RETRY RECOVERED</span><strong>{runtime.retryRecoveredCount} · {runtime.retryRecoveryRatePct.toFixed(1)}%</strong></div>
      <div><span>FINAL VALID</span><strong>{runtime.finalValidCount} · {runtime.finalValidRatePct.toFixed(1)}%</strong></div>
      <div><span>FINAL INVALID</span><strong>{runtime.finalInvalidCount}</strong></div>
      <div><span>SYSTEM FALLBACKS</span><strong>{runtime.systemFallbackCount} · {runtime.fallbackRatePct.toFixed(1)}%</strong></div>
      <div><span>FIRST ATTEMPT AVG</span><strong>{runtime.firstAttemptAverageLatencyMs.toFixed(0)} ms</strong></div>
      <div><span>RETRY AVG</span><strong>{runtime.retryAverageLatencyMs.toFixed(0)} ms</strong></div>
      <div><span>TOTAL AVG / P50 / P95</span><strong>{runtime.averageLatencyMs.toFixed(0)} / {runtime.p50LatencyMs.toFixed(0)} / {runtime.p95LatencyMs.toFixed(0)} ms</strong></div>
      <div className={runtime.slowCallCount > 0 ? "warning" : ""}><span>SLOW AI CALLS &gt; 5S</span><strong>{runtime.slowCallCount}</strong></div>
      {runtime.promptVersion === "MARKET_AI_AGENT_V4_2" && <>
        <div className={(invalidTypes.INVALID_INTENT ?? 0) > 0 ? "warning" : ""}><span>INVALID INTENT</span><strong>{invalidTypes.INVALID_INTENT ?? 0}</strong></div>
        <div className={(invalidTypes.INVALID_POSITION_INTENT ?? 0) > 0 ? "warning" : ""}><span>INVALID POSITION INTENT</span><strong>{invalidTypes.INVALID_POSITION_INTENT ?? 0}</strong></div>
        <div className={(invalidTypes.INVALID_CONFIDENCE_PCT ?? 0) > 0 ? "warning" : ""}><span>INVALID CONFIDENCE PCT</span><strong>{invalidTypes.INVALID_CONFIDENCE_PCT ?? 0}</strong></div>
      </>}
    </div>
    <div className="market-ai-invalid-types"><strong>INVALID TYPE DISTRIBUTION</strong>{Object.keys(invalidTypes).length === 0 ? <span>Nenhuma saída inválida registrada.</span> : Object.entries(invalidTypes).sort((left, right) => right[1] - left[1]).map(([type, count]) => <span key={type}>{type}<b>{count}</b></span>)}</div>
    <small>FINAL VALID RATE = decisões válidas finais ÷ oportunidades de decisão. Chamadas puladas por cadência não entram no denominador.</small>
  </section>;
}

function DecisionPipeline({ decision }: { decision: MarketAiDecisionLog }) {
  if (decision.outputAttempts.length === 0) return null;
  return <section className="market-ai-pipeline">
    <div className="panel-heading"><strong>STRUCTURED OUTPUT PIPELINE</strong><span>{decision.firstFailureType ?? "FIRST PASS VALID"}</span></div>
    {decision.outputAttempts.map((attempt) => <article key={attempt.attemptNumber}>
      <header><strong>ATTEMPT {attempt.attemptNumber}</strong><span>{attempt.status} · {attempt.latencyMs} ms{attempt.normalizedFromWrappedJson ? " · NORMALIZED_FROM_WRAPPED_JSON" : ""}</span></header>
      {attempt.errorType && <dl><div><dt>INVALID REASON</dt><dd>{attempt.errorType}</dd></div><div><dt>FIELD</dt><dd>{attempt.errorField ?? "N/A"}</dd></div><div><dt>VALUE</dt><dd>{attempt.errorValue ?? "N/A"}</dd></div><div><dt>MESSAGE</dt><dd>{attempt.errorMessage ?? "N/A"}</dd></div></dl>}
      <details><summary>AI RAW OUTPUT · RESPOSTA EFETIVAMENTE RECEBIDA</summary><pre>{attempt.rawResponse ?? "PROVIDER_ERROR · sem resposta bruta"}</pre></details>
      <div className="market-ai-pipeline__stages">{attempt.stages.map((stage, index) => <div key={`${stage.stage}-${index}`} className={stage.success ? "pass" : "fail"}><span>{stage.stage.replaceAll("_", " ")}</span><strong>{stage.success ? "PASS" : "FAIL"}</strong><small>{stage.errorCode ?? stage.message ?? "OK"}</small></div>)}</div>
    </article>)}
    <div className="market-ai-pipeline__final"><div><span>POSITION FALLBACK</span><strong>{decision.fallbackUsed ? `${decision.lifecycleAction ?? decision.action} · ${decision.fallbackReason ?? "SYSTEM FALLBACK"}` : "NOT USED"}</strong></div><div><span>RISK</span><strong>{decision.riskResult} · {decision.approvedPositionPct?.toFixed(2) ?? "N/A"}%</strong></div><div><span>EXECUTION</span><strong>{decision.executionPrice?.toFixed(4) ?? "NO ORDER"}</strong></div></div>
  </section>;
}

function V42DecisionContext({ decision }: { decision: MarketAiDecisionLog }) {
  if (decision.promptVersion !== "MARKET_AI_AGENT_V4_2") return null;
  const snapshot = decision.inputSnapshot;
  const state = valueAt(snapshot, ["position", "state"]);
  const exposure = valueAt(snapshot, ["position", "current_exposure_pct"]);
  return <section className="market-ai-pipeline">
    <div className="panel-heading"><strong>POSITION INTENT PIPELINE</strong><span>{decision.positionSizingVersion ?? "POSITION SIZING N/A"}</span></div>
    <div className="market-ai-runtime__metrics">
      <div><span>POSITION STATE</span><strong>{printable(state)}</strong></div>
      <div><span>CURRENT EXPOSURE</span><strong>{printable(exposure, "%")}</strong></div>
      <div><span>AI INTENT</span><strong>{decision.intent ?? "N/A"}</strong></div>
      <div><span>CONFIDENCE_PCT</span><strong>{decision.confidence == null ? "N/A" : `${(decision.confidence * 100).toFixed(0)}%`}</strong></div>
      <div><span>REASON CODE</span><strong>{decision.reasonCode ?? "N/A"}</strong></div>
      <div><span>GENERATED TARGET</span><strong>{decision.generatedTargetExposurePct?.toFixed(2) ?? "N/A"}%</strong></div>
      <div><span>RISK RESULT</span><strong>{decision.riskResult} · {decision.approvedPositionPct?.toFixed(2) ?? "N/A"}%</strong></div>
      <div><span>EXECUTION</span><strong>{decision.executionPrice?.toFixed(4) ?? "NO ORDER"}</strong></div>
    </div>
    <p>{decision.reason}</p>
  </section>;
}

function ConfidenceAnalysis({ decisions }: { decisions: MarketAiDecisionLog[] }) {
  const groups = useMemo(() => ([
    { name: "LOW", decisions: decisions.filter((item) => item.callStatus === "VALID" && item.confidence != null && item.confidence < 0.3) },
    { name: "MEDIUM", decisions: decisions.filter((item) => item.callStatus === "VALID" && item.confidence != null && item.confidence >= 0.3 && item.confidence < 0.6) },
    { name: "HIGH", decisions: decisions.filter((item) => item.callStatus === "VALID" && item.confidence != null && item.confidence >= 0.6) },
  ]), [decisions]);
  return <section className="market-ai-confidence-analysis"><div className="panel-heading"><strong>CONFIDENCE ANALYSIS</strong><span>POST-DECISION PERFORMANCE · NÃO É ACCURACY</span></div><div className="market-ai-confidence-analysis__grid">{groups.map((group) => <article key={group.name}><strong>{group.name}</strong><span>{group.decisions.length} decisões</span><dl><div><dt>FORWARD 1</dt><dd>{percentage(average(group.decisions.flatMap((item) => item.forwardReturn1 == null ? [] : [item.forwardReturn1])))}</dd></div><div><dt>FORWARD 5</dt><dd>{percentage(average(group.decisions.flatMap((item) => item.forwardReturn5 == null ? [] : [item.forwardReturn5])))}</dd></div><div><dt>FORWARD 10</dt><dd>{percentage(average(group.decisions.flatMap((item) => item.forwardReturn10 == null ? [] : [item.forwardReturn10])))}</dd></div></dl></article>)}</div></section>;
}

export function AiRuntimePanel({ status, runtime, decisions }: { status: MarketAiStatus | null; runtime: MarketAiRuntimeMetric | null; decisions: MarketAiDecisionLog[] }) {
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [filter, setFilter] = useState<DecisionFilter>("ALL");
  const filtered = decisions.filter((decision) => filter === "ALL" || (filter === "NO_LLM_CALL" ? decision.callStatus === filter : decision.callStatus === "VALID" && decision.action === filter));
  const selected = filtered.find((decision) => decision.decisionId === selectedId) ?? filtered[0];
  if (!runtime && decisions.length === 0) return null;
  const decisionTotal = runtime ? runtime.buyCount + runtime.sellCount + runtime.llmHoldCount : 0;
  const rate = (count: number) => decisionTotal === 0 ? "0.0%" : `${(count / decisionTotal * 100).toFixed(1)}%`;
  const snapshot = selected?.inputSnapshot ?? null;
  const v2 = Boolean(valueAt(snapshot, ["market"]));
  return <section className="market-ai-runtime">
    {runtime && <ReliabilityPanel runtime={runtime} decisions={decisions} />}
    <div className="panel-heading"><strong>AI RUNTIME & DECISION DIAGNOSTICS</strong><span>{status?.provider.toUpperCase()} / {selected?.model ?? status?.model} · {runtime?.promptVersion ?? selected?.promptVersion ?? status?.promptVersion}</span></div>
    {runtime && <><div className="market-ai-runtime__metrics"><div><span>CALLS</span><strong>{runtime.callCount}</strong></div><div><span>VALID</span><strong>{runtime.successfulCallCount}</strong></div><div><span>INVALID</span><strong>{runtime.invalidResponseCount}</strong></div><div><span>TIMEOUT</span><strong>{runtime.timeoutCount}</strong></div><div><span>RETRIES</span><strong>{runtime.retryCount}</strong></div><div><span>FALLBACKS</span><strong>{runtime.fallbackCount}</strong></div><div><span>BUY</span><strong>{runtime.buyCount} · {rate(runtime.buyCount)}</strong></div><div><span>SELL</span><strong>{runtime.sellCount} · {rate(runtime.sellCount)}</strong></div><div><span>LLM HOLD</span><strong>{runtime.llmHoldCount} · {rate(runtime.llmHoldCount)}</strong></div><div><span>NO LLM CALL</span><strong>{runtime.noLlmCallCount}</strong></div><div><span>AVG CONFIDENCE</span><strong>{runtime.averageConfidence?.toFixed(3) ?? "N/A"}</strong></div><div><span>MEDIAN / MIN / MAX</span><strong>{runtime.medianConfidence?.toFixed(2) ?? "N/A"} / {runtime.minConfidence?.toFixed(2) ?? "N/A"} / {runtime.maxConfidence?.toFixed(2) ?? "N/A"}</strong></div><div><span>AVG LATENCY</span><strong>{runtime.averageLatencyMs.toFixed(0)} ms</strong></div><div><span>MAX LATENCY</span><strong>{runtime.maxLatencyMs} ms</strong></div></div><div className="market-ai-distributions"><article><strong>AI DECISION DISTRIBUTION</strong><span>BUY {runtime.buyCount} · SELL {runtime.sellCount} · LLM HOLD {runtime.llmHoldCount}</span></article><article><strong>CONFIDENCE DISTRIBUTION</strong>{["0.00–0.20", "0.20–0.40", "0.40–0.60", "0.60–0.80", "0.80–1.00"].map((label, index) => <span key={label}>{label}<b>{runtime.confidenceDistribution[index] ?? 0}</b></span>)}</article><article><strong>AVG CONFIDENCE BY ACTION</strong><span>BUY <b>{runtime.averageBuyConfidence?.toFixed(3) ?? "N/A"}</b></span><span>SELL <b>{runtime.averageSellConfidence?.toFixed(3) ?? "N/A"}</b></span><span>HOLD <b>{runtime.averageHoldConfidence?.toFixed(3) ?? "N/A"}</b></span></article></div></>}
    {decisions.length > 0 && <div className="market-ai-inspector"><div className="market-ai-inspector__toolbar"><label>FILTER<select value={filter} onChange={(event) => { setFilter(event.target.value as DecisionFilter); setSelectedId(null); }}><option>ALL</option><option>BUY</option><option>SELL</option><option>HOLD</option><option>NO_LLM_CALL</option></select></label><label>DECISION INSPECTOR<select value={selected?.decisionId ?? ""} onChange={(event) => setSelectedId(Number(event.target.value))}><option value="">{filtered.length === 0 ? "Nenhuma decisão neste filtro" : "Selecionar..."}</option>{filtered.map((decision) => <option key={decision.decisionId} value={decision.decisionId}>{decision.timestamp} · {decision.callStatus} · {decision.action}</option>)}</select></label></div>{selected && <article><header><strong>{selected.action}</strong><span>{selected.callStatus}{selected.fallbackUsed ? " / FALLBACK" : ""}</span></header><dl><div><dt>CONFIDENCE</dt><dd>{selected.confidence?.toFixed(2) ?? "N/A"}</dd></div><div><dt>DESIRED POSITION</dt><dd>{selected.desiredPositionPct?.toFixed(2) ?? "N/A"}%</dd></div><div><dt>MARKET REGIME</dt><dd>{printable(valueAt(snapshot, ["regime", "trend"]))} / {printable(valueAt(snapshot, ["regime", "volatility"]))}</dd></div><div><dt>RETURN 1 / 5 / 10</dt><dd>{printable(valueAt(snapshot, [v2 ? "returns" : "features", "return_1"]))} / {printable(valueAt(snapshot, [v2 ? "returns" : "features", "return_5"]))} / {printable(valueAt(snapshot, ["returns", "return_10"]))}</dd></div><div><dt>SMA RELATION</dt><dd>SHORT {printable(valueAt(snapshot, ["trend", "price_vs_sma_short_pct"]), "%")} · LONG {printable(valueAt(snapshot, ["trend", "price_vs_sma_long_pct"]), "%")}</dd></div><div><dt>MOMENTUM 5 / 10</dt><dd>{printable(valueAt(snapshot, ["momentum", "momentum_5"]))} / {printable(valueAt(snapshot, ["momentum", "momentum_10"]))}</dd></div><div><dt>VOLATILITY</dt><dd>{printable(valueAt(snapshot, [v2 ? "volatility" : "features", v2 ? "rolling_volatility" : "volatility"]))}</dd></div><div><dt>CURRENT POSITION</dt><dd>{printable(valueAt(snapshot, ["portfolio", v2 ? "position_pct" : "exposure_pct"]), "%")} · {printable(valueAt(snapshot, ["portfolio", "holding_period"]), " candles")}</dd></div><div><dt>RISK RESULT</dt><dd>{selected.riskResult} · {selected.approvedPositionPct?.toFixed(2) ?? "N/A"}%</dd></div><div><dt>EXECUTION</dt><dd>{selected.executionPrice?.toFixed(4) ?? "NO ORDER"}</dd></div><div><dt>MODEL / PROMPT</dt><dd>{selected.model} · {selected.promptVersion}</dd></div><div><dt>LATENCY</dt><dd>{selected.latencyMs} ms / {selected.attempts} attempt(s)</dd></div></dl><p>{selected.reason}</p><small>{selected.riskReason}</small>{snapshot && <details><summary>AI INPUT SNAPSHOT · DADOS DISPONÍVEIS EM T</summary><pre>{JSON.stringify(snapshot, null, 2)}</pre></details>}<section className="market-ai-post-decision"><strong>POST-DECISION ANALYSIS</strong><span>Informação calculada depois da decisão; nunca enviada ao agente.</span><dl><div><dt>FORWARD 1</dt><dd>{percentage(selected.forwardReturn1)}</dd></div><div><dt>FORWARD 5</dt><dd>{percentage(selected.forwardReturn5)}</dd></div><div><dt>FORWARD 10</dt><dd>{percentage(selected.forwardReturn10)}</dd></div></dl></section></article>}</div>}
    {selected?.promptVersion === "MARKET_AI_AGENT_V3" && <section className="market-signal-trace"><div className="panel-heading"><strong>SIGNAL TRACE</strong><span>FEATURES → SIGNAL → AI → RISK → EXECUTION</span></div><dl><div><dt>TREND / MOMENTUM / VOLATILITY</dt><dd>{printable(valueAt(snapshot,["signals","trendScore"]))} / {printable(valueAt(snapshot,["signals","momentumScore"]))} / {printable(valueAt(snapshot,["signals","volatilityScore"]))}</dd></div><div><dt>BULLISH / BEARISH EVIDENCE</dt><dd>{printable(valueAt(snapshot,["signals","bullishEvidence"]))} / {printable(valueAt(snapshot,["signals","bearishEvidence"]))}</dd></div><div><dt>BIAS / STRENGTH / CONFLICT</dt><dd>{printable(valueAt(snapshot,["signals","directionalBias"]))} / {printable(valueAt(snapshot,["signals","signalStrength"]))} / {printable(valueAt(snapshot,["signals","conflictLevel"]))}</dd></div><div><dt>AI DECISION</dt><dd>{selected.action} · {selected.confidence?.toFixed(2) ?? "N/A"} · {selected.reasonCode ?? "N/A"}</dd></div><div><dt>POSITION TARGET</dt><dd>{selected.desiredPositionPct?.toFixed(2) ?? "N/A"}% TOTAL EXPOSURE</dd></div><div><dt>VALIDATION / RISK / EXECUTION</dt><dd>{selected.validationCode ?? "VALID"} / {selected.riskResult} / {selected.executionPrice?.toFixed(4) ?? "NO ORDER"}</dd></div><div><dt>AI SIGNAL DISAGREEMENT</dt><dd>{selected.signalDisagreement ? "YES" : "NO"}</dd></div></dl></section>}
    {selected && <V42DecisionContext decision={selected} />}
    {selected && <DecisionPipeline decision={selected} />}
    <ConfidenceAnalysis decisions={decisions} />
  </section>;
}

export function SignalDiagnosticsPanel({ diagnostics }: { diagnostics: MarketSignalDiagnostics | null }) {
  if (!diagnostics) return null;
  const matrix = (title: string, rows: MarketSignalDiagnostics["signalActionMatrix"]) => <article className="market-signal-matrix"><strong>{title}</strong><table><thead><tr><th>SIGNAL</th><th>BUY</th><th>SELL</th><th>HOLD</th><th>FWD 5</th></tr></thead><tbody>{rows.map((row) => <tr key={row.label}><td>{row.label}</td><td>{row.buyCount}</td><td>{row.sellCount}</td><td>{row.holdCount}</td><td>{percentage(row.averageForward5)}</td></tr>)}</tbody></table></article>;
  return <section className="market-signal-diagnostics"><div className="panel-heading"><strong>SIGNAL DIAGNOSTICS</strong><span>{diagnostics.signalEngineVersion} · {diagnostics.signalConfigVersion}</span></div>
    {(diagnostics.actionCollapse || diagnostics.confidenceCollapse) && <div className="market-signal-warnings">{diagnostics.actionCollapse && <div className="market-alert"><strong>ACTION COLLAPSE WARNING</strong><span>Respostas válidas concentradas em {diagnostics.collapsedAction}.</span></div>}{diagnostics.confidenceCollapse && <div className="market-alert"><strong>CONFIDENCE COLLAPSE WARNING</strong><span>Confiança repetida: {diagnostics.collapsedConfidence?.toFixed(4)}</span></div>}</div>}
    <div className="market-ai-runtime__metrics"><div><span>BULLISH / BEARISH / NEUTRAL</span><strong>{diagnostics.bullishCount} / {diagnostics.bearishCount} / {diagnostics.neutralCount}</strong></div><div><span>STRONG / WEAK</span><strong>{diagnostics.strongCount} / {diagnostics.weakCount}</strong></div><div><span>CONFLICT LOW / MED / HIGH</span><strong>{diagnostics.lowConflictCount} / {diagnostics.mediumConflictCount} / {diagnostics.highConflictCount}</strong></div><div><span>AI DISAGREEMENT</span><strong>{diagnostics.disagreementCount} · {diagnostics.disagreementRatePct.toFixed(1)}%</strong></div></div>
    <div className="market-signal-response"><span>STRONG BULLISH → BUY <b>{optionalRate(diagnostics.strongBullishBuyRatePct)}</b></span><span>STRONG BULLISH → HOLD <b>{optionalRate(diagnostics.strongBullishHoldRatePct)}</b></span><span>STRONG BEARISH → SELL <b>{optionalRate(diagnostics.strongBearishSellRatePct)}</b></span><span>STRONG BEARISH → HOLD <b>{optionalRate(diagnostics.strongBearishHoldRatePct)}</b></span></div>
    <div className="market-signal-matrices">{matrix("SIGNAL-ACTION MATRIX",diagnostics.signalActionMatrix)}{matrix("CONFLICT-ACTION MATRIX",diagnostics.conflictActionMatrix)}</div>
    <article className="market-reason-codes"><strong>REASON CODE DISTRIBUTION</strong>{diagnostics.reasonCodes.map((row)=><span key={row.reasonCode}>{row.reasonCode}<b>{row.count}</b></span>)}</article>
  </section>;
}

const sameConditions = (left: MarketAiExperimentComparison, right: MarketAiExperimentComparison) => left.datasetId === right.datasetId && left.riskProfileId === right.riskProfileId && left.initialCapital === right.initialCapital && left.randomSeed === right.randomSeed && left.feePct === right.feePct && left.slippagePct === right.slippagePct && left.decisionInterval === right.decisionInterval;

export function AiComparisonPanel({ entries }: { entries: MarketAiExperimentComparison[] }) {
  const [leftId, setLeftId] = useState(""); const [rightId, setRightId] = useState("");
  const left = entries.find((entry) => entry.experimentId === leftId); const right = entries.find((entry) => entry.experimentId === rightId);
  const finalRate = (entry: MarketAiExperimentComparison) => {
    const total = entry.finalValidCount + entry.finalInvalidCount;
    return total === 0 ? "N/A" : `${(entry.finalValidCount / total * 100).toFixed(1)}%`;
  };
  const fallbackRate = (entry: MarketAiExperimentComparison) => {
    const opportunities = entry.finalValidCount + entry.finalInvalidCount;
    return opportunities === 0 ? "N/A" : `${(entry.systemFallbackCount / opportunities * 100).toFixed(1)}%`;
  };
  return <section className="market-ai-comparison"><div className="panel-heading"><strong>AI PROMPT A/B</strong><span>EXPERIMENTOS SEPARADOS · UMA VARIÁVEL POR VEZ</span></div><div className="market-ai-comparison__selectors"><label>EXPERIMENTO A<select value={leftId} onChange={(event) => setLeftId(event.target.value)}><option value="">Selecionar...</option>{entries.map((entry) => <option key={entry.experimentId} value={entry.experimentId}>{entry.promptVersion} · {entry.experimentName}</option>)}</select></label><label>EXPERIMENTO B<select value={rightId} onChange={(event) => setRightId(event.target.value)}><option value="">Selecionar...</option>{entries.map((entry) => <option key={entry.experimentId} value={entry.experimentId}>{entry.promptVersion} · {entry.experimentName}</option>)}</select></label></div>{left && right && <>{!sameConditions(left, right) && <div className="market-alert"><strong>COMPARAÇÃO NÃO CONTROLADA</strong><span>Dataset, risco, capital, seed, custos ou cadência são diferentes.</span></div>}<div className="market-metrics"><table><thead><tr><th>MÉTRICA</th><th>{left.promptVersion}</th><th>{right.promptVersion}</th></tr></thead><tbody>{[["RAW CALLS",left.callCount,right.callCount],["FIRST-PASS VALID",left.firstPassValidCount,right.firstPassValidCount],["RETRY RECOVERED",left.retryRecoveredCount,right.retryRecoveredCount],["FINAL VALID",left.finalValidCount,right.finalValidCount],["FINAL INVALID",left.finalInvalidCount,right.finalInvalidCount],["FINAL VALID RATE",finalRate(left),finalRate(right)],["SYSTEM FALLBACKS",left.systemFallbackCount,right.systemFallbackCount],["FALLBACK RATE",fallbackRate(left),fallbackRate(right)],["TIMEOUT",left.timeoutCount,right.timeoutCount],["ENTER",left.enterCount,right.enterCount],["HOLD INTENT",left.intentHoldCount,right.intentHoldCount],["REDUCE",left.reduceCount,right.reduceCount],["EXIT",left.exitCount,right.exitCount],["GENERATED TARGET AVG",left.averageGeneratedTargetExposurePct==null?"N/A":`${left.averageGeneratedTargetExposurePct.toFixed(2)}%`,right.averageGeneratedTargetExposurePct==null?"N/A":`${right.averageGeneratedTargetExposurePct.toFixed(2)}%`],["TRADES",left.tradeCount,right.tradeCount],["RETURN",percentage(left.totalReturnPct),percentage(right.totalReturnPct)],["MAX DRAWDOWN",`${left.maxDrawdownPct.toFixed(2)}%`,`${right.maxDrawdownPct.toFixed(2)}%`],["EXPOSURE",`${left.averageExposurePct.toFixed(2)}%`,`${right.averageExposurePct.toFixed(2)}%`],["AVG LATENCY",`${left.averageLatencyMs.toFixed(0)} ms`,`${right.averageLatencyMs.toFixed(0)} ms`],["P95 LATENCY",`${left.p95LatencyMs.toFixed(0)} ms`,`${right.p95LatencyMs.toFixed(0)} ms`],["MAX LATENCY",`${left.maxLatencyMs} ms`,`${right.maxLatencyMs} ms`]].map(([metric,a,b]) => <tr key={metric}><td><strong>{metric}</strong></td><td>{a}</td><td>{b}</td></tr>)}</tbody></table></div></>}</section>;
}

export function AiAbcComparisonPanel({ entries }: { entries: MarketAiExperimentComparison[] }) {
  const [ids,setIds]=useState(["","",""]);
  const selected=ids.map((id)=>entries.find((entry)=>entry.experimentId===id)).filter((entry):entry is MarketAiExperimentComparison=>Boolean(entry));
  const controlled=selected.length<2||selected.every((entry)=>sameConditions(selected[0],entry));
  const rows:(readonly [string,(entry:MarketAiExperimentComparison)=>string|number])[]=[
    ["CALLS",entry=>entry.callCount],["BUY",entry=>entry.buyCount],["SELL",entry=>entry.sellCount],["LLM HOLD",entry=>entry.llmHoldCount],["TRADES",entry=>entry.tradeCount],["AVG CONFIDENCE",entry=>entry.averageConfidence?.toFixed(3)??"N/A"],["RETURN",entry=>percentage(entry.totalReturnPct)],["DRAWDOWN",entry=>`${entry.maxDrawdownPct.toFixed(2)}%`],["EXPOSURE",entry=>`${entry.averageExposurePct.toFixed(2)}%`],
  ];
  return <section className="market-ai-comparison"><div className="panel-heading"><strong>AI VERSION A/B/C</strong><span>V1 · V2 · V3 EM EXPERIMENTOS SEPARADOS</span></div><div className="market-ai-comparison__selectors market-ai-comparison__selectors--abc">{ids.map((id,index)=><label key={index}>EXPERIMENTO {String.fromCharCode(65+index)}<select value={id} onChange={(event)=>setIds((current)=>current.map((value,at)=>at===index?event.target.value:value))}><option value="">Selecionar...</option>{entries.map((entry)=><option key={entry.experimentId} value={entry.experimentId}>{entry.promptVersion} · {entry.experimentName}</option>)}</select></label>)}</div>{!controlled&&<div className="market-alert"><strong>COMPARAÇÃO NÃO CONTROLADA</strong><span>Dataset, risco, capital, seed, custos ou cadência são diferentes.</span></div>}{selected.length>0&&<div className="market-metrics"><table><thead><tr><th>MÉTRICA</th>{selected.map((entry)=><th key={entry.experimentId}>{entry.promptVersion}</th>)}</tr></thead><tbody>{rows.map(([label,get])=><tr key={label}><td><strong>{label}</strong></td>{selected.map((entry)=><td key={entry.experimentId}>{get(entry)}</td>)}</tr>)}</tbody></table></div>}</section>;
}
