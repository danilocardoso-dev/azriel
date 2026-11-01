import type { SpotifyPlayer } from "../repositories/spotifyRepository";
import { useEffect, useMemo, useState, type FormEvent, type ReactNode } from "react";
import { NowPlayingBar } from "../components/azriel/NowPlayingBar";
import { Reactor } from "../components/azriel/Reactor";
import { signedUsd, usd } from "../components/trading/tradingMath";
import { useAI } from "../contexts/useAI";
import { useDailyOperations } from "../contexts/useDailyOperations";
import { azrielStates } from "../data/system";
import { tradingRepository } from "../repositories/tradingRepository";
import { formatUptime, systemService } from "../services/systemService";
import type { AzrielState, SystemSnapshot, TradingLive, TradingOverview } from "../types";
import "../styles/nucleo.css";

interface NucleoProps {
  coreState: AzrielState;
  onOpenAI: () => void;
  onOpenDaily: () => void;
  onOpenTrading: () => void;
  onVoice: () => void;
  onOpenYouTube: () => void;
  onOpenMusic: () => void;
  azrielDeviceId: string | null;
  azrielState: SpotifyPlayer | null;
}

/** Medidor em arco: o anel inteiro é 100%. */
function Arc({ value, label }: { value: number; label: string }) {
  const radius = 24;
  const length = 2 * Math.PI * radius;
  const clamped = Math.max(0, Math.min(100, value));
  return <div className="nucleo-arc">
    <svg viewBox="-30 -30 60 60" aria-hidden="true">
      <circle className="nucleo-arc__track" r={radius} />
      <circle className="nucleo-arc__value" r={radius} strokeDasharray={`${(clamped / 100) * length} ${length}`} transform="rotate(-90)" />
    </svg>
    <strong>{Math.round(clamped)}<small>%</small></strong>
    <span>{label}</span>
  </div>;
}

/** Nó de dados em órbita: sem caixa, só o rótulo com a linha de ligação e o conteúdo. */
function Node({ position, code, title, onOpen, children }: { position: "tl" | "tr" | "bl" | "br"; code: string; title: string; onOpen?: () => void; children: ReactNode }) {
  return <section className={`nucleo-node nucleo-node--${position}`}>
    <header>
      <i aria-hidden="true" />
      <span>{code}</span>
      {onOpen ? <button type="button" onClick={onOpen}>{title} →</button> : <b>{title}</b>}
    </header>
    <div className="nucleo-node__body">{children}</div>
  </section>;
}

export function NucleoPage({ coreState, onOpenAI, onOpenDaily, onOpenTrading, onVoice, onOpenYouTube, onOpenMusic, azrielDeviceId, azrielState }: NucleoProps) {
  const { counters, tasks } = useDailyOperations();
  const { send, engine, settings, remoteSettings, engineAvailable } = useAI();
  const [snapshot, setSnapshot] = useState<SystemSnapshot | null>(null);
  const [trading, setTrading] = useState<{ overview: TradingOverview | null; live: TradingLive | null }>({ overview: null, live: null });
  const [command, setCommand] = useState("");

  useEffect(() => {
    let active = true;
    const load = () => {
      void systemService.snapshot().then((next) => { if (active) setSnapshot(next); }).catch(() => undefined);
      void Promise.all([tradingRepository.overview().catch(() => null), tradingRepository.live().catch(() => null)])
        .then(([overview, live]) => { if (active) setTrading({ overview, live }); });
    };
    const first = window.setTimeout(load, 0);
    const timer = window.setInterval(load, 15_000);
    return () => { active = false; window.clearTimeout(first); window.clearInterval(timer); };
  }, []);

  const memory = snapshot && snapshot.memory.totalBytes > 0 ? (snapshot.memory.usedBytes / snapshot.memory.totalBytes) * 100 : 0;
  const nextTasks = useMemo(() => tasks.filter((task) => !["completed", "cancelled"].includes(task.status)).slice(0, 3), [tasks]);
  const equity = trading.live?.equity ?? trading.overview?.equity.at(-1)?.equity ?? null;
  const dayStart = trading.overview?.dayStartEquity ?? null;
  const position = trading.live?.position ?? null;
  const state = azrielStates[coreState];
  const model = engine === "gemini" ? remoteSettings?.model : settings?.model;

  async function submit(event: FormEvent) {
    event.preventDefault();
    const query = command.trim();
    if (!query) return;
    setCommand("");
    onOpenAI();
    try { await send(query); } catch { /* o AI Core mostra a falha */ }
  }

  return <section className="nucleo" data-state={coreState}>
    <div className="nucleo__orbit">
      <svg className="nucleo__links" viewBox="0 0 1000 600" preserveAspectRatio="none" aria-hidden="true">
        <path d="M 250 150 L 330 150 L 400 230" />
        <path d="M 750 150 L 670 150 L 600 230" />
        <path d="M 250 450 L 330 450 L 400 370" />
        <path d="M 750 450 L 670 450 L 600 370" />
      </svg>

      <button type="button" className="nucleo__core" onClick={onVoice} title="Falar com o Azriel (Ctrl+Espaço)">
        <Reactor className="nucleo__reactor" />
        <span className="nucleo__identity"><strong>AZRIEL</strong><small>{state.label}</small></span>
      </button>

      <Node position="tl" code="SYS" title="SISTEMA">
        <div className="nucleo-arcs">
          <Arc value={snapshot?.cpu.usagePercent ?? 0} label="CPU" />
          <Arc value={memory} label="RAM" />
        </div>
        <p>{snapshot ? `Ligado há ${formatUptime(snapshot.details.uptimeSeconds)}` : "Lendo telemetria..."}</p>
      </Node>

      <Node position="tr" code="OPS" title="AGENDA" onOpen={onOpenDaily}>
        <div className="nucleo-figure"><strong>{counters.today}</strong><span>para hoje · {counters.pending} pendentes{counters.overdue ? ` · ${counters.overdue} atrasadas` : ""}</span></div>
        <ul>{nextTasks.length ? nextTasks.map((task) => <li key={task.id} data-priority={task.priority}>{task.title}</li>) : <li className="nucleo-muted">Nada pendente.</li>}</ul>
      </Node>

      <Node position="bl" code="IA" title="INTELIGÊNCIA" onOpen={onOpenAI}>
        <div className="nucleo-figure"><strong className="nucleo-figure__word">{engine === "gemini" ? "GEMINI" : "LOCAL"}</strong><span>{model || "sem modelo"}</span></div>
        <p data-tone={engineAvailable ? "up" : "down"}>{engineAvailable ? "● Motor pronto" : "● Motor indisponível"}</p>
      </Node>

      <Node position="br" code="TRD" title="TRADING" onOpen={onOpenTrading}>
        <div className="nucleo-figure"><strong>{equity != null ? usd(equity).replace(" USDT", "") : "—"}</strong><span>USDT · conta demo</span></div>
        <p data-tone={equity != null && dayStart ? (equity >= dayStart ? "up" : "down") : undefined}>
          {equity != null && dayStart ? `Hoje ${signedUsd(equity - dayStart)}` : "Sem registro hoje"}
          {position ? ` · ${position.side === "LONG" ? "COMPRADO" : "VENDIDO"} ${signedUsd(position.unrealized)}` : trading.overview ? ` · ${trading.overview.engine.running ? "agente ligado" : "agente desligado"}` : ""}
        </p>
      </Node>
    </div>

    <p className="nucleo__message">{state.message}</p>

    <NowPlayingBar onOpenYouTube={onOpenYouTube} onOpenMusic={onOpenMusic} azrielDeviceId={azrielDeviceId} azrielState={azrielState} />

    <form className="nucleo-command" onSubmit={submit}>
      <span aria-hidden="true">›</span>
      <input value={command} onChange={(event) => setCommand(event.target.value)} placeholder="Fale ou digite um comando para o Azriel" aria-label="Comando para o Azriel" />
      <button type="button" className="nucleo-command__mic" onClick={onVoice} aria-label="Falar com o Azriel" title="Falar (Ctrl+Espaço)"><i /></button>
    </form>
  </section>;
}
