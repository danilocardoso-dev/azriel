import { useState, type FormEvent } from "react";
import type { KnowledgeArea, KnowledgeInput, Priority } from "../../types";
import { RecordEditorDialog } from "./RecordEditorDialog";

const blankKnowledge = (): KnowledgeInput => ({ id: crypto.randomUUID(), name: "", category: "", description: "", coverage: 0, depth: 0, priority: "medium" });
const toInput = (area?: KnowledgeArea | null): KnowledgeInput => area ? { id: area.id, name: area.name, category: area.category, description: area.description, coverage: area.coverage, depth: area.depth, priority: area.priority } : blankKnowledge();

interface KnowledgeEditorProps { area?: KnowledgeArea | null; onCancel: () => void; onSave: (input: KnowledgeInput) => Promise<void> }

export function KnowledgeEditor({ area, onCancel, onSave }: KnowledgeEditorProps) {
  const [form, setForm] = useState<KnowledgeInput>(() => toInput(area));
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const change = <K extends keyof KnowledgeInput>(field: K, value: KnowledgeInput[K]) => setForm((current) => ({ ...current, [field]: value }));

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault(); setError(null);
    if (!form.name.trim() || !form.category.trim()) { setError("Informe o nome e a categoria do conhecimento."); return; }
    if (![form.coverage, form.depth].every((value) => Number.isInteger(value) && value >= 0 && value <= 100)) { setError("Cobertura e profundidade devem ser números inteiros entre 0 e 100."); return; }
    setBusy(true);
    try { await onSave({ ...form, name: form.name.trim(), category: form.category.trim(), description: form.description.trim() }); }
    catch (reason) { setError(reason instanceof Error ? reason.message : String(reason)); setBusy(false); }
  }

  return <RecordEditorDialog eyebrow="KNOWLEDGE CORE // REGISTRO" title={area ? `Editar ${area.name}` : "Novo conhecimento"} busy={busy} error={error} onCancel={onCancel} onSubmit={submit}>
    <label>Nome<input value={form.name} maxLength={120} required onChange={(event) => change("name", event.target.value)} /></label>
    <label>Categoria<input value={form.category} maxLength={80} required onChange={(event) => change("category", event.target.value)} /></label>
    <label>Prioridade<select value={form.priority} onChange={(event) => change("priority", event.target.value as Priority)}><option value="low">Baixa</option><option value="medium">Média</option><option value="high">Alta</option><option value="critical">Crítica</option></select></label>
    <label>Cobertura (%)<input type="number" min={0} max={100} step={1} value={form.coverage} disabled={Boolean(area)} onChange={(event) => change("coverage", Number(event.target.value))} /><small>{area ? "Use Registrar evolução para alterar a métrica." : "Métrica inicial"}</small></label>
    <label>Profundidade (%)<input type="number" min={0} max={100} step={1} value={form.depth} disabled={Boolean(area)} onChange={(event) => change("depth", Number(event.target.value))} /><small>{area ? "Use Registrar evolução para alterar a métrica." : "Métrica inicial"}</small></label>
    <label className="record-editor__wide">Descrição<textarea rows={4} value={form.description} maxLength={1200} onChange={(event) => change("description", event.target.value)} /></label>
  </RecordEditorDialog>;
}
