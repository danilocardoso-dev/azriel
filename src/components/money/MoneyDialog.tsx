import { useState, type FormEvent } from "react";
import { RecordEditorDialog } from "../core/RecordEditorDialog";
import { DeleteConfirmationDialog } from "../daily/DeleteConfirmationDialog";
import { amountField, formatDate, maskDate, parseAmount, parseDate } from "../../money/format";
import { moneyRepository, type MoneyItem, type MoneyKind, type MoneyRecurrence } from "../../repositories/moneyRepository";

const messageOf = (error: unknown) => error instanceof Error ? error.message : String(error);
const recurrences: Array<[MoneyRecurrence, string]> = [["once", "Única"], ["installments", "Parcelada"], ["monthly", "Permanente (todo mês)"]];

/** Cria ou edita uma regra de despesa/recebimento. Editar vale para todos os meses da regra. */
export function MoneyDialog({ item, kind, defaultDate, projects, onClose, onSaved }: { item: MoneyItem | null; kind: MoneyKind; defaultDate: string; projects: string[]; onClose: () => void; onSaved: () => void }) {
  const [title, setTitle] = useState(item?.title ?? "");
  const [amount, setAmount] = useState(item ? amountField(item.amountCents) : "");
  const [date, setDate] = useState(formatDate(item?.firstDate ?? defaultDate));
  const [recurrence, setRecurrence] = useState<MoneyRecurrence>(item?.recurrence ?? "once");
  const [installments, setInstallments] = useState(String(item?.installments ?? 3));
  const [project, setProject] = useState(item?.project ?? "");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const income = kind === "income";

  async function run(action: () => Promise<unknown>) {
    setBusy(true); setError(null);
    try { await action(); onSaved(); onClose(); }
    catch (reason) { setError(messageOf(reason)); setConfirmDelete(false); }
    finally { setBusy(false); }
  }

  function submit(event: FormEvent) {
    event.preventDefault();
    const cents = parseAmount(amount);
    if (!title.trim()) { setError(income ? "Dê um nome ao recebimento." : "Dê um nome à conta."); return; }
    if (cents === null || cents <= 0) { setError("Informe um valor válido, como 1.500,00."); return; }
    const isoDate = parseDate(date);
    if (!isoDate) { setError("Informe a data no formato dd/mm/aaaa."); return; }
    const count = Number(installments);
    if (recurrence === "installments" && (!Number.isInteger(count) || count < 2)) { setError("Parcelas: informe 2 ou mais."); return; }
    void run(() => moneyRepository.save({
      id: item?.id ?? crypto.randomUUID(), kind, title: title.trim(), amountCents: cents, firstDate: isoDate, recurrence,
      installments: recurrence === "installments" ? count : null, project: income && project.trim() ? project.trim() : null,
    }));
  }

  const noun = income ? "RECEBIMENTO" : "CONTA";
  return <>
    <RecordEditorDialog
      eyebrow={`FIN // ${item ? "EDITAR" : "NOVO"} ${noun}`}
      title={item?.title ?? (income ? "Novo recebimento" : "Nova conta")}
      meta={item && item.recurrence !== "once" ? "Alterações valem para todos os meses desta regra." : undefined}
      busy={busy} error={error} onCancel={onClose} onSubmit={submit}
      submitLabel={item ? "SALVAR ALTERAÇÕES" : "ADICIONAR"}
      actions={item ? <button type="button" className="record-editor__danger" disabled={busy} onClick={() => setConfirmDelete(true)}>EXCLUIR</button> : undefined}
    >
      <label className="record-editor__wide">Nome<input value={title} maxLength={200} placeholder={income ? "Ex.: mensalidade" : "Ex.: aluguel"} onChange={(event) => setTitle(event.target.value)} /></label>
      {income && <label className="record-editor__wide">Projeto<input value={project} list="money-projects" maxLength={80} placeholder="Ex.: giraComanda (opcional)" onChange={(event) => setProject(event.target.value)} />
        <datalist id="money-projects">{projects.map((name) => <option value={name} key={name} />)}</datalist></label>}
      <label>{recurrence === "installments" ? "Valor de cada parcela" : "Valor"}<input value={amount} inputMode="decimal" placeholder="0,00" onChange={(event) => setAmount(event.target.value)} /></label>
      <label>{recurrence === "once" ? (income ? "Data" : "Vencimento") : (income ? "Primeira data" : "Primeiro vencimento")}<input value={date} inputMode="numeric" placeholder="dd/mm/aaaa" maxLength={10} onChange={(event) => setDate(maskDate(event.target.value))} /></label>
      <label>Repetição<select value={recurrence} onChange={(event) => setRecurrence(event.target.value as MoneyRecurrence)}>{recurrences.map(([value, label]) => <option value={value} key={value}>{label}</option>)}</select></label>
      {recurrence === "installments" && <label>Parcelas<input type="number" min={2} max={480} value={installments} onChange={(event) => setInstallments(event.target.value)} /></label>}
    </RecordEditorDialog>
    {confirmDelete && item && <DeleteConfirmationDialog kind="lançamento" title={item.title}
      description={item.recurrence === "once" ? undefined : "Isso remove a regra inteira: todos os meses, passados e futuros."}
      busy={busy} onCancel={() => setConfirmDelete(false)} onConfirm={() => void run(() => moneyRepository.remove(item.id))} />}
  </>;
}
