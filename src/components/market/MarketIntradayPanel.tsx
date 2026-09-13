import { useMemo, useState } from "react";
import type { MarketExperimentResult } from "../../types";

const PAGE_SIZE = 40;

export function MarketIntradayPanel({ result }: { result: MarketExperimentResult }) {
  const [page, setPage] = useState(0);
  const audits = result.triggerAudits;
  const pageCount = Math.max(1, Math.ceil(audits.length / PAGE_SIZE));
  const visible = useMemo(() => audits.slice(page * PAGE_SIZE, (page + 1) * PAGE_SIZE), [audits, page]);
  const calls = audits.filter((audit) => audit.shouldEvaluate).length;
  const budgetSkips = audits.filter((audit) => audit.skipReason === "COMPUTE_BUDGET").length;
  return <section className="market-intraday">
    <div className="panel-heading"><strong>INTRADAY FOUNDATION</strong><span>{result.experiment.executionModelVersion}</span></div>
    <div className="market-intraday__summary">
      <article><span>TIMEFRAME</span><strong>{result.experiment.timeframe}</strong><small>ANNUALIZATION {result.experiment.annualizationFactor}</small></article>
      <article><span>MARKET SESSION</span><strong>{result.experiment.market}</strong><small>{result.experiment.sessionType} · {result.experiment.timezone}</small></article>
      <article><span>DECISION TRIGGERS</span><strong>{result.experiment.triggerEngineVersion ?? "LEGACY CADENCE"}</strong><small>{calls} calls · {budgetSkips} budget skips</small></article>
    </div>
    {result.sessionMetrics.length > 0 && <div className="market-metrics"><table><thead><tr><th>SESSION</th><th>AGENT</th><th>RETURN</th><th>DD</th><th>TRADES</th><th>AI CALLS</th><th>NO CALL</th></tr></thead><tbody>{result.sessionMetrics.slice(0, 100).map((metric) => <tr key={`${metric.sessionId}:${metric.agentId}`}><td>{metric.sessionId}</td><td>{metric.agentId}</td><td>{metric.returnPct.toFixed(2)}%</td><td>{metric.maxDrawdownPct.toFixed(2)}%</td><td>{metric.tradeCount}</td><td>{metric.aiCallCount}</td><td>{metric.noCallCount}</td></tr>)}</tbody></table></div>}
    {audits.length > 0 && <><div className="panel-heading"><strong>TRIGGER INSPECTOR</strong><span>{audits.length} CANDLES AUDITADOS</span></div><div className="market-metrics"><table><thead><tr><th>UTC / SESSION</th><th>AGENT</th><th>CANDLE</th><th>TRIGGER</th><th>RESULT</th></tr></thead><tbody>{visible.map((audit) => <tr key={audit.id}><td><strong>{audit.timestampUtc}</strong><small>{audit.sessionId}</small></td><td>{audit.agentId}</td><td>{audit.candleIndex}</td><td>{audit.triggerReason ?? "—"}</td><td className={audit.shouldEvaluate ? "positive" : ""}>{audit.shouldEvaluate ? `CALL ${audit.callIndex ?? ""}` : audit.skipReason}</td></tr>)}</tbody></table></div><div className="market-intraday__pager"><button className="button" disabled={page === 0} onClick={() => setPage((value) => value - 1)}>ANTERIOR</button><span>{page + 1} / {pageCount}</span><button className="button" disabled={page + 1 >= pageCount} onClick={() => setPage((value) => value + 1)}>PRÓXIMA</button></div></>}
  </section>;
}
