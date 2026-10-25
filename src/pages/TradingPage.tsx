import { useCallback, useEffect, useState, type FormEvent } from "react";
import { listen } from "@tauri-apps/api/event";
import { CandleChart } from "../components/trading/CandleChart";
import { EquityChart } from "../components/trading/EquityChart";
import { actionLabels, change, price, shortTime, signedPct, signedUsd, trend, usd } from "../components/trading/tradingMath";
import { TRADING_UPDATE_EVENT, tradingRepository } from "../repositories/tradingRepository";
import type { MarketSnapshot, TradingLive, TradingOverview, TradingSettingsInput } from "../types";
import "../styles/trading.css";

const messageOf = (error: unknown) => error instanceof Error ? error.message : String(error);
const TIMEFRAMES = ["1m", "3m", "5m", "15m", "30m", "1h", "4h", "1d"];
const AGENT_INTERVALS = ["5m", "15m", "30m", "1h", "2h", "4h"];
// Frequência de atualização do gráfico: velas curtas mudam mais rápido.
const refreshMs = (interval: string) => ["1m", "3m"].includes(interval) ? 4_000 : interval.endsWith("m") ? 10_000 : 20_000;

export function TradingPage() {
  const [overview, setOverview] = useState<TradingOverview | null>(null);
  const [live, setLive] = useState<TradingLive | null>(null);
  const [market, setMarket] = useState<MarketSnapshot | null>(null);
  const [symbols, setSymbols] = useState<string[]>([]);
  const [timeframe, setTimeframe] = useState<string | null>(null);
  const [symbolDraft, setSymbolDraft] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState<string | null>(null);
  const [configOpen, setConfigOpen] = useState(false);
  const [historyOpen, setHistoryOpen] = useState(false);

  const symbol = overview?.settings.symbol ?? null;
  const interval = timeframe ?? overview?.settings.candleInterval ?? "15m";

  const refresh = useCallback(async () => {
    try { setOverview(await tradingRepository.overview()); }
    catch (reason) { setError(messageOf(reason)); }
  }, []);
  const refreshLive = useCallback(async () => {
    try { setLive(await tradingRepository.live()); } catch { setLive(null); }
  }, []);

  useEffect(() => {
    const initial = window.setTimeout(() => {
      void refresh().then(() => refreshLive());
      void tradingRepository.symbols().then(setSymbols).catch(() => undefined);
    }, 0);
    const overviewTimer = window.setInterval(() => void refresh(), 30_000);
    const liveTimer = window.setInterval(() => void refreshLive(), 20_000);
    let unlisten: (() => void) | undefined;
    void listen(TRADING_UPDATE_EVENT, () => { void refresh(); void refreshLive(); }).then((stop) => { unlisten = stop; }).catch(() => undefined);
    return () => { window.clearTimeout(initial); window.clearInterval(overviewTimer); window.clearInterval(liveTimer); unlisten?.(); };
  }, [refresh, refreshLive]);

  useEffect(() => {
    if (!symbol) return;
    let active = true;
    const load = () => {
      if (document.hidden) return;
      void tradingRepository.market(symbol, interval).then((next) => { if (active) setMarket(next); }).catch((reason) => { if (active) setError(messageOf(reason)); });
    };
    const first = window.setTimeout(load, 0);
    const timer = window.setInterval(load, refreshMs(interval));
    return () => { active = false; window.clearTimeout(first); window.clearInterval(timer); };
  }, [symbol, interval]);

  async function run(label: string, action: () => Promise<unknown>) {
    setBusy(label); setError(null);
    try { await action(); await refresh(); await refreshLive(); }
    catch (reason) { setError(messageOf(reason)); }
    finally { setBusy(null); }
  }

  function changeSymbol(event: FormEvent) {
    event.preventDefault();
    const next = symbolDraft.trim().toUpperCase().replace(/\/|USDT$/g, "").replace(/-$/, "");
    if (!next) return;
    const target = next.includes("-") ? next : `${next}-USDT`;
    if (symbols.length && !symbols.includes(target)) { setError(`${target} não está disponível nos perpétuos da BingX.`); return; }
    setSymbolDraft("");
    setMarket(null);
    void run("symbol", () => tradingRepository.setSymbol(target));
  }

  if (!overview) return <section className="module-page trading-page">{error ? <div className="trading-alert"><strong>FALHA</strong><p>{error}</p></div> : <p className="trading-muted">Carregando...</p>}</section>;

  const { settings, engine, decisions, equity, firstEquity, dayStartEquity } = overview;
  const marketReady = market?.symbol === settings.symbol && market.interval === interval ? market : null;
  const currentEquity = live?.equity ?? equity.at(-1)?.equity ?? null;
  const today = change(currentEquity, dayStartEquity);
  const position = live?.position ?? null;
  const lastDecision = decisions[0];
  const needsSetup = !settings.apiKeyConfigured;

  return <section className="module-page trading-page">
    <header className="trading-bar">
      <div className="trading-bar__market">
        <form onSubmit={changeSymbol} className="trading-symbol">
          <input list="trading-symbols" value={symbolDraft} onChange={(event) => setSymbolDraft(event.target.value)} placeholder={settings.symbol} aria-label="Trocar par" disabled={busy === "symbol"} spellCheck={false} />
          <datalist id="trading-symbols">{symbols.map((item) => <option key={item} value={item} />)}</datalist>
        </form>
        <span className="trading-chip">DEMO</span>
        {marketReady && <>
          <strong className="trading-bar__price">{price(marketReady.lastPrice)}</strong>
          <span className="trading-bar__change" data-tone={trend(marketReady.changePct)}>{marketReady.changePct >= 0 ? "▲" : "▼"} {signedPct(marketReady.changePct)} <small>24h</small></span>
        </>}
      </div>
      <nav className="trading-timeframes" aria-label="Tempo gráfico">
        {TIMEFRAMES.map((item) => <button type="button" key={item} className={item === interval ? "active" : ""} onClick={() => setTimeframe(item)} title={item === settings.candleInterval ? "Tempo gráfico do agente" : undefined}>{item}{item === settings.candleInterval && <i />}</button>)}
      </nav>
      <div className="trading-bar__engine">
        <button type="button" className="trading-switch" data-on={engine.running} disabled={busy !== null || needsSetup} onClick={() => void run("engine", engine.running ? tradingRepository.stop : tradingRepository.start)}>
          <span /><b>{engine.cycleInProgress ? "ANALISANDO" : engine.running ? "AGENTE LIGADO" : "AGENTE DESLIGADO"}</b>
        </button>
        <button type="button" className="trading-icon" aria-label="Configuração" title="Configuração" onClick={() => setConfigOpen((open) => !open)} data-active={configOpen || needsSetup}>⚙</button>
      </div>
    </header>

    {error && <p className="trading-error" role="alert">{error} <button type="button" onClick={() => setError(null)}>×</button></p>}
    {(configOpen || needsSetup) && <TradingConfig key={settings.updatedAt} overview={overview} onSaved={async () => { await refresh(); await refreshLive(); }} />}

    {marketReady ? <CandleChart candles={marketReady.candles} interval={interval} position={position} decisions={decisions} /> : <div className="candle-chart candle-chart--empty"><p>Carregando velas de {settings.symbol} · {interval}...</p></div>}

    <div className="trading-strip">
      <div className="trading-stat"><span>PATRIMÔNIO</span><strong>{currentEquity != null ? usd(currentEquity) : "—"}</strong></div>
      <div className="trading-stat" data-tone={today ? trend(today.value) : undefined}><span>HOJE</span><strong>{today ? `${signedUsd(today.value)} · ${signedPct(today.pct)}` : "—"}</strong></div>
      <div className="trading-stat trading-stat--position" data-tone={position ? trend(position.unrealized) : undefined}>
        <span>POSIÇÃO</span>
        <strong>{position ? `${position.side === "LONG" ? "COMPRADO" : "VENDIDO"} ${position.leverage}x · ${signedUsd(position.unrealized)} USDT` : "Sem posição"}</strong>
        {position && <button type="button" disabled={busy !== null} onClick={() => void run("close", tradingRepository.closePosition)}>{busy === "close" ? "FECHANDO..." : "FECHAR"}</button>}
      </div>
      <div className="trading-stat trading-stat--agent">
        <span>AGENTE {lastDecision ? `· ${shortTime(lastDecision.createdAt)}` : ""}</span>
        <strong title={lastDecision?.reason}>{lastDecision ? `${actionLabels[lastDecision.finalAction] ?? lastDecision.finalAction} — ${lastDecision.reason}` : "Nenhuma decisão ainda"}</strong>
        <button type="button" disabled={busy !== null || needsSetup || engine.cycleInProgress} onClick={() => void run("cycle", tradingRepository.runCycleNow)}>{busy === "cycle" ? "ANALISANDO..." : "ANALISAR AGORA"}</button>
      </div>
    </div>

    <section className="trading-history">
      <button type="button" className="trading-history__toggle" onClick={() => setHistoryOpen((open) => !open)} aria-expanded={historyOpen}>
        {historyOpen ? "▾" : "▸"} HISTÓRICO <small>{decisions.length} decisões · {engine.running && engine.nextCycleAt ? `próxima análise ${new Date(engine.nextCycleAt).toLocaleTimeString("pt-BR", { hour: "2-digit", minute: "2-digit" })}` : "agente desligado"}</small>
      </button>
      {historyOpen && <>
        <EquityChart points={equity} baseline={firstEquity} />
        <ol className="trading-decisions">{decisions.map((decision) => <li key={decision.id} data-action={decision.finalAction} data-order={decision.orderStatus}>
          <header>
            <time>{shortTime(decision.createdAt)}</time>
            <b>{actionLabels[decision.finalAction] ?? decision.finalAction}</b>
            {decision.finalAction !== "ERROR" && decision.engine !== "manual" && <small>{decision.confidence}%</small>}
            {decision.orderStatus !== "NONE" && <em data-status={decision.orderStatus}>{decision.orderStatus === "FILLED" ? "✓ EXECUTADA" : "✕ FALHOU"}</em>}
          </header>
          <p>{decision.reason}</p>
          {decision.riskNote && <p className="trading-decisions__risk">{decision.riskNote}</p>}
          {decision.orderDetail && <p className="trading-decisions__order">{decision.orderDetail}</p>}
        </li>)}</ol>
      </>}
    </section>
  </section>;
}

function TradingConfig({ overview, onSaved }: { overview: TradingOverview; onSaved: () => Promise<void> }) {
  const { settings, engine } = overview;
  const [form, setForm] = useState<TradingSettingsInput>({
    symbol: settings.symbol, candleInterval: settings.candleInterval, cycleMinutes: settings.cycleMinutes, marginUsdt: settings.marginUsdt,
    leverage: settings.leverage, stopLossPct: settings.stopLossPct, takeProfitPct: settings.takeProfitPct, maxDailyLossPct: settings.maxDailyLossPct,
    minConfidence: settings.minConfidence, engine: settings.engine,
  });
  const [apiKey, setApiKey] = useState("");
  const [secretKey, setSecretKey] = useState("");
  const [message, setMessage] = useState<{ kind: "success" | "error"; text: string } | null>(null);
  const [busy, setBusy] = useState(false);

  async function save(action: () => Promise<unknown>, success: string) {
    setBusy(true); setMessage(null);
    try { await action(); await onSaved(); setMessage({ kind: "success", text: success }); }
    catch (reason) { setMessage({ kind: "error", text: messageOf(reason) }); }
    finally { setBusy(false); }
  }

  const number = (key: keyof TradingSettingsInput) => (event: { target: { value: string } }) => setForm({ ...form, [key]: Number(event.target.value) });
  const exposure = form.marginUsdt * form.leverage;
  return <section className="ai-settings trading-config">
    <header><span>CONFIGURAÇÃO DO AGENTE</span><i data-state={settings.apiKeyConfigured ? "online" : "offline"}>{settings.apiKeyConfigured ? (engine.running ? "MUDANÇAS VALEM NA PRÓXIMA ANÁLISE" : "CHAVES CONFIGURADAS") : "CHAVES NECESSÁRIAS"}</i></header>
    <form onSubmit={(event: FormEvent) => { event.preventDefault(); if (apiKey.trim() && secretKey.trim()) void save(() => tradingRepository.updateKeys(apiKey, secretKey).then(() => { setApiKey(""); setSecretKey(""); }), "Chaves salvas."); }}>
      <label><span><strong>API Key (demo)</strong><small>{settings.apiKeyConfigured ? "Configurada. Preencha só para substituir." : "Só permissão de trade, sem saque."}</small></span><input type="password" autoComplete="off" spellCheck={false} value={apiKey} onChange={(event) => setApiKey(event.target.value)} placeholder={settings.apiKeyConfigured ? "••••••••" : "API Key"} /></label>
      <label><span><strong>Secret Key</strong><small>Fica só no banco local.</small></span><input type="password" autoComplete="off" spellCheck={false} value={secretKey} onChange={(event) => setSecretKey(event.target.value)} placeholder={settings.apiKeyConfigured ? "••••••••" : "Secret Key"} /></label>
      <div className="ai-settings__actions"><button disabled={busy || !apiKey.trim() || !secretKey.trim()}>SALVAR CHAVES</button></div>
    </form>
    <form onSubmit={(event: FormEvent) => { event.preventDefault(); void save(() => tradingRepository.updateSettings({ ...form, symbol: settings.symbol }), "Parâmetros salvos."); }}>
      <label><span><strong>Tempo gráfico do agente</strong><small>As velas que a IA analisa</small></span><select value={form.candleInterval} onChange={(event) => setForm({ ...form, candleInterval: event.target.value })}>{AGENT_INTERVALS.map((item) => <option key={item}>{item}</option>)}</select></label>
      <label><span><strong>Análise a cada (min)</strong><small>Frequência das decisões</small></span><input type="number" min={5} max={240} value={form.cycleMinutes} onChange={number("cycleMinutes")} /></label>
      <label><span><strong>Margem por operação</strong><small>USDT · exposição {exposure.toLocaleString("pt-BR")} USDT</small></span><input type="number" min={1} step="any" value={form.marginUsdt} onChange={number("marginUsdt")} /></label>
      <label><span><strong>Alavancagem</strong><small>1x a 20x</small></span><input type="number" min={1} max={20} value={form.leverage} onChange={number("leverage")} /></label>
      <label><span><strong>Stop loss %</strong><small>Do preço de entrada</small></span><input type="number" min={0.1} max={10} step={0.1} value={form.stopLossPct} onChange={number("stopLossPct")} /></label>
      <label><span><strong>Take profit %</strong><small>Do preço de entrada</small></span><input type="number" min={0.1} max={30} step={0.1} value={form.takeProfitPct} onChange={number("takeProfitPct")} /></label>
      <label><span><strong>Perda diária máx. %</strong><small>Fecha tudo e desliga o agente</small></span><input type="number" min={0.5} max={50} step={0.5} value={form.maxDailyLossPct} onChange={number("maxDailyLossPct")} /></label>
      <label><span><strong>Confiança mínima</strong><small>Para abrir posição</small></span><input type="number" min={0} max={100} value={form.minConfidence} onChange={number("minConfidence")} /></label>
      <label><span><strong>Motor da IA</strong><small>Usa o modelo escolhido no AI Core</small></span><select value={form.engine} onChange={(event) => setForm({ ...form, engine: event.target.value as TradingSettingsInput["engine"] })}><option value="gemini">Gemini</option><option value="ollama">Ollama local</option></select></label>
      <div className="ai-settings__actions"><button disabled={busy}>SALVAR PARÂMETROS</button><small>Risco por operação ≈ {(exposure * form.stopLossPct / 100).toLocaleString("pt-BR", { maximumFractionDigits: 2 })} USDT no stop, sem taxas.</small></div>
    </form>
    {message && <p className="trading-config__message" data-kind={message.kind}>{message.text}</p>}
  </section>;
}
