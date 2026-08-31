import { useState, type FormEvent } from "react";
import type { EducationInput, EducationItem, EducationKind, EducationStatus } from "../../types";
import { RecordEditorDialog } from "./RecordEditorDialog";

const blankEducation = (): EducationInput => ({ id: crypto.randomUUID(), name: "", kind: "course", institution: "", status: "planned", startDate: null, expectedEndDate: null, completedAt: null, description: "", period: "", domains: [] });
const toInput = (item?: EducationItem | null): EducationInput => item ? { id: item.id, name: item.name, kind: item.kind, institution: item.institution, status: item.status, startDate: item.startDate, expectedEndDate: item.expectedEndDate, completedAt: item.completedAt, description: item.description, period: item.period, domains: [...item.domains] } : blankEducation();
const optionalDate = (value: string) => value || null;

interface EducationEditorProps { item?: EducationItem | null; onCancel: () => void; onSave: (input: EducationInput) => Promise<void> }

export function EducationEditor({ item, onCancel, onSave }: EducationEditorProps) {
  const [form, setForm] = useState<EducationInput>(() => toInput(item));
  const [domains, setDomains] = useState(form.domains.join(", "));
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const change = <K extends keyof EducationInput>(field: K, value: EducationInput[K]) => setForm((current) => ({ ...current, [field]: value }));

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault(); setError(null);
    if (!form.name.trim()) { setError("Informe o nome da formação."); return; }
    if (form.startDate && form.expectedEndDate && form.startDate > form.expectedEndDate) { setError("A previsão de conclusão não pode ser anterior ao início."); return; }
    if (form.status === "completed" && !form.completedAt) { setError("Informe a data de conclusão para uma formação concluída."); return; }
    const parsedDomains = domains.split(",").map((value) => value.trim()).filter(Boolean);
    setBusy(true);
    try { await onSave({ ...form, name: form.name.trim(), institution: form.institution.trim(), description: form.description.trim(), period: form.period.trim(), domains: Array.from(new Set(parsedDomains)) }); }
    catch (reason) { setError(reason instanceof Error ? reason.message : String(reason)); setBusy(false); }
  }

  return <RecordEditorDialog eyebrow="EDUCATION CORE // REGISTRO" title={item ? `Editar ${item.name}` : "Nova formação"} busy={busy} error={error} onCancel={onCancel} onSubmit={submit}>
    <label>Nome<input value={form.name} maxLength={160} required onChange={(event) => change("name", event.target.value)} /></label>
    <label>Instituição<input value={form.institution} maxLength={160} onChange={(event) => change("institution", event.target.value)} /></label>
    <label>Tipo<select value={form.kind} onChange={(event) => change("kind", event.target.value as EducationKind)}><option value="graduation">Graduação</option><option value="postgraduate">Pós-graduação</option><option value="masters">Mestrado</option><option value="doctorate">Doutorado</option><option value="course">Curso</option><option value="certification">Certificação</option></select></label>
    <label>Status<select value={form.status} onChange={(event) => change("status", event.target.value as EducationStatus)}><option value="planned">Planejada</option><option value="in_progress">Em andamento</option><option value="completed">Concluída</option></select></label>
    <label>Início<input type="date" value={form.startDate ?? ""} onChange={(event) => change("startDate", optionalDate(event.target.value))} /></label>
    <label>Previsão de conclusão<input type="date" value={form.expectedEndDate ?? ""} onChange={(event) => change("expectedEndDate", optionalDate(event.target.value))} /></label>
    <label>Conclusão<input type="date" value={form.completedAt ?? ""} onChange={(event) => change("completedAt", optionalDate(event.target.value))} /></label>
    <label>Período exibido<input value={form.period} maxLength={80} placeholder="2026 → 2027" onChange={(event) => change("period", event.target.value)} /></label>
    <label className="record-editor__wide">Domínios<input value={domains} placeholder="Software, IA, Biologia" onChange={(event) => setDomains(event.target.value)} /><small>Separe os domínios por vírgulas.</small></label>
    <label className="record-editor__wide">Descrição<textarea rows={4} value={form.description} maxLength={1400} onChange={(event) => change("description", event.target.value)} /></label>
  </RecordEditorDialog>;
}
