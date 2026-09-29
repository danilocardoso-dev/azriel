import type { RoadmapActivity, RoadmapStage, RoadmapTopic } from "../../../types";

const activityStatus = { pending: "PENDENTE", in_progress: "EM ANDAMENTO", completed: "CONCLUÍDA" };

type Props = {
  stage: RoadmapStage | null;
  topic: RoadmapTopic | null;
  selectedActivityId: string | null;
  activeSessionActivityId: string | null;
  onSelect: (activity: RoadmapActivity) => void;
  onStartStudy: (activity: RoadmapActivity) => void;
};

export function ActivityPreview({ stage, topic, selectedActivityId, activeSessionActivityId, onSelect, onStartStudy }: Props) {
  if (!stage || !topic) return <aside className="activity-preview activity-preview--empty"><span>ATIVIDADE</span><h2>Selecione um tópico</h2><p>Escolha onde estudar para visualizar a próxima atividade.</p></aside>;
  const selected = topic.activities.find((activity) => activity.id === selectedActivityId) ?? topic.activities[0] ?? null;
  return <aside className="activity-preview">
    <header><span>{stage.order}.{topic.order} · {topic.name}</span><small>{topic.activities.length} ATIVIDADE(S)</small></header>
    {topic.activities.length > 1 && <nav className="activity-preview__list" aria-label="Atividades do tópico">{topic.activities.map((activity) => <button key={activity.id} className={activity.id === selected?.id ? "selected" : ""} onClick={() => onSelect(activity)}><span>{activity.title}</span><small>{activityStatus[activity.status]}</small></button>)}</nav>}
    {selected ? <section className="activity-preview__content">
      <span>{selected.activityType}{selected.isValidation ? " · CHECKPOINT" : ""}</span>
      <h2>{selected.title}</h2>
      <p>{selected.learningObjective || selected.description || "Atividade pronta para iniciar."}</p>
      <div className="activity-preview__meta">
        {selected.estimatedMinutes && <span>~{selected.estimatedMinutes} MIN</span>}
        {(selected.resources?.length ?? 0) > 0 && <span>{selected.resources?.length} RECURSO(S)</span>}
        {selected.deliverable && <span>1 ENTREGA</span>}
        <span>{activityStatus[selected.status]}</span>
      </div>
      <button className="primary" onClick={() => onStartStudy(selected)}>{activeSessionActivityId === selected.id ? "RETOMAR ESTUDO" : "INICIAR ESTUDO"}</button>
    </section> : <div className="activity-preview--empty"><h2>Tópico sem atividades</h2><p>Adicione uma atividade ao roadmap para iniciar o estudo.</p></div>}
  </aside>;
}
