import { useEffect, useMemo, useState } from "react";
import { marketLabRepository } from "../../repositories/marketLabRepository";
import type { LifecycleIntelligenceComparison, LifecycleIntelligenceReport, LifecycleOutcomeAggregate, LifecycleSummary, MarketExperimentSummary } from "../../types";
import { diagnosticAvailability } from "./holdDiagnosticsView";
import { lifecycleTimeline } from "./lifecycleIntelligenceView";

const number = (value: number | null | undefined, digits = 2) => value == null ? "N/A" : value.toFixed(digits);
const pct = (value: number | null | undefined) => value == null ? "N/A" : `${value >= 0 ? "+" : ""}${value.toFixed(2)}%`;
const timestamp = (value: string | null | undefined) => value ? value.replace("T", " ").replace("Z", "") : "OPEN";

function OutcomeTable({ title, rows }: { title: string; rows: LifecycleOutcomeAggregate[] }) {
  return <section className="market-lifecycle-outcomes"><div className="panel-heading"><strong>{title}</strong><span>POST-DECISION ANALYSIS</span></div><div><table><thead><tr><th>STATE</th><th>N</th><th>AVG FWD1</th><th>AVG FWD5</th><th>AVG MFE5</th><th>AVG MAE5</th></tr></thead><tbody>{rows.length === 0 ? <tr><td colSpan={6}>SEM AMOSTRAS</td></tr> : rows.map((row) => <tr key={row.label}><td>{row.label}</td><td>{row.count}</td><td>{pct(row.averageForward1)}</td><td>{pct(row.averageForward5)}</td><td>{pct(row.averageMfe5)}</td><td>{pct(row.averageMae5)}</td></tr>)}</tbody></table></div></section>;
}

function LifecycleInspector({ report, lifecycle }: { report: LifecycleIntelligenceReport; lifecycle: LifecycleSummary | null }) {
  if (!lifecycle) return <div className="market-lifecycle-empty">Selecione um lifecycle para inspecionar.</div>;
  const events = report.events.filter((item) => item.lifecycleId === lifecycle.lifecycleId);
  const traces = report.healthTrace.filter((item) => item.lifecycleId === lifecycle.lifecycleId);
  const latest = traces.at(-1);
  const timeline = lifecycleTimeline(events, traces);
  return <section className="market-lifecycle-detail">
    <div className="panel-heading"><strong>LIFECYCLE INSPECTOR</strong><span>{lifecycle.lifecycleId}</span></div>
    <div className="market-lifecycle-separation"><span>AVAILABLE AT T · HEALTH E DETERIORATION USAM SOMENTE DADOS ATÉ T</span><span>POST-DECISION · RESULTADOS FUTUROS PERMANECEM ISOLADOS</span></div>
    <div className="market-lifecycle-detail__grid">
      <article><h4>ENTRY CONTEXT</h4><dl><div><dt>ENTRY</dt><dd>{timestamp(lifecycle.entryExecutionTimestamp)}</dd></div><div><dt>PRICE / EXPOSURE</dt><dd>{number(lifecycle.entryPrice)} · {number(lifecycle.initialExposurePct)}%</dd></div><div><dt>ATR / VWAP</dt><dd>{number(lifecycle.entryAtr)} / {number(lifecycle.entryVwap)}</dd></div><div><dt>EMA9 / EMA21</dt><dd>{number(lifecycle.entryEma9)} / {number(lifecycle.entryEma21)}</dd></div><div><dt>RSI / REL VOL</dt><dd>{number(lifecycle.entryRsi)} / {number(lifecycle.entryRelativeVolume)}</dd></div><div><dt>PHASE / REASON</dt><dd>{lifecycle.entrySessionPhase ?? "N/A"} / {lifecycle.entryReason ?? "N/A"}</dd></div></dl></article>
      <article><h4>POSITION HEALTH</h4><dl><div><dt>CURRENT</dt><dd>{latest?.positionHealth ?? "N/A"}</dd></div><div><dt>DETERIORATION</dt><dd>{latest?.deteriorationLevel ?? "N/A"} · {number(latest?.deteriorationScore, 3)}</dd></div><div><dt>AGE</dt><dd>{latest?.positionAgeCandles ?? lifecycle.durationCandles} candles · {latest?.positionAgeMarketMinutes ?? lifecycle.durationMarketMinutes} min</dd></div><div><dt>MFE / MAE</dt><dd>{pct(lifecycle.mfePct)} / {pct(lifecycle.maePct)}</dd></div><div><dt>GIVEBACK</dt><dd>{pct(lifecycle.givebackPct)}</dd></div><div><dt>RESPONSE DELAY</dt><dd>{lifecycle.responseDelayMarketMinutes ?? "N/A"} min{lifecycle.responseCensored ? " · CENSORED" : ""}</dd></div></dl></article>
    </div>
    <div className="market-lifecycle-timeline"><h4>TIMELINE · EVENTS / HEALTH</h4>{timeline.length === 0 ? <p>Sem eventos.</p> : timeline.map((item) => <article key={item.key} className={item.kind.toLowerCase()}><time>{timestamp(item.timestamp)}</time><b>{item.label}</b><span>{item.detail}</span></article>)}</div>
  </section>;
}

export function MarketPositionLifecycleIntelligence({ experiment, history, onError }: { experiment: MarketExperimentSummary; history: MarketExperimentSummary[]; onError: (message: string) => void }) {
  const [report, setReport] = useState<LifecycleIntelligenceReport | null>(null);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [comparison, setComparison] = useState<LifecycleIntelligenceComparison | null>(null);
  const [developmentId, setDevelopmentId] = useState(experiment.id);
  const [oosId, setOosId] = useState("");
  const [busy, setBusy] = useState(false);
  const available = diagnosticAvailability(experiment.agentIds, experiment.timeframe, experiment.status);
  const eligible = useMemo(() => history.filter((item) => diagnosticAvailability(item.agentIds, item.timeframe, item.status)), [history]);
  const selected = report?.lifecycles.find((item) => item.lifecycleId === selectedId) ?? report?.lifecycles[0] ?? null;

  useEffect(() => {
    if (!available) return;
    void marketLabRepository.getLifecycleIntelligence(experiment.id).then((next) => { setReport(next); setSelectedId(next.lifecycles[0]?.lifecycleId ?? null); }).catch(() => undefined);
  }, [available, experiment.id]);

  if (!available) return null;
  const generate = async () => { setBusy(true); try { const next = await marketLabRepository.generateLifecycleIntelligence(experiment.id); setReport(next); setSelectedId(next.lifecycles[0]?.lifecycleId ?? null); } catch (cause) { onError(String(cause)); } finally { setBusy(false); } };
  const compare = async () => { if (!oosId || developmentId === oosId) { onError("Selecione experimentos DEV e OOS distintos."); return; } setBusy(true); try { setComparison(await marketLabRepository.compareLifecycleIntelligence(developmentId, oosId)); } catch (cause) { onError(String(cause)); } finally { setBusy(false); } };
  const summary = report?.summary;
  const late = report?.lateReduction;
  return <section className="market-lifecycle-intelligence">
    <div className="panel-heading"><strong>POSITION LIFECYCLE OBSERVATORY</strong><span>POSITION_LIFECYCLE_CONFIG_V1 · OBSERVATIONAL / LOCAL</span></div>
    <div className="market-lifecycle-actions"><p>Reconstrói posições, saúde e deterioração usando decisões, execuções, candles e traces persistidos. Não chama o Ollama, não altera o agente e não executa ordens.</p><button className="button button--primary" disabled={busy} onClick={() => void generate()}>{busy ? "PROCESSANDO..." : report ? "REGERAR DO HISTÓRICO" : "GERAR LIFECYCLE INTELLIGENCE"}</button></div>
    {!report ? <div className="market-lifecycle-empty">Nenhuma análise persistida para este experimento.</div> : <>
      <div className="market-lifecycle-summary">{[
        ["TOTAL LIFECYCLES", summary?.totalLifecycles ?? 0], ["OPEN / CLOSED", `${summary?.openLifecycles ?? 0} / ${summary?.closedLifecycles ?? 0}`], ["AVG DURATION", `${number(summary?.averageDurationMinutes)} min`], ["AVG PNL", pct(summary?.averagePnlPct)], ["AVG MFE / MAE", `${pct(summary?.averageMfePct)} / ${pct(summary?.averageMaePct)}`], ["AVG GIVEBACK", pct(summary?.averageGivebackPct)], ["LATE REDUCTION", summary?.lateReductionEvents ?? 0], ["AVG RESPONSE", `${number(summary?.averageResponseDelayMinutes)} min`],
      ].map(([label, content]) => <div key={label}><span>{label}</span><b>{content}</b></div>)}</div>
      <div className="market-lifecycle-table"><table><thead><tr><th>ID</th><th>ENTRY</th><th>EXIT</th><th>DURATION</th><th>ENTRY / EXIT PRICE</th><th>PNL</th><th>MFE</th><th>MAE</th><th>GIVEBACK</th><th>MAX EXP.</th><th>STATUS</th></tr></thead><tbody>{report.lifecycles.map((item) => <tr key={item.lifecycleId} className={selected?.lifecycleId === item.lifecycleId ? "selected" : ""} onClick={() => setSelectedId(item.lifecycleId)}><td>{item.lifecycleId}</td><td>{timestamp(item.entryExecutionTimestamp)}</td><td>{timestamp(item.exitExecutionTimestamp)}</td><td>{item.durationMarketMinutes} min</td><td>{number(item.entryPrice)} / {number(item.exitPrice)}</td><td>{pct(item.realizedPnlPct)}</td><td>{pct(item.mfePct)}</td><td>{pct(item.maePct)}</td><td>{pct(item.givebackPct)}</td><td>{number(item.maxExposurePct)}%</td><td>{item.status}</td></tr>)}</tbody></table></div>
      <LifecycleInspector report={report} lifecycle={selected}/>
      <section className="market-late-reduction"><div className="panel-heading"><strong>LATE REDUCTION ANALYSIS</strong><span>POST-DECISION · SEM ORDEM CONTRAFACTUAL</span></div><div>{[["HOLD WHILE LONG", late?.totalHoldWhileLong ?? 0], ["POTENTIAL LATE", late?.potentialLateReductions ?? 0], ["RATE", `${number(late?.ratePct)}%`], ["AVG FWD5", pct(late?.averageForward5)], ["AVG MAE5", pct(late?.averageMae5)], ["AVG RESPONSE", `${number(late?.averageResponseDelayMinutes)} min`], ["TOP HEALTH", late?.topPositionHealth ?? "N/A"], ["TOP REASON", late?.topReasonCode ?? "N/A"]].map(([label, content]) => <article key={label}><span>{label}</span><b>{content}</b></article>)}</div></section>
      <div className="market-lifecycle-outcome-grid"><OutcomeTable title="POSITION HEALTH × OUTCOME" rows={report.healthOutcomes}/><OutcomeTable title="DETERIORATION × OUTCOME" rows={report.deteriorationOutcomes}/></div>
      <section className="market-lifecycle-outcomes"><div className="panel-heading"><strong>EXIT QUALITY BY REASON</strong><span>CLOSED LIFECYCLES</span></div><div><table><thead><tr><th>REASON CODE</th><th>N</th><th>AVG PNL</th><th>AVG MFE</th><th>AVG MAE</th><th>AVG GIVEBACK</th><th>AVG DURATION</th></tr></thead><tbody>{report.exitReasons.length === 0 ? <tr><td colSpan={7}>SEM LIFECYCLES ENCERRADOS</td></tr> : report.exitReasons.map((item) => <tr key={item.reasonCode}><td>{item.reasonCode}</td><td>{item.count}</td><td>{pct(item.averagePnlPct)}</td><td>{pct(item.averageMfePct)}</td><td>{pct(item.averageMaePct)}</td><td>{pct(item.averageGivebackPct)}</td><td>{number(item.averageDurationMinutes)} min</td></tr>)}</tbody></table></div></section>
      <section className="market-lifecycle-compare"><div className="panel-heading"><strong>DEV × OUT-OF-SAMPLE · LIFECYCLE</strong><span>MESMO ATIVO / TIMEFRAME · PERÍODOS NÃO SOBREPOSTOS</span></div><div className="market-lifecycle-compare__controls"><label>DEVELOPMENT<select value={developmentId} onChange={(event) => setDevelopmentId(event.target.value)}>{eligible.map((item) => <option key={item.id} value={item.id}>{item.name} · {item.datasetName}</option>)}</select></label><label>OUT-OF-SAMPLE<select value={oosId} onChange={(event) => setOosId(event.target.value)}><option value="">Selecione...</option>{eligible.filter((item) => item.id !== developmentId).map((item) => <option key={item.id} value={item.id}>{item.name} · {item.datasetName}</option>)}</select></label><button className="button" disabled={busy || !oosId} onClick={() => void compare()}>COMPARAR LIFECYCLES</button></div>{comparison && <div className="market-lifecycle-comparison"><table><thead><tr><th>MÉTRICA</th><th>DEV</th><th>OOS</th></tr></thead><tbody>{comparison.metrics.map((item) => <tr key={item.label}><td>{item.label}</td><td>{number(item.developmentValue)}</td><td>{number(item.outOfSampleValue)}</td></tr>)}</tbody></table><table><thead><tr><th>DETERIORATION</th><th>DEV</th><th>OOS</th></tr></thead><tbody>{comparison.deteriorationDistribution.map((item) => <tr key={item.level}><td>{item.level}</td><td>{item.developmentCount}</td><td>{item.outOfSampleCount}</td></tr>)}</tbody></table></div>}</section>
    </>}
  </section>;
}
