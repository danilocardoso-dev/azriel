import { useEffect, useMemo, useRef, useState, type PointerEvent } from "react";
import type { LivePosition, MarketCandle, TradingDecision } from "../../types";
import { intervalMs, niceTicks, price, priceDigits, sqliteDate } from "./tradingMath";

const HEIGHT = 420;
const AXIS = 76;
const BOTTOM = 24;
const TOP = 10;
const SLOT = 8;

interface Marker { index: number; kind: "long" | "short" | "close"; label: string }

/** Velas do par com o preço atual e, havendo posição, as linhas de entrada, stop e alvo. */
export function CandleChart({ candles, interval, position, decisions }: { candles: MarketCandle[]; interval: string; position: LivePosition | null; decisions: TradingDecision[] }) {
  const boxRef = useRef<HTMLDivElement>(null);
  const [width, setWidth] = useState(900);
  const [hover, setHover] = useState<{ index: number; y: number } | null>(null);

  useEffect(() => {
    const box = boxRef.current;
    if (!box) return;
    const observer = new ResizeObserver(([entry]) => setWidth(Math.max(320, Math.floor(entry.contentRect.width))));
    observer.observe(box);
    return () => observer.disconnect();
  }, []);

  const view = useMemo(() => {
    const plotWidth = width - AXIS;
    const visible = candles.slice(-Math.max(20, Math.floor(plotWidth / SLOT)));
    if (!visible.length) return null;
    const levels = position ? [position.entryPrice, position.stopLoss, position.takeProfit].filter((value): value is number => value != null) : [];
    let min = Math.min(...visible.map((c) => c.low), ...levels);
    let max = Math.max(...visible.map((c) => c.high), ...levels);
    if (max - min < 1e-12) { min *= 0.999; max *= 1.001; }
    const pad = (max - min) * 0.06;
    min -= pad; max += pad;
    const step = plotWidth / visible.length;
    const x = (index: number) => index * step + step / 2;
    const y = (value: number) => TOP + (1 - (value - min) / (max - min)) * (HEIGHT - TOP - BOTTOM);
    const valueAt = (pixel: number) => min + (1 - (pixel - TOP) / (HEIGHT - TOP - BOTTOM)) * (max - min);
    const span = intervalMs(interval);
    const markers: Marker[] = decisions
      .filter((d) => d.orderStatus === "FILLED" && ["OPEN_LONG", "OPEN_SHORT", "CLOSE"].includes(d.finalAction))
      .flatMap((d) => {
        const at = sqliteDate(d.createdAt).getTime();
        const index = visible.findIndex((c) => at >= c.time && at < c.time + span);
        if (index < 0) return [];
        const kind: Marker["kind"] = d.finalAction === "OPEN_LONG" ? "long" : d.finalAction === "OPEN_SHORT" ? "short" : "close";
        return [{ index, kind, label: `${kind === "long" ? "Compra" : kind === "short" ? "Venda" : "Fechamento"} · ${d.reason}` }];
      });
    return { visible, step, x, y, valueAt, ticks: niceTicks(min, max), digits: priceDigits(visible.at(-1)!.close), markers, plotWidth };
  }, [candles, decisions, interval, position, width]);

  if (!view) return <div className="candle-chart candle-chart--empty" ref={boxRef}><p>Carregando velas...</p></div>;

  const { visible, step, x, y, valueAt, ticks, digits, markers, plotWidth } = view;
  const last = visible.at(-1)!;
  const bodyWidth = Math.max(1, Math.min(14, step * 0.68));
  const timeLabel = (time: number) => new Date(time).toLocaleString("pt-BR", interval === "1d" ? { day: "2-digit", month: "2-digit" } : { hour: "2-digit", minute: "2-digit" });
  const timeEvery = Math.max(1, Math.ceil(90 / step));
  const active = hover ? visible[hover.index] : null;
  // Rótulos do eixo que encostariam nas etiquetas de preço atual/posição ficam escondidos.
  const tagged = [last.close, ...(position ? [position.entryPrice, position.stopLoss, position.takeProfit] : [])].filter((value): value is number => value != null).map(y);
  const freeTick = (tick: number) => tagged.every((tagY) => Math.abs(tagY - y(tick)) > 16);

  function move(event: PointerEvent<SVGRectElement>) {
    const box = event.currentTarget.getBoundingClientRect();
    const px = event.clientX - box.left;
    const py = event.clientY - box.top;
    setHover({ index: Math.min(visible.length - 1, Math.max(0, Math.floor(px / step))), y: py });
  }

  const level = (value: number | null | undefined, kind: string, label: string) => value == null ? null : <g className={`candle-chart__level candle-chart__level--${kind}`} key={kind}>
    <line x1={0} x2={plotWidth} y1={y(value)} y2={y(value)} />
    <text className="candle-chart__level-name" x={8} y={y(value) - 5}>{label}</text>
    <rect x={plotWidth + 2} y={y(value) - 9} width={AXIS - 4} height={18} rx={2} />
    <text x={plotWidth + AXIS / 2} y={y(value) + 4}>{price(value, digits)}</text>
  </g>;

  return <div className="candle-chart" ref={boxRef}>
    <svg width={width} height={HEIGHT} role="img" aria-label={`Gráfico de velas, último preço ${price(last.close, digits)}`}>
      {ticks.map((tick) => <g key={tick} className="candle-chart__tick">
        <line x1={0} x2={plotWidth} y1={y(tick)} y2={y(tick)} />
        {freeTick(tick) && <text x={plotWidth + 8} y={y(tick) + 4}>{price(tick, digits)}</text>}
      </g>)}
      {visible.map((candle, index) => index % timeEvery === 0 && x(index) > 24 && x(index) < plotWidth - 24 && <text key={`t${candle.time}`} className="candle-chart__time" x={x(index)} y={HEIGHT - 7}>{timeLabel(candle.time)}</text>)}
      {visible.map((candle, index) => {
        const up = candle.close >= candle.open;
        const top = y(Math.max(candle.open, candle.close));
        const bottom = y(Math.min(candle.open, candle.close));
        return <g key={candle.time} className={up ? "candle candle--up" : "candle candle--down"}>
          <line x1={x(index)} x2={x(index)} y1={y(candle.high)} y2={y(candle.low)} />
          <rect x={x(index) - bodyWidth / 2} y={top} width={bodyWidth} height={Math.max(1, bottom - top)} />
        </g>;
      })}
      {markers.map((marker) => {
        const candle = visible[marker.index];
        const cx = x(marker.index);
        const cy = marker.kind === "short" ? y(candle.high) - 12 : y(candle.low) + 12;
        return <g key={`${marker.index}-${marker.kind}`} className={`candle-chart__marker candle-chart__marker--${marker.kind}`}>
          <title>{marker.label}</title>
          {marker.kind === "close"
            ? <circle cx={cx} cy={y(candle.high) - 12} r={5} />
            : <path d={marker.kind === "long" ? `M${cx},${cy - 6} l6,10 h-12 z` : `M${cx},${cy + 6} l6,-10 h-12 z`} />}
        </g>;
      })}
      {position && level(position.takeProfit, "target", "ALVO")}
      {position && level(position.stopLoss, "stop", "STOP")}
      {position && level(position.entryPrice, "entry", position.side === "LONG" ? "COMPRA" : "VENDA")}
      <g className={`candle-chart__last candle-chart__last--${last.close >= last.open ? "up" : "down"}`}>
        <line x1={0} x2={plotWidth} y1={y(last.close)} y2={y(last.close)} />
        <rect x={plotWidth + 2} y={y(last.close) - 10} width={AXIS - 4} height={20} rx={2} />
        <text x={plotWidth + AXIS / 2} y={y(last.close) + 4}>{price(last.close, digits)}</text>
      </g>
      {hover && <g className="candle-chart__cross">
        <line x1={x(hover.index)} x2={x(hover.index)} y1={TOP} y2={HEIGHT - BOTTOM} />
        {hover.y > TOP && hover.y < HEIGHT - BOTTOM && <>
          <line x1={0} x2={plotWidth} y1={hover.y} y2={hover.y} />
          <rect x={plotWidth + 2} y={hover.y - 9} width={AXIS - 4} height={18} rx={2} />
          <text x={plotWidth + AXIS / 2} y={hover.y + 4}>{price(valueAt(hover.y), digits)}</text>
        </>}
      </g>}
      <rect className="candle-chart__hit" x={0} y={0} width={plotWidth} height={HEIGHT} onPointerMove={move} onPointerLeave={() => setHover(null)} />
    </svg>
    {active && <div className="candle-chart__ohlc">
      <time>{new Date(active.time).toLocaleString("pt-BR", { day: "2-digit", month: "2-digit", hour: "2-digit", minute: "2-digit" })}</time>
      <span>A <b>{price(active.open, digits)}</b></span>
      <span>M <b>{price(active.high, digits)}</b></span>
      <span>m <b>{price(active.low, digits)}</b></span>
      <span>F <b>{price(active.close, digits)}</b></span>
      <span data-tone={active.close >= active.open ? "up" : "down"}>{active.close >= active.open ? "▲" : "▼"} {(((active.close - active.open) / active.open) * 100).toLocaleString("pt-BR", { maximumFractionDigits: 2 })}%</span>
    </div>}
  </div>;
}
