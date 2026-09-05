import type { RoadmapStage, RoadmapTopic, StudyRoadmap } from "../../../types";
import { activityProgress, roadmapImpact, stageActivities, stageProgress } from "../../../services/roadmapExperience";
import type { KnowledgeEvent } from "../../../types";

const statusLabel = { planned: "PLANEJADO", active: "EM ANDAMENTO", paused: "PAUSADO", completed: "CONCLUÍDO" };
const stateLabel = { NOT_STARTED: "NÃO INICIADO", EXPOSED: "EXPOSTO", UNDERSTOOD: "COMPREENDIDO", PRACTICED: "PRATICADO", APPLIED: "APLICADO", MASTERED: "DOMINADO" };

type Props = {
  roadmap: StudyRoadmap;
  events: KnowledgeEvent[];
  expandedStages: Set<string>;
  selectedTopicId: string | null;
  currentActivityId: string | null;
  onToggleStage: (id: string) => void;
  onSelectTopic: (stage: RoadmapStage, topic: RoadmapTopic) => void;
  onContinue: () => void;
  onEdit: () => void;
  onDelete: () => void;
};

export function RoadmapStructure({ roadmap, events, expandedStages, selectedTopicId, currentActivityId, onToggleStage, onSelectTopic, onContinue, onEdit, onDelete }: Props) {
  const impact = roadmapImpact(events, roadmap.id);
  return <main className="roadmap-structure">
    <header className="roadmap-structure__hero"><div><span className="roadmap-kicker">ROADMAP SELECIONADO</span><h2>{roadmap.name}</h2><p>{roadmap.description || "Sem descrição cadastrada."}</p><div className="roadmap-meta"><span>{roadmap.stages.length} ETAPAS</span><span>{roadmap.stages.reduce((sum, stage) => sum + stage.topics.length, 0)} TÓPICOS</span><span>{roadmap.totalActivities} ATIVIDADES</span><span>{roadmap.totalActivities > 0 && roadmap.progress === 100 ? "ROADMAP CONCLUÍDO" : statusLabel[roadmap.status]}</span></div></div><div className="roadmap-total-progress"><strong>{roadmap.progress}%</strong><span>PROGRESSO</span><div><i style={{ width: `${roadmap.progress}%` }} /></div><small>{roadmap.completedActivities} / {roadmap.totalActivities} CONCLUÍDAS</small></div></header>
    <div className="roadmap-structure__actions"><button className="primary" onClick={onContinue} disabled={!roadmap.totalActivities}>▶ CONTINUAR ESTUDO</button><button onClick={onEdit}>EDITAR</button><button className="danger-link" onClick={onDelete}>EXCLUIR</button><span>IMPACTO REAL · C {impact.coverage} · P {impact.depth} · I {impact.integration}</span></div>
    <div className="roadmap-stage-list">{roadmap.stages.map((stage) => {
      const activities = stageActivities(stage); const completed = activities.filter((item) => item.status === "completed").length; const expanded = expandedStages.has(stage.id);
      const stageCompleted = activities.length > 0 && completed === activities.length;
      return <section className={`roadmap-stage ${stageCompleted ? "completed" : ""}`} key={stage.id}>
        <button className="roadmap-stage__header" aria-expanded={expanded} onClick={() => onToggleStage(stage.id)}><b>{stageCompleted ? "✓" : String(stage.order).padStart(2, "0")}</b><span><strong>{stage.name}</strong><small>{stageCompleted ? "ETAPA CONCLUÍDA" : stage.description || `${stage.topics.length} tópicos desta etapa`}</small></span><span className="roadmap-stage__progress"><i style={{ width: `${stageProgress(stage)}%` }} /></span><em>{stageProgress(stage)}%</em><small>{completed}/{activities.length}</small><i>{expanded ? "⌃" : "⌄"}</i></button>
        {expanded && <div className="roadmap-topics">{stage.topics.map((topic) => {
          const done = topic.activities.filter((item) => item.status === "completed").length;
          const isCurrent = topic.activities.some((item) => item.id === currentActivityId);
          return <button key={topic.id} className={`${selectedTopicId === topic.id ? "selected" : ""} ${isCurrent ? "current" : ""}`} onClick={() => onSelectTopic(stage, topic)}><span className="topic-state-dot" data-state={topic.state}>{topic.state === "MASTERED" ? "✓" : ""}</span><b>{stage.order}.{topic.order}</b><span><strong>{topic.name}</strong><small>{stateLabel[topic.state]}</small></span><span className="roadmap-topic-progress"><i style={{ width: `${activityProgress(topic.activities)}%` }} /></span><em>{activityProgress(topic.activities)}%</em><small>{done}/{topic.activities.length}</small></button>;
        })}{!stage.topics.length && <p className="roadmap-empty-compact">Etapa sem tópicos.</p>}</div>}
      </section>;
    })}{!roadmap.stages.length && <p className="roadmap-empty-compact">Este roadmap ainda não possui etapas.</p>}</div>
  </main>;
}
