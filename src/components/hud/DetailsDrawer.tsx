import type { ReactNode } from "react";

interface DetailsDrawerProps {
  eyebrow: string;
  title: string;
  onClose: () => void;
  children: ReactNode;
}

export function DetailsDrawer({ eyebrow, title, onClose, children }: DetailsDrawerProps) {
  return (
    <aside className="details-drawer" aria-label={`Detalhes: ${title}`}>
      <div className="details-drawer__scan" />
      <header>
        <div>
          <span className="eyebrow">{eyebrow}</span>
          <h2>{title}</h2>
        </div>
        <button className="icon-button" onClick={onClose} aria-label="Fechar painel">×</button>
      </header>
      <div className="details-drawer__content">{children}</div>
    </aside>
  );
}
