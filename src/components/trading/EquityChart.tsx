import { useMemo, useState, type PointerEvent } from "react";
import type { EquityPoint } from "../../types";
import { shortTime, signedUsd, usd } from "./tradingMath";

const WIDTH = 800;
const HEIGHT = 220;
const PAD = { top: 12, right: 12, bottom: 12, left: 12 };

/** Curva de patrimônio: uma série, referência tracejada no patrimônio inicial, crosshair com tooltip. */
export function EquityChart({ points, baseline }: { points: EquityPoint[]; baseline: number | null }) {
  const [hover, setHover] = useState<number | null>(null);
  const geometry = useMemo(() => {
    if (points.length < 2) return null;
    const values = points.map((point) => point.equity);
    if (baseline != null) values.push(baseline);
    let min = Math.min(...values);
    let max = Math.max(...values);
    if (max - min < 1e-9) { min -= 1; max += 1; }
    const span = max - min;
    min -= span * 0.08; max += span * 0.08;
    const x = (index: number) => PAD.left + (index / (points.length - 1)) * (WIDTH - PAD.left - PAD.right);
    const y = (value: number) => PAD.top + (1 - (value - min) / (max - min)) * (HEIGHT - PAD.top - PAD.bottom);
    const path = points.map((point, index) => `${index ? "L" : "M"}${x(index).toFixed(1)},${y(point.equity).toFixed(1)}`).join(" ");
    return { x, y, path };
  }, [points, baseline]);

  if (!geometry) return <div className="equity-chart equity-chart--empty"><p>A curva aparece depois de dois ciclos do motor.</p></div>;

  function move(event: PointerEvent<SVGRectElement>) {
    const box = event.currentTarget.getBoundingClientRect();
    const fraction = Math.min(1, Math.max(0, (event.clientX - box.left) / box.width));
    setHover(Math.round(fraction * (points.length - 1)));
  }

  const active = hover != null ? points[hover] : null;
  const first = points[0];
  const last = points[points.length - 1];
  const left = (index: number) => `${(geometry.x(index) / WIDTH) * 100}%`;
  return <figure className="equity-chart">
    <div className="equity-chart__plot">
      <svg viewBox={`0 0 ${WIDTH} ${HEIGHT}`} preserveAspectRatio="none" role="img" aria-label={`Patrimônio de ${usd(first.equity)} para ${usd(last.equity)}`}>
        {[0.25, 0.5, 0.75].map((step) => { const y = PAD.top + step * (HEIGHT - PAD.top - PAD.bottom); return <line key={step} className="equity-chart__grid" x1={PAD.left} x2={WIDTH - PAD.right} y1={y} y2={y} />; })}
        {baseline != null && <line className="equity-chart__baseline" x1={PAD.left} x2={WIDTH - PAD.right} y1={geometry.y(baseline)} y2={geometry.y(baseline)} />}
        <path className="equity-chart__line" d={geometry.path} />
        {hover != null && <line className="equity-chart__cross" x1={geometry.x(hover)} x2={geometry.x(hover)} y1={PAD.top} y2={HEIGHT - PAD.bottom} />}
        <rect className="equity-chart__hit" x={0} y={0} width={WIDTH} height={HEIGHT} onPointerMove={move} onPointerLeave={() => setHover(null)} />
      </svg>
      {/* Ponto em HTML: o SVG estica com preserveAspectRatio="none" e deformaria um círculo. */}
      {active && hover != null && <>
        <i className="equity-chart__dot" style={{ left: left(hover), top: `${(geometry.y(active.equity) / HEIGHT) * 100}%` }} />
        <div className="equity-chart__tooltip" data-side={hover > points.length / 2 ? "left" : "right"} style={{ left: left(hover) }}>
          <time>{shortTime(active.createdAt)}</time>
          <strong>{usd(active.equity)}</strong>
          {baseline != null && <small>{signedUsd(active.equity - baseline)} USDT desde o início</small>}
        </div>
      </>}
    </div>
    <figcaption><span>{shortTime(first.createdAt)}</span><span>{baseline != null ? `- - - patrimônio inicial ${usd(baseline)}` : ""}</span><span>{shortTime(last.createdAt)}</span></figcaption>
  </figure>;
}
