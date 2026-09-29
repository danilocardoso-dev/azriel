import type { StudyRoadmap } from "../../../types";

type Props = {
  roadmaps: StudyRoadmap[];
  selectedId: string | null;
  onSelect: (roadmap: StudyRoadmap) => void;
};

export function RoadmapSelector({ roadmaps, selectedId, onSelect }: Props) {
  return <label className="roadmap-selector">
    <span>ROADMAP</span>
    <div>
      <select value={selectedId ?? ""} onChange={(event) => {
        const roadmap = roadmaps.find((item) => item.id === event.target.value);
        if (roadmap) onSelect(roadmap);
      }}>
        {roadmaps.map((roadmap) => <option key={roadmap.id} value={roadmap.id}>{roadmap.name} · {roadmap.progress}%</option>)}
      </select>
      <small>{roadmaps.length} roadmap(s) disponível(is)</small>
    </div>
  </label>;
}
