import { useCallback, useEffect, useState, type FormEvent } from "react";
import { listen } from "@tauri-apps/api/event";
import { ModuleIntro } from "../components/layout/ModuleIntro";
import { EquityChart } from "../components/trading/EquityChart";
import { actionLabels, change, shortTime, signedPct, signedUsd, trend, usd } from "../components/trading/tradingMath";
import { TRADING_UPDATE_EVENT, tradingRepository } from "../repositories/tradingRepository";
import type { TradingLive, TradingOverview, TradingSettingsInput } from "../types";
import "../styles/trading.css";

const messageOf = (error: unknown) => error instanceof Error ? error.message : String(error);
const INTERVALS = ["5m", "15m", "30m", "1h", "2h", "4h"];

export function TradingPage() {
  const [overview, setOverview] = useState<TradingOverview | null>(null);
  const [live, setLive] = useState<TradingLive | null>(null);
  const [liveError, setLiveError] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState<string | null>(null);
  const [configOpen, setConfigOpen] = useState(false);

  const refresh = useCallback(async () => {
    try { setOverview(await tradingRepository.overview()); }
    catch (reason) { setError(messageOf(reason)); }
  }, []);

  const refreshLive = useCallback(async () => {
    try { setLive(await tradingRepository.live()); setLiveError(null); }
    catch (reason) { setLive(null); setLiveError(messageOf(reason)); }
  }, []);

  useEffect(() => {
    const initial = window.setTimeout(() => void refresh().then(() => refreshLive()), 0);
    const overviewTimer = window.setInterval(() => void refresh(), 30_000);
    const liveTimer = window.setInterval(() => void refreshLive(), 60_000);
    let unlisten: (() => void) | undefined;
    void listen(TRADING_UPDATE_EVENT, () => { void refresh(); void refreshLive(); }).then((stop) => { unlisten = stop; }).catch(() => undefined);
    return () => { window.clearTimeout(initial); window.clearInterval(overviewTimer); window.clearInterval(liveTimer); unlisten?.(); };
  }, [refresh, refreshLive]);

  async function run(label: string, action: () => Promise<void>) {
    setBusy(label); setError(null);
    try { await action(); await refresh(); await refreshLive(); }
    catch (reason) { setError(messageOf(reason)); }
    finally { setBusy(null); }
  }

  if (!overview) return <section className="module-page trading-page"><ModuleIntro code="TRD-01" title="Trading" description="Agente autônomo na conta demo da BingX." />{error ? <div className="trading-alert"><strong>FALHA</strong><p>{error}</p></div> : <p className="trading-muted">Carregando...</p>}</section>;

  const { settings, engine, decisions, equity, firstEquity, dayStartEquity } = overview;
  const currentEquity = live?.equity ?? equity.at(-1)?.equity ?? null;
  const today = change(currentEquity, dayStartEquity);
  const total = change(currentEquity, firstEquity);
  const entries = decisions.filter((decision) => ["OPEN_LONG", "OPEN_SHORT"].includes(decision.finalAction) && decision.orderStatus === "FILLED").length;
  const needsSetup = !settings.apiKeyConfigured;

  return <section className="module-page trading-page">
    <ModuleIntro code="TRD-01" title="Trading" description="Agente autônomo operando perpétuos na conta DEMO da BingX. A IA propõe, os limites de risco decidem o que pode ser executado." metric={`${engine.running ? "RODANDO" : "PARADO"} // ${settings.symbol}`} />
    <div className="trading-demo-banner"><strong>CONTA DEMO (VST)</strong><span>Dinheiro virtual. O endpoint de conta real não existe neste código.</span></div>

    <div className="trading-controls">
      <div className="trading-controls__state" data-state={engine.running ? "online" : "offline"}>
        <i />
        <div>
          <strong>{engine.cycleInProgress ? "CICLO EM ANDAMENTO" : engine.running ? "MOTOR LIGADO" : "MOTOR DESLIGADO"}</strong>
          <small>{engine.running && engine.nextCycleAt ? `Próximo ciclo ${new Date(engine.nextCycleAt).toLocaleTimeString("pt-BR", { hour: "2-digit", minute: "2-digit" })}` : "Ligue para operar a cada ciclo"} · {settings.engine === "gemini" ? "Gemini" : "Ollama local"} · a cada {settings.cycleMinutes} min</small>
        </div>
      </div>
      <div className="trading-controls__actions">
        {engine.running
          ? <button type="button" disabled={busy !== null} onClick={() => void run("stop", tradingRepository.stop)}>{busy === "stop" ? "DESLIGANDO..." : "DESLIGAR MOTOR"}</button>
          : <button type="button" className="primary" disabled={busy !== null || needsSetup} onClick={() => void run("start", tradingRepository.start)}>{busy === "start" ? "LIGANDO..." : "LIGAR MOTOR"}</button>}
        <button type="button" disabled={busy !== null || needsSetup || engine.cycleInProgress} onClick={() => void run("cycle", tradingRepository.runCycleNow)}>{busy === "cycle" ? "ANALISANDO..." : "RODAR CICLO AGORA"}</button>
        <button type="button" className="danger" disabled={busy !== null || !live?.position} onClick={() => void run("close", tradingRepository.closePosition)}>{busy === "close" ? "FECHANDO..." : "FECHAR POSIÇÃO"}</button>
        <button type="button" onClick={() => setConfigOpen((open) => !open)}>{configOpen || needsSetup ? "OCULTAR CONFIGURAÇÃO" : "CONFIGURAÇÃO"}</button>
      </div>
    </div>
    {error && <div className="trading-alert"><strong>FALHA</strong><p>{error}</p></div>}
    {engine.lastError && !error && <div className="trading-alert"><strong>ÚLTIMO CICLO FALHOU</strong><p>{engine.lastError}</p></div>}

    {(configOpen || needsSetup) && <TradingConfig key={settings.updatedAt} overview={overview} onSaved={async () => { await refresh(); await refreshLive(); }} />}

    <div className="trading-kpis">
      <Kpi label="PATRIMÔNIO" value={currentEquity != null ? usd(currentEquity) : "—"} detail={liveError && !needsSetup ? "Sem conexão ao vivo" : live ? `Margem livre ${usd(live.availableMargin)}` : "Aguardando conexão"} />
      <Kpi label="RESULTADO HOJE" value={today ? `${signedUsd(today.value)} USDT` : "—"} detail={today ? signedPct(today.pct) : "Sem registro hoje"} tone={today ? trend(today.value) : undefined} />
      <Kpi label="RESULTADO TOTAL" value={total ? `${signedUsd(total.value)} USDT` : "—"} detail={total ? `${signedPct(total.pct)} desde o 1º ciclo` : "Sem histórico"} tone={total ? trend(total.value) : undefined} />
      <Kpi label="POSIÇÃO" value={live?.position ? `${live.position.side === "LONG" ? "COMPRADO" : "VENDIDO"} ${live.position.leverage}x` : "SEM POSIÇÃO"} detail={live?.position ? `${live.position.quantity} @ ${live.position.entryPrice.toLocaleString("pt-BR")} · ${signedUsd(live.position.unrealized)} USDT` : `${entries} entradas registradas`} tone={live?.position ? trend(live.position.unrealized) : undefined} />
    </div>

    <section className="trading-panel">
      <header><span>CURVA DE PATRIMÔNIO</span><small>{equity.length} registros · um por ciclo</small></header>
      <EquityChart points={equity} baseline={firstEquity} />
    </section>

    <section className="trading-panel">
      <header><span>DECISÕES DO AGENTE</span><small>proposta da IA → decisão final após os limites de risco</small></header>
      {decisions.length === 0
        ? <p className="trading-muted">Nenhum ciclo ainda. Configure as chaves e clique em RODAR CICLO AGORA para ver a primeira decisão.</p>
        : <ol className="trading-decisions">{decisions.map((decision) => <li key={decision.id} data-action={decision.finalAction} data-order={decision.orderStatus}>
          <header>
            <time>{shortTime(decision.createdAt)}</time>
            <b>{actionLabels[decision.finalAction] ?? decision.finalAction}</b>
            {decision.proposedAction !== decision.finalAction && decision.proposedAction !== "MANUAL_CLOSE" && <small>IA propôs {actionLabels[decision.proposedAction] ?? decision.proposedAction}</small>}
            {decision.finalAction !== "ERROR" && decision.engine !== "manual" && <small>confiança {decision.confidence}%</small>}
            {decision.price > 0 && <small>preço {decision.price.toLocaleString("pt-BR")}</small>}
            {decision.orderStatus !== "NONE" && <em data-status={decision.orderStatus}>{decision.orderStatus === "FILLED" ? "✓ EXECUTADA" : "✕ ORDEM FALHOU"}</em>}
          </header>
          <p>{decision.reason}</p>
          {decision.riskNote && <p className="trading-decisions__risk">RISCO: {decision.riskNote}</p>}
          {decision.orderDetail && <p className="trading-decisions__order">{decision.orderDetail}</p>}
        </li>)}</ol>}
    </section>
  </section>;
}

function Kpi({ label, value, detail, tone }: { label: string; value: string; detail: string; tone?: "up" | "down" | "flat" }) {
  return <article className="trading-kpi" data-tone={tone}><span>{label}</span><strong>{tone === "up" ? "▲ " : tone === "down" ? "▼ " : ""}{value}</strong><small>{detail}</small></article>;
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
    <header><span>CONFIGURAÇÃO DO AGENTE</span><i data-state={settings.apiKeyConfigured ? "online" : "offline"}>{settings.apiKeyConfigured ? (engine.running ? "MUDANÇAS VALEM NO PRÓXIMO CICLO" : "CHAVES CONFIGURADAS") : "CHAVES NECESSÁRIAS"}</i></header>
    <form onSubmit={(event: FormEvent) => { event.preventDefault(); if (apiKey.trim() && secretKey.trim()) void save(() => tradingRepository.updateKeys(apiKey, secretKey).then(() => { setApiKey(""); setSecretKey(""); }), "Chaves salvas. Teste com RODAR CICLO AGORA."); }}>
      <label><span><strong>API Key (demo)</strong><small>{settings.apiKeyConfigured ? "Configurada. Preencha só para substituir." : "Crie na BingX só com permissão de trade, sem saque."}</small></span><input type="password" autoComplete="off" spellCheck={false} value={apiKey} onChange={(event) => setApiKey(event.target.value)} placeholder={settings.apiKeyConfigured ? "••••••••" : "API Key"} /></label>
      <label><span><strong>Secret Key</strong><small>Fica só no banco local, nunca é exibida de volta.</small></span><input type="password" autoComplete="off" spellCheck={false} value={secretKey} onChange={(event) => setSecretKey(event.target.value)} placeholder={settings.apiKeyConfigured ? "••••••••" : "Secret Key"} /></label>
      <div className="ai-settings__actions"><button disabled={busy || !apiKey.trim() || !secretKey.trim()}>SALVAR CHAVES</button></div>
    </form>
    <form onSubmit={(event: FormEvent) => { event.preventDefault(); void save(() => tradingRepository.updateSettings({ ...form, symbol: form.symbol.trim().toUpperCase() }), "Parâmetros salvos."); }}>
      <label><span><strong>Par</strong><small>Perpétuo USDT-M</small></span><input value={form.symbol} onChange={(event) => setForm({ ...form, symbol: event.target.value.toUpperCase() })} /></label>
      <label><span><strong>Candle</strong><small>Tempo gráfico analisado</small></span><select value={form.candleInterval} onChange={(event) => setForm({ ...form, candleInterval: event.target.value })}>{INTERVALS.map((interval) => <option key={interval}>{interval}</option>)}</select></label>
      <label><span><strong>Ciclo (min)</strong><small>Frequência das decisões</small></span><input type="number" min={5} max={240} value={form.cycleMinutes} onChange={number("cycleMinutes")} /></label>
      <label><span><strong>Motor da IA</strong><small>Usa o modelo escolhido no AI Core</small></span><select value={form.engine} onChange={(event) => setForm({ ...form, engine: event.target.value as TradingSettingsInput["engine"] })}><option value="gemini">Gemini</option><option value="ollama">Ollama local</option></select></label>
      <label><span><strong>Margem por operação</strong><small>USDT · exposição {exposure.toLocaleString("pt-BR")} USDT</small></span><input type="number" min={1} step="any" value={form.marginUsdt} onChange={number("marginUsdt")} /></label>
      <label><span><strong>Alavancagem</strong><small>1x a 20x</small></span><input type="number" min={1} max={20} value={form.leverage} onChange={number("leverage")} /></label>
      <label><span><strong>Stop loss %</strong><small>Do preço de entrada</small></span><input type="number" min={0.1} max={10} step={0.1} value={form.stopLossPct} onChange={number("stopLossPct")} /></label>
      <label><span><strong>Take profit %</strong><small>Do preço de entrada</small></span><input type="number" min={0.1} max={30} step={0.1} value={form.takeProfitPct} onChange={number("takeProfitPct")} /></label>
      <label><span><strong>Perda diária máx. %</strong><small>Fecha tudo e desliga o motor</small></span><input type="number" min={0.5} max={50} step={0.5} value={form.maxDailyLossPct} onChange={number("maxDailyLossPct")} /></label>
      <label><span><strong>Confiança mínima</strong><small>Para abrir posição</small></span><input type="number" min={0} max={100} value={form.minConfidence} onChange={number("minConfidence")} /></label>
      <div className="ai-settings__actions"><button disabled={busy}>SALVAR PARÂMETROS</button><small>Risco por operação ≈ {(exposure * form.stopLossPct / 100).toLocaleString("pt-BR", { maximumFractionDigits: 2 })} USDT no stop (sem contar taxas).</small></div>
    </form>
    {message && <p className="trading-config__message" data-kind={message.kind}>{message.text}</p>}
  </section>;
}
