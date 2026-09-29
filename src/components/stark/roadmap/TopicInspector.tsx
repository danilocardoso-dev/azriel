import { useState } from "react";
import type { StudyAICardDraft } from "../../../ai/study/studyAIDrafts";
import { StudyAIToolPanel } from "../../study/StudyAIToolPanel";
import { StudyRelatedMaterials } from "../../study/StudyRelatedMaterials";
import { activityProgress, cleanTopicDescription } from "../../../services/roadmapExperience";
import { studyLabService } from "../../../services/studyLabService";
import type { RoadmapActivity, RoadmapActivityStatus, RoadmapStage, RoadmapTopic, StudyCardInput, StudyRoadmap } from "../../../types";

const activityLabel = { pending: "PENDENTE", in_progress: "EM ANDAMENTO", completed: "CONCLUÍDA" };

type Props = {
  roadmap: StudyRoadmap;
  stage: RoadmapStage | null;
  topic: RoadmapTopic | null;
  currentActivityId: string | null;
  activeSessionActivityId: string | null;
  busyActivityId: string | null;
  sessionBusy: boolean;
  startInActivities: boolean;
  onActivityStatus: (activity: RoadmapActivity, status: RoadmapActivityStatus) => Promise<void>;
  onStartSession: (roadmap: StudyRoadmap, activity: RoadmapActivity) => Promise<void>;
  onNotes: (roadmap: StudyRoadmap, stage: RoadmapStage, topic: RoadmapTopic, activity: RoadmapActivity, mode: "open" | "create") => void;
  onCards: (roadmap: StudyRoadmap, stage: RoadmapStage, topic: RoadmapTopic, activity: RoadmapActivity, mode: "open" | "create") => void;
  onMaterials: (activity: RoadmapActivity) => void;
};

export function TopicInspector({ roadmap, stage, topic, currentActivityId, activeSessionActivityId, busyActivityId, sessionBusy, startInActivities, onActivityStatus, onStartSession, onNotes, onCards, onMaterials }: Props) {
  const [tab, setTab] = useState<"overview" | "activities">(startInActivities ? "activities" : "overview");
  const [activityFilter, setActivityFilter] = useState<"all" | RoadmapActivityStatus>("all");
  const [selectedActivityId, setSelectedActivityId] = useState<string | null>(startInActivities ? currentActivityId : null);
  const allTopics = roadmap.stages.flatMap((item) => item.topics);
  if (!topic) return <aside className="topic-inspector topic-inspector--empty"><strong>INSPETOR DE TÓPICO</strong><p>Selecione um tópico para consultar atividades, pré-requisitos e progresso.</p></aside>;
  const selectedTopic = topic;
  const prerequisites = (topic.prerequisiteTopicIds ?? []).map((id) => allTopics.find((item) => item.id === id)).filter(Boolean) as RoadmapTopic[];
  const nextTopics = allTopics.filter((item) => (item.prerequisiteTopicIds ?? []).includes(topic.id));
  const visibleActivities = topic.activities.filter((item) => activityFilter === "all" || item.status === activityFilter);
  const completed = topic.activities.filter((item) => item.status === "completed").length;
  const status = topic.activities.length > 0 && completed === topic.activities.length ? "CONCLUÍDO" : topic.activities.some((item) => item.status === "in_progress") ? "EM ANDAMENTO" : "NÃO INICIADO";
  async function saveGeneratedCards(activity: RoadmapActivity, drafts: StudyAICardDraft[]) {
    if (!stage) throw new Error("A atividade não possui uma etapa válida.");
    const inputs: StudyCardInput[] = drafts.map((card) => ({ id: card.id, front: card.front, back: card.back, roadmapId: roadmap.id, stageId: stage.id, topicId: selectedTopic.id, activityId: activity.id, notebookId: null, noteId: null }));
    await studyLabService.saveCards(inputs);
  }
  return <aside className="topic-inspector">
    <header><span>TÓPICO · {stage?.name.toUpperCase()}</span><h2>{topic.name}</h2><small>{status}</small></header>
    <div className="topic-inspector__tabs" role="tablist"><button role="tab" aria-selected={tab === "overview"} className={tab === "overview" ? "active" : ""} onClick={() => setTab("overview")}>VISÃO GERAL</button><button role="tab" aria-selected={tab === "activities"} className={tab === "activities" ? "active" : ""} onClick={() => setTab("activities")}>ATIVIDADES</button></div>
    {tab === "overview" ? <div className="topic-inspector__body">
      <section><div className="inspector-heading"><strong>PROGRESSO</strong><span>{activityProgress(topic.activities)}%</span></div><div className="inspector-progress"><i style={{ width: `${activityProgress(topic.activities)}%` }} /></div><small>{completed} / {topic.activities.length} atividades</small></section>
      <section><strong>DESCRIÇÃO</strong><p>{cleanTopicDescription(topic.description) || "Sem descrição cadastrada."}</p></section>
      <section><strong>PRÉ-REQUISITOS</strong><div className="inspector-relations">{prerequisites.map((item) => { const done = item.activities.length > 0 && item.activities.every((activity) => activity.status === "completed"); return <span key={item.id} data-complete={done}>{done ? "✓" : "○"} {item.name}</span>; })}{!prerequisites.length && <small>Nenhum pré-requisito estruturado.</small>}</div>{prerequisites.some((item) => !item.activities.length || item.activities.some((activity) => activity.status !== "completed")) && <small className="prerequisite-warning">ATENÇÃO · EXISTEM PRÉ-REQUISITOS PENDENTES</small>}</section>
      <section><strong>PRÓXIMOS TÓPICOS</strong><div className="inspector-relations">{nextTopics.map((item) => <span key={item.id}>○ {item.name}</span>)}{!nextTopics.length && <small>Nenhum tópico dependente.</small>}</div></section>
      <button className="inspector-primary" onClick={() => setTab("activities")}>ABRIR ATIVIDADES</button>
    </div> : <div className="topic-inspector__body">
      <div className="activity-filter">{(["all", "pending", "in_progress", "completed"] as const).map((item) => <button key={item} className={activityFilter === item ? "active" : ""} onClick={() => setActivityFilter(item)}>{item === "all" ? "TODAS" : activityLabel[item]}</button>)}</div>
      <div className="inspector-activities">{visibleActivities.map((activity) => { const expanded = selectedActivityId === activity.id; const sessionActive = activeSessionActivityId === activity.id; return <article key={activity.id} className={activity.id === currentActivityId ? "current" : ""}><button className="activity-summary" aria-expanded={expanded} onClick={() => setSelectedActivityId(expanded ? null : activity.id)}><span><small>{activity.activityType}</small><strong>{activity.title}</strong></span><b data-status={activity.status}>{activityLabel[activity.status]}</b><i>{expanded ? "⌃" : "⌄"}</i></button>{expanded && <div className="activity-details"><p>{activity.description || "Sem instruções adicionais."}</p><dl><div><dt>TIPO</dt><dd>{activity.activityType}</dd></div><div><dt>SESSÃO</dt><dd>{sessionActive ? "ATIVA" : "DISPONÍVEL"}</dd></div><div><dt>CONCLUSÃO</dt><dd>{activity.completedAt ? new Date(activity.completedAt).toLocaleString("pt-BR") : "—"}</dd></div><div><dt>STATUS</dt><dd>{activityLabel[activity.status]}</dd></div></dl>{stage && <><section className="activity-related-notes"><div><strong>NOTAS RELACIONADAS</strong><span>ROADMAP / TÓPICO / ATIVIDADE</span></div><nav><button onClick={() => onNotes(roadmap, stage, topic, activity, "open")}>ABRIR NOTAS</button><button onClick={() => onNotes(roadmap, stage, topic, activity, "create")}>＋ CRIAR NOTA</button></nav></section><section className="activity-related-notes"><div><strong>STUDY CARDS RELACIONADOS</strong><span>ACTIVE RECALL / CONTEXTO REAL</span></div><nav><button onClick={() => onCards(roadmap, stage, topic, activity, "open")}>VISUALIZAR</button><button onClick={() => onCards(roadmap, stage, topic, activity, "create")}>＋ CRIAR CARD</button></nav></section><StudyRelatedMaterials relationType="ACTIVITY" relationId={activity.id} onOpenLibrary={() => onMaterials(activity)} /><StudyAIToolPanel context={{ roadmap: { id: roadmap.id, name: roadmap.name }, stage: { id: stage.id, name: stage.name }, topic: { id: topic.id, name: topic.name }, activity: { id: activity.id, name: activity.title, description: activity.description, activityType: activity.activityType } }} onSaveCards={(drafts) => saveGeneratedCards(activity, drafts)} /></>}</div>}<footer><button disabled={sessionBusy || Boolean(activeSessionActivityId)} onClick={() => void onStartSession(roadmap, activity)}>{sessionActive ? "SESSÃO ATIVA" : "INICIAR SESSÃO"}</button>{activity.status === "pending" && <><button disabled={busyActivityId === activity.id} onClick={() => void onActivityStatus(activity, "in_progress")}>MARCAR EM ANDAMENTO</button><button disabled={busyActivityId === activity.id} onClick={() => void onActivityStatus(activity, "completed")}>CONCLUIR</button></>}{activity.status === "in_progress" && <button disabled={busyActivityId === activity.id} onClick={() => void onActivityStatus(activity, "completed")}>CONCLUIR</button>}{activity.status === "completed" && <button disabled={busyActivityId === activity.id} onClick={() => void onActivityStatus(activity, "pending")}>REABRIR</button>}</footer></article>; })}{!visibleActivities.length && <p className="roadmap-empty-compact">Nenhuma atividade neste filtro.</p>}</div>
    </div>}
  </aside>;
}
