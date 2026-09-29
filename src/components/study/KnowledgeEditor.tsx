import { useState, type FormEvent } from "react";
import type { KnowledgeArea, KnowledgeInput, KnowledgeNodeType, Priority } from "../../types";
import { RecordEditorDialog } from "../core/RecordEditorDialog";

const blankKnowledge = (): KnowledgeInput => ({
  id: "",
  name: "",
  category: "",
  description: "",
  coverage: 0,
  depth: 0,
  priority: "high",
  nodeType: "area",
  parentId: null,
});

const toInput = (area?: KnowledgeArea | null): KnowledgeInput => area ? {
  id: area.id,
  name: area.name,
  category: area.category,
  description: area.description,
  coverage: area.coverage,
  depth: area.depth,
  priority: area.priority,
  nodeType: area.nodeType,
  parentId: area.parentId,
} : blankKnowledge();

interface KnowledgeEditorProps {
  area?: KnowledgeArea | null;
  areas: KnowledgeArea[];
  onCancel: () => void;
  onSave: (input: KnowledgeInput) => Promise<void>;
}

export function KnowledgeEditor({ area, areas, onCancel, onSave }: KnowledgeEditorProps) {
  const [form, setForm] = useState<KnowledgeInput>(() => toInput(area));
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const change = <K extends keyof KnowledgeInput>(field: K, value: KnowledgeInput[K]) => setForm((current) => ({ ...current, [field]: value }));

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setError(null);
    const id = form.id.trim();
    if (!id || !/^[A-Za-z0-9_-]+$/.test(id)) {
      setError("Use um ID único com letras, números, hífen ou sublinhado.");
      return;
    }
    if (!form.name.trim() || !form.category.trim()) {
      setError("Informe o nome e a categoria do conhecimento.");
      return;
    }
    setBusy(true);
    try {
      await onSave({
        ...form,
        id,
        name: form.name.trim(),
        category: form.category.trim(),
        description: form.description.trim(),
        coverage: area?.coverage ?? 0,
        depth: area?.depth ?? 0,
      });
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
      setBusy(false);
    }
  }

  return <RecordEditorDialog eyebrow="STUDY LAB // CONHECIMENTO" title={area ? `Editar ${area.name}` : "Novo conhecimento"} busy={busy} error={error} onCancel={onCancel} onSubmit={submit}>
    <label>ID<input value={form.id} maxLength={100} required disabled={Boolean(area)} placeholder="software-engineering" onChange={(event) => change("id", event.target.value)} /><small>{area ? "O ID é imutável após a criação." : "Use o mesmo ID nos arquivos de roadmap."}</small></label>
    <label>Nome<input value={form.name} maxLength={120} required onChange={(event) => change("name", event.target.value)} /></label>
    <label>Categoria<input value={form.category} maxLength={80} required onChange={(event) => change("category", event.target.value)} /></label>
    <label>Tipo<select value={form.nodeType} onChange={(event) => change("nodeType", event.target.value as KnowledgeNodeType)}><option value="area">Área</option><option value="discipline">Disciplina</option><option value="topic">Tópico</option><option value="competency">Competência</option></select></label>
    <label>Conhecimento pai<select value={form.parentId ?? ""} onChange={(event) => change("parentId", event.target.value || null)}><option value="">Sem vínculo superior</option>{areas.filter((item) => item.id !== form.id).map((item) => <option key={item.id} value={item.id}>{item.name} · {item.id}</option>)}</select></label>
    <label>Prioridade<select value={form.priority} onChange={(event) => change("priority", event.target.value as Priority)}><option value="low">Baixa</option><option value="medium">Média</option><option value="high">Alta</option><option value="critical">Crítica</option></select></label>
    <label className="record-editor__wide">Descrição<textarea rows={5} value={form.description} maxLength={1200} onChange={(event) => change("description", event.target.value)} /></label>
  </RecordEditorDialog>;
}
