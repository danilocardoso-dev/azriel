import { useEffect, useMemo, useState } from "react";
import { marketLabRepository } from "../../repositories/marketLabRepository";
import type { HoldAggregateRow, HoldDiagnosticItem, HoldDiagnosticsComparison, HoldDiagnosticsReport, MarketExperimentSummary } from "../../types";
import { diagnosticAvailability, qualityLabel, qualityRate } from "./holdDiagnosticsView";

const pct = (value: number | null | undefined) => value == null ? "N/A" : `${value >= 0 ? "+" : ""}${value.toFixed(2)}%`;
const value = (input: number | null | undefined, digits = 2) => input == null ? "N/A" : input.toFixed(digits);

function AggregateTable({ title, rows }: { title: string; rows: HoldAggregateRow[] }) {
  return <section className="market-hold-table"><h4>{title}</h4><div className="market-hold-table__scroll"><table><thead><tr><th>GRUPO</th><th>N</th><th>CONF.</th><th>FWD5</th><th>MFE5</th><th>MAE5</th><th>GOOD</th><th>MISSED</th><th>LATE</th></tr></thead><tbody>{rows.length === 0 ? <tr><td colSpan={9}>SEM AMOSTRAS</td></tr> : rows.map((row) => <tr key={row.key}><td>{row.key}</td><td>{row.count}</td><td>{value(row.averageConfidence, 3)}</td><td>{pct(row.averageForward5)}</td><td>{pct(row.averageMfe5)}</td><td>{pct(row.averageMae5)}</td><td>{row.goodHoldRatePct.toFixed(1)}%</td><td>{row.missedOpportunityRatePct.toFixed(1)}%</td><td>{row.lateReductionRatePct.toFixed(1)}%</td></tr>)}</tbody></table></div></section>;
}

function TriggerTable({ report }: { report: HoldDiagnosticsReport }) {
  return <section className="market-hold-table"><h4>TRIGGER ANALYSIS</h4><div className="market-hold-table__scroll"><table><thead><tr><th>TRIGGER</th><th>CALLS</th><th>HOLD</th><th>ENTER</th><th>HOLD %</th><th>FWD5 HOLD</th><th>MISSED %</th></tr></thead><tbody>{report.triggerAnalysis.map((row) => <tr key={row.trigger}><td>{row.trigger}</td><td>{row.callCount}</td><td>{row.holdCount}</td><td>{row.enterCount}</td><td>{row.holdRatePct.toFixed(1)}%</td><td>{pct(row.averageForward5AfterHold)}</td><td>{row.missedOpportunityRatePct.toFixed(1)}%</td></tr>)}</tbody></table></div></section>;
}

function DecisionInspector({ item }: { item: HoldDiagnosticItem | null }) {
  if (!item) return <div className="market-hold-empty">Selecione um HOLD para inspecionar.</div>;
  return <section className="market-hold-inspector"><div className="panel-heading"><strong>HOLD INSPECTOR</strong><span>PÓS-DECISÃO · NÃO ENVIADO AO AGENTE</span></div><header><div><small>{item.timestamp} · {item.sessionPhase}</small><h4>{qualityLabel(item.quality, item.positionState)}</h4><p>{item.qualityContext}</p></div><b className={`hold-quality hold-quality--${item.quality.toLowerCase()}`}>{item.positionState}</b></header><dl><div><dt>REASON</dt><dd>{item.reasonCode}</dd></div><div><dt>CONFIDENCE</dt><dd>{value(item.confidence, 3)} / {item.confidenceBucket}</dd></div><div><dt>EXPOSURE</dt><dd>{item.currentExposurePct.toFixed(2)}%</dd></div><div><dt>TRIGGER</dt><dd>{item.triggerReason}</dd></div><div><dt>FWD 1 / 5 / 10</dt><dd>{pct(item.outcome.forward1)} / {pct(item.outcome.forward5)} / {pct(item.outcome.forward10)}</dd></div><div><dt>MFE 1 / 5 / 10</dt><dd>{pct(item.outcome.mfe1)} / {pct(item.outcome.mfe5)} / {pct(item.outcome.mfe10)}</dd></div><div><dt>MAE 1 / 5 / 10</dt><dd>{pct(item.outcome.mae1)} / {pct(item.outcome.mae5)} / {pct(item.outcome.mae10)}</dd></div><div><dt>RSI / REL VOL.</dt><dd>{value(item.rsi14)} / {value(item.relativeVolume)}</dd></div><div><dt>EMA / VWAP ATR</dt><dd>{value(item.emaSpreadAtr)} / {value(item.distanceFromVwapAtr)}</dd></div><div><dt>STATE</dt><dd>{item.trend} / {item.momentum} / {item.volatility}</dd></div></dl><p className="market-hold-inspector__reason">{item.reason}</p>{item.conflictSignature && <code>{item.conflictSignature}</code>}</section>;
}

export function MarketHoldDiagnostics({ experiment, history, onError }: { experiment: MarketExperimentSummary; history: MarketExperimentSummary[]; onError: (message: string) => void }) {
  const [report, setReport] = useState<HoldDiagnosticsReport | null>(null);
  const [comparison, setComparison] = useState<HoldDiagnosticsComparison | null>(null);
  const [selected, setSelected] = useState<HoldDiagnosticItem | null>(null);
  const [busy, setBusy] = useState(false);
  const eligible = useMemo(() => history.filter((item) => diagnosticAvailability(item.agentIds, item.timeframe, item.status)), [history]);
  const [developmentId, setDevelopmentId] = useState(experiment.id);
  const [oosId, setOosId] = useState("");
  const available = diagnosticAvailability(experiment.agentIds, experiment.timeframe, experiment.status);

  useEffect(() => {
    if (!available) return;
    void marketLabRepository.getHoldDiagnostics(experiment.id).then((next) => { setReport(next); setSelected(next.items[0] ?? null); }).catch(() => undefined);
  }, [available, experiment.id]);

  if (!available) return null;
  const generate = async () => { setBusy(true); try { const next = await marketLabRepository.generateHoldDiagnostics(experiment.id); setReport(next); setSelected(next.items[0] ?? null); } catch (cause) { onError(String(cause)); } finally { setBusy(false); } };
  const compare = async () => { if (!developmentId || !oosId) { onError("Selecione experimentos distintos para DEV e OOS."); return; } setBusy(true); try { setComparison(await marketLabRepository.compareHoldDiagnostics(developmentId, oosId)); } catch (cause) { onError(String(cause)); } finally { setBusy(false); } };
  const q = report?.quality;
  return <section className="market-hold-diagnostics">
    <div className="panel-heading"><strong>INTRADAY HOLD DIAGNOSTICS</strong><span>HOLD_DIAGNOSTICS_V1 · ANÁLISE LOCAL PÓS-DECISÃO</span></div>
    <div className="market-hold-actions"><div><p>Explica por que o V1 manteve HOLD e mede o que ocorreu depois. O prompt, o contexto, o Trigger V2, o risco e a execução permanecem congelados.</p></div><button className="button button--primary" disabled={busy} onClick={() => void generate()}>{busy ? "PROCESSANDO..." : report ? "REGERAR DO HISTÓRICO" : "GERAR DIAGNÓSTICO HOLD"}</button></div>
    {!report ? <div className="market-hold-empty">Nenhum diagnóstico persistido. A geração reutiliza decisões e candles existentes e não chama o Ollama.</div> : <>
      {report.holdConcentrationWarning && <div className="market-hold-warning"><strong>ACTION CONCENTRATION</strong><span>HOLD representa {report.holdRatePct.toFixed(1)}% das decisões válidas. Isso é um alerta investigativo, não uma conclusão de falha.</span></div>}
      <div className="market-hold-summary"><div><span>AI CALLS / HOLD</span><b>{report.totalAiCalls} / {report.totalHolds}</b><small>{report.holdRatePct.toFixed(1)}% HOLD</small></div><div><span>FLAT / LONG</span><b>{report.flatCount} / {report.longCount}</b><small>SEM POSIÇÃO / POSICIONADO</small></div><div><span>GOOD HOLD</span><b>{q?.goodHoldCount ?? 0}</b><small>{qualityRate(report, q?.goodHoldCount ?? 0).toFixed(1)}%</small></div><div><span>MISSED</span><b>{q?.missedOpportunityCount ?? 0}</b><small>{qualityRate(report, q?.missedOpportunityCount ?? 0).toFixed(1)}%</small></div><div><span>LATE REDUCTION</span><b>{q?.lateReductionCount ?? 0}</b><small>{qualityRate(report, q?.lateReductionCount ?? 0).toFixed(1)}%</small></div><div><span>INCONCLUSIVE</span><b>{q?.inconclusiveCount ?? 0}</b><small>{qualityRate(report, q?.inconclusiveCount ?? 0).toFixed(1)}%</small></div></div>
      <AggregateTable title="HOLD QUALITY MATRIX" rows={report.qualityMatrix}/>
      <AggregateTable title="REASON CODE PERFORMANCE" rows={report.reasons}/>
      <div className="market-hold-grid"><AggregateTable title="CONFIDENCE BUCKETS" rows={report.confidence}/><AggregateTable title="SESSION PHASE" rows={report.sessionPhases}/></div>
      <TriggerTable report={report}/><AggregateTable title="FEATURE BUCKETS" rows={report.featureBuckets}/>
      {report.conflicts.length > 0 && <AggregateTable title="SIGNAL CONFLICT ANALYSIS" rows={report.conflicts}/>}
      <section className="market-hold-candidates"><h4>CALIBRATION CANDIDATES · NÃO APLICADOS</h4>{report.calibrationCandidates.length === 0 ? <p>Nenhum padrão atingiu os critérios mínimos congelados.</p> : report.calibrationCandidates.map((candidate) => <article key={`${candidate.reasonCode}-${candidate.issue}`}><b>{candidate.reasonCode}</b><span>{candidate.issue} · N {candidate.count} · {candidate.issueRatePct.toFixed(1)}% · FWD5 {pct(candidate.averageForward5)}</span></article>)}</section>
      <section className="market-hold-decisions"><h4>HOLD DECISIONS</h4><div>{report.items.slice().reverse().map((item) => <button key={item.decisionId} className={selected?.decisionId === item.decisionId ? "active" : ""} onClick={() => setSelected(item)}><span>{item.timestamp}</span><b>{item.reasonCode}</b><small>{item.positionState} · {qualityLabel(item.quality, item.positionState)} · FWD5 {pct(item.outcome.forward5)}</small></button>)}</div></section>
      <DecisionInspector item={selected}/>
      <section className="market-hold-compare"><div className="panel-heading"><strong>DEV × OUT-OF-SAMPLE</strong><span>MESMO ATIVO / 15M · DATASETS NÃO SOBREPOSTOS</span></div><div className="market-hold-compare__controls"><label>DEVELOPMENT<select value={developmentId} onChange={(event) => setDevelopmentId(event.target.value)}>{eligible.map((item) => <option value={item.id} key={item.id}>{item.name} · {item.datasetName}</option>)}</select></label><label>OUT-OF-SAMPLE<select value={oosId} onChange={(event) => setOosId(event.target.value)}><option value="">Selecione...</option>{eligible.filter((item) => item.id !== developmentId).map((item) => <option value={item.id} key={item.id}>{item.name} · {item.datasetName}</option>)}</select></label><button className="button" disabled={busy || !oosId} onClick={() => void compare()}>COMPARAR EVIDÊNCIA</button></div>{comparison && <><div className="market-hold-comparison-table"><table><thead><tr><th>MÉTRICA</th><th>DEV</th><th>OOS</th></tr></thead><tbody>{comparison.metrics.map((metric) => <tr key={metric.label}><td>{metric.label}</td><td>{metric.developmentValue.toFixed(2)}%</td><td>{metric.outOfSampleValue.toFixed(2)}%</td></tr>)}</tbody></table></div><div className="market-hold-comparison-tops"><span>TOP REASON · DEV <b>{comparison.developmentTopReason ?? "N/A"}</b></span><span>TOP REASON · OOS <b>{comparison.outOfSampleTopReason ?? "N/A"}</b></span><span>TOP CANDIDATE · DEV <b>{comparison.developmentTopCandidate ?? "N/A"}</b></span><span>TOP CANDIDATE · OOS <b>{comparison.outOfSampleTopCandidate ?? "N/A"}</b></span></div><div className="market-hold-evidence">{comparison.candidateEvidence.length === 0 ? <p>Evidência insuficiente: nenhum candidato de calibração comum atingiu o piso mínimo.</p> : comparison.candidateEvidence.map((item) => <article key={`${item.reasonCode}-${item.issue}`}><b>{item.evidence}</b><span>{item.reasonCode} / {item.issue}</span><small>DEV {item.development?.issueRatePct.toFixed(1) ?? "N/A"}% · OOS {item.outOfSample?.issueRatePct.toFixed(1) ?? "N/A"}%</small></article>)}</div></> }</section>
    </>}
  </section>;
}
