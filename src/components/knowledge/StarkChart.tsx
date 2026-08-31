import type { KnowledgeArea } from "../../types";

interface StarkChartProps {
  areas: KnowledgeArea[];
  selectedId?: string;
  onSelect: (area: KnowledgeArea) => void;
}

export function StarkChart({ areas, selectedId, onSelect }: StarkChartProps) {
  return (
    <div className="stark-chart">
      <div className="stark-chart__legend">
        <span><i className="legend-dot legend-dot--coverage" />Cobertura</span>
        <span><i className="legend-dot legend-dot--depth" />Profundidade</span>
      </div>
      <div className="stark-chart__rows">
        {areas.map((area) => (
          <button
            className={`stark-row ${selectedId === area.id ? "stark-row--active" : ""}`}
            key={area.id}
            onClick={() => onSelect(area)}
            title={`Abrir detalhes de ${area.name}`}
          >
            <span className="stark-row__label">{area.name}</span>
            <span className="stark-row__bar"><i className="coverage" style={{ width: `${area.coverage}%` }} /></span>
            <strong>{area.coverage}%</strong>
            <span className="stark-row__bar"><i className="depth" style={{ width: `${area.depth}%` }} /></span>
            <strong>{area.depth}%</strong>
          </button>
        ))}
      </div>
    </div>
  );
}
