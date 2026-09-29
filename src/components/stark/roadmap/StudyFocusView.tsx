import { useState } from "react";
import type { StudyAICardDraft } from "../../../ai/study/studyAIDrafts";
import { studyLabService } from "../../../services/studyLabService";
import type { RoadmapActivityResource, RoadmapActivityStatus, StudyCardInput, StudySession } from "../../../types";
import type { RoadmapContext } from "../../../services/roadmapExperience";
import { StudyAIToolPanel } from "../../study/StudyAIToolPanel";
import { StudyRelatedMaterials } from "../../study/StudyRelatedMaterials";
import { StudySessionPanel } from "../../study/StudySessionPanel";

type Props = {
  context: RoadmapContext;
  activeSession: StudySession | null;
  sessionBusy: boolean;
  busyActivityId: string | null;
  onBack: () => void;
  onStartSession: () => Promise<void>;
  onPauseSession: () => Promise<void>;
  onResumeSession: () => Promise<void>;
  onCompleteSession: () => Promise<void>;
  onCancelSession: () => Promise<void>;
  onOpenSessionNotes: () => Promise<void>;
  onCreateSessionNote: () => Promise<void>;
  onActivityStatus: (status: RoadmapActivityStatus) => Promise<void>;
  onNotes: (mode: "open" | "create") => void;
  onCards: (mode: "open" | "create") => void;
  onOpenLibrary: (materialId?: string) => void;
  onOpenResource: (resource: RoadmapActivityResource) => Promise<void>;
  onAddResourceToLibrary: (resource: RoadmapActivityResource) => Promise<void>;
};

export function StudyFocusView(props: Props) {
  const { context, activeSession, sessionBusy, busyActivityId, onBack, onStartSession, onPauseSession, onResumeSession, onCompleteSession, onCancelSession, onOpenSessionNotes, onCreateSessionNote, onActivityStatus, onNotes, onCards, onOpenLibrary, onOpenResource, onAddResourceToLibrary } = props;
  const { roadmap, stage, topic, activity } = context;
  const [selectedResourceIds, setSelectedResourceIds] = useState<Set<string>>(new Set());
  const [aiOpen, setAiOpen] = useState(false);
  const resources = activity.resources ?? [];
  const sessionMatches = activeSession?.activityId === activity.id;
  const aiResources = resources.filter((resource) => selectedResourceIds.has(resource.id));
  const toggleResource = (resource: RoadmapActivityResource) => setSelectedResourceIds((current) => {
    const next = new Set(current);
    if (next.has(resource.id)) next.delete(resource.id); else next.add(resource.id);
    return next;
  });
  async function saveGeneratedCards(drafts: StudyAICardDraft[]) {
    const inputs: StudyCardInput[] = drafts.map((card) => ({ id: card.id, front: card.front, back: card.back, roadmapId: roadmap.id, stageId: stage.id, topicId: topic.id, activityId: activity.id, notebookId: null, noteId: null }));
    await studyLabService.saveCards(inputs);
  }

  return <article className="study-focus-view">
    <header className="study-focus-view__header">
      <button onClick={onBack}>← VOLTAR AO ROADMAP</button>
      <span>{roadmap.name} · {stage.order}.{topic.order}</span>
      <h1>{activity.title}</h1>
      {activity.description && <p>{activity.description}</p>}
      <div>{activity.activityType && <span>{activity.activityType}</span>}{activity.isValidation && <span>CHECKPOINT</span>}{activity.estimatedMinutes && <span>~{activity.estimatedMinutes} MIN ESTIMADOS</span>}</div>
    </header>

    <section className="study-focus-timer">
      {activeSession ? <><StudySessionPanel compact session={activeSession} busy={sessionBusy} onPause={onPauseSession} onResume={onResumeSession} onComplete={onCompleteSession} onCancel={onCancelSession} onOpenNotes={onOpenSessionNotes} onCreateNote={onCreateSessionNote} />{!sessionMatches && <p>Existe uma sessão ativa em outra atividade. Conclua ou cancele essa sessão antes de iniciar esta.</p>}</> : <div><span>FOCO</span><strong>PRONTO PARA INICIAR</strong><small>Pomodoro configurado nas preferências do Study Lab.</small><button className="primary" disabled={sessionBusy} onClick={() => void onStartSession()}>▶ INICIAR FOCO</button></div>}
    </section>

    <main className="study-focus-view__content">
      {activity.learningObjective && <section className="study-focus-primary"><span>OBJETIVO</span><p>{activity.learningObjective}</p></section>}
      {activity.instructions && <section className="study-focus-primary"><span>O QUE FAZER</span><p>{activity.instructions}</p></section>}
      {resources.length > 0 && <section className="study-focus-primary"><span>RECURSOS</span><div className="study-focus-resources">{resources.map((resource) => <article key={resource.id}>
        <div><strong>{resource.title}</strong><small>{[resource.provider, resource.type, resource.language?.toUpperCase()].filter(Boolean).join(" · ")}</small>{resource.required && <b>OBRIGATÓRIO</b>}</div>
        <label title="Incluir este recurso no contexto da IA"><input type="checkbox" checked={selectedResourceIds.has(resource.id)} onChange={() => toggleResource(resource)} /> CONTEXTO IA</label>
        <nav>{resource.url && <button onClick={() => void onOpenResource(resource)}>ABRIR</button>}<button onClick={() => void onAddResourceToLibrary(resource)}>{resource.studyMaterialId ? "NA BIBLIOTECA" : "+ BIBLIOTECA"}</button></nav>
      </article>)}</div></section>}

      {activity.learningMethod && <details className="study-focus-disclosure"><summary>MÉTODO</summary><div><strong>{activity.learningMethod.type}</strong>{activity.learningMethod.instructions && <p>{activity.learningMethod.instructions}</p>}</div></details>}
      {(activity.deliverable || activity.completionCriteria) && <details className="study-focus-disclosure"><summary>ENTREGA E CRITÉRIO</summary><div>{activity.deliverable && <><strong>ENTREGA</strong><p>{activity.deliverable}</p></>}{activity.completionCriteria && <><strong>CRITÉRIO DE CONCLUSÃO</strong><p>{activity.completionCriteria}</p></>}</div></details>}
      {activity.reflectionPrompt && <details className="study-focus-disclosure"><summary>REFLEXÃO</summary><div><p>{activity.reflectionPrompt}</p><button onClick={() => onNotes("create")}>REGISTRAR REFLEXÃO</button></div></details>}
      <details className="study-focus-disclosure"><summary>NOTAS RELACIONADAS{sessionMatches ? ` (${activeSession.linkedNoteCount})` : ""}</summary><div className="study-focus-actions"><button onClick={() => onNotes("open")}>ABRIR NOTAS</button><button onClick={() => onNotes("create")}>＋ CRIAR NOTA</button>{activity.deliverable && <button onClick={() => onNotes("create")}>NOTA DE ENTREGA</button>}</div></details>
      <details className="study-focus-disclosure"><summary>STUDY CARDS</summary><div className="study-focus-actions"><button onClick={() => onCards("open")}>VISUALIZAR</button><button onClick={() => onCards("create")}>＋ CRIAR CARD</button></div></details>
      <details className="study-focus-disclosure"><summary>MATERIAIS RELACIONADOS</summary><StudyRelatedMaterials relationType="ACTIVITY" relationId={activity.id} onOpenLibrary={onOpenLibrary} /></details>
      <section className="study-focus-ai"><button onClick={() => setAiOpen((value) => !value)}>AZRIEL AI {aiOpen ? "⌃" : "⌄"}</button>{aiOpen && <StudyAIToolPanel context={{ roadmap: { id: roadmap.id, name: roadmap.name }, stage: { id: stage.id, name: stage.name }, topic: { id: topic.id, name: topic.name }, activity: { id: activity.id, name: activity.title, description: activity.description, activityType: activity.activityType, learningObjective: activity.learningObjective, instructions: activity.instructions, completionCriteria: activity.completionCriteria, deliverable: activity.deliverable, estimatedMinutes: activity.estimatedMinutes, learningMethod: activity.learningMethod, isValidation: activity.isValidation, reflectionPrompt: activity.reflectionPrompt, resources: aiResources } }} onSaveCards={saveGeneratedCards} onClose={() => setAiOpen(false)} />}</section>
    </main>

    <footer className="study-focus-view__footer">
      {activity.status !== "completed" ? <button className="primary" disabled={busyActivityId === activity.id} onClick={() => void onActivityStatus("completed")}>✓ CONCLUIR ATIVIDADE</button> : <button disabled={busyActivityId === activity.id} onClick={() => void onActivityStatus("pending")}>REABRIR ATIVIDADE</button>}
      {activity.status === "pending" && <button disabled={busyActivityId === activity.id} onClick={() => void onActivityStatus("in_progress")}>MARCAR EM ANDAMENTO</button>}
    </footer>
  </article>;
}
