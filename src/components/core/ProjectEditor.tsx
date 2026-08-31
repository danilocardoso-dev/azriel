import { useState, type FormEvent } from "react";
import type { KnowledgeArea, Project, ProjectInput, ProjectStatus } from "../../types";
import { RecordEditorDialog } from "./RecordEditorDialog";

const blankProject = (): ProjectInput => ({ id: crypto.randomUUID(), name: "", category: "", description: "", objective: "", status: "planned", progress: 0, nextStep: "", knowledgeAreaIds: [] });
const toInput = (project?: Project | null): ProjectInput => project ? { id: project.id, name: project.name, category: project.category, description: project.description, objective: project.objective, status: project.status, progress: project.progress, nextStep: project.nextStep, knowledgeAreaIds: [...project.knowledgeAreaIds] } : blankProject();

interface ProjectEditorProps {
  project?: Project | null;
  knowledgeAreas: KnowledgeArea[];
  onCancel: () => void;
  onSave: (input: ProjectInput) => Promise<void>;
}

export function ProjectEditor({ project, knowledgeAreas, onCancel, onSave }: ProjectEditorProps) {
  const [form, setForm] = useState<ProjectInput>(() => toInput(project));
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const change = <K extends keyof ProjectInput>(field: K, value: ProjectInput[K]) => setForm((current) => ({ ...current, [field]: value }));

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault(); setError(null);
    if (!form.name.trim() || !form.category.trim()) { setError("Informe o nome e a categoria do projeto."); return; }
    if (!Number.isInteger(form.progress) || form.progress < 0 || form.progress > 100) { setError("O progresso deve ser um número inteiro entre 0 e 100."); return; }
    setBusy(true);
    try { await onSave({ ...form, name: form.name.trim(), category: form.category.trim(), description: form.description.trim(), objective: form.objective.trim(), nextStep: form.nextStep.trim() }); }
    catch (reason) { setError(reason instanceof Error ? reason.message : String(reason)); setBusy(false); }
  }

  return <RecordEditorDialog eyebrow="PROJECT CORE // REGISTRO" title={project ? `Editar ${project.name}` : "Novo projeto"} busy={busy} error={error} onCancel={onCancel} onSubmit={submit}>
    <label>Nome<input value={form.name} maxLength={120} required onChange={(event) => change("name", event.target.value)} /></label>
    <label>Categoria<input value={form.category} maxLength={80} required onChange={(event) => change("category", event.target.value)} /></label>
    <label>Status<select value={form.status} onChange={(event) => change("status", event.target.value as ProjectStatus)}><option value="active">Ativo</option><option value="research">Pesquisa e desenvolvimento</option><option value="paused">Pausado</option><option value="planned">Planejado</option><option value="completed">Concluído</option></select></label>
    <label>Progresso (%)<input type="number" min={0} max={100} step={1} value={form.progress} onChange={(event) => change("progress", Number(event.target.value))} /></label>
    <label className="record-editor__wide">Descrição<textarea rows={3} value={form.description} maxLength={1200} onChange={(event) => change("description", event.target.value)} /></label>
    <label className="record-editor__wide">Objetivo<textarea rows={3} value={form.objective} maxLength={1200} onChange={(event) => change("objective", event.target.value)} /></label>
    <label className="record-editor__wide">Próximo passo<textarea rows={2} value={form.nextStep} maxLength={600} onChange={(event) => change("nextStep", event.target.value)} /></label>
    <fieldset className="record-editor__wide"><legend>Conhecimentos relacionados</legend><div className="record-editor__checks">{knowledgeAreas.map((area) => <label key={area.id}><input type="checkbox" checked={form.knowledgeAreaIds.includes(area.id)} onChange={(event) => change("knowledgeAreaIds", event.target.checked ? [...form.knowledgeAreaIds, area.id] : form.knowledgeAreaIds.filter((id) => id !== area.id))} />{area.name}</label>)}</div></fieldset>
  </RecordEditorDialog>;
}
