import { useEffect, useMemo, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { MarketObservatory } from "../components/market/MarketObservatory";
import { marketLabRepository } from "../repositories/marketLabRepository";
import type { MarketAgentDefinition, MarketDataset, MarketExperimentResult, MarketExperimentSummary, MarketRiskProfile } from "../types";

const money = (value: number, currency = "BRL") => new Intl.NumberFormat("pt-BR", { style: "currency", currency }).format(value);
const pct = (value: number) => `${value >= 0 ? "+" : ""}${value.toFixed(2)}%`;
const colors = ["#21dff3", "#55f0a7", "#ffbd4a", "#b58cff", "#ff6f91"];

function EquityChart({ result }: { result: MarketExperimentResult }) {
  const series = useMemo(() => result.metrics.map((metric) => ({ metric, points: result.equity.filter((point) => point.agentId === metric.agentId) })), [result]);
  const values = series.flatMap((entry) => entry.points.map((point) => point.equity));
  const min = Math.min(...values); const max = Math.max(...values); const range = Math.max(max - min, 1);
  return <div className="market-chart"><svg viewBox="0 0 900 250" preserveAspectRatio="none" role="img" aria-label="Curvas patrimoniais dos agentes">
    {[0, 1, 2, 3, 4].map((line) => <line key={line} x1="0" x2="900" y1={line * 62.5} y2={line * 62.5} />)}
    {series.map((entry, seriesIndex) => { const denominator = Math.max(entry.points.length - 1, 1); const points = entry.points.map((point, index) => `${index / denominator * 900},${235 - (point.equity - min) / range * 220}`).join(" "); return <polyline key={entry.metric.agentId} points={points} style={{ stroke: colors[seriesIndex % colors.length] }} />; })}
  </svg><div className="market-chart__legend">{series.map((entry, index) => <span key={entry.metric.agentId}><i style={{ background: colors[index % colors.length] }} />{entry.metric.agentName}</span>)}</div></div>;
}

export function MarketLabPage() {
  const [datasets, setDatasets] = useState<MarketDataset[]>([]); const [agents, setAgents] = useState<MarketAgentDefinition[]>([]); const [profiles, setProfiles] = useState<MarketRiskProfile[]>([]); const [history, setHistory] = useState<MarketExperimentSummary[]>([]);
  const [result, setResult] = useState<MarketExperimentResult | null>(null); const [selectedAgents, setSelectedAgents] = useState<string[]>(["cash", "buy-hold", "simple-trend"]); const [tab, setTab] = useState<"overview" | "experiments" | "agents" | "observatory" | "datasets">("overview");
  const [name, setName] = useState("Experimento determinístico"); const [datasetId, setDatasetId] = useState(""); const [profileId, setProfileId] = useState(""); const [capital, setCapital] = useState(10_000); const [seed, setSeed] = useState(42); const [fee, setFee] = useState(0.1); const [slippage, setSlippage] = useState(0.05);
  const [busy, setBusy] = useState(false); const [error, setError] = useState<string | null>(null);

  const load = async () => { const [nextDatasets, nextAgents, nextProfiles, nextHistory] = await Promise.all([marketLabRepository.listDatasets(), marketLabRepository.listAgents(), marketLabRepository.listRiskProfiles(), marketLabRepository.listExperiments()]); setDatasets(nextDatasets); setAgents(nextAgents); setProfiles(nextProfiles); setHistory(nextHistory); setDatasetId((current) => current || nextDatasets[0]?.id || ""); setProfileId((current) => current || nextProfiles[0]?.id || ""); };
  useEffect(() => {
    void Promise.all([marketLabRepository.listDatasets(), marketLabRepository.listAgents(), marketLabRepository.listRiskProfiles(), marketLabRepository.listExperiments()]).then(([nextDatasets, nextAgents, nextProfiles, nextHistory]) => {
      setDatasets(nextDatasets); setAgents(nextAgents); setProfiles(nextProfiles); setHistory(nextHistory); setDatasetId(nextDatasets[0]?.id || ""); setProfileId(nextProfiles[0]?.id || "");
    }).catch((cause) => setError(String(cause)));
  }, []);

  const importCsv = async () => { const path = await open({ multiple: false, filters: [{ name: "Historical candles", extensions: ["csv"] }] }); if (!path) return; const basename = path.split(/[\\/]/).pop()?.replace(/\.csv$/i, "") || "Dataset"; const asset = window.prompt("Código do ativo", "TEST")?.trim(); if (!asset) return; const timeframe = window.prompt("Timeframe", "1D")?.trim(); if (!timeframe) return; setBusy(true); setError(null); try { const dataset = await marketLabRepository.importDataset({ path, name: basename, asset, timeframe, currency: "BRL" }); await load(); setDatasetId(dataset.id); } catch (cause) { setError(String(cause)); } finally { setBusy(false); } };
  const run = async () => { if (!datasetId || !profileId) { setError("Importe um dataset e selecione o perfil de risco."); return; } setBusy(true); setError(null); try { const next = await marketLabRepository.runExperiment({ name, datasetId, riskProfileId: profileId, agentIds: selectedAgents, initialCapital: capital, randomSeed: seed, feePct: fee, slippagePct: slippage }); setResult(next); setTab("overview"); await load(); } catch (cause) { setError(String(cause)); } finally { setBusy(false); } };
  const openExperiment = async (id: string) => { setBusy(true); setError(null); try { setResult(await marketLabRepository.getExperiment(id)); setTab("overview"); } catch (cause) { setError(String(cause)); } finally { setBusy(false); } };
  const rerun = async () => { if (!result) return; setBusy(true); setError(null); try { setResult(await marketLabRepository.rerunExperiment(result.experiment.id)); await load(); } catch (cause) { setError(String(cause)); } finally { setBusy(false); } };
  const toggleAgent = (id: string) => setSelectedAgents((current) => current.includes(id) ? current.filter((item) => item !== id) : current.length < 5 ? [...current, id] : current);

  return <section className="market-lab">
    <header className="module-header market-lab__header"><div><span className="eyebrow">LAB / MARKET LAB v0.2</span><h1>Multi-Agent Behavioral Observatory</h1><p>Coortes determinísticas, comparação comportamental e auditoria local. Sem corretora, dinheiro real ou decisões por IA.</p></div><div className="market-header-actions"><button className="button market-kill" onClick={() => void marketLabRepository.activateKillSwitch()}>KILL SWITCH</button><button className="button button--primary" onClick={() => void importCsv()} disabled={busy}>+ IMPORTAR CSV</button></div></header>
    {error && <div className="market-alert"><strong>OPERAÇÃO INTERROMPIDA</strong><span>{error}</span><button onClick={() => setError(null)}>×</button></div>}
    <div className="market-lab__grid">
      <aside className="hud-panel market-config"><div className="panel-heading"><strong>NOVO EXPERIMENTO</strong><span>MAX 5 AGENTES</span></div>
        <label>NOME<input value={name} onChange={(event) => setName(event.target.value)} /></label>
        <label>DATASET<select value={datasetId} onChange={(event) => setDatasetId(event.target.value)}><option value="">Nenhum dataset</option>{datasets.map((dataset) => <option key={dataset.id} value={dataset.id}>{dataset.asset} / {dataset.timeframe} · {dataset.candleCount}</option>)}</select></label>
        {datasets.find((item) => item.id === datasetId) && <div className="market-dataset-card"><strong>{datasets.find((item) => item.id === datasetId)?.name}</strong><span>{datasets.find((item) => item.id === datasetId)?.startAt} → {datasets.find((item) => item.id === datasetId)?.endAt}</span><small>{datasets.find((item) => item.id === datasetId)?.fingerprint}</small></div>}
        <label>RISK PROFILE<select value={profileId} onChange={(event) => setProfileId(event.target.value)}>{profiles.map((profile) => <option key={profile.id} value={profile.id}>{profile.name} · max {profile.maxPositionPct}%</option>)}</select></label>
        <div className="market-form-row"><label>CAPITAL / AGENTE<input type="number" min="1" value={capital} onChange={(event) => setCapital(Number(event.target.value))} /></label><label>SEED<input type="number" value={seed} onChange={(event) => setSeed(Number(event.target.value))} /></label></div>
        <div className="market-form-row"><label>FEE %<input type="number" step="0.01" min="0" value={fee} onChange={(event) => setFee(Number(event.target.value))} /></label><label>SLIPPAGE %<input type="number" step="0.01" min="0" value={slippage} onChange={(event) => setSlippage(Number(event.target.value))} /></label></div>
        <fieldset><legend>AGENT REGISTRY · {selectedAgents.length}/5</legend>{agents.map((agent) => <label className="market-agent-check" key={agent.id}><input type="checkbox" checked={selectedAgents.includes(agent.id)} onChange={() => toggleAgent(agent.id)} disabled={!agent.enabled || (!selectedAgents.includes(agent.id) && selectedAgents.length >= 5)} /><span><strong>{agent.name}</strong><small>{agent.strategyType} / {agent.strategyVersion}</small></span></label>)}</fieldset>
        <button className="button button--primary market-run" onClick={() => void run()} disabled={busy || selectedAgents.length < 3}>{busy ? "RUNNING..." : "▶ RUN COHORT"}</button>
        {selectedAgents.length < 3 && <small className="market-config__hint">Selecione pelo menos 3 agentes para formar uma coorte.</small>}
      </aside>
      <main className="hud-panel market-results"><div className="market-tabs market-tabs--v2"><button className={tab === "overview" ? "active" : ""} onClick={() => setTab("overview")}>OVERVIEW</button><button className={tab === "experiments" ? "active" : ""} onClick={() => setTab("experiments")}>EXPERIMENTS</button><button className={tab === "agents" ? "active" : ""} onClick={() => setTab("agents")}>AGENTS</button><button className={tab === "observatory" ? "active" : ""} onClick={() => setTab("observatory")}>OBSERVATORY</button><button className={tab === "datasets" ? "active" : ""} onClick={() => setTab("datasets")}>DATASETS</button></div>
        {!result && (tab === "overview" || tab === "observatory") && <div className="market-empty"><span>◇</span><strong>NENHUMA COORTE SELECIONADA</strong><p>Importe candles OHLCV locais, selecione de 3 a 5 agentes e execute um experimento.</p></div>}
        {result && tab === "overview" && <><div className="market-result-head"><div><span>EXPERIMENTO / {result.experiment.status.toUpperCase()}</span><h2>{result.experiment.name}</h2><p>{result.experiment.asset} · {result.experiment.timeframe} · {result.experiment.datasetName}</p></div><button className="button" onClick={() => void rerun()} disabled={busy}>CLONAR / EXECUTAR NOVAMENTE</button></div><div className="market-metrics"><table><thead><tr><th>AGENTE</th><th>EQUITY FINAL</th><th>RETORNO</th><th>DRAWDOWN</th><th>TRADES</th><th>HOLD</th><th>EXPOSIÇÃO</th></tr></thead><tbody>{result.metrics.map((metric) => <tr key={metric.agentId}><td><strong>{metric.agentName}</strong><small>{metric.status}</small></td><td>{money(metric.finalEquity, result.experiment.currency)}</td><td className={metric.totalReturnPct >= 0 ? "positive" : "negative"}>{pct(metric.totalReturnPct)}</td><td>{metric.maxDrawdownPct.toFixed(2)}%</td><td>{metric.tradeCount}</td><td>{metric.holdCount}</td><td>{metric.averageExposurePct.toFixed(1)}% / {metric.maxExposurePct.toFixed(1)}%</td></tr>)}</tbody></table></div><div className="panel-heading"><strong>EQUITY CURVE</strong><span>MARK-TO-MARKET / SEM LIQUIDAÇÃO FORÇADA</span></div><EquityChart result={result} /></>}
        {result && tab === "observatory" && <MarketObservatory result={result} />}
        {tab === "experiments" && <div className="market-history">{history.length === 0 ? <div className="market-empty"><strong>HISTÓRICO VAZIO</strong></div> : history.map((experiment) => <button key={experiment.id} onClick={() => void openExperiment(experiment.id)}><div><strong>{experiment.name}</strong><span>{experiment.datasetName} · {experiment.asset} / {experiment.timeframe}</span></div><div><strong>{experiment.status.toUpperCase()}</strong><span>{experiment.agentIds.length} agentes · seed {experiment.randomSeed}</span></div></button>)}</div>}
        {tab === "agents" && <div className="market-registry-view">{agents.map((agent) => <article key={agent.id}><div><strong>{agent.name}</strong><span>{agent.strategyType} / {agent.strategyVersion}</span></div><b>{agent.enabled ? "ENABLED" : "DISABLED"}</b><code>{agent.defaultConfigJson}</code></article>)}</div>}
        {tab === "datasets" && <div className="market-registry-view">{datasets.length === 0 ? <div className="market-empty"><strong>NENHUM DATASET</strong></div> : datasets.map((dataset) => <article key={dataset.id}><div><strong>{dataset.name}</strong><span>{dataset.asset} / {dataset.timeframe} · {dataset.candleCount} candles</span></div><b>{dataset.startAt} → {dataset.endAt}</b><code>{dataset.fingerprint}</code></article>)}</div>}
      </main>
    </div>
  </section>;
}
