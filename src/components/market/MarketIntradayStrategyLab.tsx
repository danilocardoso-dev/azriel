import { useMemo, useState } from "react";
import type { MarketExperimentResult } from "../../types";

const format = (value: unknown) => typeof value === "number" ? value.toFixed(4) : String(value ?? "N/A");

export function MarketIntradayStrategyLab({ result }: { result: MarketExperimentResult }) {
  const report = result.intradayStrategy;
  const [decisionId, setDecisionId] = useState(report.decisions[0]?.decisionId ?? 0);
  const selected = report.decisions.find((item) => item.decisionId === decisionId) ?? report.decisions[0];
  const feature = useMemo(() => selected ? report.featureTraces.find((item) => item.candleIndex === selected.candleIndex) : undefined, [report.featureTraces, selected]);
  if (!report.featureEngineVersion) return null;

  return <section className="market-strategy-lab">
    <div className="panel-heading"><strong>STRATEGY OBSERVATORY</strong><span>{report.featureEngineVersion} · POSITION SIZING V1 · RISK POLICY V2 · EXECUTION T+1</span></div>
    <div className="market-strategy-lab__metrics">{report.metrics.map((metric) => <article key={metric.agentId}>
      <span>{metric.style}</span><strong>{result.metrics.find((item) => item.agentId === metric.agentId)?.agentName ?? metric.agentId}</strong>
      <dl><div><dt>ENTRIES</dt><dd>{metric.trendEntries + metric.breakoutEntries + metric.meanReversionEntries}</dd></div><div><dt>DURAÇÃO MÉDIA</dt><dd>{metric.averageTrendDurationMinutes.toFixed(0)} min</dd></div><div><dt>CONFIRMAÇÕES</dt><dd>{metric.emaCrossEntries + metric.breakoutEntries + metric.successfulReversions}</dd></div><div><dt>FALHAS / DESVIOS</dt><dd>{metric.failedBreakouts + metric.vwapDeviationEvents}</dd></div></dl>
    </article>)}</div>

    <div className="market-strategy-lab__tables">
      <div><strong>PERFORMANCE POR FASE</strong><table><thead><tr><th>AGENTE</th><th>FASE</th><th>TRADES</th><th>RETORNO</th><th>WIN</th><th>PNL MÉDIO</th></tr></thead><tbody>{report.phasePerformance.map((item) => <tr key={`${item.agentId}-${item.sessionPhase}`}><td>{item.agentId}</td><td>{item.sessionPhase}</td><td>{item.trades}</td><td>{item.returnPct.toFixed(2)}%</td><td>{item.winRatePct.toFixed(1)}%</td><td>{item.averagePnl.toFixed(2)}</td></tr>)}</tbody></table></div>
      <div><strong>SOBREPOSIÇÃO COMPORTAMENTAL</strong><table><thead><tr><th>PAR</th><th>MESMA DIREÇÃO</th><th>ENTRADAS</th><th>SAÍDAS</th></tr></thead><tbody>{report.overlaps.map((item) => <tr key={`${item.agentAId}-${item.agentBId}`}><td>{item.agentAId}<br />{item.agentBId}</td><td>{item.sameDirectionDecisionRate.toFixed(1)}%</td><td>{item.sameEntryWindowCount}</td><td>{item.sameExitWindowCount}</td></tr>)}</tbody></table><div className="market-holding-buckets">{report.holdingDistribution.map((item) => <span key={`${item.agentId}-${item.bucket}`}>{item.agentId} · {item.bucket} min <b>{item.tradeCount}</b></span>)}</div></div>
    </div>

    <div className="market-strategy-lab__inspectors">
      <aside><strong>DECISION TRACE</strong>{report.decisions.slice(0, 250).map((item) => <button key={item.decisionId} className={selected?.decisionId === item.decisionId ? "active" : ""} onClick={() => setDecisionId(item.decisionId)}><span>{item.timestamp} · {item.agentId}</span><b>{item.intent} / {item.reasonCode}</b><small>{item.riskResult} · alvo {item.generatedTargetExposurePct.toFixed(1)}%</small></button>)}</aside>
      {selected && <div className="market-feature-inspector"><header><div><span>STRATEGY INSPECTOR</span><strong>{selected.style} · {selected.intent}</strong></div><b>{selected.reasonCode}</b></header><p>{selected.reason}</p><div><section><strong>INDICADORES OBSERVADOS</strong><dl>{Object.entries(selected.indicators).map(([key, value]) => <div key={key}><dt>{key}</dt><dd>{format(value)}</dd></div>)}</dl></section><section><strong>THRESHOLDS CONGELADOS</strong><dl>{Object.entries(selected.thresholds).map(([key, value]) => <div key={key}><dt>{key}</dt><dd>{format(value)}</dd></div>)}</dl></section></div>{feature && <section className="market-feature-inspector__trace"><strong>FEATURE INSPECTOR · CANDLE {feature.candleIndex}</strong><span>{feature.timestampUtc} · {feature.sessionPhase} · sessão {(feature.sessionProgress * 100).toFixed(1)}% · {feature.ready ? "READY" : "WARMUP"}</span><dl>{Object.entries(feature.features).filter(([, value]) => typeof value === "number" || value == null).map(([key, value]) => <div key={key}><dt>{key}</dt><dd>{format(value)}</dd></div>)}</dl></section>}</div>}
    </div>
  </section>;
}
