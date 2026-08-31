import { gapAreas } from "../../data/knowledge";
import type { KnowledgeArea } from "../../types";

interface GapDiagnosticsProps {
  onSelect?: (area: KnowledgeArea) => void;
}

const priorityLabel = {
  critical: "CRÍTICA",
  high: "ALTA",
  medium: "MÉDIA",
  low: "BAIXA",
};

export function GapDiagnostics({ onSelect }: GapDiagnosticsProps) {
  return (
    <div className="gap-list">
      {gapAreas.map((area) => (
        <button className="gap-item" key={area.id} onClick={() => onSelect?.(area)}>
          <span className={`priority priority--${area.priority}`}>{priorityLabel[area.priority]}</span>
          <strong>{area.name}</strong>
          <small>Profundidade {area.depth}% · diferença de {area.coverage - area.depth} pontos</small>
        </button>
      ))}
    </div>
  );
}
