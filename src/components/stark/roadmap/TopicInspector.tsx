import { useMemo, useState } from "react";
import { activityProgress, cleanTopicDescription, evidenceDimension } from "../../../services/roadmapExperience";
import type { KnowledgeArea, KnowledgeEvent, LearningMutation, Project, RoadmapActivity, RoadmapActivityStatus, RoadmapStage, RoadmapTopic, StudyRoadmap } from "../../../types";

const masteryStates: RoadmapTopic["state"][] = ["NOT_STARTED", "EXPOSED", "UNDERSTOOD", "PRACTICED", "APPLIED", "MASTERED"];
const masteryLabel = { NOT_STARTED: "NÃO INICIADO", EXPOSED: "EXPOSTO", UNDERSTOOD: "COMPREENDIDO", PRACTICED: "PRATICADO", APPLIED: "APLICADO", MASTERED: "DOMINADO" };
const activityLabel = { pending: "PENDENTE", in_progress: "EM ANDAMENTO", completed: "CONCLUÍDA" };

type Props = {
  roadmap: StudyRoadmap;
  stage: RoadmapStage | null;
  topic: RoadmapTopic | null;
  currentActivityId: string | null;
  knowledge: KnowledgeArea[];
  projects: Project[];
  events: KnowledgeEvent[];
  busyActivityId: string | null;
  lastMutation: LearningMutation | null;
  startInActivities: boolean;
  onActivityStatus: (activity: RoadmapActivity, status: RoadmapActivityStatus) => Promise<void>;
  onKnowledge: (knowledge: KnowledgeArea) => void;
};

export function TopicInspector({ roadmap, stage, topic, currentActivityId, knowledge, projects, events, busyActivityId, lastMutation, startInActivities, onActivityStatus, onKnowledge }: Props) {
  const [tab, setTab] = useState<"overview" | "activities">(startInActivities ? "activities" : "overview");
  const [activityFilter, setActivityFilter] = useState<"all" | RoadmapActivityStatus>("all");
  const [selectedActivityId, setSelectedActivityId] = useState<string | null>(startInActivities ? currentActivityId : null);
  const allTopics = roadmap.stages.flatMap((item) => item.topics);
  const relatedKnowledge = useMemo(() => topic ? [...new Set([topic.knowledgeNodeId, ...topic.activities.flatMap((activity) => [activity.primaryKnowledgeNodeId, ...(activity.secondaryKnowledgeNodeIds ?? [])])].filter(Boolean))].map((id) => knowledge.find((item) => item.id === id)).filter(Boolean) as KnowledgeArea[] : [], [knowledge, topic]);
  if (!topic) return <aside className="topic-inspector topic-inspector--empty"><strong>INSPETOR DE TÓPICO</strong><p>Selecione um tópico para consultar domínio, atividades, pré-requisitos e impacto no conhecimento.</p></aside>;
  const prerequisites = (topic.prerequisiteTopicIds ?? []).map((id) => allTopics.find((item) => item.id === id)).filter(Boolean) as RoadmapTopic[];
  const nextTopics = allTopics.filter((item) => (item.prerequisiteTopicIds ?? []).includes(topic.id));
  const visibleActivities = topic.activities.filter((item) => activityFilter === "all" || item.status === activityFilter);
  const topicEvents = events.filter((item) => item.topicId === topic.id && !item.reversalOfEventId);
  const impact = topicEvents.reduce((total, item) => ({ coverage: total.coverage + item.coverageImpact, depth: total.depth + item.depthImpact, integration: total.integration + item.integrationImpact }), { coverage: 0, depth: 0, integration: 0 });
  return <aside className="topic-inspector">
    <header><span>TÓPICO · {stage?.name.toUpperCase()}</span><h2>{topic.name}</h2><small>{masteryLabel[topic.state]}</small></header>
    <div className="topic-inspector__tabs" role="tablist"><button role="tab" aria-selected={tab === "overview"} className={tab === "overview" ? "active" : ""} onClick={() => setTab("overview")}>VISÃO GERAL</button><button role="tab" aria-selected={tab === "activities"} className={tab === "activities" ? "active" : ""} onClick={() => setTab("activities")}>ATIVIDADES</button></div>
    {tab === "overview" ? <div className="topic-inspector__body">
      <section><div className="inspector-heading"><strong>ESTADO DE DOMÍNIO</strong><span>{masteryLabel[topic.state]}</span></div><div className="mastery-rail">{masteryStates.map((state) => <i key={state} className={masteryStates.indexOf(state) <= masteryStates.indexOf(topic.state) ? "reached" : ""} title={masteryLabel[state]} />)}</div><small>Derivado das evidências concluídas; não é o progresso estrutural.</small></section>
      <section><div className="inspector-heading"><strong>PROGRESSO</strong><span>{activityProgress(topic.activities)}%</span></div><div className="inspector-progress"><i style={{ width: `${activityProgress(topic.activities)}%` }} /></div><small>{topic.activities.filter((item) => item.status === "completed").length} / {topic.activities.length} atividades</small></section>
      <section><strong>DESCRIÇÃO</strong><p>{cleanTopicDescription(topic.description) || "Sem descrição cadastrada."}</p></section>
      <section><strong>CONHECIMENTOS RELACIONADOS</strong><div className="inspector-tags">{relatedKnowledge.map((item) => <button key={item.id} onClick={() => onKnowledge(item)}>{item.name}</button>)}{!relatedKnowledge.length && <small>Nenhum vínculo cadastrado.</small>}</div></section>
      <section><strong>PRÉ-REQUISITOS</strong><div className="inspector-relations">{prerequisites.map((item) => { const completed = item.activities.length > 0 && item.activities.every((activity) => activity.status === "completed"); return <span key={item.id} data-complete={completed}>{completed ? "✓" : "○"} {item.name}</span>; })}{!prerequisites.length && <small>Nenhum pré-requisito estruturado.</small>}</div>{prerequisites.some((item) => !item.activities.length || item.activities.some((activity) => activity.status !== "completed")) && <small className="prerequisite-warning">ATENÇÃO · EXISTEM PRÉ-REQUISITOS PENDENTES</small>}</section>
      <section><strong>PRÓXIMOS TÓPICOS</strong><div className="inspector-relations">{nextTopics.map((item) => <span key={item.id}>○ {item.name}</span>)}{!nextTopics.length && <small>Nenhum tópico dependente.</small>}</div></section>
      <section className="inspector-impact"><strong>IMPACTO AUDITADO</strong><div><span>COBERTURA <b>{impact.coverage}</b></span><span>PROFUNDIDADE <b>{impact.depth}</b></span><span>INTEGRAÇÃO <b>{impact.integration}</b></span></div></section>
      <button className="inspector-primary" onClick={() => setTab("activities")}>ABRIR ATIVIDADES</button>
    </div> : <div className="topic-inspector__body">
      <div className="activity-filter">{(["all", "pending", "in_progress", "completed"] as const).map((status) => <button key={status} className={activityFilter === status ? "active" : ""} onClick={() => setActivityFilter(status)}>{status === "all" ? "TODAS" : activityLabel[status]}</button>)}</div>
      <div className="inspector-activities">{visibleActivities.map((activity) => { const expanded = selectedActivityId === activity.id; const primaryId = activity.primaryKnowledgeNodeId ?? topic.knowledgeNodeId; const primary = knowledge.find((item) => item.id === primaryId); const project = projects.find((item) => item.id === activity.projectId); return <article key={activity.id} className={activity.id === currentActivityId ? "current" : ""}><button className="activity-summary" aria-expanded={expanded} onClick={() => setSelectedActivityId(expanded ? null : activity.id)}><span><small>{activity.activityType}</small><strong>{activity.title}</strong></span><b data-status={activity.status}>{activityLabel[activity.status]}</b><i>{expanded ? "⌃" : "⌄"}</i></button>{expanded && <div className="activity-details"><p>{activity.description || "Sem instruções adicionais."}</p><dl><div><dt>CONHECIMENTO</dt><dd>{primary?.name ?? "SEM VÍNCULO"}</dd></div><div><dt>PROJETO</dt><dd>{project?.name ?? "SEM PROJETO"}</dd></div><div><dt>CONCLUSÃO</dt><dd>{activity.completedAt ? new Date(activity.completedAt).toLocaleString("pt-BR") : "—"}</dd></div><div><dt>EVIDÊNCIA</dt><dd>{evidenceDimension(activity.activityType)}</dd></div></dl></div>}<footer>{activity.status === "pending" && <><button disabled={busyActivityId === activity.id} onClick={() => void onActivityStatus(activity, "in_progress")}>INICIAR</button><button disabled={busyActivityId === activity.id} onClick={() => void onActivityStatus(activity, "completed")}>CONCLUIR</button></>}{activity.status === "in_progress" && <button disabled={busyActivityId === activity.id} onClick={() => void onActivityStatus(activity, "completed")}>CONCLUIR</button>}{activity.status === "completed" && <button disabled={busyActivityId === activity.id} onClick={() => void onActivityStatus(activity, "pending")}>REABRIR</button>}</footer></article>; })}{!visibleActivities.length && <p className="roadmap-empty-compact">Nenhuma atividade neste filtro.</p>}</div>
      {lastMutation && <div className="learning-feedback"><strong>{lastMutation.createdEvents.length ? "APRENDIZADO REGISTRADO" : "STATUS ATUALIZADO"}</strong><span>{lastMutation.createdEvents.length} EVENTOS · INTEGRAÇÃO {lastMutation.integration}</span><small>{lastMutation.createdEvents.map((event) => `${event.eventType}: C ${event.coverageImpact} / P ${event.depthImpact} / I ${event.integrationImpact}`).join(" · ") || "A transição não gerou nova evidência."}</small></div>}
    </div>}
  </aside>;
}
