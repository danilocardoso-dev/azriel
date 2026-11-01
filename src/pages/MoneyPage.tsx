import { useCallback, useEffect, useState } from "react";
import { MoneyDialog } from "../components/money/MoneyDialog";
import { formatDate, formatMoney, monthLabel, monthOf, shiftMonth } from "../money/format";
import { moneyRepository, type MoneyEntry, type MoneyItem, type MoneyKind, type MoneyMonth } from "../repositories/moneyRepository";
import { localDateKey } from "../services/dateService";
import "../styles/money.css";

const messageOf = (error: unknown) => error instanceof Error ? error.message : String(error);
interface Editing { item: MoneyItem | null; kind: MoneyKind }

export function MoneyPage() {
  const today = localDateKey();
  const [month, setMonth] = useState(monthOf(today));
  const [data, setData] = useState<MoneyMonth | null>(null);
  const [projects, setProjects] = useState<string[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [editing, setEditing] = useState<Editing | null>(null);

  const load = useCallback(async () => {
    try {
      const [next, names] = await Promise.all([moneyRepository.month(month), moneyRepository.projects()]);
      setData(next); setProjects(names); setError(null);
    } catch (reason) { setError(messageOf(reason)); }
  }, [month]);

  useEffect(() => {
    const first = window.setTimeout(() => void load(), 0);
    return () => window.clearTimeout(first);
  }, [load]);

  async function toggle(entry: MoneyEntry) {
    try { await moneyRepository.setPaid(entry.item.id, month, !entry.paid); await load(); }
    catch (reason) { setError(messageOf(reason)); }
  }

  const totals = data?.totals;
  const defaultDate = month === monthOf(today) ? today : `${month}-01`;
  const row = (entry: MoneyEntry, income: boolean) => {
    const late = !income && !entry.paid && entry.date < today;
    const check = entry.paid ? (income ? "Marcar como não recebido" : "Marcar como não paga") : (income ? "Marcar como recebido" : "Marcar como paga");
    return <li key={entry.item.id} className="money-row" data-paid={entry.paid}>
      <button type="button" className="money-row__check" aria-pressed={entry.paid} aria-label={check} onClick={() => void toggle(entry)}>{entry.paid ? "✓" : ""}</button>
      <button type="button" className="money-row__body" onClick={() => setEditing({ item: entry.item, kind: entry.item.kind })}>
        <strong>{entry.item.title}</strong>
        <small>
          {formatDate(entry.date)}{entry.label ? ` · ${entry.label}` : ""}{entry.item.project ? ` · ${entry.item.project}` : ""}
          {late && <em> · ATRASADA</em>}
        </small>
      </button>
      <b>{formatMoney(entry.item.amountCents)}</b>
    </li>;
  };

  return <section className="module-page money-page">
    <header className="money-head">
      <div><span>MÓDULO // FIN</span><h1>Finanças</h1></div>
      <nav className="money-month" aria-label="Mês">
        <button type="button" onClick={() => setMonth((current) => shiftMonth(current, -1))} aria-label="Mês anterior">‹</button>
        <strong>{monthLabel(month)}</strong>
        <button type="button" onClick={() => setMonth((current) => shiftMonth(current, 1))} aria-label="Próximo mês">›</button>
        {month !== monthOf(today) && <button type="button" className="money-month__today" onClick={() => setMonth(monthOf(today))}>HOJE</button>}
      </nav>
    </header>

    {error && <p className="money-error" role="alert">{error}</p>}

    <div className="money-totals">
      <article><span>GASTOS DO MÊS</span><strong>{formatMoney(totals?.expensesCents ?? 0)}</strong><small>pago {formatMoney(totals?.expensesPaidCents ?? 0)}</small></article>
      <article><span>RECEBIDO</span><strong>{formatMoney(totals?.incomeReceivedCents ?? 0)}</strong><small>previsto {formatMoney(totals?.incomeCents ?? 0)}</small></article>
      <article data-negative={(totals?.balanceCents ?? 0) < 0}><span>SALDO PREVISTO</span><strong>{formatMoney(totals?.balanceCents ?? 0)}</strong><small>recebimentos − gastos</small></article>
    </div>

    <div className="money-columns">
      <section className="money-panel">
        <header><h2>Contas</h2><button type="button" onClick={() => setEditing({ item: null, kind: "expense" })}>+ CONTA</button></header>
        {data?.expenses.length ? <ul>{data.expenses.map((entry) => row(entry, false))}</ul> : <p className="money-empty">Nenhuma conta neste mês.</p>}
      </section>
      <section className="money-panel">
        <header><h2>Recebimentos</h2><button type="button" onClick={() => setEditing({ item: null, kind: "income" })}>+ RECEBIMENTO</button></header>
        {data?.income.length ? <ul>{data.income.map((entry) => row(entry, true))}</ul> : <p className="money-empty">Nenhum recebimento neste mês.</p>}
      </section>
    </div>

    <section className="money-panel money-projects">
      <header><h2>Por projeto</h2><small>média = total recebido ÷ meses desde o primeiro recebimento do projeto</small></header>
      {data?.projects.length ? <table>
        <thead><tr><th>Projeto</th><th>No mês</th><th>Recebido</th><th>Média/mês</th><th>Total</th></tr></thead>
        <tbody>{data.projects.map((project) => <tr key={project.name}>
          <td>{project.name}</td><td>{formatMoney(project.monthCents)}</td><td>{formatMoney(project.receivedCents)}</td><td>{formatMoney(project.averageCents)}</td><td>{formatMoney(project.totalCents)}</td>
        </tr>)}</tbody>
      </table> : <p className="money-empty">Dê um projeto a um recebimento (ex.: giraComanda) para ver a média por projeto.</p>}
    </section>

    {editing && <MoneyDialog key={editing.item?.id ?? editing.kind} item={editing.item} kind={editing.kind} defaultDate={defaultDate} projects={projects} onClose={() => setEditing(null)} onSaved={() => void load()} />}
  </section>;
}
