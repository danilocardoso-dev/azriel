use super::finance_planning_models::*;
use chrono::{Datelike, Duration, Months, NaiveDate};
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::{BTreeMap, HashSet};

fn err(error: rusqlite::Error) -> String {
    error.to_string()
}
fn date(value: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|_| "Data inválida; use AAAA-MM-DD".into())
}
fn month(value: &str) -> Result<NaiveDate, String> {
    date(&format!("{value}-01")).map_err(|_| "Mês inválido; use AAAA-MM".into())
}
fn fmt(value: NaiveDate) -> String {
    value.format("%Y-%m-%d").to_string()
}
fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 120
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}
fn validate_scope(value: &str) -> Result<(), String> {
    if ["PERSONAL", "PROFESSIONAL"].contains(&value) {
        Ok(())
    } else {
        Err("Classificação financeira inválida".into())
    }
}
fn clean(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|x| !x.is_empty())
        .map(str::to_string)
}

pub fn list_budgets(
    connection: &Connection,
    year_month: &str,
    view: &str,
) -> Result<Vec<FinancialBudget>, String> {
    month(year_month)?;
    if !["CASH", "COMPETENCE"].contains(&view) {
        return Err("Visão financeira inválida".into());
    }
    let transaction_date = if view == "CASH" {
        "t.paid_at"
    } else {
        "t.competence_date"
    };
    let planned_date = if view == "CASH" {
        "t.due_date"
    } else {
        "t.competence_date"
    };
    let commitment_date = if view == "CASH" {
        "m.due_date"
    } else {
        "m.competence_date"
    };
    let sql = format!(
        r#"
      SELECT b.id,b.year_month,b.scope,b.category_id,c.name,c.enabled,b.limit_cents,
        COALESCE((SELECT SUM(t.amount_cents) FROM financial_transactions t
          WHERE t.category_id=b.category_id AND t.scope=b.scope AND t.transaction_type='EXPENSE'
            AND t.status='PAID' AND substr({transaction_date},1,7)=b.year_month),0),
        COALESCE((SELECT SUM(t.amount_cents) FROM financial_transactions t
          WHERE t.category_id=b.category_id AND t.scope=b.scope AND t.transaction_type='EXPENSE'
            AND t.status='PLANNED' AND substr({planned_date},1,7)=b.year_month),0)
        + COALESCE((SELECT SUM(m.expected_amount_cents) FROM finance_commitments m
          WHERE m.category_id=b.category_id AND m.scope=b.scope AND m.direction='EXPENSE'
            AND m.status='PENDING' AND substr({commitment_date},1,7)=b.year_month),0),
        b.created_at,b.updated_at
      FROM finance_budgets b JOIN financial_categories c ON c.id=b.category_id
      WHERE b.year_month=?1 ORDER BY b.scope,c.name COLLATE NOCASE"#
    );
    let mut statement = connection.prepare(&sql).map_err(err)?;
    let result = statement
        .query_map([year_month], |row| {
            let limit: i64 = row.get(6)?;
            let realized: i64 = row.get(7)?;
            let committed: i64 = row.get(8)?;
            Ok(FinancialBudget {
                id: row.get(0)?,
                year_month: row.get(1)?,
                scope: row.get(2)?,
                category_id: row.get(3)?,
                category_name: row.get(4)?,
                category_enabled: row.get(5)?,
                limit_cents: limit,
                realized_cents: realized,
                committed_cents: committed,
                available_cents: limit.saturating_sub(realized).saturating_sub(committed),
                usage_basis_points: realized.saturating_add(committed).saturating_mul(10_000)
                    / limit,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })
        .map_err(err)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(err)?;
    let projected = projection(
        connection,
        &format!("{year_month}-01"),
        view,
        3,
        0,
        &ProjectionScenario::default(),
    )?;
    let mut result = result;
    for budget in &mut result {
        budget.committed_cents = projected
            .items
            .iter()
            .filter(|item| {
                item.direction == "EXPENSE"
                    && item.scope == budget.scope
                    && item.category_id == budget.category_id
                    && item.expected_date.starts_with(year_month)
            })
            .map(|item| item.amount_cents)
            .fold(0_i64, i64::saturating_add);
        budget.available_cents = budget
            .limit_cents
            .saturating_sub(budget.realized_cents)
            .saturating_sub(budget.committed_cents);
        budget.usage_basis_points = budget
            .realized_cents
            .saturating_add(budget.committed_cents)
            .saturating_mul(10_000)
            / budget.limit_cents;
    }
    Ok(result)
}

pub fn save_budget(
    connection: &Connection,
    input: &FinancialBudgetInput,
) -> Result<FinancialBudget, String> {
    if !valid_id(&input.id) {
        return Err("ID de orçamento inválido".into());
    }
    month(&input.year_month)?;
    validate_scope(&input.scope)?;
    if input.limit_cents <= 0 {
        return Err("O limite deve ser maior que zero".into());
    }
    let category: Option<(String, bool)> = connection
        .query_row(
            "SELECT scope,enabled FROM financial_categories WHERE id=?1",
            [&input.category_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()
        .map_err(err)?;
    let (category_scope, enabled) = category.ok_or("Categoria financeira não encontrada")?;
    if category_scope != input.scope {
        return Err("A categoria não pertence à classificação do orçamento".into());
    }
    let existing: bool = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM finance_budgets WHERE id=?1)",
            [&input.id],
            |r| r.get(0),
        )
        .map_err(err)?;
    if !enabled && !existing {
        return Err("Não é possível criar orçamento para uma categoria desativada".into());
    }
    connection.execute("INSERT INTO finance_budgets(id,year_month,scope,category_id,limit_cents) VALUES (?1,?2,?3,?4,?5) ON CONFLICT(id) DO UPDATE SET year_month=excluded.year_month,scope=excluded.scope,category_id=excluded.category_id,limit_cents=excluded.limit_cents,updated_at=CURRENT_TIMESTAMP",params![input.id,input.year_month,input.scope,input.category_id,input.limit_cents]).map_err(|e| if e.to_string().contains("UNIQUE") { "Já existe orçamento para esta categoria no mês".into() } else { err(e) })?;
    list_budgets(connection, &input.year_month, "COMPETENCE")?
        .into_iter()
        .find(|x| x.id == input.id)
        .ok_or("Orçamento não encontrado após salvar".into())
}

pub fn delete_budget(connection: &Connection, id: &str) -> Result<(), String> {
    if connection
        .execute("DELETE FROM finance_budgets WHERE id=?1", [id])
        .map_err(err)?
        == 0
    {
        return Err("Orçamento não encontrado".into());
    }
    Ok(())
}

pub fn copy_budgets(
    connection: &Connection,
    from_month: &str,
    to_month: &str,
) -> Result<BudgetCopyResult, String> {
    month(from_month)?;
    month(to_month)?;
    if from_month == to_month {
        return Err("Selecione meses diferentes para copiar".into());
    }
    let transaction = connection.unchecked_transaction().map_err(err)?;
    let total: i64 = transaction
        .query_row(
            "SELECT COUNT(*) FROM finance_budgets WHERE year_month=?1",
            [from_month],
            |r| r.get(0),
        )
        .map_err(err)?;
    let mut statement = transaction
        .prepare("SELECT id,scope,category_id,limit_cents FROM finance_budgets WHERE year_month=?1")
        .map_err(err)?;
    let rows = statement
        .query_map([from_month], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, i64>(3)?,
            ))
        })
        .map_err(err)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(err)?;
    drop(statement);
    let mut created = 0;
    for (source_id, scope, category, limit) in rows {
        let id = format!("budget-{}-{}", to_month, source_id);
        created+=transaction.execute("INSERT OR IGNORE INTO finance_budgets(id,year_month,scope,category_id,limit_cents) VALUES (?1,?2,?3,?4,?5)",params![id,to_month,scope,category,limit]).map_err(err)? as i64;
    }
    transaction.commit().map_err(err)?;
    Ok(BudgetCopyResult {
        created,
        skipped: total - created,
    })
}

pub fn list_goals(
    connection: &Connection,
    include_archived: bool,
) -> Result<Vec<FinancialGoal>, String> {
    let where_clause = if include_archived {
        ""
    } else {
        "WHERE g.status<>'ARCHIVED'"
    };
    let sql=format!("SELECT g.id,g.title,g.scope,g.target_cents,g.target_date,g.status,COALESCE(SUM(a.amount_cents),0),g.created_at,g.updated_at FROM finance_goals g LEFT JOIN finance_goal_contributions a ON a.goal_id=g.id {where_clause} GROUP BY g.id ORDER BY CASE g.status WHEN 'ACTIVE' THEN 0 WHEN 'COMPLETED' THEN 1 ELSE 2 END,g.target_date,g.created_at");
    let mut statement = connection.prepare(&sql).map_err(err)?;
    let result = statement
        .query_map([], |r| {
            let target: i64 = r.get(3)?;
            let contributed: i64 = r.get(6)?;
            Ok(FinancialGoal {
                id: r.get(0)?,
                title: r.get(1)?,
                scope: r.get(2)?,
                target_cents: target,
                target_date: r.get(4)?,
                status: r.get(5)?,
                contributed_cents: contributed,
                remaining_cents: target.saturating_sub(contributed).max(0),
                progress_basis_points: contributed.saturating_mul(10_000) / target,
                created_at: r.get(7)?,
                updated_at: r.get(8)?,
            })
        })
        .map_err(err)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(err)?;
    Ok(result)
}

pub fn save_goal(
    connection: &Connection,
    input: &FinancialGoalInput,
) -> Result<FinancialGoal, String> {
    if !valid_id(&input.id) || input.title.trim().is_empty() {
        return Err("Informe uma meta válida".into());
    }
    validate_scope(&input.scope)?;
    if input.target_cents <= 0 {
        return Err("O valor-alvo deve ser maior que zero".into());
    }
    if let Some(x) = input.target_date.as_deref() {
        date(x)?;
    }
    if !["ACTIVE", "COMPLETED", "ARCHIVED"].contains(&input.status.as_str()) {
        return Err("Status de meta inválido".into());
    }
    connection.execute("INSERT INTO finance_goals(id,title,scope,target_cents,target_date,status) VALUES (?1,?2,?3,?4,?5,?6) ON CONFLICT(id) DO UPDATE SET title=excluded.title,scope=excluded.scope,target_cents=excluded.target_cents,target_date=excluded.target_date,status=excluded.status,updated_at=CURRENT_TIMESTAMP",params![input.id,input.title.trim(),input.scope,input.target_cents,input.target_date,input.status]).map_err(err)?;
    list_goals(connection, true)?
        .into_iter()
        .find(|x| x.id == input.id)
        .ok_or("Meta não encontrada após salvar".into())
}

pub fn list_contributions(
    connection: &Connection,
    goal_id: &str,
) -> Result<Vec<GoalContribution>, String> {
    let mut statement=connection.prepare("SELECT id,goal_id,amount_cents,contribution_date,note,created_at,updated_at FROM finance_goal_contributions WHERE goal_id=?1 ORDER BY contribution_date DESC,created_at DESC").map_err(err)?;
    let result = statement
        .query_map([goal_id], |r| {
            Ok(GoalContribution {
                id: r.get(0)?,
                goal_id: r.get(1)?,
                amount_cents: r.get(2)?,
                contribution_date: r.get(3)?,
                note: r.get(4)?,
                created_at: r.get(5)?,
                updated_at: r.get(6)?,
            })
        })
        .map_err(err)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(err)?;
    Ok(result)
}

pub fn save_contribution(
    connection: &Connection,
    input: &GoalContributionInput,
) -> Result<GoalContribution, String> {
    if !valid_id(&input.id) || !valid_id(&input.goal_id) {
        return Err("ID de aporte inválido".into());
    }
    if input.amount_cents <= 0 {
        return Err("O aporte deve ser maior que zero".into());
    }
    date(&input.contribution_date)?;
    let transaction = connection.unchecked_transaction().map_err(err)?;
    let exists: bool = transaction
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM finance_goals WHERE id=?1)",
            [&input.goal_id],
            |r| r.get(0),
        )
        .map_err(err)?;
    if !exists {
        return Err("Meta não encontrada".into());
    }
    let old:Option<(String,i64,String,Option<String>)>=transaction.query_row("SELECT goal_id,amount_cents,contribution_date,note FROM finance_goal_contributions WHERE id=?1",[&input.id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional().map_err(err)?;
    if let Some((goal, amount, when, note)) = old {
        transaction.execute("INSERT INTO finance_goal_contribution_history(contribution_id,goal_id,previous_amount_cents,previous_date,previous_note) VALUES (?1,?2,?3,?4,?5)",params![input.id,goal,amount,when,note]).map_err(err)?;
    }
    transaction.execute("INSERT INTO finance_goal_contributions(id,goal_id,amount_cents,contribution_date,note) VALUES (?1,?2,?3,?4,?5) ON CONFLICT(id) DO UPDATE SET goal_id=excluded.goal_id,amount_cents=excluded.amount_cents,contribution_date=excluded.contribution_date,note=excluded.note,updated_at=CURRENT_TIMESTAMP",params![input.id,input.goal_id,input.amount_cents,input.contribution_date,clean(input.note.as_deref())]).map_err(err)?;
    transaction.commit().map_err(err)?;
    list_contributions(connection, &input.goal_id)?
        .into_iter()
        .find(|x| x.id == input.id)
        .ok_or("Aporte não encontrado após salvar".into())
}

pub fn delete_contribution(connection: &Connection, id: &str) -> Result<(), String> {
    let transaction = connection.unchecked_transaction().map_err(err)?;
    let previous: Option<(String, i64, String, Option<String>)> = transaction
        .query_row("SELECT goal_id,amount_cents,contribution_date,note FROM finance_goal_contributions WHERE id=?1",[id],|row|Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?)))
        .optional().map_err(err)?;
    let (goal_id, amount, contribution_date, note) = previous.ok_or("Aporte não encontrado")?;
    transaction.execute("INSERT INTO finance_goal_contribution_history(contribution_id,goal_id,previous_amount_cents,previous_date,previous_note) VALUES (?1,?2,?3,?4,?5)",params![id,goal_id,amount,contribution_date,note]).map_err(err)?;
    transaction
        .execute("DELETE FROM finance_goal_contributions WHERE id=?1", [id])
        .map_err(err)?;
    transaction.commit().map_err(err)?;
    Ok(())
}

fn last_day(start: NaiveDate, horizon: u32) -> Result<NaiveDate, String> {
    start
        .checked_add_months(Months::new(horizon))
        .map(|x| x - Duration::days(1))
        .ok_or("Horizonte inválido".into())
}
fn clamp_date(year: i32, month: u32, day: u32) -> NaiveDate {
    let mut d = day;
    loop {
        if let Some(x) = NaiveDate::from_ymd_opt(year, month, d) {
            return x;
        }
        d -= 1;
    }
}
fn occurrence_dates(
    frequency: &str,
    first: &str,
    start: &str,
    end: Option<&str>,
    resume: Option<&str>,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Vec<NaiveDate>, String> {
    let first = date(first)?;
    let start = date(start)?;
    let end = end.map(date).transpose()?;
    let resume = resume.map(date).transpose()?;
    let mut result = Vec::new();
    let mut current = first;
    let mut guard = 0;
    while current <= to && guard < 700 {
        if current >= from
            && current >= start
            && end.is_none_or(|x| current <= x)
            && resume.is_none_or(|x| current >= x)
        {
            result.push(current);
        }
        current = match frequency {
            "WEEKLY" => current + Duration::days(7),
            "MONTHLY" => {
                let base = NaiveDate::from_ymd_opt(current.year(), current.month(), 1)
                    .unwrap()
                    .checked_add_months(Months::new(1))
                    .ok_or("Data recorrente inválida")?;
                clamp_date(base.year(), base.month(), first.day())
            }
            "YEARLY" => clamp_date(current.year() + 1, first.month(), first.day()),
            "ONE_TIME" => break,
            _ => return Err("Periodicidade inválida".into()),
        };
        guard += 1;
    }
    Ok(result)
}

pub fn projection(
    connection: &Connection,
    today: &str,
    view: &str,
    horizon: u32,
    base_cents: i64,
    scenario: &ProjectionScenario,
) -> Result<FinancialProjection, String> {
    let today = date(today)?;
    if ![3, 6, 12].contains(&horizon) {
        return Err("Horizonte deve ser 3, 6 ou 12 meses".into());
    }
    if !["CASH", "COMPETENCE"].contains(&view) {
        return Err("Visão financeira inválida".into());
    }
    if !(0..=100).contains(&scenario.professional_income_reduction_percent)
        || !(0..=500).contains(&scenario.operating_cost_increase_percent)
        || scenario.new_monthly_commitment_cents < 0
    {
        return Err("Parâmetros de cenário inválidos".into());
    }
    let from = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap();
    let to = last_day(from, horizon)?;
    let mut items = Vec::new();
    let tx_date = if view == "CASH" {
        "due_date"
    } else {
        "competence_date"
    };
    let sql=format!("SELECT id,description,transaction_type,scope,category_id,amount_cents,{tx_date} FROM financial_transactions WHERE status='PLANNED' AND {tx_date} BETWEEN ?1 AND ?2");
    let mut statement = connection.prepare(&sql).map_err(err)?;
    let rows = statement
        .query_map(params![fmt(from), fmt(to)], |r| {
            Ok(ProjectionItem {
                origin_key: format!("TRANSACTION:{}", r.get::<_, String>(0)?),
                source_type: "TRANSACTION".into(),
                source_name: r.get(1)?,
                direction: r.get(2)?,
                scope: r.get(3)?,
                category_id: r.get(4)?,
                amount_cents: r.get(5)?,
                expected_date: r.get(6)?,
                estimated: false,
            })
        })
        .map_err(err)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(err)?;
    items.extend(rows);
    drop(statement);
    let commitment_date = if view == "CASH" {
        "due_date"
    } else {
        "competence_date"
    };
    let sql=format!("SELECT source_type,COALESCE(contract_id,recurrence_id,loan_installment_id),occurrence_key,CASE source_type WHEN 'CONTRACT' THEN COALESCE((SELECT name FROM finance_contracts WHERE id=contract_id),'Contrato') WHEN 'RECURRING_EXPENSE' THEN COALESCE((SELECT title FROM finance_recurrences WHERE id=recurrence_id),'Recorrência') ELSE COALESCE((SELECT l.name FROM finance_loan_installments i JOIN finance_loans l ON l.id=i.loan_id WHERE i.id=loan_installment_id),'Parcela') END,direction,scope,category_id,expected_amount_cents,{commitment_date},estimated,status FROM finance_commitments WHERE {commitment_date} BETWEEN ?1 AND ?2");
    let mut statement = connection.prepare(&sql).map_err(err)?;
    let persisted = statement
        .query_map(params![fmt(from), fmt(to)], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, String>(5)?,
                r.get::<_, String>(6)?,
                r.get::<_, i64>(7)?,
                r.get::<_, String>(8)?,
                r.get::<_, bool>(9)?,
                r.get::<_, String>(10)?,
            ))
        })
        .map_err(err)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(err)?;
    drop(statement);
    let mut represented = HashSet::new();
    for (kind, id, key, name, direction, scope, category, amount, when, estimated, status) in
        persisted
    {
        let origin = format!("{kind}:{id}:{key}");
        represented.insert(origin.clone());
        if status == "PENDING" {
            items.push(ProjectionItem {
                origin_key: origin,
                source_type: kind,
                source_name: name,
                direction,
                scope,
                category_id: category,
                amount_cents: amount,
                expected_date: when,
                estimated,
            });
        }
    }
    let mut statement=connection.prepare("SELECT id,name,scope,amount_cents,frequency,start_date,end_date,first_due_date,category_id,resume_date FROM finance_contracts WHERE status='ACTIVE'").map_err(err)?;
    let contracts = statement
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, String>(5)?,
                r.get::<_, Option<String>>(6)?,
                r.get::<_, String>(7)?,
                r.get::<_, String>(8)?,
                r.get::<_, Option<String>>(9)?,
            ))
        })
        .map_err(err)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(err)?;
    drop(statement);
    for (id, name, scope, amount, freq, start, end, first, category, resume) in contracts {
        for due in occurrence_dates(
            &freq,
            &first,
            &start,
            end.as_deref(),
            resume.as_deref(),
            from,
            to,
        )? {
            let key = format!("CONTRACT:{id}:{}", fmt(due));
            if !represented.contains(&key) {
                items.push(ProjectionItem {
                    origin_key: key,
                    source_type: "CONTRACT".into(),
                    source_name: name.clone(),
                    direction: "INCOME".into(),
                    scope: scope.clone(),
                    category_id: category.clone(),
                    amount_cents: amount,
                    expected_date: fmt(due),
                    estimated: false,
                });
            }
        }
    }
    let mut statement=connection.prepare("SELECT id,title,scope,amount_cents,frequency,start_date,end_date,first_due_date,category_id,resume_date,estimated FROM finance_recurrences WHERE status='ACTIVE'").map_err(err)?;
    let recurrences = statement
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, String>(5)?,
                r.get::<_, Option<String>>(6)?,
                r.get::<_, String>(7)?,
                r.get::<_, String>(8)?,
                r.get::<_, Option<String>>(9)?,
                r.get::<_, bool>(10)?,
            ))
        })
        .map_err(err)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(err)?;
    drop(statement);
    for (id, name, scope, amount, freq, start, end, first, category, resume, estimated) in
        recurrences
    {
        for due in occurrence_dates(
            &freq,
            &first,
            &start,
            end.as_deref(),
            resume.as_deref(),
            from,
            to,
        )? {
            let key = format!("RECURRING_EXPENSE:{id}:{}", fmt(due));
            if !represented.contains(&key) {
                items.push(ProjectionItem {
                    origin_key: key,
                    source_type: "RECURRING_EXPENSE".into(),
                    source_name: name.clone(),
                    direction: "EXPENSE".into(),
                    scope: scope.clone(),
                    category_id: category.clone(),
                    amount_cents: amount,
                    expected_date: fmt(due),
                    estimated,
                });
            }
        }
    }
    let mut statement=connection.prepare("SELECT i.id,l.name,l.scope,l.category_id,i.amount_cents,i.due_date,i.installment_number FROM finance_loan_installments i JOIN finance_loans l ON l.id=i.loan_id WHERE l.status='ACTIVE' AND i.due_date BETWEEN ?1 AND ?2").map_err(err)?;
    let installments = statement
        .query_map(params![fmt(from), fmt(to)], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, i64>(4)?,
                r.get::<_, String>(5)?,
                r.get::<_, i64>(6)?,
            ))
        })
        .map_err(err)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(err)?;
    drop(statement);
    for (id, name, scope, category, amount, when, number) in installments {
        let key = format!("LOAN_INSTALLMENT:{id}:installment-{number}");
        if !represented.contains(&key) {
            items.push(ProjectionItem {
                origin_key: key,
                source_type: "LOAN_INSTALLMENT".into(),
                source_name: name,
                direction: "EXPENSE".into(),
                scope,
                category_id: category,
                amount_cents: amount,
                expected_date: when,
                estimated: false,
            });
        }
    }
    items.sort_by(|a, b| {
        a.expected_date
            .cmp(&b.expected_date)
            .then(a.origin_key.cmp(&b.origin_key))
    });
    let base_result = items.iter().fold(0_i64, |total, item| {
        total.saturating_add(if item.direction == "INCOME" {
            item.amount_cents
        } else {
            -item.amount_cents
        })
    });
    let closed = scenario.closed_contract_id.as_deref();
    let mut scenario_items = items
        .into_iter()
        .filter(|x| {
            !(x.source_type == "CONTRACT"
                && closed.is_some_and(|id| x.origin_key.starts_with(&format!("CONTRACT:{id}:"))))
        })
        .map(|mut x| {
            if x.direction == "INCOME" && x.scope == "PROFESSIONAL" {
                x.amount_cents = x
                    .amount_cents
                    .saturating_mul(100 - scenario.professional_income_reduction_percent)
                    / 100;
            } else if x.direction == "EXPENSE" && x.scope == "PROFESSIONAL" {
                x.amount_cents = x
                    .amount_cents
                    .saturating_mul(100 + scenario.operating_cost_increase_percent)
                    / 100;
            }
            x
        })
        .collect::<Vec<_>>();
    if scenario.new_monthly_commitment_cents > 0 {
        for offset in 0..horizon {
            let d = from.checked_add_months(Months::new(offset)).unwrap();
            scenario_items.push(ProjectionItem {
                origin_key: format!("SCENARIO:NEW:{}", d.format("%Y-%m")),
                source_type: "SCENARIO".into(),
                source_name: "Novo compromisso hipotético".into(),
                direction: "EXPENSE".into(),
                scope: "PROFESSIONAL".into(),
                category_id: "scenario".into(),
                amount_cents: scenario.new_monthly_commitment_cents,
                expected_date: fmt(d),
                estimated: true,
            });
        }
    }
    let mut grouped: BTreeMap<String, (i64, i64)> = BTreeMap::new();
    for item in &scenario_items {
        let entry = grouped.entry(item.expected_date[..7].into()).or_default();
        if item.direction == "INCOME" {
            entry.0 = entry.0.saturating_add(item.amount_cents)
        } else {
            entry.1 = entry.1.saturating_add(item.amount_cents)
        }
    }
    let mut accumulated = base_cents;
    let mut months = Vec::new();
    for offset in 0..horizon {
        let key = from
            .checked_add_months(Months::new(offset))
            .unwrap()
            .format("%Y-%m")
            .to_string();
        let (income, expense) = grouped.get(&key).copied().unwrap_or_default();
        let result = income.saturating_sub(expense);
        accumulated = accumulated.saturating_add(result);
        months.push(ProjectionMonth {
            month: key,
            income_cents: income,
            expense_cents: expense,
            result_cents: result,
            accumulated_cents: accumulated,
        });
    }
    let scenario_result = months
        .iter()
        .fold(0_i64, |total, item| total.saturating_add(item.result_cents));
    Ok(FinancialProjection {
        view: view.into(),
        horizon_months: horizon,
        base_cents,
        items: scenario_items,
        months,
        base_result_cents: base_result,
        scenario_result_cents: scenario_result,
        scenario_difference_cents: scenario_result.saturating_sub(base_result),
    })
}

pub fn alerts(
    connection: &Connection,
    today: &str,
    year_month: &str,
    view: &str,
    upcoming_days: u32,
) -> Result<Vec<FinanceAlert>, String> {
    let today = date(today)?;
    if upcoming_days > 90 {
        return Err("A janela de alertas deve ser de até 90 dias".into());
    }
    let end = today + Duration::days(upcoming_days as i64);
    let mut alerts = Vec::new();
    let mut statement=connection.prepare("SELECT id,source_type,due_date,expected_amount_cents FROM finance_commitments WHERE status='PENDING' AND due_date<=?1 ORDER BY due_date").map_err(err)?;
    let rows = statement
        .query_map([fmt(end)], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, i64>(3)?,
            ))
        })
        .map_err(err)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(err)?;
    for (id, kind, due, amount) in rows {
        let overdue = due < fmt(today);
        alerts.push(FinanceAlert {
            id: format!("commitment-{id}"),
            level: if overdue { "CRITICAL" } else { "WARNING" }.into(),
            alert_type: if overdue { "OVERDUE" } else { "UPCOMING" }.into(),
            title: if overdue {
                "Compromisso atrasado"
            } else {
                "Vencimento próximo"
            }
            .into(),
            description: format!("{kind} · {due} · {} centavos", amount),
            reference_id: id,
        });
    }
    for budget in list_budgets(connection, year_month, view)? {
        if budget.usage_basis_points >= 8_000 {
            let exceeded = budget.usage_basis_points > 10_000;
            alerts.push(FinanceAlert {
                id: format!("budget-{}", budget.id),
                level: if exceeded { "CRITICAL" } else { "WARNING" }.into(),
                alert_type: if exceeded {
                    "BUDGET_EXCEEDED"
                } else {
                    "BUDGET_WARNING"
                }
                .into(),
                title: if exceeded {
                    "Orçamento ultrapassado"
                } else {
                    "Orçamento próximo do limite"
                }
                .into(),
                description: format!(
                    "{} · {:.1}% utilizado",
                    budget.category_name,
                    budget.usage_basis_points as f64 / 100.0
                ),
                reference_id: budget.id,
            });
        }
    }
    for goal in list_goals(connection, false)? {
        if goal.status == "ACTIVE" {
            if let Some(target) = goal.target_date.as_deref() {
                let target = date(target)?;
                let elapsed = (today
                    - NaiveDate::parse_from_str(&goal.created_at[..10], "%Y-%m-%d")
                        .unwrap_or(today))
                .num_days()
                .max(0);
                let total = (target
                    - NaiveDate::parse_from_str(&goal.created_at[..10], "%Y-%m-%d")
                        .unwrap_or(today))
                .num_days();
                if total >= 14
                    && elapsed >= 7
                    && today < target
                    && goal.progress_basis_points < (elapsed.saturating_mul(10_000) / total)
                {
                    alerts.push(FinanceAlert {
                        id: format!("goal-{}", goal.id),
                        level: "INFO".into(),
                        alert_type: "GOAL_PACE".into(),
                        title: "Meta abaixo do ritmo registrado".into(),
                        description: goal.title,
                        reference_id: goal.id,
                    });
                }
            }
        }
    }
    Ok(alerts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database;
    fn db() -> Connection {
        let mut c = Connection::open_in_memory().unwrap();
        database::initialize(&mut c).unwrap();
        c
    }
    #[test]
    fn budget_is_unique_and_keeps_disabled_category() {
        let c = db();
        let input = FinancialBudgetInput {
            id: "b1".into(),
            year_month: "2026-12".into(),
            scope: "PERSONAL".into(),
            category_id: "personal-food".into(),
            limit_cents: 100_000,
        };
        save_budget(&c, &input).unwrap();
        let mut duplicate = input;
        duplicate.id = "b2".into();
        assert!(save_budget(&c, &duplicate).is_err());
        c.execute(
            "UPDATE financial_categories SET enabled=0 WHERE id='personal-food'",
            [],
        )
        .unwrap();
        assert_eq!(list_budgets(&c, "2026-12", "CASH").unwrap().len(), 1);
    }
    #[test]
    fn contribution_is_progress_not_cash_and_correction_is_audited() {
        let c = db();
        save_goal(
            &c,
            &FinancialGoalInput {
                id: "g1".into(),
                title: "Reserva".into(),
                scope: "PERSONAL".into(),
                target_cents: 100_000,
                target_date: None,
                status: "ACTIVE".into(),
            },
        )
        .unwrap();
        let mut x = GoalContributionInput {
            id: "a1".into(),
            goal_id: "g1".into(),
            amount_cents: 10_000,
            contribution_date: "2026-10-14".into(),
            note: None,
        };
        save_contribution(&c, &x).unwrap();
        x.amount_cents = 20_000;
        save_contribution(&c, &x).unwrap();
        assert_eq!(list_goals(&c, false).unwrap()[0].contributed_cents, 20_000);
        assert_eq!(
            c.query_row("SELECT COUNT(*) FROM financial_transactions", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            0
        );
        assert_eq!(
            c.query_row(
                "SELECT COUNT(*) FROM finance_goal_contribution_history",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
            1
        );
        delete_contribution(&c, "a1").unwrap();
        assert!(list_contributions(&c, "g1").unwrap().is_empty());
        assert_eq!(
            c.query_row(
                "SELECT COUNT(*) FROM finance_goal_contribution_history",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
            2
        );
    }
    #[test]
    fn projections_support_horizons_and_scenarios_do_not_write() {
        let c = db();
        let before = c.total_changes();
        let p = projection(
            &c,
            "2026-10-14",
            "CASH",
            12,
            0,
            &ProjectionScenario {
                new_monthly_commitment_cents: 10_000,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(p.months.len(), 12);
        assert_eq!(c.total_changes(), before);
        assert_eq!(p.items.len(), 12);
        assert!(projection(
            &c,
            "2026-10-14",
            "CASH",
            5,
            0,
            &ProjectionScenario::default()
        )
        .is_err());
        let large = projection(
            &c,
            "2026-10-14",
            "CASH",
            12,
            i64::MAX,
            &ProjectionScenario {
                new_monthly_commitment_cents: i64::MAX,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(large.scenario_result_cents, i64::MIN);
    }
    #[test]
    fn budget_distinguishes_cash_competence_realized_and_committed() {
        let c = db();
        save_budget(
            &c,
            &FinancialBudgetInput {
                id: "b-cash".into(),
                year_month: "2026-10".into(),
                scope: "PERSONAL".into(),
                category_id: "personal-food".into(),
                limit_cents: 10_000,
            },
        )
        .unwrap();
        c.execute("INSERT INTO financial_transactions(id,transaction_type,scope,description,amount_cents,category_id,competence_date,due_date,paid_at,status) VALUES ('paid','EXPENSE','PERSONAL','Pago',1000,'personal-food','2026-09-30','2026-10-01','2026-10-02','PAID'),('planned','EXPENSE','PERSONAL','Previsto',2000,'personal-food','2026-11-01','2026-10-20',NULL,'PLANNED')",[]).unwrap();
        let cash = list_budgets(&c, "2026-10", "CASH").unwrap().remove(0);
        assert_eq!(
            (
                cash.realized_cents,
                cash.committed_cents,
                cash.available_cents
            ),
            (1000, 2000, 7000)
        );
        let competence = list_budgets(&c, "2026-10", "COMPETENCE").unwrap().remove(0);
        assert_eq!(
            (competence.realized_cents, competence.committed_cents),
            (0, 0)
        );
    }
    #[test]
    fn projection_deduplicates_persisted_occurrence_and_settlement_removes_it() {
        let c = db();
        c.execute("INSERT INTO finance_contracts(id,name,counterparty,scope,amount_cents,frequency,start_date,first_due_date,category_id) VALUES ('contract-a','Receita','Cliente','PROFESSIONAL',5000,'MONTHLY','2026-10-01','2026-10-15','professional-other')",[]).unwrap();
        c.execute("INSERT INTO finance_commitments(id,source_type,contract_id,occurrence_key,direction,scope,category_id,expected_amount_cents,competence_date,due_date) VALUES ('commit-a','CONTRACT','contract-a','2026-10-15','INCOME','PROFESSIONAL','professional-other',5000,'2026-10-15','2026-10-15')",[]).unwrap();
        let first = projection(
            &c,
            "2026-10-01",
            "CASH",
            3,
            0,
            &ProjectionScenario::default(),
        )
        .unwrap();
        assert_eq!(
            first
                .items
                .iter()
                .filter(|x| x.expected_date == "2026-10-15")
                .count(),
            1
        );
        c.execute("INSERT INTO financial_transactions(id,transaction_type,scope,description,amount_cents,category_id,competence_date,due_date,paid_at,status) VALUES ('settled-a','INCOME','PROFESSIONAL','Recebido',5000,'professional-other','2026-10-15','2026-10-15','2026-10-15','PAID')",[]).unwrap();
        c.execute("UPDATE finance_commitments SET status='SETTLED',transaction_id='settled-a',settled_at='2026-10-15' WHERE id='commit-a'",[]).unwrap();
        let settled = projection(
            &c,
            "2026-10-01",
            "CASH",
            3,
            0,
            &ProjectionScenario::default(),
        )
        .unwrap();
        assert!(!settled
            .items
            .iter()
            .any(|x| x.expected_date == "2026-10-15"));
    }
    #[test]
    fn recurrence_clamps_year_and_month_boundaries() {
        let dates = occurrence_dates(
            "MONTHLY",
            "2026-12-31",
            "2026-12-01",
            None,
            None,
            NaiveDate::from_ymd_opt(2026, 12, 1).unwrap(),
            NaiveDate::from_ymd_opt(2027, 2, 28).unwrap(),
        )
        .unwrap();
        assert_eq!(
            dates.iter().map(|x| fmt(*x)).collect::<Vec<_>>(),
            vec!["2026-12-31", "2027-01-31", "2027-02-28"]
        );
    }
    #[test]
    fn alerts_cover_overdue_upcoming_and_budget_threshold_without_duplicates() {
        let c = db();
        save_budget(
            &c,
            &FinancialBudgetInput {
                id: "b-alert".into(),
                year_month: "2026-10".into(),
                scope: "PERSONAL".into(),
                category_id: "personal-food".into(),
                limit_cents: 1000,
            },
        )
        .unwrap();
        c.execute("INSERT INTO financial_transactions(id,transaction_type,scope,description,amount_cents,category_id,competence_date,paid_at,status) VALUES ('spent','EXPENSE','PERSONAL','Gasto',1100,'personal-food','2026-10-01','2026-10-01','PAID')",[]).unwrap();
        c.execute("INSERT INTO finance_recurrences(id,title,scope,amount_cents,frequency,start_date,first_due_date,category_id) VALUES ('r-alert','Conta','PERSONAL',500,'MONTHLY','2026-09-01','2026-09-01','personal-food')",[]).unwrap();
        c.execute("INSERT INTO finance_commitments(id,source_type,recurrence_id,occurrence_key,direction,scope,category_id,expected_amount_cents,competence_date,due_date) VALUES ('late','RECURRING_EXPENSE','r-alert','2026-10-10','EXPENSE','PERSONAL','personal-food',500,'2026-10-10','2026-10-10'),('soon','RECURRING_EXPENSE','r-alert','2026-10-17','EXPENSE','PERSONAL','personal-food',500,'2026-10-17','2026-10-17')",[]).unwrap();
        let result = alerts(&c, "2026-10-14", "2026-10", "CASH", 7).unwrap();
        assert_eq!(
            result.iter().filter(|x| x.alert_type == "OVERDUE").count(),
            1
        );
        assert_eq!(
            result.iter().filter(|x| x.alert_type == "UPCOMING").count(),
            1
        );
        assert_eq!(
            result
                .iter()
                .filter(|x| x.alert_type == "BUDGET_EXCEEDED")
                .count(),
            1
        );
    }
    #[test]
    fn copying_budgets_is_idempotent() {
        let c = db();
        save_budget(
            &c,
            &FinancialBudgetInput {
                id: "source".into(),
                year_month: "2026-10".into(),
                scope: "PERSONAL".into(),
                category_id: "personal-food".into(),
                limit_cents: 1000,
            },
        )
        .unwrap();
        assert_eq!(copy_budgets(&c, "2026-10", "2026-11").unwrap().created, 1);
        let again = copy_budgets(&c, "2026-10", "2026-11").unwrap();
        assert_eq!((again.created, again.skipped), (0, 1));
    }
}
