import { useState, type FormEvent } from "react";
import type { KnowledgeArea, Project, ResearchItem, RoadmapActivity, RoadmapActivityStatus, RoadmapActivityType, RoadmapStatus, StudyRoadmap, StudyRoadmapInput } from "../../types";
import { RecordEditorDialog } from "../core/RecordEditorDialog";

const uid = () => crypto.randomUUID();
const activity = (order: number): RoadmapActivity => ({ id: uid(), title: "", description: "", activityType: "READING", status: "pending", completedAt: null, order, primaryKnowledgeNodeId: null, secondaryKnowledgeNodeIds: [], projectId: null, researchId: null });
const inputOf = (item?: StudyRoadmap | null): StudyRoadmapInput => item ? { id: item.id, name: item.name, description: item.description, status: item.status, stages: structuredClone(item.stages) } : { id: uid(), name: "", description: "", status: "planned", stages: [] };
type Props = { roadmap?: StudyRoadmap | null; knowledge: KnowledgeArea[]; projects: Project[]; research: ResearchItem[]; onCancel: () => void; onSave: (input: StudyRoadmapInput) => Promise<void> };

export function RoadmapEditor({ roadmap, knowledge, projects, research, onCancel, onSave }: Props) {
  const [form, setForm] = useState(() => inputOf(roadmap));
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const changeStage = (si: number, fn: (stage: StudyRoadmapInput["stages"][number]) => StudyRoadmapInput["stages"][number]) => setForm((current) => ({ ...current, stages: current.stages.map((stage, index) => index === si ? fn(stage) : stage) }));
  const changeActivity = (si: number, ti: number, ai: number, fn: (entry: RoadmapActivity) => RoadmapActivity) => changeStage(si, (stage) => ({ ...stage, topics: stage.topics.map((topic, index) => index === ti ? { ...topic, activities: topic.activities.map((entry, position) => position === ai ? fn(entry) : entry) } : topic) }));
  async function submit(event: FormEvent) { event.preventDefault(); setError(null); setBusy(true); try { await onSave(form); } catch (reason) { setError(reason instanceof Error ? reason.message : String(reason)); setBusy(false); } }

  return <RecordEditorDialog eyebrow="MAPA STARK // LEARNING ENGINE" title={roadmap ? `Editar ${roadmap.name}` : "Novo roadmap"} busy={busy} error={error} onCancel={onCancel} onSubmit={submit}>
    <label>Nome<input required maxLength={120} value={form.name} onChange={(event) => setForm({ ...form, name: event.target.value })} /></label>
    <label>Status<select value={form.status} onChange={(event) => setForm({ ...form, status: event.target.value as RoadmapStatus })}><option value="planned">Planejado</option><option value="active">Ativo</option><option value="paused">Pausado</option><option value="completed">Concluído</option></select></label>
    <label className="record-editor__wide">Descrição<textarea rows={3} value={form.description} onChange={(event) => setForm({ ...form, description: event.target.value })} /></label>
    <div className="record-editor__wide roadmap-builder"><header><strong>ETAPAS, TÓPICOS E EVIDÊNCIAS</strong><button type="button" onClick={() => setForm((current) => ({ ...current, stages: [...current.stages, { id: uid(), name: "Nova etapa", description: "", order: current.stages.length + 1, topics: [] }] }))}>＋ ETAPA</button></header>
      {form.stages.map((stage, si) => <section className="roadmap-stage-editor" key={stage.id}>
        <div className="roadmap-builder__line"><b>ETAPA {si + 1}</b><input value={stage.name} onChange={(event) => changeStage(si, (item) => ({ ...item, name: event.target.value }))} /><button type="button" className="danger-link" onClick={() => setForm((current) => ({ ...current, stages: current.stages.filter((_, index) => index !== si) }))}>×</button></div>
        {stage.topics.map((topic, ti) => <article className="roadmap-topic-editor" key={topic.id}>
          <div className="roadmap-builder__line"><span>TÓPICO {ti + 1}</span><input value={topic.name} onChange={(event) => changeStage(si, (item) => ({ ...item, topics: item.topics.map((entry, index) => index === ti ? { ...entry, name: event.target.value } : entry) }))} /><button type="button" onClick={() => changeStage(si, (item) => ({ ...item, topics: item.topics.filter((_, index) => index !== ti) }))}>×</button></div>
          <div className="roadmap-builder__fields"><label>Conhecimento padrão<select value={topic.knowledgeNodeId ?? ""} onChange={(event) => changeStage(si, (item) => ({ ...item, topics: item.topics.map((entry, index) => index === ti ? { ...entry, knowledgeNodeId: event.target.value || null } : entry) }))}><option value="">Selecione</option>{knowledge.map((area) => <option key={area.id} value={area.id}>{area.name}</option>)}</select></label><label>Domínio derivado<input value={topic.state} readOnly title="Calculado pelas evidências" /></label></div>
          {topic.activities.map((entry, ai) => <div className="roadmap-activity-editor learning-activity" key={entry.id}>
            <input placeholder="Atividade" value={entry.title} onChange={(event) => changeActivity(si, ti, ai, (item) => ({ ...item, title: event.target.value }))} />
            <select value={entry.activityType} onChange={(event) => changeActivity(si, ti, ai, (item) => ({ ...item, activityType: event.target.value as RoadmapActivityType }))}>{["READING", "LESSON", "QUIZ", "EXERCISE", "SIMULATION", "EXPERIMENT", "PROJECT", "DOCUMENTATION", "RESEARCH", "OTHER"].map((type) => <option key={type}>{type}</option>)}</select>
            <select value={entry.status} onChange={(event) => changeActivity(si, ti, ai, (item) => ({ ...item, status: event.target.value as RoadmapActivityStatus, completedAt: event.target.value === "completed" ? item.completedAt ?? new Date().toISOString() : null }))}><option value="pending">Pendente</option><option value="in_progress">Em andamento</option><option value="completed">Concluída</option></select>
            <button type="button" onClick={() => changeStage(si, (item) => ({ ...item, topics: item.topics.map((current, index) => index === ti ? { ...current, activities: current.activities.filter((_, position) => position !== ai) } : current) }))}>×</button>
            <div className="learning-activity__relations">
              <label>Conhecimento primário<select value={entry.primaryKnowledgeNodeId ?? ""} onChange={(event) => changeActivity(si, ti, ai, (item) => ({ ...item, primaryKnowledgeNodeId: event.target.value || null }))}><option value="">Usar o tópico</option>{knowledge.map((area) => <option key={area.id} value={area.id}>{area.name}</option>)}</select></label>
              <label>Secundários<select multiple value={entry.secondaryKnowledgeNodeIds ?? []} onChange={(event) => changeActivity(si, ti, ai, (item) => ({ ...item, secondaryKnowledgeNodeIds: [...event.target.selectedOptions].map((option) => option.value) }))}>{knowledge.filter((area) => area.id !== (entry.primaryKnowledgeNodeId ?? topic.knowledgeNodeId)).map((area) => <option key={area.id} value={area.id}>{area.name}</option>)}</select></label>
              <label>Projeto<select value={entry.projectId ?? ""} onChange={(event) => changeActivity(si, ti, ai, (item) => ({ ...item, projectId: event.target.value || null }))}><option value="">Sem projeto</option>{projects.map((project) => <option key={project.id} value={project.id}>{project.name}</option>)}</select></label>
              <label>Pesquisa<select value={entry.researchId ?? ""} onChange={(event) => changeActivity(si, ti, ai, (item) => ({ ...item, researchId: event.target.value || null }))}><option value="">Sem pesquisa</option>{research.map((item) => <option key={item.id} value={item.id}>{item.title}</option>)}</select></label>
            </div>
          </div>)}
          <button type="button" onClick={() => changeStage(si, (stageItem) => ({ ...stageItem, topics: stageItem.topics.map((current, index) => index === ti ? { ...current, activities: [...current.activities, activity(current.activities.length + 1)] } : current) }))}>＋ EVIDÊNCIA</button>
        </article>)}
        <button type="button" onClick={() => changeStage(si, (item) => ({ ...item, topics: [...item.topics, { id: uid(), name: "Novo tópico", description: "", knowledgeNodeId: null, state: "NOT_STARTED", order: item.topics.length + 1, activities: [] }] }))}>＋ TÓPICO</button>
      </section>)}
    </div>
  </RecordEditorDialog>;
}
