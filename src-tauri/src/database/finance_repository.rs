use super::finance_models::*;
use chrono::{Datelike, NaiveDate};
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::BTreeMap;

pub fn list_categories(
    connection: &Connection,
    include_disabled: bool,
) -> Result<Vec<FinancialCategory>, String> {
    let sql = if include_disabled {
        "SELECT id,name,scope,enabled,created_at,updated_at FROM financial_categories ORDER BY scope,name COLLATE NOCASE"
    } else {
        "SELECT id,name,scope,enabled,created_at,updated_at FROM financial_categories WHERE enabled=1 ORDER BY scope,name COLLATE NOCASE"
    };
    let mut statement = connection.prepare(sql).map_err(err)?;
    let rows = statement
        .query_map([], map_category)
        .map_err(err)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(err)?;
    Ok(rows)
}

pub fn get_category(
    connection: &Connection,
    id: &str,
) -> Result<Option<FinancialCategory>, String> {
    connection.query_row(
        "SELECT id,name,scope,enabled,created_at,updated_at FROM financial_categories WHERE id=?1",
        [id], map_category,
    ).optional().map_err(err)
}

pub fn save_category(
    connection: &Connection,
    input: &FinancialCategoryInput,
) -> Result<FinancialCategory, String> {
    validate_id(&input.id)?;
    validate_scope(&input.scope)?;
    let name = input.name.trim();
    if name.is_empty() {
        return Err("Nome da categoria é obrigatório".into());
    }
    if let Some(existing) = get_category(connection, &input.id)? {
        let has_transactions: bool = connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM financial_transactions WHERE category_id=?1)",
                [&input.id],
                |row| row.get(0),
            )
            .map_err(err)?;
        if has_transactions && existing.scope != input.scope {
            return Err(
                "A classificação de uma categoria utilizada em lançamentos não pode ser alterada"
                    .into(),
            );
        }
    }
    connection.execute(
        "INSERT INTO financial_categories(id,name,scope,enabled) VALUES (?1,?2,?3,?4)
         ON CONFLICT(id) DO UPDATE SET name=excluded.name,scope=excluded.scope,enabled=excluded.enabled,updated_at=CURRENT_TIMESTAMP",
        params![input.id, name, input.scope, input.enabled],
    ).map_err(|error| match error {
        rusqlite::Error::SqliteFailure(_, Some(message)) if message.contains("UNIQUE") => "Já existe uma categoria com este nome e classificação".into(),
        other => err(other),
    })?;
    get_category(connection, &input.id)?
        .ok_or_else(|| "Categoria não encontrada após salvar".into())
}

pub fn set_category_enabled(
    connection: &Connection,
    id: &str,
    enabled: bool,
) -> Result<FinancialCategory, String> {
    let changed = connection
        .execute(
            "UPDATE financial_categories SET enabled=?2,updated_at=CURRENT_TIMESTAMP WHERE id=?1",
            params![id, enabled],
        )
        .map_err(err)?;
    if changed == 0 {
        return Err("Categoria não encontrada".into());
    }
    get_category(connection, id)?.ok_or_else(|| "Categoria não encontrada após atualizar".into())
}

pub fn list_transactions(
    connection: &Connection,
    filter: &FinancialTransactionFilter,
) -> Result<Vec<FinancialTransaction>, String> {
    validate_filter(filter)?;
    let mut transactions = query_transactions(connection)?;
    let view = filter.view.as_deref().unwrap_or("COMPETENCE");
    transactions.retain(|item| {
        if filter
            .category_id
            .as_deref()
            .is_some_and(|value| value != item.category_id)
        {
            return false;
        }
        if filter
            .transaction_type
            .as_deref()
            .is_some_and(|value| value != item.transaction_type)
        {
            return false;
        }
        if filter
            .scope
            .as_deref()
            .is_some_and(|value| value != item.scope)
        {
            return false;
        }
        if filter
            .status
            .as_deref()
            .is_some_and(|value| value != item.status)
        {
            return false;
        }
        let date = reference_date(item, view);
        if filter
            .date_from
            .as_deref()
            .is_some_and(|from| date.is_none_or(|value| value < from))
        {
            return false;
        }
        if filter
            .date_to
            .as_deref()
            .is_some_and(|to| date.is_none_or(|value| value > to))
        {
            return false;
        }
        true
    });
    transactions.sort_by(|left, right| {
        reference_date(right, view)
            .cmp(&reference_date(left, view))
            .then_with(|| right.created_at.cmp(&left.created_at))
    });
    Ok(transactions)
}

pub fn get_transaction(
    connection: &Connection,
    id: &str,
) -> Result<Option<FinancialTransaction>, String> {
    connection
        .query_row(
            transaction_select("WHERE t.id=?1").as_str(),
            [id],
            map_transaction,
        )
        .optional()
        .map_err(err)
}

pub fn save_transaction(
    connection: &Connection,
    input: &FinancialTransactionInput,
) -> Result<FinancialTransaction, String> {
    validate_transaction(input)?;
    validate_category_for_transaction(connection, input)?;
    if let Some(existing) = get_transaction(connection, &input.id)? {
        let linked = connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM finance_commitments WHERE transaction_id=?1)",
                [&input.id],
                |row| row.get::<_, bool>(0),
            )
            .unwrap_or(false);
        if linked
            && (input.status != "PAID"
                || input.transaction_type != existing.transaction_type
                || input.scope != existing.scope
                || input.category_id != existing.category_id)
        {
            return Err("Lançamento vinculado: tipo, classificação, categoria e status realizado não podem ser alterados".into());
        }
    }
    let description = input.description.trim();
    let notes = clean_optional(input.notes.as_deref());
    connection.execute(
        "INSERT INTO financial_transactions(id,transaction_type,scope,description,amount_cents,category_id,competence_date,due_date,paid_at,status,notes)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)
         ON CONFLICT(id) DO UPDATE SET transaction_type=excluded.transaction_type,scope=excluded.scope,description=excluded.description,
           amount_cents=excluded.amount_cents,category_id=excluded.category_id,competence_date=excluded.competence_date,due_date=excluded.due_date,
           paid_at=excluded.paid_at,status=excluded.status,notes=excluded.notes,updated_at=CURRENT_TIMESTAMP",
        params![input.id,input.transaction_type,input.scope,description,input.amount_cents,input.category_id,input.competence_date,input.due_date,input.paid_at,input.status,notes],
    ).map_err(err)?;
    get_transaction(connection, &input.id)?
        .ok_or_else(|| "Lançamento não encontrado após salvar".into())
}

pub fn mark_transaction_paid(
    connection: &Connection,
    id: &str,
    paid_at: &str,
) -> Result<FinancialTransaction, String> {
    validate_date(paid_at)?;
    let changed = connection.execute(
        "UPDATE financial_transactions SET status='PAID',paid_at=?2,updated_at=CURRENT_TIMESTAMP WHERE id=?1 AND status<>'CANCELLED'",
        params![id, paid_at],
    ).map_err(err)?;
    if changed == 0 {
        return Err("Lançamento não encontrado ou já cancelado".into());
    }
    get_transaction(connection, id)?
        .ok_or_else(|| "Lançamento não encontrado após confirmar pagamento".into())
}

pub fn cancel_transaction(
    connection: &Connection,
    id: &str,
) -> Result<FinancialTransaction, String> {
    let transaction = connection.unchecked_transaction().map_err(err)?;
    let changed = transaction.execute(
        "UPDATE financial_transactions SET status='CANCELLED',updated_at=CURRENT_TIMESTAMP WHERE id=?1",
        [id],
    ).map_err(err)?;
    if changed == 0 {
        return Err("Lançamento não encontrado".into());
    }
    transaction.execute(
        "UPDATE finance_commitments SET status='PENDING',transaction_id=NULL,settled_at=NULL,updated_at=CURRENT_TIMESTAMP WHERE transaction_id=?1",
        [id],
    ).map_err(err)?;
    transaction.commit().map_err(err)?;
    get_transaction(connection, id)?.ok_or_else(|| "Lançamento não encontrado após cancelar".into())
}

pub fn dashboard(
    connection: &Connection,
    month: &str,
    view: &str,
    today: &str,
) -> Result<FinancialDashboard, String> {
    validate_month(month)?;
    validate_view(view)?;
    validate_date(today)?;
    let transactions = query_transactions(connection)?;
    let current = transactions
        .iter()
        .filter(|item| belongs_to_month(item, month, view));
    let realized_items: Vec<_> = current
        .clone()
        .filter(|item| item.status == "PAID")
        .collect();
    let planned_items: Vec<_> = current.filter(|item| item.status == "PLANNED").collect();
    let mut upcoming_due = transactions
        .iter()
        .filter(|item| {
            item.status == "PLANNED" && item.due_date.as_deref().is_some_and(|date| date >= today)
        })
        .cloned()
        .collect::<Vec<_>>();
    upcoming_due.sort_by(|left, right| left.due_date.cmp(&right.due_date));
    upcoming_due.truncate(8);
    Ok(FinancialDashboard {
        month: month.into(),
        view: view.into(),
        realized: totals(realized_items.iter().copied()),
        planned: totals(planned_items.iter().copied()),
        personal_realized: totals(
            realized_items
                .iter()
                .copied()
                .filter(|item| item.scope == "PERSONAL"),
        ),
        personal_planned: totals(
            planned_items
                .iter()
                .copied()
                .filter(|item| item.scope == "PERSONAL"),
        ),
        professional_realized: totals(
            realized_items
                .iter()
                .copied()
                .filter(|item| item.scope == "PROFESSIONAL"),
        ),
        professional_planned: totals(
            planned_items
                .iter()
                .copied()
                .filter(|item| item.scope == "PROFESSIONAL"),
        ),
        upcoming_due,
    })
}

pub fn monthly_report(
    connection: &Connection,
    month: &str,
    view: &str,
) -> Result<FinancialMonthlyReport, String> {
    validate_month(month)?;
    validate_view(view)?;
    let transactions = query_transactions(connection)?;
    let current_paid: Vec<_> = transactions
        .iter()
        .filter(|item| item.status == "PAID" && belongs_to_month(item, month, view))
        .collect();
    let current_planned: Vec<_> = transactions
        .iter()
        .filter(|item| item.status == "PLANNED" && belongs_to_month(item, month, view))
        .collect();
    let previous_month = previous_month(month)?;
    let previous_paid: Vec<_> = transactions
        .iter()
        .filter(|item| item.status == "PAID" && belongs_to_month(item, &previous_month, view))
        .collect();
    let realized = totals(current_paid.iter().copied());
    let previous_realized = totals(previous_paid.iter().copied());
    let expense_total = current_paid
        .iter()
        .filter(|item| item.transaction_type == "EXPENSE")
        .map(|item| item.amount_cents)
        .sum::<i64>();
    let mut grouped: BTreeMap<(String, String, String), i64> = BTreeMap::new();
    for item in current_paid
        .iter()
        .filter(|item| item.transaction_type == "EXPENSE")
    {
        *grouped
            .entry((
                item.category_id.clone(),
                item.category_name.clone(),
                item.scope.clone(),
            ))
            .or_default() += item.amount_cents;
    }
    let mut category_expenses = grouped
        .into_iter()
        .map(
            |((category_id, category_name, scope), amount_cents)| FinancialCategoryTotal {
                category_id,
                category_name,
                scope,
                amount_cents,
                share_basis_points: if expense_total > 0 {
                    Some(amount_cents.saturating_mul(10_000) / expense_total)
                } else {
                    None
                },
            },
        )
        .collect::<Vec<_>>();
    category_expenses.sort_by(|left, right| right.amount_cents.cmp(&left.amount_cents));
    let result_change_cents = realized.result_cents - previous_realized.result_cents;
    let result_change_basis_points = if previous_realized.result_cents != 0 {
        Some(result_change_cents.saturating_mul(10_000) / previous_realized.result_cents.abs())
    } else {
        None
    };
    Ok(FinancialMonthlyReport {
        month: month.into(),
        view: view.into(),
        realized,
        planned: totals(current_planned.iter().copied()),
        personal: totals(
            current_paid
                .iter()
                .copied()
                .filter(|item| item.scope == "PERSONAL"),
        ),
        professional: totals(
            current_paid
                .iter()
                .copied()
                .filter(|item| item.scope == "PROFESSIONAL"),
        ),
        category_expenses,
        previous_month,
        previous_realized,
        previous_has_data: !previous_paid.is_empty(),
        result_change_cents,
        result_change_basis_points,
    })
}

fn query_transactions(connection: &Connection) -> Result<Vec<FinancialTransaction>, String> {
    let sql = transaction_select("");
    let mut statement = connection.prepare(&sql).map_err(err)?;
    let rows = statement
        .query_map([], map_transaction)
        .map_err(err)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(err)?;
    Ok(rows)
}

fn transaction_select(suffix: &str) -> String {
    format!("SELECT t.id,t.transaction_type,t.scope,t.description,t.amount_cents,t.category_id,c.name,t.competence_date,t.due_date,t.paid_at,t.status,t.notes,t.created_at,t.updated_at FROM financial_transactions t JOIN financial_categories c ON c.id=t.category_id {suffix}")
}

fn map_category(row: &rusqlite::Row<'_>) -> rusqlite::Result<FinancialCategory> {
    Ok(FinancialCategory {
        id: row.get(0)?,
        name: row.get(1)?,
        scope: row.get(2)?,
        enabled: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

fn map_transaction(row: &rusqlite::Row<'_>) -> rusqlite::Result<FinancialTransaction> {
    Ok(FinancialTransaction {
        id: row.get(0)?,
        transaction_type: row.get(1)?,
        scope: row.get(2)?,
        description: row.get(3)?,
        amount_cents: row.get(4)?,
        category_id: row.get(5)?,
        category_name: row.get(6)?,
        competence_date: row.get(7)?,
        due_date: row.get(8)?,
        paid_at: row.get(9)?,
        status: row.get(10)?,
        notes: row.get(11)?,
        created_at: row.get(12)?,
        updated_at: row.get(13)?,
    })
}

fn validate_transaction(input: &FinancialTransactionInput) -> Result<(), String> {
    validate_id(&input.id)?;
    validate_type(&input.transaction_type)?;
    validate_scope(&input.scope)?;
    validate_status(&input.status)?;
    if input.description.trim().is_empty() {
        return Err("Descrição do lançamento é obrigatória".into());
    }
    if input.amount_cents <= 0 {
        return Err("O valor deve ser maior que zero".into());
    }
    validate_id(&input.category_id)?;
    validate_date(&input.competence_date)?;
    if let Some(value) = input.due_date.as_deref() {
        validate_date(value)?;
    }
    if let Some(value) = input.paid_at.as_deref() {
        validate_date(value)?;
    }
    if input.status == "PAID" && input.paid_at.is_none() {
        return Err("Informe a data de pagamento para um lançamento realizado".into());
    }
    Ok(())
}

fn validate_category_for_transaction(
    connection: &Connection,
    input: &FinancialTransactionInput,
) -> Result<(), String> {
    let category = get_category(connection, &input.category_id)?
        .ok_or_else(|| "Categoria financeira não encontrada".to_string())?;
    if category.scope != input.scope {
        return Err("A categoria não pertence à classificação do lançamento".into());
    }
    if !category.enabled {
        let keeps_existing = get_transaction(connection, &input.id)?
            .is_some_and(|existing| existing.category_id == input.category_id);
        if !keeps_existing {
            return Err("A categoria selecionada está desativada".into());
        }
    }
    Ok(())
}

fn validate_filter(filter: &FinancialTransactionFilter) -> Result<(), String> {
    if let Some(value) = filter.date_from.as_deref() {
        validate_date(value)?;
    }
    if let Some(value) = filter.date_to.as_deref() {
        validate_date(value)?;
    }
    if filter
        .date_from
        .as_deref()
        .zip(filter.date_to.as_deref())
        .is_some_and(|(from, to)| from > to)
    {
        return Err("Período financeiro inválido".into());
    }
    if let Some(value) = filter.transaction_type.as_deref() {
        validate_type(value)?;
    }
    if let Some(value) = filter.scope.as_deref() {
        validate_scope(value)?;
    }
    if let Some(value) = filter.status.as_deref() {
        validate_status(value)?;
    }
    if let Some(value) = filter.view.as_deref() {
        validate_view(value)?;
    }
    Ok(())
}

fn validate_id(value: &str) -> Result<(), String> {
    if !value.is_empty()
        && value.len() <= 100
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || character == '-' || character == '_'
        })
    {
        Ok(())
    } else {
        Err("ID financeiro inválido".into())
    }
}
fn validate_type(value: &str) -> Result<(), String> {
    if ["INCOME", "EXPENSE"].contains(&value) {
        Ok(())
    } else {
        Err("Tipo financeiro inválido".into())
    }
}
fn validate_scope(value: &str) -> Result<(), String> {
    if ["PERSONAL", "PROFESSIONAL"].contains(&value) {
        Ok(())
    } else {
        Err("Classificação financeira inválida".into())
    }
}
fn validate_status(value: &str) -> Result<(), String> {
    if ["PLANNED", "PAID", "CANCELLED"].contains(&value) {
        Ok(())
    } else {
        Err("Status financeiro inválido".into())
    }
}
fn validate_view(value: &str) -> Result<(), String> {
    if ["COMPETENCE", "CASH"].contains(&value) {
        Ok(())
    } else {
        Err("Visão financeira inválida".into())
    }
}
fn validate_date(value: &str) -> Result<(), String> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map(|_| ())
        .map_err(|_| "Data inválida; use AAAA-MM-DD".into())
}
fn validate_month(value: &str) -> Result<(), String> {
    NaiveDate::parse_from_str(&format!("{value}-01"), "%Y-%m-%d")
        .map(|_| ())
        .map_err(|_| "Mês inválido; use AAAA-MM".into())
}
fn clean_optional(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}
fn reference_date<'a>(item: &'a FinancialTransaction, view: &str) -> Option<&'a str> {
    if view == "CASH" {
        if item.status == "PAID" {
            item.paid_at.as_deref()
        } else if item.status == "PLANNED" {
            item.due_date.as_deref()
        } else {
            item.due_date
                .as_deref()
                .or(Some(item.competence_date.as_str()))
        }
    } else {
        Some(item.competence_date.as_str())
    }
}
fn belongs_to_month(item: &FinancialTransaction, month: &str, view: &str) -> bool {
    item.status != "CANCELLED"
        && reference_date(item, view).is_some_and(|date| date.starts_with(month))
}
fn totals<'a>(items: impl Iterator<Item = &'a FinancialTransaction>) -> FinancialTotals {
    let mut result = FinancialTotals::default();
    for item in items {
        if item.transaction_type == "INCOME" {
            result.income_cents += item.amount_cents;
        } else {
            result.expense_cents += item.amount_cents;
        }
    }
    result.result_cents = result.income_cents - result.expense_cents;
    result
}
fn previous_month(value: &str) -> Result<String, String> {
    let date = NaiveDate::parse_from_str(&format!("{value}-01"), "%Y-%m-%d")
        .map_err(|_| "Mês inválido; use AAAA-MM".to_string())?;
    let (year, month) = if date.month() == 1 {
        (date.year() - 1, 12)
    } else {
        (date.year(), date.month() - 1)
    };
    Ok(format!("{year:04}-{month:02}"))
}
fn err(error: rusqlite::Error) -> String {
    error.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database;
    use std::fs;

    fn database() -> Connection {
        let mut connection = Connection::open_in_memory().unwrap();
        database::initialize(&mut connection).unwrap();
        connection
    }
    fn input(
        id: &str,
        transaction_type: &str,
        scope: &str,
        category_id: &str,
        competence: &str,
    ) -> FinancialTransactionInput {
        FinancialTransactionInput {
            id: id.into(),
            transaction_type: transaction_type.into(),
            scope: scope.into(),
            description: format!("Lançamento {id}"),
            amount_cents: 10_000,
            category_id: category_id.into(),
            competence_date: competence.into(),
            due_date: Some(competence.into()),
            paid_at: None,
            status: "PLANNED".into(),
            notes: None,
        }
    }

    #[test]
    fn creates_edits_filters_pays_and_cancels_transactions_in_cents() {
        let connection = database();
        let mut item = input("tx-1", "EXPENSE", "PERSONAL", "personal-food", "2026-10-05");
        assert_eq!(
            save_transaction(&connection, &item).unwrap().amount_cents,
            10_000
        );
        item.amount_cents = 12_345;
        assert_eq!(
            save_transaction(&connection, &item).unwrap().amount_cents,
            12_345
        );
        let paid = mark_transaction_paid(&connection, "tx-1", "2026-11-02").unwrap();
        assert_eq!(paid.status, "PAID");
        let filter = FinancialTransactionFilter {
            date_from: Some("2026-11-01".into()),
            date_to: Some("2026-11-30".into()),
            view: Some("CASH".into()),
            ..Default::default()
        };
        assert_eq!(list_transactions(&connection, &filter).unwrap().len(), 1);
        assert_eq!(
            cancel_transaction(&connection, "tx-1").unwrap().status,
            "CANCELLED"
        );
    }

    #[test]
    fn rejects_invalid_money_scope_category_and_paid_state() {
        let connection = database();
        let mut item = input(
            "invalid",
            "EXPENSE",
            "PERSONAL",
            "professional-tools",
            "2026-10-05",
        );
        assert!(save_transaction(&connection, &item)
            .unwrap_err()
            .contains("categoria"));
        item.category_id = "personal-food".into();
        item.amount_cents = 0;
        assert!(save_transaction(&connection, &item).is_err());
        item.amount_cents = 100;
        item.status = "PAID".into();
        assert!(save_transaction(&connection, &item)
            .unwrap_err()
            .contains("pagamento"));
    }

    #[test]
    fn competence_cash_planned_paid_cancelled_and_month_comparison_are_distinct() {
        let connection = database();
        let mut paid = input(
            "income",
            "INCOME",
            "PROFESSIONAL",
            "professional-other",
            "2026-09-28",
        );
        paid.paid_at = Some("2026-10-03".into());
        paid.status = "PAID".into();
        paid.amount_cents = 60_000;
        save_transaction(&connection, &paid).unwrap();
        let planned = input(
            "expense",
            "EXPENSE",
            "PROFESSIONAL",
            "professional-servers",
            "2026-10-05",
        );
        save_transaction(&connection, &planned).unwrap();
        let cancelled = input(
            "cancelled",
            "EXPENSE",
            "PERSONAL",
            "personal-leisure",
            "2026-10-06",
        );
        save_transaction(&connection, &cancelled).unwrap();
        cancel_transaction(&connection, "cancelled").unwrap();
        assert_eq!(
            dashboard(&connection, "2026-09", "COMPETENCE", "2026-09-01")
                .unwrap()
                .realized
                .income_cents,
            60_000
        );
        let cash = dashboard(&connection, "2026-10", "CASH", "2026-10-01").unwrap();
        assert_eq!(cash.realized.income_cents, 60_000);
        assert_eq!(cash.planned.expense_cents, 10_000);
        assert_eq!(
            monthly_report(&connection, "2026-10", "CASH")
                .unwrap()
                .professional
                .expense_cents,
            0
        );
    }

    #[test]
    fn categories_deactivate_without_breaking_history_and_database_reopens() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("azriel-finance-{nonce}.db"));
        {
            let mut connection = Connection::open(&path).unwrap();
            database::initialize(&mut connection).unwrap();
            let item = input(
                "persisted",
                "EXPENSE",
                "PERSONAL",
                "personal-food",
                "2026-10-05",
            );
            save_transaction(&connection, &item).unwrap();
            set_category_enabled(&connection, "personal-food", false).unwrap();
            assert_eq!(
                get_transaction(&connection, "persisted")
                    .unwrap()
                    .unwrap()
                    .category_name,
                "Alimentação"
            );
        }
        {
            let mut connection = Connection::open(&path).unwrap();
            database::initialize(&mut connection).unwrap();
            assert!(get_transaction(&connection, "persisted").unwrap().is_some());
        }
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn category_scope_is_immutable_after_use_and_cash_does_not_invent_due_dates() {
        let connection = database();
        let mut item = input(
            "without-due-date",
            "EXPENSE",
            "PERSONAL",
            "personal-food",
            "2026-10-05",
        );
        item.due_date = None;
        save_transaction(&connection, &item).unwrap();

        let cash = dashboard(&connection, "2026-10", "CASH", "2026-10-01").unwrap();
        assert_eq!(cash.planned.expense_cents, 0);
        assert_eq!(
            list_transactions(
                &connection,
                &FinancialTransactionFilter {
                    date_from: Some("2026-10-01".into()),
                    date_to: Some("2026-10-31".into()),
                    view: Some("CASH".into()),
                    ..Default::default()
                }
            )
            .unwrap()
            .len(),
            0
        );

        let error = save_category(
            &connection,
            &FinancialCategoryInput {
                id: "personal-food".into(),
                name: "Alimentação".into(),
                scope: "PROFESSIONAL".into(),
                enabled: true,
            },
        )
        .unwrap_err();
        assert!(error.contains("não pode ser alterada"));
    }
}
