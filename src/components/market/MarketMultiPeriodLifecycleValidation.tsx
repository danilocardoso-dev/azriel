import { save } from "@tauri-apps/plugin-dialog";
import { useEffect, useMemo, useState } from "react";
import { marketLabRepository } from "../../repositories/marketLabRepository";
import type { DeteriorationLevel, LifecycleValidationBatch, LifecycleValidationSourceRole, MarketExperimentSummary, MultiPeriodLifecycleValidationReport } from "../../types";
import { diagnosticAvailability } from "./holdDiagnosticsView";
import { chartBars, filterValidationLifecycles, responseLabel, type LifecycleValidationFilters } from "./multiPeriodLifecycleView";

const number = (value: number | null | undefined, digits = 2) => value == null ? "N/A" : value.toFixed(digits);
const pct = (value: number | null | undefined) => value == null ? "N/A" : `${value >= 0 ? "+" : ""}${value.toFixed(2)}%`;
const date = (value: string | null | undefined) => value ? value.replace("T", " ").replace("Z", "") : "OPEN";
const roles: LifecycleValidationSourceRole[] = ["DEVELOPMENT", "OOS", "HOLDOUT", "ADDITIONAL_VALIDATION"];

function MiniDistribution({ title, values, suffix = "" }: { title: string; values: number[]; suffix?: string }) {
  const bars = chartBars(values);
  const maximum = Math.max(...bars, 1);
  return <article className="market-multi-chart"><header><strong>{title}</strong><span>N {values.length}</span></header><div>{bars.map((value, index) => <i key={`${title}-${index}`} style={{ height: `${Math.max(3, value / maximum * 100)}%` }} title={`${value} amostras`} />)}</div><footer><span>{values.length ? `${number(Math.min(...values))}${suffix}` : "N/A"}</span><span>{values.length ? `${number(Math.max(...values))}${suffix}` : "N/A"}</span></footer></article>;
}

function PeriodRateChart({ report }: { report: MultiPeriodLifecycleValidationReport }) {
  const maximum = Math.max(...report.periods.map((period) => period.lifecycleLateReductionRatePct), 1);
  return <article className="market-multi-chart market-multi-chart--period"><header><strong>LIFECYCLE LATE REDUCTION BY PERIOD</strong><span>%</span></header><div>{report.periods.map((period) => <i key={period.periodId} style={{ height: `${Math.max(3, period.lifecycleLateReductionRatePct / maximum * 100)}%` }} title={`${period.datasetName}: ${number(period.lifecycleLateReductionRatePct)}%`} />)}</div><footer><span>{report.periods[0]?.datasetName ?? "N/A"}</span><span>{report.periods.at(-1)?.datasetName ?? "N/A"}</span></footer></article>;
}

const initialFilters: LifecycleValidationFilters = { periodId: "", status: "ALL", deterioration: "ALL", lateReduction: "ALL", overnight: "ALL" };

export function MarketMultiPeriodLifecycleValidation({ experiment, history, onError }: { experiment: MarketExperimentSummary; history: MarketExperimentSummary[]; onError: (message: string) => void }) {
  const [selection, setSelection] = useState<Record<string, LifecycleValidationSourceRole>>({ [experiment.id]: "DEVELOPMENT" });
  const [name, setName] = useState(`Multi-Period ${experiment.asset} ${experiment.timeframe}`);
  const [report, setReport] = useState<MultiPeriodLifecycleValidationReport | null>(null);
  const [batches, setBatches] = useState<LifecycleValidationBatch[]>([]);
  const [selectedBatchId, setSelectedBatchId] = useState("");
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState("");
  const [filters, setFilters] = useState<LifecycleValidationFilters>(initialFilters);
  const eligible = useMemo(() => history.filter((item) => diagnosticAvailability(item.agentIds, item.timeframe, item.status)), [history]);
  const selectedCount = Object.keys(selection).length;
  const filtered = useMemo(() => report ? filterValidationLifecycles(report.lifecycles, filters) : [], [report, filters]);

  useEffect(() => {
    let active = true;
    void marketLabRepository.listMultiPeriodLifecycleValidations().then((items) => { if (active) setBatches(items); }).catch(() => undefined);
    return () => { active = false; };
  }, []);

  const toggle = (id: string) => setSelection((current) => {
    if (current[id]) {
      const next = { ...current };
      delete next[id];
      return next;
    }
    return { ...current, [id]: Object.keys(current).length === 0 ? "DEVELOPMENT" : "ADDITIONAL_VALIDATION" };
  });

  const run = async () => {
    if (selectedCount < 2) { onError("Selecione ao menos dois experimentos independentes."); return; }
    setBusy(true); setMessage("");
    try {
      const result = await marketLabRepository.runMultiPeriodLifecycleValidation({ name, periods: Object.entries(selection).map(([experimentId, sourceRole]) => ({ experimentId, sourceRole })) });
      setReport(result);
      setSelectedBatchId(result.batch.batchId);
      setFilters(initialFilters);
      setBatches(await marketLabRepository.listMultiPeriodLifecycleValidations());
      setMessage("BATCH CONCLUÍDO · ARTEFATOS LOCAIS REUTILIZADOS");
    } catch (error) { onError(String(error)); } finally { setBusy(false); }
  };

  const openBatch = async (batchId: string) => {
    setSelectedBatchId(batchId);
    if (!batchId) return;
    setBusy(true); setMessage("");
    try { setReport(await marketLabRepository.getMultiPeriodLifecycleValidation(batchId)); } catch (error) { onError(String(error)); } finally { setBusy(false); }
  };

  const resumeBatch = async () => {
    if (!selectedBatchId) return;
    setBusy(true); setMessage("");
    try {
      const result = await marketLabRepository.resumeMultiPeriodLifecycleValidation(selectedBatchId);
      setReport(result);
      setBatches(await marketLabRepository.listMultiPeriodLifecycleValidations());
      setMessage(`BATCH ${result.batch.status} · CHECKPOINTS PRESERVADOS`);
    } catch (error) { onError(String(error)); } finally { setBusy(false); }
  };

  const exportJson = async () => {
    if (!report) return;
    const safeName = report.batch.name.replace(/[^a-z0-9_-]+/gi, "-").replace(/^-|-$/g, "");
    const path = await save({ title: "Exportar Multi-Period Lifecycle Validation", defaultPath: `${safeName || report.batch.batchId}.json`, filters: [{ name: "JSON", extensions: ["json"] }] });
    if (!path) return;
    try { await marketLabRepository.exportMultiPeriodLifecycleValidation(report.batch.batchId, path); setMessage("AUDITORIA JSON EXPORTADA"); } catch (error) { onError(String(error)); }
  };

  return <section className="market-multi-period">
    <div className="panel-heading"><strong>MULTI-PERIOD LIFECYCLE VALIDATION</strong><span>V0.5.4 · LIFECYCLE É A UNIDADE PRIMÁRIA</span></div>
    <div className="market-multi-period__notice"><span>AVAILABLE AT T permanece isolado.</span><span>OUTCOMES e classificações futuras são POST-DECISION ONLY.</span><span>Nenhuma chamada ao Ollama, otimização ou regra operacional.</span></div>
    <div className="market-multi-period__builder">
      <label>NOME DO BATCH<input value={name} maxLength={120} onChange={(event) => setName(event.target.value)} /></label>
      <label>BATCHS PERSISTIDOS<select value={selectedBatchId} onChange={(event) => void openBatch(event.target.value)}><option value="">Selecione...</option>{batches.map((batch) => <option key={batch.batchId} value={batch.batchId}>{batch.name} · {batch.lifecycleCount} lifecycles · {batch.status}</option>)}</select></label>
      <button className="button button--primary" disabled={busy || selectedCount < 2} onClick={() => void run()}>{busy ? "VALIDANDO..." : `▶ EXECUTAR BATCH (${selectedCount})`}</button>
      {selectedBatchId && batches.find((batch) => batch.batchId === selectedBatchId)?.status !== "COMPLETED" && <button className="button" disabled={busy} onClick={() => void resumeBatch()}>RETOMAR BATCH</button>}
    </div>
    <div className="market-multi-period__selector">{eligible.map((item) => <article key={item.id} className={selection[item.id] ? "selected" : ""}><label><input type="checkbox" checked={Boolean(selection[item.id])} onChange={() => toggle(item.id)} /><span><strong>{item.name}</strong><small>{item.asset} · {item.timeframe} · {item.datasetName}</small></span></label><select disabled={!selection[item.id]} value={selection[item.id] ?? "ADDITIONAL_VALIDATION"} onChange={(event) => setSelection((current) => ({ ...current, [item.id]: event.target.value as LifecycleValidationSourceRole }))}>{roles.map((role) => <option key={role} value={role}>{role}</option>)}</select></article>)}</div>
    {message && <p className="market-multi-period__message">{message}</p>}
    {!report ? <div className="market-lifecycle-empty">Selecione períodos compatíveis e não sobrepostos para criar um batch auditável.</div> : <>
      <div className="market-multi-period__toolbar"><span>{report.batch.asset} · {report.batch.timeframe} · {report.batch.agentVersion}</span><div><button className="button" onClick={() => void exportJson()}>EXPORTAR AUDITORIA JSON</button><button className="button" onClick={() => window.print()}>EXPORTAR PDF</button></div></div>
      <div className="market-multi-period__cards">{[
        ["DATASETS / PERIODS", `${report.sample.datasets} / ${report.sample.periods}`], ["LIFECYCLES", report.sample.lifecycles], ["CLOSED / OPEN / CENSORED", `${report.sample.closed} / ${report.sample.open} / ${report.sample.censored}`], ["DATE COVERAGE", `${date(report.sample.dateCoverageStart)} → ${date(report.sample.dateCoverageEnd)}`], ["LATE LIFECYCLE", `${number(report.sample.lifecycleLateReductionRatePct)}%`], ["LATE EVENT", `${number(report.sample.eventLateReductionRatePct)}%`], ["MEDIAN RESPONSE", report.sample.medianResponseDelayMinutes == null ? "UNRESOLVED" : `${number(report.sample.medianResponseDelayMinutes)} min`], ["SCORE STABILITY", report.sample.scoreStability], ["SAMPLE STATUS", report.sample.sampleStatus],
      ].map(([label, value]) => <article key={label}><span>{label}</span><b>{value}</b></article>)}</div>
      {report.warnings.length > 0 && <div className="market-multi-period__warnings">{report.warnings.map((warning) => <span key={warning}>⚠ {warning}</span>)}</div>}
      <section className="market-multi-period__section"><div className="panel-heading"><strong>DATASET QUALITY GATE</strong><span>OHLCV · TIMESTAMPS · US_EQUITIES REGULAR SESSION</span></div><div className="market-multi-period__table"><table><thead><tr><th>DATASET</th><th>STATUS</th><th>CANDLES</th><th>SESSIONS</th><th>DUPLICATES</th><th>INVALID OHLC</th><th>UNEXPECTED GAPS</th><th>SOURCE FILE</th></tr></thead><tbody>{report.quality.map((item) => <tr key={item.datasetId}><td>{item.datasetName}</td><td>{item.status}</td><td>{item.candleCount}</td><td>{item.sessionCount}</td><td>{item.duplicateTimestampCount}</td><td>{item.invalidHighCount + item.invalidLowCount}</td><td>{item.unexpectedGapCount}</td><td>{item.sourceFileAvailable ? "AVAILABLE" : "UNAVAILABLE · PERSISTED DB"}</td></tr>)}</tbody></table></div></section>
      <section className="market-multi-period__section"><div className="panel-heading"><strong>PERIOD SUMMARY</strong><span>SEM SOBREPOSIÇÃO · PNL REALIZADO SOMENTE CLOSED</span></div><div className="market-multi-period__table"><table><thead><tr><th>PERIOD</th><th>ROLE</th><th>DATE RANGE</th><th>CANDLES</th><th>SESSIONS</th><th>LIFECYCLES</th><th>CLOSED / OPEN</th><th>AVG PNL</th><th>MEDIAN MFE</th><th>MEDIAN MAE</th><th>GIVEBACK</th><th>LATE LIFECYCLE</th><th>LATE EVENT</th><th>RESPONSE</th><th>CONTEXT</th></tr></thead><tbody>{report.periods.map((period) => <tr key={period.periodId}><td>{period.datasetName}</td><td>{period.sourceRole}</td><td>{date(period.startAt)} → {date(period.endAt)}</td><td>{period.candleCount}</td><td>{period.sessionCount}</td><td>{period.lifecycleCount}</td><td>{period.closedCount} / {period.openCount}</td><td>{pct(period.pnl.average)}</td><td>{pct(period.mfe.median)}</td><td>{pct(period.mae.median)}</td><td>{pct(period.giveback.median)}</td><td>{number(period.lifecycleLateReductionRatePct)}%</td><td>{number(period.eventLateReductionRatePct)}%</td><td>{period.unresolvedResponseCount ? `${period.unresolvedResponseCount} UNRESOLVED` : `${number(period.responseDelay.median)} min`}</td><td>{period.context.trendProxy} · RET {pct(period.context.periodReturnPct)} · VOL {pct(period.context.realizedVolatilityPct)}</td></tr>)}</tbody></table></div></section>
      <section className="market-multi-period__section"><div className="panel-heading"><strong>CONSOLIDATED LIFECYCLES</strong><span>{filtered.length} / {report.lifecycles.length}</span></div><div className="market-multi-period__filters"><select value={filters.periodId} onChange={(event) => setFilters((current) => ({ ...current, periodId: event.target.value }))}><option value="">TODOS OS PERÍODOS</option>{report.periods.map((period) => <option key={period.periodId} value={period.periodId}>{period.datasetName}</option>)}</select><select value={filters.status} onChange={(event) => setFilters((current) => ({ ...current, status: event.target.value as LifecycleValidationFilters["status"] }))}><option value="ALL">OPEN + CLOSED</option><option value="OPEN">OPEN</option><option value="CLOSED">CLOSED</option></select><select value={filters.deterioration} onChange={(event) => setFilters((current) => ({ ...current, deterioration: event.target.value as LifecycleValidationFilters["deterioration"] }))}><option value="ALL">QUALQUER DETERIORATION</option>{(["NONE", "LOW", "MODERATE", "HIGH", "CRITICAL"] as DeteriorationLevel[]).map((level) => <option key={level} value={level}>{level} OU PIOR</option>)}</select><select value={filters.lateReduction} onChange={(event) => setFilters((current) => ({ ...current, lateReduction: event.target.value as LifecycleValidationFilters["lateReduction"] }))}><option value="ALL">LATE: TODOS</option><option value="YES">COM LATE</option><option value="NO">SEM LATE</option></select><select value={filters.overnight} onChange={(event) => setFilters((current) => ({ ...current, overnight: event.target.value as LifecycleValidationFilters["overnight"] }))}><option value="ALL">OVERNIGHT: TODOS</option><option value="YES">OVERNIGHT</option><option value="NO">INTRADAY</option></select></div><div className="market-multi-period__table"><table><thead><tr><th>PERIOD</th><th>LIFECYCLE</th><th>ENTRY</th><th>EXIT</th><th>STATUS</th><th>DURATION</th><th>PNL</th><th>MFE</th><th>MAE</th><th>GIVEBACK</th><th>WORST HEALTH</th><th>MAX DETERIORATION</th><th>LATE EVENTS</th><th>RESPONSE</th></tr></thead><tbody>{filtered.map((row) => <tr key={`${row.periodId}-${row.lifecycleId}`}><td>{report.periods.find((period) => period.periodId === row.periodId)?.datasetName}</td><td>{row.lifecycleId}</td><td>{date(row.entryAt)}</td><td>{date(row.exitAt)}</td><td>{row.status}{row.censoredAtDatasetEnd ? " · CENSORED" : ""}</td><td>{row.durationMinutes} min</td><td>{pct(row.pnlPct)}</td><td>{pct(row.mfePct)}</td><td>{pct(row.maePct)}</td><td>{pct(row.givebackPct)}</td><td>{row.worstHealth}</td><td>{row.maxDeteriorationLevel} · {number(row.maxDeteriorationScore, 3)}</td><td>{row.lateReductionEvents}</td><td>{responseLabel(row.responseDelayMinutes, row.responseStatus)}</td></tr>)}</tbody></table></div></section>
      <section className="market-multi-period__section"><div className="panel-heading"><strong>DISTRIBUTIONS</strong><span>DESCRITIVAS · SEM INFLAR N POR CANDLE</span></div><div className="market-multi-period__charts"><MiniDistribution title="LIFECYCLE PNL" suffix="%" values={report.lifecycles.flatMap((row) => row.pnlPct == null ? [] : [row.pnlPct])}/><MiniDistribution title="MFE" suffix="%" values={report.lifecycles.map((row) => row.mfePct)}/><MiniDistribution title="MAE" suffix="%" values={report.lifecycles.map((row) => row.maePct)}/><MiniDistribution title="GIVEBACK" suffix="%" values={report.lifecycles.map((row) => row.givebackPct)}/><MiniDistribution title="RESPONSE DELAY" suffix=" min" values={report.lifecycles.flatMap((row) => row.responseDelayMinutes == null ? [] : [row.responseDelayMinutes])}/><MiniDistribution title="MAX DETERIORATION" values={report.lifecycles.map((row) => row.maxDeteriorationScore)}/><PeriodRateChart report={report}/><MiniDistribution title="CRITICAL REACH BY PERIOD" suffix="%" values={report.deteriorationByPeriod.filter((row) => row.level === "CRITICAL").map((row) => row.lifecycleReachPct)}/></div></section>
      <div className="market-multi-period__analysis-grid"><section className="market-multi-period__section"><div className="panel-heading"><strong>CROSS-PERIOD CONSISTENCY</strong><span>MEDIAN · RANGE · DISPERSION</span></div><div className="market-multi-period__table"><table><thead><tr><th>METRIC</th><th>MEDIAN</th><th>MIN</th><th>MAX</th><th>DISPERSION</th></tr></thead><tbody>{report.consistency.map((item) => <tr key={item.label}><td>{item.label}</td><td>{number(item.median)}</td><td>{number(item.minimum)}</td><td>{number(item.maximum)}</td><td>{number(item.dispersion)}</td></tr>)}</tbody></table></div></section><section className="market-multi-period__section"><div className="panel-heading"><strong>EVIDENCE MATRIX</strong><span>SEM SCORE 0–100</span></div><dl className="market-multi-period__evidence"><div><dt>SAMPLE SIZE</dt><dd>{report.evidence.sampleSize}</dd></div><div><dt>PERIOD COVERAGE</dt><dd>{report.evidence.periodCoverage}</dd></div><div><dt>LATE REDUCTION</dt><dd>{report.evidence.lateReduction}</dd></div><div><dt>DETERIORATION STABILITY</dt><dd>{report.evidence.deteriorationStability}</dd></div><div><dt>HEALTH × OUTCOME</dt><dd>{report.evidence.healthOutcomeRelation}</dd></div><div><dt>COMPONENT CONSISTENCY</dt><dd>{report.evidence.componentConsistency}</dd></div><div><dt>GIVEBACK CONSISTENCY</dt><dd>{report.evidence.givebackConsistency}</dd></div></dl></section></div>
      <section className="market-multi-period__section"><div className="panel-heading"><strong>COMPONENT × OUTCOME</strong><span>AGREGADO POR LIFECYCLE · PESOS CONGELADOS</span></div><div className="market-multi-period__table"><table><thead><tr><th>SCOPE</th><th>COMPONENT</th><th>LIFECYCLES</th><th>AVG / MAX</th><th>AVG PNL</th><th>AVG MAE</th><th>AVG GIVEBACK</th><th>LATE %</th></tr></thead><tbody>{report.components.map((item) => <tr key={`${item.periodId ?? "ALL"}-${item.component}`}><td>{item.periodId ? report.periods.find((period) => period.periodId === item.periodId)?.datasetName : "ALL PERIODS"}</td><td>{item.component}</td><td>{item.lifecycleCount}</td><td>{number(item.averageValue, 3)} / {number(item.maximumValue, 3)}</td><td>{pct(item.averagePnlPct)}</td><td>{pct(item.averageMaePct)}</td><td>{pct(item.averageGivebackPct)}</td><td>{number(item.lateReductionRatePct)}%</td></tr>)}</tbody></table></div></section>
      <section className="market-multi-period__section"><div className="panel-heading"><strong>OUTLIER SENSITIVITY</strong><span>NENHUM LIFECYCLE É EXCLUÍDO AUTOMATICAMENTE</span></div><div className="market-multi-period__table"><table><thead><tr><th>METRIC</th><th>LARGEST OUTLIER</th><th>FULL AVG</th><th>WITHOUT OUTLIER</th><th>DELTA</th></tr></thead><tbody>{report.outlierSensitivity.map((item) => <tr key={item.metric}><td>{item.metric}</td><td>{item.lifecycleId ?? "N/A"}</td><td>{number(item.fullAverage)}</td><td>{number(item.withoutLargestOutlierAverage)}</td><td>{number(item.delta)}</td></tr>)}</tbody></table></div></section>
      <section className="market-multi-period__audit"><div className="panel-heading"><strong>BATCH AUDIT</strong><span>REPRODUCIBLE · LOCAL · READ-ONLY SOBRE O HISTÓRICO</span></div><code>{report.batch.batchId}</code><span>{report.batch.validationEngineVersion}</span><span>{report.batch.lifecycleConfigVersion}</span><span>{report.batch.deteriorationConfigVersion}</span><span>{report.batch.holdDiagnosticsVersion}</span><span>{report.batch.executionModelVersion}</span><span>REUSED {report.audit.reusedExperiments}</span><span>REBUILT {report.audit.rebuiltArtifacts}</span><span>NEW LLM RUNS {report.audit.newLlmRuns}</span><span>FAILED PERIODS {report.audit.failedPeriods}</span></section>
    </>}
  </section>;
}
