import { useEffect, useMemo, useState } from "react";
import { marketLabRepository } from "../../repositories/marketLabRepository";
import type { MarketAgentDefinition, MarketDataset, MarketRegimeMetric, MarketRiskProfile, MarketRollingMetric, MarketValidationInput, MarketValidationResult, MarketValidationSummary } from "../../types";

type ValidationDetail = "summary" | "windows" | "regimes" | "rolling" | "benchmarks" | "audit";
type RollingMode = "rollingReturnPct" | "rollingVolatilityPct" | "rollingSharpe" | "rollingDrawdownPct";
type ReportSort = "positiveWindowRatioPct" | "outOfSampleReturnPct" | "outOfSampleDrawdownPct" | "returnStdAcrossWindows";
const fmt = (value: number | null, suffix = "") => value == null ? "N/A" : `${value.toFixed(2)}${suffix}`;
const label = (value: string) => value.replaceAll("_", " ").toUpperCase();

interface Props {
  datasets: MarketDataset[];
  agents: MarketAgentDefinition[];
  profiles: MarketRiskProfile[];
  datasetId: string;
  riskProfileId: string;
  agentIds: string[];
  initialCapital: number;
  randomSeed: number;
  feePct: number;
  slippagePct: number;
  onError: (message: string) => void;
}

function RollingChart({ points, mode }: { points: MarketRollingMetric[]; mode: RollingMode }) {
  const usable = points.filter((point) => point[mode] != null);
  const values = usable.map((point) => point[mode] as number);
  if (values.length < 2) return <div className="market-validation__no-data">DADOS INSUFICIENTES PARA A JANELA ROLLING</div>;
  const min = Math.min(...values); const max = Math.max(...values); const range = Math.max(max - min, 0.0001);
  const polyline = values.map((value, index) => `${index / (values.length - 1) * 900},${205 - (value - min) / range * 185}`).join(" ");
  return <div className="market-chart market-validation__chart"><svg viewBox="0 0 900 220" preserveAspectRatio="none" role="img" aria-label={label(mode)}>{[0, 1, 2, 3, 4].map((line) => <line key={line} x1="0" x2="900" y1={line * 52} y2={line * 52} />)}<polyline points={polyline} /></svg><div className="market-validation__range"><span>MIN {fmt(min)}</span><span>MAX {fmt(max)}</span></div></div>;
}

function RegimeMatrix({ metrics, agents }: { metrics: MarketRegimeMetric[]; agents: MarketAgentDefinition[] }) {
  const regimes = ["bull", "bear", "sideways", "high_volatility", "low_volatility"];
  return <div className="market-regime-matrix" style={{ gridTemplateColumns: `minmax(170px,1.2fr) repeat(${regimes.length},minmax(115px,1fr))` }}>
    <strong>AGENTE / REGIME</strong>{regimes.map((regime) => <strong key={regime}>{label(regime)}</strong>)}
    {agents.map((agent) => [<strong key={`${agent.id}-label`}>{agent.name}</strong>, ...regimes.map((regime) => { const metric = metrics.find((item) => item.agentId === agent.id && item.regime === regime); return <div key={`${agent.id}-${regime}`} className={metric?.lowSampleSize ? "low-sample" : ""}><b>{fmt(metric?.totalReturnPct ?? null, "%")}</b><span>DD {fmt(metric?.maxDrawdownPct ?? null, "%")}</span><small>{metric?.candleCount ?? 0} candles · {metric?.tradeCount ?? 0} trades</small>{metric?.lowSampleSize && <em>LOW SAMPLE SIZE</em>}</div>; })])}
  </div>;
}

export function MarketValidation({ datasets, agents, profiles, datasetId, riskProfileId, agentIds, initialCapital, randomSeed, feePct, slippagePct, onError }: Props) {
  const dataset = datasets.find((item) => item.id === datasetId);
  const profile = profiles.find((item) => item.id === riskProfileId);
  const candleCount = dataset?.candleCount ?? 30;
  const suggestedTest = Math.max(2, Math.floor(candleCount * 0.2));
  const [name, setName] = useState("VAL-001");
  const [split, setSplit] = useState({ inSamplePct: 60, validationPct: 20, outOfSamplePct: 20 });
  const [walk, setWalk] = useState({ trainWindowSize: Math.max(4, Math.floor(candleCount * 0.4)), testWindowSize: suggestedTest, stepSize: suggestedTest });
  const [rollingWindow, setRollingWindow] = useState(Math.max(3, Math.min(20, Math.floor(candleCount / 6))));
  const [annualizationFactor, setAnnualizationFactor] = useState(252);
  const [result, setResult] = useState<MarketValidationResult | null>(null);
  const [history, setHistory] = useState<MarketValidationSummary[]>([]);
  const [busy, setBusy] = useState(false);
  const [selectedAgent, setSelectedAgent] = useState(agentIds[0] ?? "");
  const [detail, setDetail] = useState<ValidationDetail>("summary");
  const [rollingMode, setRollingMode] = useState<RollingMode>("rollingReturnPct");
  const [reportSort, setReportSort] = useState<ReportSort>("positiveWindowRatioPct");
  useEffect(() => { void marketLabRepository.listValidations().then(setHistory).catch(() => undefined); }, []);

  const selectedReport = result?.reports.find((item) => item.agentId === selectedAgent) ?? result?.reports[0];
  const selectedId = selectedReport?.agentId ?? selectedAgent;
  const selectedMetrics = useMemo(() => result?.metrics.filter((item) => item.agentId === selectedId) ?? [], [result, selectedId]);
  const sortedReports = useMemo(() => [...(result?.reports ?? [])].sort((left,right) => reportSort === "outOfSampleDrawdownPct" || reportSort === "returnStdAcrossWindows" ? left[reportSort]-right[reportSort] : right[reportSort]-left[reportSort]), [result, reportSort]);
  const splitWindows = result?.windows.filter((item) => item.windowType !== "walk_forward") ?? [];
  const run = async () => {
    if (!datasetId || !riskProfileId) { onError("Selecione dataset e perfil de risco antes da validação."); return; }
    if (agentIds.length < 3 || agentIds.length > 5) { onError("A validação exige uma coorte de 3 a 5 agentes."); return; }
    const input: MarketValidationInput = { name, datasetId, riskProfileId, agentIds, initialCapital, randomSeed, feePct, slippagePct, splitConfig: split, walkForwardConfig: walk, regimeConfig: { trendWindow: Math.min(20, Math.max(3, rollingWindow)), volatilityWindow: Math.min(20, Math.max(3, rollingWindow)), bullThresholdPct: 2, bearThresholdPct: -2, highVolatilityThresholdPct: 1.5, lowVolatilityThresholdPct: 0.25, minimumSampleCandles: Math.min(20, Math.max(3, Math.floor(candleCount * 0.05))), minimumSampleTrades: 2 }, rollingWindow, annualizationFactor };
    setBusy(true);
    try { const next = await marketLabRepository.runValidation(input); setResult(next); setSelectedAgent(next.reports[0]?.agentId ?? ""); setHistory(await marketLabRepository.listValidations()); }
    catch (cause) { onError(String(cause)); }
    finally { setBusy(false); }
  };
  const open = async (id: string) => { setBusy(true); try { const next = await marketLabRepository.getValidation(id); setResult(next); setSelectedAgent(next.reports[0]?.agentId ?? ""); } catch (cause) { onError(String(cause)); } finally { setBusy(false); } };

  return <div className="market-validation">
    <section className="market-validation__config">
      <div className="panel-heading"><strong>SCIENTIFIC VALIDATION</strong><span>CONFIGURAÇÃO EXPLÍCITA E IMUTÁVEL</span></div>
      <div className="market-validation__form">
        <label>VALIDATION RUN<input value={name} onChange={(event) => setName(event.target.value)} /></label>
        <label>ANNUALIZATION FACTOR<input type="number" min="1" value={annualizationFactor} onChange={(event) => setAnnualizationFactor(Number(event.target.value))} /></label>
        <label>ROLLING WINDOW<input type="number" min="2" max={candleCount} value={rollingWindow} onChange={(event) => setRollingWindow(Number(event.target.value))} /></label>
      </div>
      <div className="market-validation__method">
        <fieldset><legend>TEMPORAL SPLIT · SOMA 100%</legend><div><label>IN-SAMPLE %<input type="number" value={split.inSamplePct} onChange={(event) => setSplit({ ...split, inSamplePct:Number(event.target.value) })} /></label><label>VALIDATION %<input type="number" value={split.validationPct} onChange={(event) => setSplit({ ...split, validationPct:Number(event.target.value) })} /></label><label>OUT-OF-SAMPLE %<input type="number" value={split.outOfSamplePct} onChange={(event) => setSplit({ ...split, outOfSamplePct:Number(event.target.value) })} /></label></div></fieldset>
        <fieldset><legend>WALK-FORWARD · CANDLES</legend><div><label>TRAIN<input type="number" min="2" value={walk.trainWindowSize} onChange={(event) => setWalk({ ...walk, trainWindowSize:Number(event.target.value) })} /></label><label>TEST<input type="number" min="2" value={walk.testWindowSize} onChange={(event) => setWalk({ ...walk, testWindowSize:Number(event.target.value) })} /></label><label>STEP<input type="number" min="1" value={walk.stepSize} onChange={(event) => setWalk({ ...walk, stepSize:Number(event.target.value) })} /></label></div></fieldset>
      </div>
      <div className="market-validation__actions"><button className="button button--primary" onClick={() => void run()} disabled={busy || !datasetId || agentIds.length < 3}>{busy ? "VALIDATING..." : "▶ RUN SCIENTIFIC VALIDATION"}</button><span>{dataset ? `${dataset.name} · ${dataset.candleCount} candles` : "SELECIONE UM DATASET"} · {profile?.name ?? "SEM RISCO"} · {agentIds.length} agentes</span></div>
      {history.length > 0 && <label className="market-validation__history">VALIDAÇÕES PERSISTIDAS<select value={result?.validation.id ?? ""} onChange={(event) => void open(event.target.value)}><option value="">Selecionar...</option>{history.map((item) => <option key={item.id} value={item.id}>{item.name} · {item.status.toUpperCase()} · {item.datasetName}</option>)}</select></label>}
    </section>

    {!result && <div className="market-empty market-validation__empty"><span>◇</span><strong>NENHUMA VALIDATION RUN SELECIONADA</strong><p>Configure splits temporais e walk-forward. Nenhum parâmetro será otimizado automaticamente.</p></div>}
    {result && <>
      <section className="market-validation__header"><div><span>VALIDATION RUN</span><strong>{result.validation.name}</strong><small>{result.validation.datasetName} · {result.validation.agentIds.length} agentes</small></div><div><span>METHOD</span><strong>SPLIT + WALK-FORWARD</strong><small>{result.windows.filter((item) => item.windowType === "walk_forward").length} janelas de teste</small></div><div><span>STATUS</span><strong>{result.validation.status.toUpperCase()}</strong><small>{result.audit.validationEngineVersion}</small></div></section>
      <section className="market-split"><div className="panel-heading"><strong>TEMPORAL SPLIT</strong><span>SEM SHUFFLE · RANGES REAIS</span></div><div className="market-split__bar">{splitWindows.map((window) => <div key={window.id} className={window.windowType} style={{ flex: window.endIndex - window.startIndex + 1 }}><strong>{label(window.windowType)}</strong><span>{window.startIndex + 1}–{window.endIndex + 1}</span><small>{window.startAt} → {window.endAt}</small></div>)}</div></section>
      <section className="market-validation__report-table"><div className="panel-heading"><strong>AGENT VALIDATION TABLE</strong><label>ORDENAR POR <select value={reportSort} onChange={(event)=>setReportSort(event.target.value as ReportSort)}><option value="positiveWindowRatioPct">Janelas positivas</option><option value="outOfSampleReturnPct">Retorno OOS</option><option value="outOfSampleDrawdownPct">Menor DD OOS</option><option value="returnStdAcrossWindows">Menor dispersão</option></select></label></div><div className="market-metrics"><table><thead><tr><th>AGENT</th><th>IS RETURN</th><th>VAL RETURN</th><th>OOS RETURN</th><th>OOS DD</th><th>SHARPE</th><th>SORTINO</th><th>CALMAR</th><th>POSITIVE WINDOWS</th><th>STATUS</th></tr></thead><tbody>{sortedReports.map((report)=>{const oos=result.metrics.find((metric)=>metric.agentId===report.agentId&&metric.windowType==="out_of_sample");return <tr key={report.agentId} className={selectedId===report.agentId?"selected":""} onClick={()=>setSelectedAgent(report.agentId)}><td><strong>{report.agentName}</strong></td><td>{fmt(report.inSampleReturnPct,"%")}</td><td>{fmt(report.validationReturnPct,"%")}</td><td className="market-validation__oos-cell">{fmt(report.outOfSampleReturnPct,"%")}</td><td>{fmt(report.outOfSampleDrawdownPct,"%")}</td><td>{fmt(oos?.sharpe??null)}</td><td>{fmt(oos?.sortino??null)}</td><td>{fmt(oos?.calmar??null)}</td><td>{fmt(report.positiveWindowRatioPct,"%")}</td><td>{label(report.robustnessStatus)}{report.possibleOverfitting&&<small>POSSIBLE OVERFITTING</small>}</td></tr>})}</tbody></table></div></section>
      <div className="market-validation__agent-tabs">{result.reports.map((report) => <button key={report.agentId} className={selectedId === report.agentId ? "active" : ""} onClick={() => setSelectedAgent(report.agentId)}>{report.agentName}</button>)}</div>
      <div className="market-validation__detail-tabs">{(["summary","windows","regimes","rolling","benchmarks","audit"] as ValidationDetail[]).map((item) => <button key={item} className={detail === item ? "active" : ""} onClick={() => setDetail(item)}>{item.toUpperCase()}</button>)}</div>

      {detail === "summary" && selectedReport && <><div className="market-validation__oos"><span>OUT OF SAMPLE</span><strong>{fmt(selectedReport.outOfSampleReturnPct,"%")}</strong><small>DD {fmt(selectedReport.outOfSampleDrawdownPct,"%")} · VS BUY & HOLD {fmt(selectedReport.benchmarkExcessPct,"%")}</small></div><div className="market-stat-grid market-validation__stats"><div><span>IN-SAMPLE</span><strong>{fmt(selectedReport.inSampleReturnPct,"%")}</strong></div><div><span>VALIDATION</span><strong>{fmt(selectedReport.validationReturnPct,"%")}</strong></div><div><span>POSITIVE WINDOWS</span><strong>{fmt(selectedReport.positiveWindowRatioPct,"%")}</strong></div><div><span>DISPERSION</span><strong>{fmt(selectedReport.returnStdAcrossWindows)}</strong></div><div><span>BEST / WORST</span><strong>{fmt(selectedReport.bestWindowReturnPct,"%")} / {fmt(selectedReport.worstWindowReturnPct,"%")}</strong></div><div><span>OVERFITTING GAP</span><strong>{fmt(selectedReport.overfittingGapPct,"%")}</strong></div><div className="market-validation__status"><span>ROBUSTNESS STATUS</span><strong>{label(selectedReport.robustnessStatus)}</strong></div>{selectedReport.possibleOverfitting && <div className="market-validation__warning"><span>SCIENTIFIC WARNING</span><strong>POSSIBLE OVERFITTING</strong></div>}</div></>}

      {detail === "windows" && <div className="market-metrics"><table><thead><tr><th>WINDOW</th><th>RANGE</th><th>RETURN</th><th>DD</th><th>SHARPE</th><th>SORTINO</th><th>CALMAR</th><th>TRADES</th></tr></thead><tbody>{selectedMetrics.map((metric) => { const window=result.windows.find((item)=>item.id===metric.windowId); return <tr key={metric.windowId}><td><strong>{metric.windowType === "walk_forward" ? `W${String(metric.windowIndex+1).padStart(2,"0")}` : label(metric.windowType)}</strong></td><td>{window ? `${window.startIndex+1}–${window.endIndex+1}` : "—"}</td><td>{fmt(metric.totalReturnPct,"%")}</td><td>{fmt(metric.maxDrawdownPct,"%")}</td><td>{fmt(metric.sharpe)}</td><td>{fmt(metric.sortino)}</td><td>{fmt(metric.calmar)}</td><td>{metric.tradeCount}</td></tr>;})}</tbody></table></div>}
      {detail === "regimes" && <><div className="panel-heading"><strong>PERFORMANCE BY REGIME MATRIX</strong><span>DIAGNÓSTICO DO DATASET COMPLETO · DIREÇÃO + VOLATILIDADE · LOW SAMPLE DESTACADO</span></div><RegimeMatrix metrics={result.regimeMetrics} agents={agents.filter((agent)=>result.validation.agentIds.includes(agent.id))} /></>}
      {detail === "rolling" && <><div className="market-segmented market-validation__rolling-tabs">{(["rollingReturnPct","rollingVolatilityPct","rollingSharpe","rollingDrawdownPct"] as RollingMode[]).map((mode)=><button key={mode} className={rollingMode===mode?"active":""} onClick={()=>setRollingMode(mode)}>{label(mode.replace("rolling",""))}</button>)}</div><RollingChart points={result.rollingMetrics.filter((item)=>item.agentId===selectedId)} mode={rollingMode} /></>}
      {detail === "benchmarks" && <div className="market-metrics"><table><thead><tr><th>PERÍODO</th><th>AGENT RETURN</th><th>VS CASH</th><th>VS BUY & HOLD</th></tr></thead><tbody>{selectedMetrics.filter((item)=>item.windowType!=="walk_forward").map((metric)=><tr key={metric.windowId}><td><strong>{label(metric.windowType)}</strong></td><td>{fmt(metric.totalReturnPct,"%")}</td><td>{fmt(metric.benchmarkCashExcessPct,"%")}</td><td>{fmt(metric.benchmarkBuyHoldExcessPct,"%")}</td></tr>)}</tbody></table></div>}
      {detail === "audit" && <div className="market-validation__audit"><div><span>DATASET HASH</span><code>{result.validation.datasetHash}</code></div><div><span>FORMULA VERSIONS</span><code>{result.audit.validationEngineVersion} · {result.audit.metricFormulaVersion} · {result.audit.regimeEngineVersion}</code></div><div><span>FROZEN CONFIG</span><pre>{JSON.stringify(JSON.parse(result.audit.frozenConfigJson),null,2)}</pre></div><div><span>SPLIT / WALK-FORWARD / REGIMES</span><pre>{[result.audit.splitConfigJson,result.audit.walkForwardConfigJson,result.audit.regimeConfigJson].join("\n")}</pre></div></div>}
    </>}
  </div>;
}
