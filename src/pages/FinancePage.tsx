import { useCallback, useEffect, useMemo, useState } from "react";
import { DataState } from "../components/layout/DataState";
import { ModuleIntro } from "../components/layout/ModuleIntro";
import { CommitmentSummaryCards, CommitmentsPanel, OriginsPanel } from "../components/finance/FinanceV2";
import { FinanceAlerts, PlanningPanel, ProjectionsPanel } from "../components/finance/FinancePlanning";
import { financeService, formatBrazilianDate, formatCurrency, formatCurrencyInput, formatMonth, maskBrazilianDate, parseBrazilianDate, parseCurrencyToCents, shiftMonth } from "../services/financeService";
import { localDateKey } from "../services/dateService";
import type {
  FinancialCategory, FinancialCategoryInput, FinancialDashboard, FinancialMonthlyReport,
  FinancialScope, FinancialStatus, FinancialTransaction, FinancialTransactionInput,
  FinancialTransactionType, FinancialView, FinanceAlert, FinanceCommitmentSummary, FinanceContract,
} from "../types";

type FinanceTab = "overview" | "transactions" | "commitments" | "origins" | "planning" | "projections" | "reports" | "categories";

const currentMonth = () => localDateKey().slice(0, 7);
const monthBounds = (month: string) => {
  const [year, value] = month.split("-").map(Number);
  const last = new Date(year, value, 0).getDate();
  return [`${month}-01`, `${month}-${String(last).padStart(2, "0")}`];
};
const newId = (prefix: string) => `${prefix}-${crypto.randomUUID()}`;

function Totals({ title, values, tone = "" }: { title: string; values: { incomeCents: number; expenseCents: number; resultCents: number }; tone?: string }) {
  return <article className={`finance-total ${tone}`}><span>{title}</span><div><small>ENTRADAS</small><strong>{formatCurrency(values.incomeCents)}</strong></div><div><small>SAÍDAS</small><strong>{formatCurrency(values.expenseCents)}</strong></div><div><small>RESULTADO</small><strong className={values.resultCents < 0 ? "negative" : "positive"}>{formatCurrency(values.resultCents)}</strong></div></article>;
}

function BalanceSummary({ values }: { values: { incomeCents: number; expenseCents: number; resultCents: number } }) {
  return <div className="finance-balance-summary"><article><span>ENTRADAS</span><strong className="positive">{formatCurrency(values.incomeCents)}</strong><small>RECEITAS DO PERÍODO</small></article><article><span>SAÍDAS</span><strong className="negative">{formatCurrency(values.expenseCents)}</strong><small>DESPESAS DO PERÍODO</small></article><article className="finance-balance-summary__remaining"><span>RESTANTE</span><strong className={values.resultCents < 0 ? "negative" : "positive"}>{formatCurrency(values.resultCents)}</strong><small>ENTRADAS − SAÍDAS</small></article></div>;
}

function ScopeBalance({ title, values }: { title: string; values: { incomeCents: number; expenseCents: number; resultCents: number } }) {
  return <article className="finance-scope-balance"><strong>{title}</strong><span><small>ENTRADAS</small><b>{formatCurrency(values.incomeCents)}</b></span><span><small>SAÍDAS</small><b>{formatCurrency(values.expenseCents)}</b></span><span><small>RESTANTE</small><b className={values.resultCents < 0 ? "negative" : "positive"}>{formatCurrency(values.resultCents)}</b></span></article>;
}

function TransactionForm({ categories, transaction, onClose, onSaved }: { categories: FinancialCategory[]; transaction: FinancialTransaction | null; onClose: () => void; onSaved: () => Promise<void> }) {
  const [type, setType] = useState<FinancialTransactionType>(transaction?.transactionType ?? "EXPENSE");
  const [scope, setScope] = useState<FinancialScope>(transaction?.scope ?? "PERSONAL");
  const available = categories.filter((item) => item.scope === scope && (item.enabled || item.id === transaction?.categoryId));
  const [description, setDescription] = useState(transaction?.description ?? "");
  const [amount, setAmount] = useState(transaction ? formatCurrencyInput(transaction.amountCents) : "");
  const [categoryId, setCategoryId] = useState(transaction?.categoryId ?? "");
  const [competenceDate, setCompetenceDate] = useState(formatBrazilianDate(transaction?.competenceDate ?? localDateKey()));
  const [dueDate, setDueDate] = useState(formatBrazilianDate(transaction?.dueDate ?? null));
  const [paidAt, setPaidAt] = useState(formatBrazilianDate(transaction?.paidAt ?? null));
  const [status, setStatus] = useState<Exclude<FinancialStatus, "CANCELLED">>(transaction?.status === "PAID" ? "PAID" : "PLANNED");
  const [notes, setNotes] = useState(transaction?.notes ?? "");
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const readonly = transaction?.status === "CANCELLED";
  const effectiveCategoryId = available.some((item) => item.id === categoryId) ? categoryId : (available[0]?.id ?? "");

  const submit = async (event: React.FormEvent) => {
    event.preventDefault(); setError(null); setSaving(true);
    try {
      const input: FinancialTransactionInput = {
        id: transaction?.id ?? newId("finance-entry"), transactionType: type, scope,
        description, amountCents: parseCurrencyToCents(amount), categoryId: effectiveCategoryId,
        competenceDate: parseBrazilianDate(competenceDate, "a data de competência")!,
        dueDate: parseBrazilianDate(dueDate, "a data de vencimento", false),
        paidAt: status === "PAID" ? parseBrazilianDate(paidAt || formatBrazilianDate(localDateKey()), "a data de pagamento ou recebimento") : null,
        status, notes: notes || null,
      };
      await financeService.saveTransaction(input); await onSaved(); onClose();
    } catch (reason) { setError(String(reason)); } finally { setSaving(false); }
  };

  return <div className="finance-modal" role="dialog" aria-modal="true"><form onSubmit={submit}>
    <header><div><span>LANÇAMENTO FINANCEIRO</span><h2>{transaction ? "Editar lançamento" : "Novo lançamento"}</h2></div><button type="button" onClick={onClose}>×</button></header>
    {readonly && <p className="finance-alert">Lançamentos cancelados permanecem somente para auditoria e não podem ser alterados.</p>}
    {error && <p className="finance-error">{error}</p>}
    <div className="finance-form-grid">
      <label className="wide">DESCRIÇÃO<input value={description} onChange={(event) => setDescription(event.target.value)} disabled={readonly} /></label>
      <label>TIPO<select value={type} onChange={(event) => setType(event.target.value as FinancialTransactionType)} disabled={readonly}><option value="INCOME">ENTRADA</option><option value="EXPENSE">SAÍDA</option></select></label>
      <label>CONTEXTO<select value={scope} onChange={(event) => { setScope(event.target.value as FinancialScope); setCategoryId(""); }} disabled={readonly}><option value="PERSONAL">PESSOAL</option><option value="PROFESSIONAL">PROFISSIONAL</option></select></label>
      <label>VALOR (R$)<input inputMode="decimal" value={amount} onChange={(event) => setAmount(event.target.value)} onBlur={() => { try { setAmount(formatCurrencyInput(parseCurrencyToCents(amount))); } catch { /* A validação completa é exibida ao salvar. */ } }} disabled={readonly} placeholder="0,00" /></label>
      <label>CATEGORIA<select value={effectiveCategoryId} onChange={(event) => setCategoryId(event.target.value)} disabled={readonly}>{!available.length && <option value="">SEM CATEGORIAS ATIVAS</option>}{available.map((item) => <option value={item.id} key={item.id}>{item.name}</option>)}</select></label>
      <label>COMPETÊNCIA<input className="finance-date-input" inputMode="numeric" maxLength={10} value={competenceDate} onChange={(event) => setCompetenceDate(maskBrazilianDate(event.target.value))} disabled={readonly} placeholder="DD/MM/AAAA" /></label>
      <label>VENCIMENTO<input className="finance-date-input" inputMode="numeric" maxLength={10} value={dueDate} onChange={(event) => setDueDate(maskBrazilianDate(event.target.value))} disabled={readonly} placeholder="DD/MM/AAAA" /></label>
      <label>STATUS<select value={status} onChange={(event) => setStatus(event.target.value as Exclude<FinancialStatus, "CANCELLED">)} disabled={readonly}><option value="PLANNED">PLANEJADO</option><option value="PAID">REALIZADO</option></select></label>
      <label>PAGAMENTO / RECEBIMENTO<input className="finance-date-input" inputMode="numeric" maxLength={10} value={paidAt} onChange={(event) => setPaidAt(maskBrazilianDate(event.target.value))} disabled={readonly || status !== "PAID"} placeholder="DD/MM/AAAA" /></label>
      <label className="wide">OBSERVAÇÕES<textarea value={notes} onChange={(event) => setNotes(event.target.value)} disabled={readonly} /></label>
    </div>
    <footer><button type="button" onClick={onClose}>{readonly ? "FECHAR" : "CANCELAR"}</button>{!readonly && <button className="primary" disabled={saving}>{saving ? "SALVANDO" : "SALVAR LANÇAMENTO"}</button>}</footer>
  </form></div>;
}

export function FinancePage() {
  const [tab, setTab] = useState<FinanceTab>("overview");
  const [month, setMonth] = useState(currentMonth);
  const [view, setView] = useState<FinancialView>("COMPETENCE");
  const [categories, setCategories] = useState<FinancialCategory[]>([]);
  const [transactions, setTransactions] = useState<FinancialTransaction[]>([]);
  const [dashboard, setDashboard] = useState<FinancialDashboard | null>(null);
  const [report, setReport] = useState<FinancialMonthlyReport | null>(null);
  const [commitmentSummary, setCommitmentSummary] = useState<FinanceCommitmentSummary | null>(null);
  const [alerts, setAlerts] = useState<FinanceAlert[]>([]);
  const [contracts, setContracts] = useState<FinanceContract[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [editing, setEditing] = useState<FinancialTransaction | "new" | null>(null);
  const [query, setQuery] = useState("");
  const [statusFilter, setStatusFilter] = useState<FinancialStatus | "ALL">("ALL");
  const [categoryFilter, setCategoryFilter] = useState("ALL");
  const [typeFilter, setTypeFilter] = useState<FinancialTransactionType | "ALL">("ALL");
  const [scopeFilter, setScopeFilter] = useState<FinancialScope | "ALL">("ALL");
  const [summaryStatus, setSummaryStatus] = useState<"REALIZED" | "PLANNED">("REALIZED");

  const reload = useCallback(async () => {
    setLoading(true); setError(null);
    try {
      const [from, to] = monthBounds(month);
      const [nextCategories, nextTransactions, nextDashboard, nextReport, nextCommitmentSummary, nextAlerts, nextContracts] = await Promise.all([
        financeService.listCategories(true),
        financeService.listTransactions({ dateFrom: from, dateTo: to, view }),
        financeService.dashboard(month, view, localDateKey()),
        financeService.monthlyReport(month, view),
        financeService.commitmentSummary(localDateKey()),
        financeService.alerts(localDateKey(), month, view, 7),
        financeService.listContracts(),
      ]);
      setCategories(nextCategories); setTransactions(nextTransactions); setDashboard(nextDashboard); setReport(nextReport); setCommitmentSummary(nextCommitmentSummary); setAlerts(nextAlerts); setContracts(nextContracts);
    } catch (reason) { setError(String(reason)); } finally { setLoading(false); }
  }, [month, view]);
  useEffect(() => {
    const timer = window.setTimeout(() => void reload(), 0);
    return () => window.clearTimeout(timer);
  }, [reload]);

  const filtered = useMemo(() => transactions.filter((item) => {
    const matchesStatus = statusFilter === "ALL" || item.status === statusFilter;
    const matchesCategory = categoryFilter === "ALL" || item.categoryId === categoryFilter;
    const matchesType = typeFilter === "ALL" || item.transactionType === typeFilter;
    const matchesScope = scopeFilter === "ALL" || item.scope === scopeFilter;
    const term = query.trim().toLocaleLowerCase("pt-BR");
    return matchesStatus && matchesCategory && matchesType && matchesScope && (!term || `${item.description} ${item.categoryName} ${item.notes ?? ""}`.toLocaleLowerCase("pt-BR").includes(term));
  }), [categoryFilter, query, scopeFilter, statusFilter, transactions, typeFilter]);

  const markPaid = async (item: FinancialTransaction) => { await financeService.markTransactionPaid(item.id, localDateKey()); await reload(); };
  const cancel = async (item: FinancialTransaction) => { if (window.confirm(`Cancelar o lançamento “${item.description}”?`)) { await financeService.cancelTransaction(item.id); await reload(); } };

  return <div className="finance-page">
    <ModuleIntro code="FIN-01" title="Finanças" description="Controle financeiro local, pessoal e profissional, com separação entre competência e caixa." metric={dashboard ? `${formatCurrency(dashboard.realized.resultCents)} REALIZADO` : "SQLITE LOCAL"} />
    <div className="finance-toolbar">
      <nav>{(["overview","transactions","commitments","origins","planning","projections","reports","categories"] as FinanceTab[]).map((item) => <button className={tab === item ? "active" : ""} onClick={() => setTab(item)} key={item}>{({ overview:"VISÃO GERAL",transactions:"LANÇAMENTOS",commitments:"COMPROMISSOS",origins:"ORIGENS",planning:"PLANEJAMENTO",projections:"PROJEÇÕES",reports:"RELATÓRIOS",categories:"CATEGORIAS" })[item]}</button>)}</nav>
      <div className="finance-month-control" aria-label="Navegação mensal"><span>MÊS</span><div><button type="button" onClick={() => setMonth((value) => shiftMonth(value, -1))} aria-label="Mês anterior">‹</button><strong>{formatMonth(month).toLocaleUpperCase("pt-BR")}</strong><button type="button" onClick={() => setMonth((value) => shiftMonth(value, 1))} aria-label="Próximo mês">›</button></div>{month !== currentMonth() && <button className="finance-month-control__today" type="button" onClick={() => setMonth(currentMonth())}>MÊS ATUAL</button>}</div>
      <label>VISÃO<select value={view} onChange={(event) => setView(event.target.value as FinancialView)}><option value="COMPETENCE">COMPETÊNCIA</option><option value="CASH">CAIXA</option></select></label>
      <button className="primary" onClick={() => setEditing("new")}>＋ NOVO LANÇAMENTO</button>
    </div>
    {loading || error ? <DataState loading={loading} error={error} onRetry={() => void reload()} /> : <>
      {tab === "overview" && dashboard && <section className="finance-overview">
        <header><div><span>PAINEL MENSAL</span><h2>{formatMonth(month)}</h2></div><p>Resumo do período. O restante representa entradas menos saídas, não o saldo de uma conta bancária.</p></header>
        <div className="finance-summary-switch" aria-label="Tipo de resumo"><button className={summaryStatus === "REALIZED" ? "active" : ""} onClick={() => setSummaryStatus("REALIZED")}>REALIZADO</button><button className={summaryStatus === "PLANNED" ? "active" : ""} onClick={() => setSummaryStatus("PLANNED")}>PLANEJADO</button></div>
        <BalanceSummary values={summaryStatus === "REALIZED" ? dashboard.realized : dashboard.planned} />
        <div className="finance-scope-summary"><ScopeBalance title="PESSOAL" values={summaryStatus === "REALIZED" ? dashboard.personalRealized : dashboard.personalPlanned} /><ScopeBalance title="PROFISSIONAL" values={summaryStatus === "REALIZED" ? dashboard.professionalRealized : dashboard.professionalPlanned} /></div>
        <CommitmentSummaryCards summary={commitmentSummary} />
        <FinanceAlerts items={alerts} />
        <div className="finance-upcoming"><header><strong>PRÓXIMOS VENCIMENTOS</strong><span>{dashboard.upcomingDue.length} REGISTROS</span></header>{dashboard.upcomingDue.length ? dashboard.upcomingDue.map((item) => <button key={item.id} onClick={() => setEditing(item)}><span><strong>{item.description}</strong><small>{item.categoryName} · {item.scope === "PERSONAL" ? "PESSOAL" : "PROFISSIONAL"}</small></span><span><strong>{formatCurrency(item.amountCents)}</strong><small>{formatBrazilianDate(item.dueDate) || "SEM VENCIMENTO"}</small></span></button>) : <p>Nenhum vencimento planejado no período.</p>}</div>
      </section>}
      {tab === "transactions" && <section className="finance-transactions">
        <header><div><span>REGISTRO FINANCEIRO</span><h2>{transactions.length} lançamentos em {formatMonth(month)}</h2></div><div><input placeholder="Buscar descrição ou categoria" value={query} onChange={(event) => setQuery(event.target.value)} /><select value={statusFilter} onChange={(event) => setStatusFilter(event.target.value as FinancialStatus | "ALL")}><option value="ALL">TODOS OS STATUS</option><option value="PLANNED">PLANEJADOS</option><option value="PAID">REALIZADOS</option><option value="CANCELLED">CANCELADOS</option></select></div></header>
        <div className="finance-filter-row"><select value={categoryFilter} onChange={(event) => setCategoryFilter(event.target.value)}><option value="ALL">TODAS AS CATEGORIAS</option>{categories.map((item) => <option value={item.id} key={item.id}>{item.name}</option>)}</select><select value={typeFilter} onChange={(event) => setTypeFilter(event.target.value as FinancialTransactionType | "ALL")}><option value="ALL">ENTRADAS E SAÍDAS</option><option value="INCOME">ENTRADAS</option><option value="EXPENSE">SAÍDAS</option></select><select value={scopeFilter} onChange={(event) => setScopeFilter(event.target.value as FinancialScope | "ALL")}><option value="ALL">PESSOAL E PROFISSIONAL</option><option value="PERSONAL">PESSOAL</option><option value="PROFESSIONAL">PROFISSIONAL</option></select><span>{filtered.length} RESULTADOS</span></div>
        <div className="finance-table"><div className="finance-table__head"><span>DESCRIÇÃO</span><span>CATEGORIA</span><span>CONTEXTO</span><span>STATUS</span><span>VALOR</span><span>AÇÕES</span></div>{filtered.length ? filtered.map((item) => <article key={item.id} className={item.status === "CANCELLED" ? "cancelled" : ""}><button className="finance-description" onClick={() => setEditing(item)}><strong>{item.description}</strong><small>{view === "CASH" ? (formatBrazilianDate(item.paidAt ?? item.dueDate) || "SEM DATA DE CAIXA") : formatBrazilianDate(item.competenceDate)}</small></button><span>{item.categoryName}</span><span>{item.scope === "PERSONAL" ? "PESSOAL" : "PROFISSIONAL"}</span><b className={`status-${item.status.toLowerCase()}`}>{item.status === "PAID" ? "REALIZADO" : item.status === "PLANNED" ? "PLANEJADO" : "CANCELADO"}</b><strong className={item.transactionType === "INCOME" ? "positive" : "negative"}>{item.transactionType === "INCOME" ? "+" : "−"}{formatCurrency(item.amountCents)}</strong><nav>{item.status === "PLANNED" && <button onClick={() => void markPaid(item)}>REALIZAR</button>}<button onClick={() => setEditing(item)}>ABRIR</button>{item.status !== "CANCELLED" && <button className="danger" onClick={() => void cancel(item)}>CANCELAR</button>}</nav></article>) : <p className="finance-empty">Nenhum lançamento corresponde aos filtros.</p>}</div>
      </section>}
      {tab === "commitments" && <CommitmentsPanel transactions={transactions} onChanged={reload} />}
      {tab === "origins" && <OriginsPanel categories={categories} onChanged={reload} />}
      {tab === "planning" && <PlanningPanel month={month} view={view} categories={categories} alerts={alerts} onChanged={reload} />}
      {tab === "projections" && <ProjectionsPanel view={view} contracts={contracts} />}
      {tab === "categories" && <CategoryManager categories={categories} onChanged={reload} />}
      {tab === "reports" && report && <section className="finance-reports"><header><span>RELATÓRIO MENSAL</span><h2>{formatMonth(month)} · {view === "COMPETENCE" ? "COMPETÊNCIA" : "CAIXA"}</h2></header><div className="finance-total-grid"><Totals title="REALIZADO" values={report.realized} tone="realized" /><Totals title="PLANEJADO" values={report.planned} /></div><div className="finance-total-grid"><Totals title="PESSOAL · REALIZADO" values={report.personal} /><Totals title="PROFISSIONAL · REALIZADO" values={report.professional} /></div><div className="finance-category-report"><header><strong>SAÍDAS POR CATEGORIA</strong><span>PARTICIPAÇÃO NO TOTAL REALIZADO</span></header>{report.categoryExpenses.length ? report.categoryExpenses.map((item) => <article key={item.categoryId}><span><strong>{item.categoryName}</strong><small>{item.scope === "PERSONAL" ? "PESSOAL" : "PROFISSIONAL"}</small></span><div><i style={{ width: `${(item.shareBasisPoints ?? 0) / 100}%` }} /></div><b>{formatCurrency(item.amountCents)} · {((item.shareBasisPoints ?? 0) / 100).toFixed(1)}%</b></article>) : <p>Nenhuma saída realizada no mês.</p>}</div><div className="finance-comparison"><span>MÊS ANTERIOR · {formatMonth(report.previousMonth)}</span>{report.previousHasData ? <><strong>{formatCurrency(report.previousRealized.resultCents)}</strong><small>Variação do resultado: {formatCurrency(report.resultChangeCents)}{report.resultChangeBasisPoints === null ? "" : ` · ${(report.resultChangeBasisPoints / 100).toFixed(1)}%`}</small></> : <strong>SEM BASE PARA COMPARAÇÃO</strong>}</div></section>}
    </>}
    {editing && <TransactionForm categories={categories} transaction={editing === "new" ? null : editing} onClose={() => setEditing(null)} onSaved={reload} />}
  </div>;
}

function CategoryManager({ categories, onChanged }: { categories: FinancialCategory[]; onChanged: () => Promise<void> }) {
  const [editing, setEditing] = useState<FinancialCategory | null>(null);
  const [name, setName] = useState(""); const [scope, setScope] = useState<FinancialScope>("PERSONAL"); const [error, setError] = useState<string | null>(null);
  const save = async (event: React.FormEvent) => { event.preventDefault(); setError(null); try { const input: FinancialCategoryInput = { id: editing?.id ?? newId("finance-category"), name, scope, enabled: editing?.enabled ?? true }; await financeService.saveCategory(input); setEditing(null); setName(""); await onChanged(); } catch (reason) { setError(String(reason)); } };
  const edit = (item: FinancialCategory) => { setEditing(item); setName(item.name); setScope(item.scope); setError(null); };
  return <section className="finance-categories"><header><div><span>CLASSIFICAÇÃO</span><h2>Categorias financeiras</h2><p>Categorias utilizadas não são excluídas: elas podem ser desativadas para preservar o histórico.</p></div></header><form onSubmit={save}><label>NOME<input value={name} onChange={(event) => setName(event.target.value)} /></label><label>CONTEXTO<select value={scope} onChange={(event) => setScope(event.target.value as FinancialScope)} disabled={Boolean(editing)}><option value="PERSONAL">PESSOAL</option><option value="PROFESSIONAL">PROFISSIONAL</option></select></label><button className="primary">{editing ? "SALVAR ALTERAÇÃO" : "CRIAR CATEGORIA"}</button>{editing && <button type="button" onClick={() => { setEditing(null); setName(""); }}>CANCELAR</button>}{error && <p>{error}</p>}</form><div className="finance-category-list">{categories.map((item) => <article className={!item.enabled ? "disabled" : ""} key={item.id}><span><strong>{item.name}</strong><small>{item.scope === "PERSONAL" ? "PESSOAL" : "PROFISSIONAL"}</small></span><b>{item.enabled ? "ATIVA" : "INATIVA"}</b><button onClick={() => edit(item)}>EDITAR</button><button onClick={() => void financeService.setCategoryEnabled(item.id, !item.enabled).then(onChanged)}>{item.enabled ? "DESATIVAR" : "ATIVAR"}</button></article>)}</div></section>;
}
