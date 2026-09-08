import { useMemo, useState } from "react";
import type { MarketAgentMetric, MarketBehaviorMetric, MarketExperimentResult } from "../../types";

const colors = ["#21dff3", "#55f0a7", "#ffbd4a", "#b58cff", "#ff6f91"];
const number = (value: number, digits = 2) => value.toFixed(digits);

type SortKey = "totalReturnPct" | "maxDrawdownPct" | "holdRatePct" | "turnoverPct" | "averageExposurePct";
type DetailTab = "performance" | "behavior" | "risk" | "decisions" | "positions" | "benchmarks";
type ChartMode = "equity" | "drawdown" | "exposure";

function metricValue(metric: MarketAgentMetric, behavior: MarketBehaviorMetric | undefined, key: SortKey) {
  if (key === "holdRatePct") return behavior?.holdRatePct ?? 0;
  if (key === "turnoverPct") return behavior?.turnoverPct ?? 0;
  if (key === "averageExposurePct") return behavior?.averageExposurePct ?? 0;
  return metric[key];
}

function ComparisonChart({ result, agents, mode }: { result: MarketExperimentResult; agents: string[]; mode: ChartMode }) {
  const series = agents.map((agentId) => {
    const raw = result.equity.filter((point) => point.agentId === agentId);
    let peak = raw[0]?.equity ?? 0;
    const values = raw.map((point) => {
      peak = Math.max(peak, point.equity);
      if (mode === "equity") return point.equity;
      if (mode === "exposure") return point.exposurePct;
      return peak > 0 ? ((peak - point.equity) / peak) * 100 : 0;
    });
    return { agentId, values };
  });
  const values = series.flatMap((entry) => entry.values);
  const min = Math.min(...values, 0); const max = Math.max(...values, 1); const range = Math.max(max - min, 1);
  return <div className="market-chart market-observatory__chart"><svg viewBox="0 0 900 230" preserveAspectRatio="none" role="img" aria-label={`Comparação de ${mode}`}>
    {[0, 1, 2, 3, 4].map((line) => <line key={line} x1="0" x2="900" y1={line * 57.5} y2={line * 57.5} />)}
    {series.map((entry, index) => <polyline key={entry.agentId} style={{ stroke: colors[index % colors.length] }} points={entry.values.map((value, pointIndex) => `${pointIndex / Math.max(entry.values.length - 1, 1) * 900},${215 - (value - min) / range * 200}`).join(" ")} />)}
  </svg><div className="market-chart__legend">{series.map((entry, index) => <span key={entry.agentId}><i style={{ background: colors[index % colors.length] }} />{result.metrics.find((metric) => metric.agentId === entry.agentId)?.agentName ?? entry.agentId}</span>)}</div></div>;
}

function Formula({ children }: { children: React.ReactNode }) { return <small className="market-formula">{children}</small>; }

export function MarketObservatory({ result }: { result: MarketExperimentResult }) {
  const [sortKey, setSortKey] = useState<SortKey>("totalReturnPct");
  const [selectedAgent, setSelectedAgent] = useState(result.metrics[0]?.agentId ?? "");
  const [detailTab, setDetailTab] = useState<DetailTab>("performance");
  const [comparisonAgents, setComparisonAgents] = useState(() => result.metrics.slice(0, 5).map((metric) => metric.agentId));
  const [chartMode, setChartMode] = useState<ChartMode>("equity");
  const behaviorMap = useMemo(() => new Map(result.observatory.behavior.map((metric) => [metric.agentId, metric])), [result]);
  const sorted = useMemo(() => [...result.metrics].sort((left, right) => metricValue(right, behaviorMap.get(right.agentId), sortKey) - metricValue(left, behaviorMap.get(left.agentId), sortKey)), [result, behaviorMap, sortKey]);
  const metric = result.metrics.find((item) => item.agentId === selectedAgent) ?? result.metrics[0];
  const behavior = behaviorMap.get(metric?.agentId ?? "");
  const benchmark = result.observatory.benchmarks.find((item) => item.agentId === metric?.agentId);
  const episodes = result.observatory.episodes.filter((item) => item.agentId === metric?.agentId);
  const decisions = result.decisions.filter((item) => item.agentId === metric?.agentId);
  const toggleComparison = (agentId: string) => setComparisonAgents((current) => current.includes(agentId) ? current.length > 2 ? current.filter((id) => id !== agentId) : current : current.length < 5 ? [...current, agentId] : current);
  const correlation = (left: string, right: string) => result.observatory.correlations.find((item) => (item.agentAId === left && item.agentBId === right) || (item.agentAId === right && item.agentBId === left));

  return <div className="market-observatory">
    <section className="market-observatory__summary">
      <div><span>COORTE</span><strong>{result.experiment.agentIds.length} agentes</strong><small>Mesmo dataset, capital, risco, custos e seed</small></div>
      <div><span>DATASET SNAPSHOT</span><strong>{result.experiment.asset} / {result.experiment.timeframe}</strong><small>{result.experiment.datasetName}</small></div>
      <div><span>STATUS</span><strong>{result.experiment.status.toUpperCase()}</strong><small>Identidade e configuração congeladas</small></div>
    </section>

    <div className="panel-heading"><strong>AGENT COMPARISON</strong><label>ORDENAR POR <select value={sortKey} onChange={(event) => setSortKey(event.target.value as SortKey)}><option value="totalReturnPct">Retorno</option><option value="maxDrawdownPct">Drawdown</option><option value="holdRatePct">Taxa de HOLD</option><option value="turnoverPct">Turnover</option><option value="averageExposurePct">Exposição média</option></select></label></div>
    <div className="market-metrics market-observatory__table"><table><thead><tr><th>AGENTE</th><th>RETORNO</th><th>DRAWDOWN</th><th>PF</th><th>WIN RATE</th><th>HOLD</th><th>EXPOSIÇÃO</th><th>TURNOVER</th><th>FREQ.</th></tr></thead><tbody>{sorted.map((item) => { const itemBehavior = behaviorMap.get(item.agentId); return <tr key={item.agentId} className={item.agentId === metric?.agentId ? "selected" : ""} onClick={() => setSelectedAgent(item.agentId)}><td><strong>{item.agentName}</strong><small>{item.agentId}</small></td><td>{number(item.totalReturnPct)}%</td><td>{number(item.maxDrawdownPct)}%</td><td>{item.profitFactor == null ? "N/A" : number(item.profitFactor)}</td><td>{number(item.winRatePct)}%</td><td>{number(itemBehavior?.holdRatePct ?? 0)}%</td><td>{number(itemBehavior?.averageExposurePct ?? 0)}%</td><td>{number(itemBehavior?.turnoverPct ?? 0)}%</td><td>{number(itemBehavior?.tradeFrequencyPct ?? 0)}%</td></tr>; })}</tbody></table></div>

    <section className="market-comparison">
      <header><div><strong>COMPARISON MODE</strong><span>Selecione entre 2 e 5 agentes · sem pontuação única</span></div><div className="market-segmented">{(["equity", "drawdown", "exposure"] as ChartMode[]).map((mode) => <button key={mode} className={chartMode === mode ? "active" : ""} onClick={() => setChartMode(mode)}>{mode.toUpperCase()}</button>)}</div></header>
      <div className="market-agent-selector">{result.metrics.map((item) => <label key={item.agentId}><input type="checkbox" checked={comparisonAgents.includes(item.agentId)} onChange={() => toggleComparison(item.agentId)} />{item.agentName}</label>)}</div>
      <ComparisonChart result={result} agents={comparisonAgents} mode={chartMode} />
    </section>

    <section className="market-correlation">
      <div className="panel-heading"><strong>CORRELAÇÃO / SIMILARIDADE</strong><span>RETORNOS DA EQUITY · ALERTA ≥ {number(result.observatory.similarityThreshold * 100, 0)}%</span></div>
      <div className="market-correlation__matrix" style={{ gridTemplateColumns: `150px repeat(${result.metrics.length}, minmax(82px, 1fr))` }}><span />{result.metrics.map((item) => <strong key={item.agentId}>{item.agentName}</strong>)}{result.metrics.flatMap((row) => [<strong key={`${row.agentId}-label`}>{row.agentName}</strong>, ...result.metrics.map((column) => { const cell = correlation(row.agentId, column.agentId); return <div key={`${row.agentId}-${column.agentId}`} className={cell?.highSimilarity ? "warning" : ""}><b>{cell?.equityReturnCorrelation == null ? "N/A" : number(cell.equityReturnCorrelation)}</b><small>DEC {number((cell?.decisionSimilarity ?? 0) * 100, 0)}%</small></div>; })])}</div>
    </section>

    {metric && <section className="market-agent-detail">
      <header><div><span>AGENT OBSERVATORY</span><h3>{metric.agentName}</h3><small>Configuração imutável no experimento {result.experiment.id}</small></div><div className="market-agent-detail__tabs">{(["performance", "behavior", "risk", "decisions", "positions", "benchmarks"] as DetailTab[]).map((item) => <button key={item} className={detailTab === item ? "active" : ""} onClick={() => setDetailTab(item)}>{item.toUpperCase()}</button>)}</div></header>
      {detailTab === "performance" && <div className="market-stat-grid"><div><span>EQUITY FINAL</span><strong>{number(metric.finalEquity)}</strong></div><div><span>RETORNO</span><strong>{number(metric.totalReturnPct)}%</strong></div><div><span>MAX DRAWDOWN</span><strong>{number(metric.maxDrawdownPct)}%</strong></div><div><span>PROFIT FACTOR</span><strong>{metric.profitFactor == null ? "N/A" : number(metric.profitFactor)}</strong></div><div><span>WIN RATE</span><strong>{number(metric.winRatePct)}%</strong></div><div><span>TRADES</span><strong>{metric.tradeCount}</strong></div></div>}
      {detailTab === "behavior" && behavior && <div className="market-stat-grid"><div><span>BUY / SELL / HOLD</span><strong>{behavior.buyCount} / {behavior.sellCount} / {behavior.holdCount}</strong><Formula>Contagem das decisões por ação</Formula></div><div><span>HOLD RATE</span><strong>{number(behavior.holdRatePct)}%</strong><Formula>HOLD ÷ decisões × 100</Formula></div><div><span>TIME IN MARKET</span><strong>{number(behavior.timeInMarketPct)}%</strong><Formula>Candles expostos ÷ snapshots × 100</Formula></div><div><span>TIME IN CASH</span><strong>{number(behavior.timeInCashPct)}%</strong><Formula>Candles sem posição ÷ snapshots × 100</Formula></div><div><span>TURNOVER</span><strong>{number(behavior.turnoverPct)}%</strong><Formula>Valor bruto negociado ÷ equity média × 100</Formula></div><div><span>HOLDING MÉDIO / MEDIANO / MÁX.</span><strong>{number(behavior.averageHoldingCandles, 1)} / {number(behavior.medianHoldingCandles, 1)} / {behavior.maxHoldingCandles}</strong><Formula>Duração em candles expostos</Formula></div><div><span>POSIÇÃO MÉDIA / MÁX.</span><strong>{number(behavior.averagePositionSizePct)}% / {number(behavior.maxPositionSizePct)}%</strong><Formula>Exposição apenas durante posições abertas</Formula></div><div><span>FREQUÊNCIA</span><strong>{number(behavior.tradeFrequencyPct)}%</strong><Formula>Execuções ÷ candles × 100</Formula></div></div>}
      {detailTab === "risk" && behavior && <div className="market-stat-grid"><div><span>REJEIÇÕES</span><strong>{behavior.riskRejectionCount}</strong><Formula>{number(behavior.riskRejectionRatePct)}% das decisões</Formula></div><div><span>MODIFICAÇÕES</span><strong>{behavior.riskModificationCount}</strong><Formula>{number(behavior.riskModificationRatePct)}% das decisões</Formula></div><div><span>TRIGGERS DRAWDOWN</span><strong>{behavior.drawdownTriggerCount}</strong></div><div><span>TRIGGERS PERDA DIÁRIA</span><strong>{behavior.dailyLossTriggerCount}</strong></div></div>}
      {detailTab === "decisions" && <div className="market-timeline">{decisions.map((decision) => <article key={decision.id}><i className={decision.action.toLowerCase()}>{decision.action}</i><div><strong>{decision.timestamp}</strong><span>{decision.reasoning}</span></div><b>{decision.riskResult}</b></article>)}</div>}
      {detailTab === "positions" && <div className="market-episodes">{episodes.length === 0 ? <p>Nenhum episódio de posição.</p> : episodes.map((episode) => <article key={episode.episodeIndex}><strong>EPISÓDIO {String(episode.episodeIndex).padStart(2, "0")}</strong><span>{episode.openedAt} → {episode.closedAt ?? "ABERTO"}</span><b>{episode.durationCandles} candles · exposição máx. {number(episode.maxExposurePct)}%</b></article>)}</div>}
      {detailTab === "benchmarks" && benchmark && <div className="market-stat-grid"><div><span>EXCESSO VS CASH</span><strong>{number(benchmark.excessVsCashPct)}%</strong></div><div><span>EXCESSO VS BUY & HOLD</span><strong>{benchmark.excessVsBuyHoldPct == null ? "N/A" : `${number(benchmark.excessVsBuyHoldPct)}%`}</strong></div><div><span>RETORNO CASH</span><strong>{benchmark.cashReturnPct == null ? "N/A" : `${number(benchmark.cashReturnPct)}%`}</strong></div><div><span>RETORNO BUY & HOLD</span><strong>{benchmark.buyHoldReturnPct == null ? "N/A" : `${number(benchmark.buyHoldReturnPct)}%`}</strong></div></div>}
    </section>}
  </div>;
}
