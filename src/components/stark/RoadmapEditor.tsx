import { useState, type FormEvent } from "react";
import type { KnowledgeArea, RoadmapActivity, RoadmapActivityStatus, RoadmapActivityType, RoadmapStatus, RoadmapTopicState, StudyRoadmap, StudyRoadmapInput } from "../../types";
import { RecordEditorDialog } from "../core/RecordEditorDialog";

const id = () => crypto.randomUUID();
const activity = (order: number): RoadmapActivity => ({ id: id(), title: "", description: "", activityType: "READING", status: "pending", completedAt: null, order });
const blank = (): StudyRoadmapInput => ({ id: id(), name: "", description: "", status: "planned", stages: [] });
const inputOf = (roadmap?: StudyRoadmap | null): StudyRoadmapInput => roadmap ? { id: roadmap.id, name: roadmap.name, description: roadmap.description, status: roadmap.status, stages: structuredClone(roadmap.stages) } : blank();

export function RoadmapEditor({ roadmap, knowledge, onCancel, onSave }: { roadmap?: StudyRoadmap | null; knowledge: KnowledgeArea[]; onCancel: () => void; onSave: (input: StudyRoadmapInput) => Promise<void> }) {
  const [form, setForm] = useState(() => inputOf(roadmap));
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const changeStage = (stageIndex: number, updater: (stage: StudyRoadmapInput["stages"][number]) => StudyRoadmapInput["stages"][number]) => setForm((current) => ({ ...current, stages: current.stages.map((stage, index) => index === stageIndex ? updater(stage) : stage) }));
  const move = <T,>(items: T[], index: number, direction: -1 | 1) => { const target = index + direction; if (target < 0 || target >= items.length) return items; const next = [...items]; [next[index], next[target]] = [next[target], next[index]]; return next; };

  async function submit(event: FormEvent) {
    event.preventDefault(); setError(null);
    if (!form.name.trim()) { setError("Informe o nome do roadmap."); return; }
    setBusy(true);
    try { await onSave(form); } catch (reason) { setError(reason instanceof Error ? reason.message : String(reason)); setBusy(false); }
  }

  return <RecordEditorDialog eyebrow="MAPA STARK // ROADMAP" title={roadmap ? `Editar ${roadmap.name}` : "Novo roadmap"} busy={busy} error={error} onCancel={onCancel} onSubmit={submit}>
    <label>Nome<input value={form.name} required maxLength={120} onChange={(event) => setForm({ ...form, name: event.target.value })} /></label>
    <label>Status<select value={form.status} onChange={(event) => setForm({ ...form, status: event.target.value as RoadmapStatus })}><option value="planned">Planejado</option><option value="active">Ativo</option><option value="paused">Pausado</option><option value="completed">Concluído</option></select></label>
    <label className="record-editor__wide">Descrição<textarea rows={3} value={form.description} onChange={(event) => setForm({ ...form, description: event.target.value })} /></label>
    <div className="record-editor__wide roadmap-builder">
      <header><strong>ETAPAS E ATIVIDADES</strong><button type="button" onClick={() => setForm((current) => ({ ...current, stages: [...current.stages, { id: id(), name: "Nova etapa", description: "", order: current.stages.length + 1, topics: [] }] }))}>＋ ETAPA</button></header>
      {form.stages.map((stage, stageIndex) => <section className="roadmap-stage-editor" key={stage.id}>
        <div className="roadmap-builder__line"><b>ETAPA {String(stageIndex + 1).padStart(2, "0")}</b><input value={stage.name} onChange={(event) => changeStage(stageIndex, (item) => ({ ...item, name: event.target.value }))} /><button type="button" disabled={stageIndex === 0} onClick={() => setForm((current) => ({ ...current, stages: move(current.stages, stageIndex, -1) }))}>↑</button><button type="button" disabled={stageIndex === form.stages.length - 1} onClick={() => setForm((current) => ({ ...current, stages: move(current.stages, stageIndex, 1) }))}>↓</button><button type="button" className="danger-link" onClick={() => setForm((current) => ({ ...current, stages: current.stages.filter((_, index) => index !== stageIndex) }))}>×</button></div>
        {stage.topics.map((topic, topicIndex) => <article className="roadmap-topic-editor" key={topic.id}>
          <div className="roadmap-builder__line"><span>TÓPICO {topicIndex + 1}</span><input value={topic.name} onChange={(event) => changeStage(stageIndex, (item) => ({ ...item, topics: item.topics.map((entry, index) => index === topicIndex ? { ...entry, name: event.target.value } : entry) }))} /><button type="button" onClick={() => changeStage(stageIndex, (item) => ({ ...item, topics: item.topics.filter((_, index) => index !== topicIndex) }))}>×</button></div>
          <div className="roadmap-builder__fields"><select aria-label="Conhecimento relacionado" value={topic.knowledgeNodeId ?? ""} onChange={(event) => changeStage(stageIndex, (item) => ({ ...item, topics: item.topics.map((entry, index) => index === topicIndex ? { ...entry, knowledgeNodeId: event.target.value || null } : entry) }))}><option value="">Sem conhecimento relacionado</option>{knowledge.map((area) => <option key={area.id} value={area.id}>{area.name}</option>)}</select><select aria-label="Estado do tópico" value={topic.state} onChange={(event) => changeStage(stageIndex, (item) => ({ ...item, topics: item.topics.map((entry, index) => index === topicIndex ? { ...entry, state: event.target.value as RoadmapTopicState } : entry) }))}>{["NOT_STARTED", "EXPOSED", "UNDERSTOOD", "PRACTICED", "APPLIED", "MASTERED"].map((state) => <option key={state}>{state}</option>)}</select></div>
          {topic.activities.map((entry, activityIndex) => <div className="roadmap-activity-editor" key={entry.id}><input aria-label="Título da atividade" placeholder="Atividade" value={entry.title} onChange={(event) => changeStage(stageIndex, (item) => ({ ...item, topics: item.topics.map((currentTopic, index) => index === topicIndex ? { ...currentTopic, activities: currentTopic.activities.map((currentActivity, position) => position === activityIndex ? { ...currentActivity, title: event.target.value } : currentActivity) } : currentTopic) }))} /><select value={entry.activityType} onChange={(event) => changeStage(stageIndex, (item) => ({ ...item, topics: item.topics.map((currentTopic, index) => index === topicIndex ? { ...currentTopic, activities: currentTopic.activities.map((currentActivity, position) => position === activityIndex ? { ...currentActivity, activityType: event.target.value as RoadmapActivityType } : currentActivity) } : currentTopic) }))}>{["READING", "LESSON", "QUIZ", "EXERCISE", "SIMULATION", "EXPERIMENT", "PROJECT", "DOCUMENTATION", "RESEARCH", "OTHER"].map((type) => <option key={type}>{type}</option>)}</select><select value={entry.status} onChange={(event) => changeStage(stageIndex, (item) => ({ ...item, topics: item.topics.map((currentTopic, index) => index === topicIndex ? { ...currentTopic, activities: currentTopic.activities.map((currentActivity, position) => position === activityIndex ? { ...currentActivity, status: event.target.value as RoadmapActivityStatus, completedAt: event.target.value === "completed" ? currentActivity.completedAt ?? new Date().toISOString() : null } : currentActivity) } : currentTopic) }))}><option value="pending">Pendente</option><option value="in_progress">Em andamento</option><option value="completed">Concluída</option></select><button type="button" onClick={() => changeStage(stageIndex, (item) => ({ ...item, topics: item.topics.map((currentTopic, index) => index === topicIndex ? { ...currentTopic, activities: currentTopic.activities.filter((_, position) => position !== activityIndex) } : currentTopic) }))}>×</button></div>)}
          <button type="button" onClick={() => changeStage(stageIndex, (item) => ({ ...item, topics: item.topics.map((currentTopic, index) => index === topicIndex ? { ...currentTopic, activities: [...currentTopic.activities, activity(currentTopic.activities.length + 1)] } : currentTopic) }))}>＋ ATIVIDADE</button>
        </article>)}
        <button type="button" onClick={() => changeStage(stageIndex, (item) => ({ ...item, topics: [...item.topics, { id: id(), name: "Novo tópico", description: "", knowledgeNodeId: null, state: "NOT_STARTED", order: item.topics.length + 1, activities: [] }] }))}>＋ TÓPICO</button>
      </section>)}
    </div>
  </RecordEditorDialog>;
}
