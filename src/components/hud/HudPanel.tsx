import type { ReactNode } from "react";

interface HudPanelProps {
  title: string;
  code?: string;
  className?: string;
  children: ReactNode;
}

export function HudPanel({ title, code, className = "", children }: HudPanelProps) {
  return (
    <section className={`hud-panel ${className}`.trim()}>
      <header className="hud-panel__header">
        <h2>{title}</h2>
        {code && <span>{code}</span>}
      </header>
      <div className="hud-panel__body">{children}</div>
    </section>
  );
}
