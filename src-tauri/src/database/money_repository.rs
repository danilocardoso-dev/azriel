use super::money_models::*;
use rusqlite::{params, Connection};
use std::collections::{HashMap, HashSet};

const COLUMNS: &str = "id,kind,title,amount_cents,first_date,recurrence,installments,project";
const MAX_INSTALLMENTS: i64 = 480;
/// Teto de meses percorridos ao somar um projeto (50 anos), só para não varrer sem fim.
const MAX_SPAN: i64 = 600;
fn err(error: rusqlite::Error) -> String { error.to_string() }

// ---------- datas (puras) ----------

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        _ if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 => 29,
        _ => 28,
    }
}

fn parse_month(text: &str) -> Result<(i32, u32), String> {
    let invalid = || format!("Mês inválido: {text}");
    let (year, month) = text.split_once('-').ok_or_else(invalid)?;
    if year.len() != 4 || month.len() != 2 { return Err(invalid()); }
    let (year, month) = (year.parse::<i32>().map_err(|_| invalid())?, month.parse::<u32>().map_err(|_| invalid())?);
    if !(1..=12).contains(&month) || year < 1900 { return Err(invalid()); }
    Ok((year, month))
}

fn parse_date(text: &str) -> Result<(i32, u32, u32), String> {
    let invalid = || format!("Data inválida: {text}");
    let mut parts = text.split('-');
    let (year, month, day) = (parts.next().ok_or_else(invalid)?, parts.next().ok_or_else(invalid)?, parts.next().ok_or_else(invalid)?);
    if parts.next().is_some() || year.len() != 4 || month.len() != 2 || day.len() != 2 { return Err(invalid()); }
    let (year, month, day) = (year.parse::<i32>().map_err(|_| invalid())?, month.parse::<u32>().map_err(|_| invalid())?, day.parse::<u32>().map_err(|_| invalid())?);
    if !(1..=12).contains(&month) || year < 1900 || day == 0 || day > days_in_month(year, month) { return Err(invalid()); }
    Ok((year, month, day))
}

fn month_index(year: i32, month: u32) -> i64 { i64::from(year) * 12 + i64::from(month) - 1 }

fn start_index(item: &MoneyItem) -> Option<(i64, u32)> {
    let (year, month, day) = parse_date(&item.first_date).ok()?;
    Some((month_index(year, month), day))
}

/// Data e rótulo da ocorrência da regra no mês `index`, se houver. O dia é ajustado ao tamanho do mês (31 → 28 em fevereiro).
fn occurrence(item: &MoneyItem, index: i64) -> Option<(String, String)> {
    let (start, day) = start_index(item)?;
    let offset = index - start;
    if offset < 0 { return None; }
    let label = match item.recurrence.as_str() {
        "once" if offset == 0 => String::new(),
        "monthly" => "permanente".to_string(),
        "installments" => {
            let total = item.installments.unwrap_or(1);
            if offset >= total { return None; }
            format!("{}/{}", offset + 1, total)
        }
        _ => return None,
    };
    let (year, month) = (index.div_euclid(12) as i32, (index.rem_euclid(12) + 1) as u32);
    Some((format!("{year:04}-{month:02}-{:02}", day.min(days_in_month(year, month))), label))
}

// ---------- gravação ----------

fn validate(input: &MoneyItemInput) -> Result<MoneyItemInput, String> {
    if input.id.trim().is_empty() { return Err("Lançamento sem identificador".into()); }
    if !["expense", "income"].contains(&input.kind.as_str()) { return Err("Tipo inválido: use despesa ou recebimento".into()); }
    let title = input.title.trim();
    if title.is_empty() { return Err("Dê um nome ao lançamento.".into()); }
    if title.chars().count() > 200 { return Err("O nome pode ter no máximo 200 caracteres.".into()); }
    if input.amount_cents <= 0 { return Err("O valor precisa ser maior que zero.".into()); }
    parse_date(&input.first_date)?;
    if !["once", "monthly", "installments"].contains(&input.recurrence.as_str()) { return Err("Repetição inválida".into()); }
    let installments = if input.recurrence == "installments" {
        let count = input.installments.unwrap_or(0);
        if !(2..=MAX_INSTALLMENTS).contains(&count) { return Err(format!("Parcelas: informe de 2 a {MAX_INSTALLMENTS}.")); }
        Some(count)
    } else { None };
    let project = input.project.as_deref().map(str::trim).filter(|name| !name.is_empty()).map(|name| name.chars().take(80).collect::<String>());
    Ok(MoneyItemInput { id: input.id.trim().into(), kind: input.kind.clone(), title: title.into(), amount_cents: input.amount_cents, first_date: input.first_date.clone(), recurrence: input.recurrence.clone(), installments, project })
}

pub fn save_item(c: &Connection, input: &MoneyItemInput) -> Result<MoneyItem, String> {
    let input = validate(input)?;
    c.execute(
        "INSERT INTO money_items(id,kind,title,amount_cents,first_date,recurrence,installments,project) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)
         ON CONFLICT(id) DO UPDATE SET kind=?2,title=?3,amount_cents=?4,first_date=?5,recurrence=?6,installments=?7,project=?8,updated_at=CURRENT_TIMESTAMP",
        params![input.id, input.kind, input.title, input.amount_cents, input.first_date, input.recurrence, input.installments, input.project],
    ).map_err(err)?;
    Ok(MoneyItem { id: input.id, kind: input.kind, title: input.title, amount_cents: input.amount_cents, first_date: input.first_date, recurrence: input.recurrence, installments: input.installments, project: input.project })
}

pub fn delete_item(c: &Connection, id: &str) -> Result<(), String> {
    c.execute("DELETE FROM money_marks WHERE item_id=?1", [id]).map_err(err)?;
    if c.execute("DELETE FROM money_items WHERE id=?1", [id]).map_err(err)? == 0 { return Err("Lançamento não encontrado".into()); }
    Ok(())
}

/// Marca (ou desmarca) uma ocorrência como paga/recebida; cada mês de uma regra tem a sua marca.
pub fn set_paid(c: &Connection, id: &str, month: &str, paid: bool) -> Result<(), String> {
    let (year, number) = parse_month(month)?;
    let month = format!("{year:04}-{number:02}");
    let exists: i64 = c.query_row("SELECT COUNT(*) FROM money_items WHERE id=?1", [id], |row| row.get(0)).map_err(err)?;
    if exists == 0 { return Err("Lançamento não encontrado".into()); }
    let result = if paid { c.execute("INSERT OR IGNORE INTO money_marks(item_id,month) VALUES(?1,?2)", params![id, month]) } else { c.execute("DELETE FROM money_marks WHERE item_id=?1 AND month=?2", params![id, month]) };
    result.map_err(err)?;
    Ok(())
}

// ---------- leitura ----------

pub fn list_items(c: &Connection) -> Result<Vec<MoneyItem>, String> {
    let mut statement = c.prepare(&format!("SELECT {COLUMNS} FROM money_items ORDER BY created_at,id")).map_err(err)?;
    let rows = statement.query_map([], |row| Ok(MoneyItem {
        id: row.get(0)?, kind: row.get(1)?, title: row.get(2)?, amount_cents: row.get(3)?, first_date: row.get(4)?,
        recurrence: row.get(5)?, installments: row.get(6)?, project: row.get(7)?,
    })).map_err(err)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(err)
}

pub fn project_names(c: &Connection) -> Result<Vec<String>, String> {
    let mut statement = c.prepare("SELECT DISTINCT project FROM money_items WHERE project IS NOT NULL AND project<>'' ORDER BY project COLLATE NOCASE").map_err(err)?;
    let rows = statement.query_map([], |row| row.get::<_, String>(0)).map_err(err)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(err)
}

fn paid_marks(c: &Connection, month: &str) -> Result<HashSet<String>, String> {
    let mut statement = c.prepare("SELECT item_id FROM money_marks WHERE month=?1").map_err(err)?;
    let rows = statement.query_map([month], |row| row.get::<_, String>(0)).map_err(err)?;
    rows.collect::<Result<HashSet<_>, _>>().map_err(err)
}

/// Tudo de um mês: despesas, recebimentos, totais e o resumo por projeto.
pub fn month_view(c: &Connection, month: &str) -> Result<MoneyMonth, String> {
    let (year, number) = parse_month(month)?;
    let month = format!("{year:04}-{number:02}");
    let index = month_index(year, number);
    let items = list_items(c)?;
    let paid = paid_marks(c, &month)?;

    let mut expenses = Vec::new();
    let mut income = Vec::new();
    for item in &items {
        let Some((date, label)) = occurrence(item, index) else { continue };
        let entry = MoneyEntry { paid: paid.contains(&item.id), item: item.clone(), date, label };
        if item.kind == "income" { income.push(entry); } else { expenses.push(entry); }
    }
    let order = |a: &MoneyEntry, b: &MoneyEntry| a.date.cmp(&b.date).then_with(|| a.item.title.to_lowercase().cmp(&b.item.title.to_lowercase()));
    expenses.sort_by(order);
    income.sort_by(order);

    let sum = |entries: &[MoneyEntry], only_paid: bool| entries.iter().filter(|entry| !only_paid || entry.paid).map(|entry| entry.item.amount_cents).sum::<i64>();
    let totals = MoneyTotals {
        expenses_cents: sum(&expenses, false), expenses_paid_cents: sum(&expenses, true),
        income_cents: sum(&income, false), income_received_cents: sum(&income, true),
        balance_cents: sum(&income, false) - sum(&expenses, false),
    };
    Ok(MoneyMonth { projects: project_summaries(&items, &income, index), month, expenses, income, totals })
}

fn project_summaries(items: &[MoneyItem], income_of_month: &[MoneyEntry], index: i64) -> Vec<MoneyProjectSummary> {
    // Chave em minúsculas, para "GiraComanda" e "giracomanda" serem o mesmo projeto.
    let mut groups: HashMap<String, (String, Vec<&MoneyItem>)> = HashMap::new();
    for item in items.iter().filter(|item| item.kind == "income") {
        let Some(name) = item.project.as_deref() else { continue };
        groups.entry(name.to_lowercase()).or_insert_with(|| (name.to_string(), Vec::new())).1.push(item);
    }
    let mut summaries = Vec::new();
    for (name, group) in groups.into_values() {
        let Some(first) = group.iter().filter_map(|item| start_index(item).map(|(start, _)| start)).min() else { continue };
        if first > index { continue; }
        let span = (index - first + 1).min(MAX_SPAN);
        let total: i64 = group.iter().map(|item| (first.max(index - span + 1)..=index).filter(|month| occurrence(item, *month).is_some()).map(|_| item.amount_cents).sum::<i64>()).sum();
        let in_month = |entry: &&MoneyEntry| group.iter().any(|item| item.id == entry.item.id);
        let month_cents = income_of_month.iter().filter(in_month).map(|entry| entry.item.amount_cents).sum();
        let received_cents = income_of_month.iter().filter(in_month).filter(|entry| entry.paid).map(|entry| entry.item.amount_cents).sum();
        summaries.push(MoneyProjectSummary { name, month_cents, received_cents, average_cents: total / span, total_cents: total });
    }
    summaries.sort_by(|a, b| b.month_cents.cmp(&a.month_cents).then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase())));
    summaries
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(kind: &str, first: &str, recurrence: &str, installments: Option<i64>, project: Option<&str>, cents: i64) -> MoneyItem {
        MoneyItem { id: format!("{kind}-{first}-{recurrence}-{}", project.unwrap_or("")), kind: kind.into(), title: "x".into(), amount_cents: cents, first_date: first.into(), recurrence: recurrence.into(), installments, project: project.map(Into::into) }
    }
    fn input(recurrence: &str, installments: Option<i64>) -> MoneyItemInput {
        MoneyItemInput { id: "a".into(), kind: "expense".into(), title: " Aluguel ".into(), amount_cents: 150000, first_date: "2026-10-05".into(), recurrence: recurrence.into(), installments, project: Some("  ".into()) }
    }
    fn database() -> Connection {
        let mut connection = Connection::open_in_memory().unwrap();
        crate::database::initialize(&mut connection).unwrap();
        connection
    }
    fn idx(text: &str) -> i64 { let (y, m) = parse_month(text).unwrap(); month_index(y, m) }

    #[test]
    fn unica_so_aparece_no_proprio_mes() {
        let conta = item("expense", "2026-10-05", "once", None, None, 100);
        assert_eq!(occurrence(&conta, idx("2026-10")), Some(("2026-10-05".into(), String::new())));
        assert_eq!(occurrence(&conta, idx("2026-09")), None);
        assert_eq!(occurrence(&conta, idx("2026-11")), None);
    }

    #[test]
    fn permanente_reflete_nos_meses_seguintes_e_nao_nos_anteriores() {
        let conta = item("expense", "2026-10-05", "monthly", None, None, 100);
        assert_eq!(occurrence(&conta, idx("2026-09")), None);
        assert_eq!(occurrence(&conta, idx("2027-03")), Some(("2027-03-05".into(), "permanente".into())));
    }

    #[test]
    fn parcelada_numera_as_parcelas_e_termina() {
        let conta = item("expense", "2026-11-20", "installments", Some(3), None, 100);
        assert_eq!(occurrence(&conta, idx("2026-11")).unwrap().1, "1/3");
        assert_eq!(occurrence(&conta, idx("2027-01")), Some(("2027-01-20".into(), "3/3".into())));
        assert_eq!(occurrence(&conta, idx("2027-02")), None);
        assert_eq!(occurrence(&conta, idx("2026-10")), None);
    }

    #[test]
    fn dia_31_cai_no_ultimo_dia_do_mes_curto() {
        let conta = item("expense", "2026-01-31", "monthly", None, None, 100);
        assert_eq!(occurrence(&conta, idx("2026-02")).unwrap().0, "2026-02-28");
        assert_eq!(occurrence(&conta, idx("2028-02")).unwrap().0, "2028-02-29");
        assert_eq!(occurrence(&conta, idx("2026-04")).unwrap().0, "2026-04-30");
        assert_eq!(occurrence(&conta, idx("2026-12")).unwrap().0, "2026-12-31");
    }

    #[test]
    fn datas_e_meses_invalidos_sao_recusados() {
        assert!(parse_date("2026-02-30").is_err());
        assert!(parse_date("2026-13-01").is_err());
        assert!(parse_date("26-01-01").is_err());
        assert!(parse_date("2026-1-01").is_err());
        assert!(parse_month("2026-00").is_err());
        assert!(parse_month("2026").is_err());
        assert_eq!(idx("2027-01"), idx("2026-12") + 1);
    }

    #[test]
    fn validacao_limpa_e_recusa() {
        let ok = validate(&input("once", Some(9))).unwrap();
        assert_eq!((ok.title.as_str(), ok.installments, ok.project), ("Aluguel", None, None));
        assert!(validate(&input("installments", Some(1))).is_err());
        assert!(validate(&input("installments", None)).is_err());
        assert_eq!(validate(&input("installments", Some(12))).unwrap().installments, Some(12));
        assert!(validate(&MoneyItemInput { amount_cents: 0, ..input("once", None) }).is_err());
        assert!(validate(&MoneyItemInput { title: "  ".into(), ..input("once", None) }).is_err());
        assert!(validate(&MoneyItemInput { kind: "x".into(), ..input("once", None) }).is_err());
        assert!(validate(&MoneyItemInput { recurrence: "weekly".into(), ..input("once", None) }).is_err());
        assert!(validate(&MoneyItemInput { first_date: "2026-02-31".into(), ..input("once", None) }).is_err());
    }

    #[test]
    fn mes_soma_gastos_recebidos_e_saldo() {
        let c = database();
        save_item(&c, &MoneyItemInput { id: "aluguel".into(), ..input("monthly", None) }).unwrap();
        save_item(&c, &MoneyItemInput { id: "tv".into(), title: "TV".into(), amount_cents: 30000, first_date: "2026-09-10".into(), ..input("installments", Some(3)) }).unwrap();
        save_item(&c, &MoneyItemInput { id: "gira".into(), kind: "income".into(), title: "Gira".into(), amount_cents: 400000, project: Some("giraComanda".into()), ..input("once", None) }).unwrap();
        set_paid(&c, "aluguel", "2026-10", true).unwrap();
        set_paid(&c, "gira", "2026-10", true).unwrap();

        let october = month_view(&c, "2026-10").unwrap();
        assert_eq!(october.expenses.iter().map(|e| e.item.id.as_str()).collect::<Vec<_>>(), ["aluguel", "tv"]);
        assert_eq!(october.totals, MoneyTotals { expenses_cents: 180000, expenses_paid_cents: 150000, income_cents: 400000, income_received_cents: 400000, balance_cents: 220000 });
        assert!(october.expenses[0].paid && !october.expenses[1].paid);

        let november = month_view(&c, "2026-11").unwrap();
        assert_eq!(november.totals.expenses_cents, 180000);
        assert_eq!(november.totals.expenses_paid_cents, 0, "a marca de pago vale só para o mês marcado");
        assert_eq!(november.totals.income_cents, 0);
        let december = month_view(&c, "2026-12").unwrap();
        assert_eq!(december.totals.expenses_cents, 150000, "a parcela 3/3 da TV foi em novembro");
        assert!(month_view(&c, "2026-1").is_err());
    }

    #[test]
    fn projeto_tem_valor_do_mes_e_media_desde_o_primeiro_recebimento() {
        let items = vec![
            item("income", "2026-08-10", "installments", Some(3), Some("Mapcloid"), 90000),
            item("income", "2026-10-01", "once", None, Some("mapcloid"), 30000),
            item("income", "2026-10-15", "once", None, Some("giraComanda"), 50000),
            item("income", "2026-10-20", "once", None, None, 99999),
            item("expense", "2026-08-10", "monthly", None, Some("Mapcloid"), 1),
        ];
        let october: Vec<MoneyEntry> = items.iter().filter_map(|i| occurrence(i, idx("2026-10")).map(|(date, label)| MoneyEntry { item: i.clone(), date, label, paid: false })).filter(|e| e.item.kind == "income").collect();
        let summary = project_summaries(&items, &october, idx("2026-10"));
        assert_eq!(summary.len(), 2, "recebimento sem projeto e despesa não entram");
        let mapcloid = summary.iter().find(|p| p.name == "Mapcloid").unwrap();
        // ago, set, out (3 parcelas de 900) + 300 avulsos em out = 3000 em 3 meses.
        assert_eq!((mapcloid.month_cents, mapcloid.total_cents, mapcloid.average_cents), (120000, 300000, 100000));
        let gira = summary.iter().find(|p| p.name == "giraComanda").unwrap();
        assert_eq!((gira.month_cents, gira.total_cents, gira.average_cents), (50000, 50000, 50000));
        assert_eq!(summary[0].name, "Mapcloid", "ordenado pelo maior valor do mês");
        assert!(project_summaries(&items, &[], idx("2026-07")).is_empty(), "antes do primeiro recebimento não há projeto");
    }

    #[test]
    fn editar_e_excluir_atualizam_a_regra_e_as_marcas() {
        let c = database();
        save_item(&c, &input("once", None)).unwrap();
        save_item(&c, &MoneyItemInput { title: "Aluguel novo".into(), amount_cents: 200000, ..input("once", None) }).unwrap();
        let items = list_items(&c).unwrap();
        assert_eq!((items.len(), items[0].title.as_str(), items[0].amount_cents), (1, "Aluguel novo", 200000));
        set_paid(&c, "a", "2026-10", true).unwrap();
        set_paid(&c, "a", "2026-10", true).unwrap();
        set_paid(&c, "a", "2026-10", false).unwrap();
        assert_eq!(month_view(&c, "2026-10").unwrap().totals.expenses_paid_cents, 0);
        assert!(set_paid(&c, "nada", "2026-10", true).is_err());
        set_paid(&c, "a", "2026-10", true).unwrap();
        delete_item(&c, "a").unwrap();
        assert!(list_items(&c).unwrap().is_empty());
        let marks: i64 = c.query_row("SELECT COUNT(*) FROM money_marks", [], |row| row.get(0)).unwrap();
        assert_eq!(marks, 0);
        assert!(delete_item(&c, "a").is_err());
    }

    #[test]
    fn nomes_de_projeto_para_sugestao() {
        let c = database();
        save_item(&c, &MoneyItemInput { id: "1".into(), kind: "income".into(), project: Some("Mapcloid".into()), ..input("once", None) }).unwrap();
        save_item(&c, &MoneyItemInput { id: "2".into(), kind: "income".into(), project: Some("giraComanda".into()), ..input("once", None) }).unwrap();
        assert_eq!(project_names(&c).unwrap(), ["giraComanda", "Mapcloid"]);
    }
}
