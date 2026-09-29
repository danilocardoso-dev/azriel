import type { RoadmapStage, RoadmapTopic, StudyRoadmap } from "../../../types";
import { stageActivities } from "../../../services/roadmapExperience";

function topicStatus(topic: RoadmapTopic) {
  if (topic.activities.length > 0 && topic.activities.every((activity) => activity.status === "completed")) return "CONCLUÍDO";
  if (topic.activities.some((activity) => activity.status === "in_progress")) return "EM ANDAMENTO";
  return "NÃO INICIADO";
}

type Props = {
  roadmap: StudyRoadmap;
  expandedStages: Set<string>;
  selectedTopicId: string | null;
  currentActivityId: string | null;
  onToggleStage: (id: string) => void;
  onSelectTopic: (stage: RoadmapStage, topic: RoadmapTopic) => void;
  onContinue: () => void;
};

export function RoadmapStructure({ roadmap, expandedStages, selectedTopicId, currentActivityId, onToggleStage, onSelectTopic, onContinue }: Props) {
  return <main className="roadmap-structure">
    <header className="roadmap-structure__hero">
      <div><span className="roadmap-kicker">ROADMAP SELECIONADO</span><h2>{roadmap.name}</h2><p>{roadmap.description || "Sem descrição cadastrada."}</p></div>
      <div className="roadmap-total-progress"><strong>{roadmap.progress}%</strong><span>PROGRESSO</span><div><i style={{ width: `${roadmap.progress}%` }} /></div><small>{roadmap.completedActivities} / {roadmap.totalActivities} ATIVIDADES</small></div>
    </header>
    <div className="roadmap-structure__actions"><button className="primary" onClick={onContinue} disabled={!roadmap.totalActivities}>▶ CONTINUAR ESTUDO</button><span>{roadmap.stages.length} ETAPAS · {roadmap.stages.reduce((sum, stage) => sum + stage.topics.length, 0)} TÓPICOS</span></div>
    <div className="roadmap-stage-list">{roadmap.stages.map((stage) => {
      const activities = stageActivities(stage);
      const completed = activities.filter((item) => item.status === "completed").length;
      const expanded = expandedStages.has(stage.id);
      const stageCompleted = activities.length > 0 && completed === activities.length;
      return <section className={`roadmap-stage ${stageCompleted ? "completed" : ""}`} key={stage.id}>
        <button className="roadmap-stage__header" aria-expanded={expanded} onClick={() => onToggleStage(stage.id)}><b>{stageCompleted ? "✓" : String(stage.order).padStart(2, "0")}</b><span><strong>{stage.name}</strong><small>{stageCompleted ? "ETAPA CONCLUÍDA" : stage.description || `${stage.topics.length} tópicos desta etapa`}</small></span><small>{completed}/{activities.length}</small><i>{expanded ? "⌃" : "⌄"}</i></button>
        {expanded && <div className="roadmap-topics">{stage.topics.map((topic) => {
          const done = topic.activities.filter((item) => item.status === "completed").length;
          const isCurrent = topic.activities.some((item) => item.id === currentActivityId);
          return <button key={topic.id} className={`${selectedTopicId === topic.id ? "selected" : ""} ${isCurrent ? "current" : ""}`} onClick={() => onSelectTopic(stage, topic)}><span className="topic-state-dot" data-state={topicStatus(topic)}>{topicStatus(topic) === "CONCLUÍDO" ? "✓" : ""}</span><b>{stage.order}.{topic.order}</b><span><strong>{topic.name}</strong><small>{topicStatus(topic)}</small></span><small>{done}/{topic.activities.length}</small></button>;
        })}{!stage.topics.length && <p className="roadmap-empty-compact">Etapa sem tópicos.</p>}</div>}
      </section>;
    })}{!roadmap.stages.length && <p className="roadmap-empty-compact">Este roadmap ainda não possui etapas.</p>}</div>
  </main>;
}
