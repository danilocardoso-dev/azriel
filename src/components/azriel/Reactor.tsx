import { useEffect, useRef, type ReactNode } from "react";
import "../../styles/reactor.css";

const BARS = 72;
export type Levels = () => { az: number; you: number };

/**
 * O núcleo do Azriel: anéis girando, um anel de barras e o centro que reage ao áudio.
 * `levels` é lido a cada quadro (sem re-render); sem ele, o núcleo apenas respira.
 */
export function Reactor({ levels, className = "", children }: { levels?: Levels; className?: string; children?: ReactNode }) {
  const rootRef = useRef<HTMLDivElement>(null);
  const barsRef = useRef<SVGGElement>(null);

  useEffect(() => {
    let frame = 0;
    let smoothAz = 0;
    let smoothYou = 0;
    const tick = (time: number) => {
      const current = levels?.() ?? { az: 0.08 + 0.06 * Math.sin(time / 900), you: 0 };
      smoothAz += (current.az - smoothAz) * 0.25;
      smoothYou += (current.you - smoothYou) * 0.25;
      rootRef.current?.style.setProperty("--az", smoothAz.toFixed(3));
      rootRef.current?.style.setProperty("--you", smoothYou.toFixed(3));
      const bars = barsRef.current?.children;
      if (bars) {
        const energy = Math.max(smoothAz, smoothYou);
        for (let i = 0; i < bars.length; i += 1) {
          const wave = 0.5 + 0.5 * Math.sin(time / 180 + i * 0.55) * Math.cos(time / 410 - i * 0.21);
          (bars[i] as SVGLineElement).setAttribute("y2", String(-(138 + energy * 54 * wave + 3 * wave)));
        }
      }
      frame = requestAnimationFrame(tick);
    };
    frame = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(frame);
  }, [levels]);

  return <div className={`reactor ${className}`.trim()} ref={rootRef}>
    <svg className="reactor__svg" viewBox="-200 -200 400 400" aria-hidden="true">
      <circle className="reactor__halo" r={190} />
      <g className="reactor__ring reactor__ring--outer"><circle r={176} /></g>
      <g className="reactor__ring reactor__ring--ticks">{Array.from({ length: 60 }, (_, i) => <line key={i} y1={-158} y2={i % 5 ? -152 : -146} transform={`rotate(${i * 6})`} />)}</g>
      <g className="reactor__bars" ref={barsRef}>{Array.from({ length: BARS }, (_, i) => <line key={i} y1={-132} y2={-138} transform={`rotate(${(i * 360) / BARS})`} />)}</g>
      <g className="reactor__ring reactor__ring--segments"><circle r={118} /></g>
      <g className="reactor__ring reactor__ring--inner"><circle r={96} /></g>
      <circle className="reactor__core-glow" r={70} />
      <circle className="reactor__core" r={46} />
    </svg>
    {children}
  </div>;
}
