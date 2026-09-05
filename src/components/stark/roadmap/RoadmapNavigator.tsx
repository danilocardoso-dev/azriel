import type { StudyRoadmap } from "../../../types";
import { filterRoadmaps, type RoadmapFilter } from "../../../services/roadmapExperience";

const statusLabel = { planned: "PLANEJADO", active: "EM ANDAMENTO", paused: "PAUSADO", completed: "CONCLUÍDO" };

type Props = {
  roadmaps: StudyRoadmap[];
  selectedId: string | null;
  query: string;
  filter: RoadmapFilter;
  collapsed: boolean;
  onQuery: (value: string) => void;
  onFilter: (value: RoadmapFilter) => void;
  onSelect: (roadmap: StudyRoadmap) => void;
  onToggleCollapsed: () => void;
};

export function RoadmapNavigator({ roadmaps, selectedId, query, filter, collapsed, onQuery, onFilter, onSelect, onToggleCollapsed }: Props) {
  const visible = filterRoadmaps(roadmaps, query, filter);
  const filters: Array<{ id: RoadmapFilter; label: string }> = [{ id: "all", label: "TODOS" }, { id: "active", label: "ATIVOS" }, { id: "paused", label: "PAUSADOS" }, { id: "completed", label: "CONCLUÍDOS" }];
  return <aside className={`roadmap-navigator ${collapsed ? "collapsed" : ""}`} aria-label="Seus roadmaps">
    <header>{!collapsed && <><strong>SEUS ROADMAPS</strong><span>{roadmaps.length} REGISTROS</span></>}<button className="roadmap-navigator__toggle" type="button" onClick={onToggleCollapsed} aria-label={collapsed ? "Expandir lista de roadmaps" : "Recolher lista de roadmaps"} title={collapsed ? "Expandir lista de roadmaps" : "Recolher lista de roadmaps"}>{collapsed ? "»" : "«"}</button></header>
    {!collapsed && <><label className="roadmap-search"><span>BUSCAR ROADMAP</span><input value={query} onChange={(event) => onQuery(event.target.value)} placeholder="Nome ou descrição..." /></label>
      <div className="roadmap-filter" role="group" aria-label="Filtrar roadmaps">{filters.map((item) => <button key={item.id} className={filter === item.id ? "active" : ""} onClick={() => onFilter(item.id)}>{item.label} <i>{item.id === "all" ? roadmaps.length : roadmaps.filter((roadmap) => roadmap.status === item.id).length}</i></button>)}</div>
      <div className="roadmap-navigator__list">{visible.map((roadmap) => <button key={roadmap.id} className={roadmap.id === selectedId ? "selected" : ""} onClick={() => onSelect(roadmap)}>
        <span className="roadmap-nav-icon">◇</span><span><strong>{roadmap.name}</strong><small>{statusLabel[roadmap.status]}</small><span className="roadmap-mini-progress"><i style={{ width: `${roadmap.progress}%` }} /></span></span><b>{roadmap.progress}%</b>
      </button>)}{!visible.length && <p className="roadmap-empty-compact">Nenhum roadmap corresponde ao filtro.</p>}</div></>}
  </aside>;
}
