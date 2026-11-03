import { useState } from "react";
import { tradingRepository } from "../../repositories/tradingRepository";
import type { TradeExitReason, TradingOverview } from "../../types";
import { price, shortTime, signedUsd, trend } from "./tradingMath";

const exitLabels: Record<TradeExitReason, string> = {
  STOP: "stop", TARGET: "alvo", LOCK: "trava de lucro", TIMEOUT: "tempo máximo", AGENT: "agente", MANUAL: "manual", DAILY_LIMIT: "limite diário", EXCHANGE: "corretora",
};

/** Autoajuste: diário de trades, revisões do Gemini (diagnóstico, lições, parâmetros) e o desfazer. */
export function TradingTuning({ overview, busy, run }: { overview: TradingOverview; busy: string | null; run: (label: string, action: () => Promise<unknown>) => Promise<void> }) {
  const [open, setOpen] = useState(false);
  const { trades, reviews, lessons, autoTune, tradesUntilReview, engine } = overview;
  const closed = trades.filter((trade) => trade.pnlUsdt !== null);
  const wins = closed.filter((trade) => (trade.pnlUsdt ?? 0) > 0).length;
  const total = closed.reduce((sum, trade) => sum + (trade.pnlUsdt ?? 0), 0);
  const latest = reviews[0];
  const status = engine.reviewInProgress ? "revisando agora..."
    : autoTune ? `próxima revisão em ${tradesUntilReview} ${tradesUntilReview === 1 ? "trade fechado" : "trades fechados"}`
    : "autoajuste desligado";

  return <section className="trading-history trading-tuning">
    <button type="button" className="trading-history__toggle" onClick={() => setOpen((value) => !value)} aria-expanded={open}>
      {open ? "▾" : "▸"} AUTOAJUSTE <small>{closed.length} trades · {wins} ganhos · {signedUsd(total)} USDT · {status}</small>
    </button>
    {open && <div className="trading-tuning__body">
      <div className="trading-tuning__controls">
        <label className="trading-tuning__switch">
          <input type="checkbox" checked={autoTune} disabled={busy !== null} onChange={(event) => void run("tune", () => tradingRepository.setAutoTune(event.target.checked))} />
          <span>Revisar sozinho a cada 5 trades fechados</span>
        </label>
        <button type="button" disabled={busy !== null || engine.reviewInProgress || closed.length === 0} onClick={() => void run("review", tradingRepository.review)}>
          {busy === "review" || engine.reviewInProgress ? "REVISANDO..." : "REVISAR AGORA"}
        </button>
      </div>
      <p className="trading-tuning__note">O Gemini estuda os trades fechados e propõe lições e ajustes. Contra a tendência de 1h, só com reversão clara (confiança 85%+). Sem preço esticado nem mercado parado. O sistema só aceita stop entre 0,15% e 1%, alvo de pelo menos 1,5× o stop, confiança entre 55% e 90%, velas de 5m ou 15m e passos pequenos por revisão. Com menos de 5 trades, ele só escreve lições. Margem, alavancagem e perda diária nunca mudam.</p>
      {engine.lastReviewError && <p className="trading-tuning__error">Última revisão falhou: {engine.lastReviewError}</p>}

      {latest ? <article className="trading-review" data-reverted={latest.reverted}>
        <header>
          <b>REVISÃO · {shortTime(latest.createdAt)}</b>
          <small>{latest.tradesCount} trades · {Math.round(latest.winRate)}% de acerto · {signedUsd(latest.pnlUsdt)} USDT</small>
          {latest.reverted ? <em>DESFEITA</em> : <button type="button" disabled={busy !== null} onClick={() => void run("revert", tradingRepository.revertReview)}>DESFAZER</button>}
        </header>
        <p>{latest.diagnosis}</p>
        {latest.changes.length > 0
          ? <ul className="trading-review__changes">{latest.changes.map((change) => <li key={change.field}>{change.label}: {change.from} → <b>{change.to}</b></li>)}</ul>
          : <p className="trading-review__muted">Sem mudança de parâmetros.</p>}
      </article> : <p className="trading-review__muted">Nenhuma revisão ainda. A primeira acontece depois de 5 trades fechados, ou quando você pedir.</p>}

      {lessons.length > 0 && <div className="trading-lessons">
        <span>LIÇÕES EM VIGOR</span>
        <ol>{lessons.map((lesson) => <li key={lesson}><span>{lesson}</span><button type="button" aria-label="Apagar lição" title="Apagar lição" disabled={busy !== null} onClick={() => void run("lesson", () => tradingRepository.removeLesson(lesson))}>×</button></li>)}</ol>
      </div>}

      {reviews.length > 1 && <details className="trading-review__older">
        <summary>{reviews.length - 1} revisões anteriores</summary>
        {reviews.slice(1).map((review) => <article key={review.id} className="trading-review" data-reverted={review.reverted}>
          <header><b>{shortTime(review.createdAt)}</b><small>{review.tradesCount} trades · {Math.round(review.winRate)}% · {signedUsd(review.pnlUsdt)}</small>{review.reverted && <em>DESFEITA</em>}</header>
          <p>{review.diagnosis}</p>
          {review.changes.length > 0 && <ul className="trading-review__changes">{review.changes.map((change) => <li key={change.field}>{change.label}: {change.from} → <b>{change.to}</b></li>)}</ul>}
        </article>)}
      </details>}

      {trades.length > 0 && <table className="trading-trades">
        <thead><tr><th>Abertura</th><th>Lado</th><th>Entrada</th><th>Saída</th><th>Motivo</th><th>Resultado</th></tr></thead>
        <tbody>{trades.map((trade) => <tr key={trade.id} title={trade.entryReason}>
          <td>{shortTime(trade.openedAt)}</td>
          <td>{trade.side === "LONG" ? "COMPRA" : "VENDA"}</td>
          <td>{price(trade.entryPrice)}</td>
          <td>{trade.exitPrice !== null ? price(trade.exitPrice) : "—"}</td>
          <td>{trade.exitReason ? exitLabels[trade.exitReason] : "aberto"}</td>
          <td data-tone={trade.pnlUsdt !== null ? trend(trade.pnlUsdt) : undefined}>{trade.pnlUsdt !== null ? signedUsd(trade.pnlUsdt) : "—"}</td>
        </tr>)}</tbody>
      </table>}
    </div>}
  </section>;
}
