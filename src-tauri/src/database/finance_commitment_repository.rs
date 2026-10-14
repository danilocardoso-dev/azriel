use super::{finance_commitment_models::*, finance_models::*, finance_repository};
use chrono::{Datelike, Duration, Months, NaiveDate};
use rusqlite::{params, Connection, OptionalExtension, Row};

pub fn list_contracts(connection: &Connection) -> Result<Vec<FinanceContract>, String> {
    query_all(connection, "SELECT c.id,c.name,c.counterparty,c.description,c.scope,c.amount_cents,c.currency,c.frequency,c.start_date,c.end_date,c.first_due_date,c.status,c.category_id,k.name,c.notes,c.resume_date,c.created_at,c.updated_at FROM finance_contracts c JOIN financial_categories k ON k.id=c.category_id ORDER BY c.status,c.name COLLATE NOCASE", map_contract)
}

pub fn save_contract(
    connection: &Connection,
    input: &FinanceContractInput,
) -> Result<FinanceContract, String> {
    validate_origin(
        &input.id,
        &input.name,
        &input.scope,
        input.amount_cents,
        &input.frequency,
        &input.start_date,
        input.end_date.as_deref(),
        &input.first_due_date,
    )?;
    validate_category(
        connection,
        &input.category_id,
        &input.scope,
        "INCOME",
        existing_contract_category(connection, &input.id)?.as_deref(),
    )?;
    connection.execute("INSERT INTO finance_contracts(id,name,counterparty,description,scope,amount_cents,frequency,start_date,end_date,first_due_date,category_id,notes) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12) ON CONFLICT(id) DO UPDATE SET name=excluded.name,counterparty=excluded.counterparty,description=excluded.description,scope=excluded.scope,amount_cents=excluded.amount_cents,frequency=excluded.frequency,start_date=excluded.start_date,end_date=excluded.end_date,first_due_date=excluded.first_due_date,category_id=excluded.category_id,notes=excluded.notes,updated_at=CURRENT_TIMESTAMP", params![input.id,input.name.trim(),required(&input.counterparty,"Contraparte")?,clean(input.description.as_deref()),input.scope,input.amount_cents,input.frequency,input.start_date,input.end_date,input.first_due_date,input.category_id,clean(input.notes.as_deref())]).map_err(err)?;
    get_contract(connection, &input.id)?.ok_or_else(|| "Contrato não encontrado após salvar".into())
}

pub fn list_recurrences(connection: &Connection) -> Result<Vec<FinanceRecurrence>, String> {
    query_all(connection, "SELECT r.id,r.title,r.scope,r.amount_cents,r.estimated,r.frequency,r.start_date,r.end_date,r.first_due_date,r.status,r.category_id,k.name,r.notes,r.resume_date,r.created_at,r.updated_at FROM finance_recurrences r JOIN financial_categories k ON k.id=r.category_id ORDER BY r.status,r.title COLLATE NOCASE", map_recurrence)
}

pub fn save_recurrence(
    connection: &Connection,
    input: &FinanceRecurrenceInput,
) -> Result<FinanceRecurrence, String> {
    validate_origin(
        &input.id,
        &input.title,
        &input.scope,
        input.amount_cents,
        &input.frequency,
        &input.start_date,
        input.end_date.as_deref(),
        &input.first_due_date,
    )?;
    validate_category(
        connection,
        &input.category_id,
        &input.scope,
        "EXPENSE",
        existing_recurrence_category(connection, &input.id)?.as_deref(),
    )?;
    connection.execute("INSERT INTO finance_recurrences(id,title,scope,amount_cents,estimated,frequency,start_date,end_date,first_due_date,category_id,notes) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11) ON CONFLICT(id) DO UPDATE SET title=excluded.title,scope=excluded.scope,amount_cents=excluded.amount_cents,estimated=excluded.estimated,frequency=excluded.frequency,start_date=excluded.start_date,end_date=excluded.end_date,first_due_date=excluded.first_due_date,category_id=excluded.category_id,notes=excluded.notes,updated_at=CURRENT_TIMESTAMP", params![input.id,input.title.trim(),input.scope,input.amount_cents,input.estimated,input.frequency,input.start_date,input.end_date,input.first_due_date,input.category_id,clean(input.notes.as_deref())]).map_err(err)?;
    get_recurrence(connection, &input.id)?
        .ok_or_else(|| "Recorrência não encontrada após salvar".into())
}

pub fn list_loans(connection: &Connection) -> Result<Vec<FinanceLoan>, String> {
    query_all(connection, "SELECT l.id,l.name,l.creditor,l.purpose,l.scope,l.original_principal_cents,l.informed_balance_cents,l.balance_as_of,l.installment_count,l.interest_rate_basis_points,l.interest_rate_period,l.status,l.category_id,k.name,l.notes,l.resume_date,l.created_at,l.updated_at FROM finance_loans l JOIN financial_categories k ON k.id=l.category_id ORDER BY l.status,l.name COLLATE NOCASE", map_loan)
}

pub fn save_loan(connection: &Connection, input: &FinanceLoanInput) -> Result<FinanceLoan, String> {
    validate_id(&input.id)?;
    required(&input.name, "Nome")?;
    required(&input.creditor, "Credor")?;
    validate_scope(&input.scope)?;
    validate_date(&input.balance_as_of)?;
    if input.original_principal_cents <= 0
        || input.informed_balance_cents < 0
        || input.installment_count <= 0
    {
        return Err("Valores do empréstimo são inválidos".into());
    }
    match (
        input.interest_rate_basis_points,
        input.interest_rate_period.as_deref(),
    ) {
        (None, None) => {}
        (Some(value), Some(period)) if value >= 0 && ["MONTHLY", "YEARLY"].contains(&period) => {}
        _ => return Err("Taxa de juros deve possuir valor e período válidos".into()),
    }
    validate_category(
        connection,
        &input.category_id,
        &input.scope,
        "EXPENSE",
        existing_loan_category(connection, &input.id)?.as_deref(),
    )?;
    connection.execute("INSERT INTO finance_loans(id,name,creditor,purpose,scope,original_principal_cents,informed_balance_cents,balance_as_of,installment_count,interest_rate_basis_points,interest_rate_period,category_id,notes) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13) ON CONFLICT(id) DO UPDATE SET name=excluded.name,creditor=excluded.creditor,purpose=excluded.purpose,scope=excluded.scope,original_principal_cents=excluded.original_principal_cents,informed_balance_cents=excluded.informed_balance_cents,balance_as_of=excluded.balance_as_of,installment_count=excluded.installment_count,interest_rate_basis_points=excluded.interest_rate_basis_points,interest_rate_period=excluded.interest_rate_period,category_id=excluded.category_id,notes=excluded.notes,updated_at=CURRENT_TIMESTAMP", params![input.id,input.name.trim(),input.creditor.trim(),clean(input.purpose.as_deref()),input.scope,input.original_principal_cents,input.informed_balance_cents,input.balance_as_of,input.installment_count,input.interest_rate_basis_points,input.interest_rate_period,input.category_id,clean(input.notes.as_deref())]).map_err(err)?;
    get_loan(connection, &input.id)?.ok_or_else(|| "Empréstimo não encontrado após salvar".into())
}

pub fn list_installments(
    connection: &Connection,
    loan_id: &str,
) -> Result<Vec<FinanceLoanInstallment>, String> {
    validate_id(loan_id)?;
    let mut statement=connection.prepare("SELECT i.id,i.loan_id,l.name,i.installment_number,i.due_date,i.amount_cents,i.principal_cents,i.interest_cents,i.fees_cents,i.notes,c.id,c.status,i.created_at,i.updated_at FROM finance_loan_installments i JOIN finance_loans l ON l.id=i.loan_id LEFT JOIN finance_commitments c ON c.loan_installment_id=i.id WHERE i.loan_id=?1 ORDER BY i.installment_number").map_err(err)?;
    let result = statement
        .query_map([loan_id], map_installment)
        .map_err(err)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(err);
    result
}

pub fn save_installment(
    connection: &Connection,
    input: &FinanceLoanInstallmentInput,
) -> Result<FinanceLoanInstallment, String> {
    validate_id(&input.id)?;
    validate_id(&input.loan_id)?;
    validate_date(&input.due_date)?;
    if input.installment_number <= 0
        || input.amount_cents <= 0
        || [
            input.principal_cents,
            input.interest_cents,
            input.fees_cents,
        ]
        .into_iter()
        .flatten()
        .any(|value| value < 0)
    {
        return Err("Dados da parcela são inválidos".into());
    }
    if connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM finance_loans WHERE id=?1)",
            [&input.loan_id],
            |r| r.get::<_, bool>(0),
        )
        .map_err(err)?
        == false
    {
        return Err("Empréstimo não encontrado".into());
    }
    let settled: bool=connection.query_row("SELECT EXISTS(SELECT 1 FROM finance_commitments WHERE loan_installment_id=?1 AND status='SETTLED')",[&input.id],|r|r.get(0)).map_err(err)?;
    if settled {
        return Err("Parcela quitada não pode ser alterada".into());
    }
    connection.execute("INSERT INTO finance_loan_installments(id,loan_id,installment_number,due_date,amount_cents,principal_cents,interest_cents,fees_cents,notes) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9) ON CONFLICT(id) DO UPDATE SET loan_id=excluded.loan_id,installment_number=excluded.installment_number,due_date=excluded.due_date,amount_cents=excluded.amount_cents,principal_cents=excluded.principal_cents,interest_cents=excluded.interest_cents,fees_cents=excluded.fees_cents,notes=excluded.notes,updated_at=CURRENT_TIMESTAMP",params![input.id,input.loan_id,input.installment_number,input.due_date,input.amount_cents,input.principal_cents,input.interest_cents,input.fees_cents,clean(input.notes.as_deref())]).map_err(err)?;
    list_installments(connection, &input.loan_id)?
        .into_iter()
        .find(|item| item.id == input.id)
        .ok_or_else(|| "Parcela não encontrada após salvar".into())
}

pub fn set_origin_status(
    connection: &Connection,
    source_type: &str,
    id: &str,
    status: &str,
    effective_date: &str,
) -> Result<(), String> {
    validate_id(id)?;
    validate_date(effective_date)?;
    let (table, valid) = match source_type {
        "CONTRACT" => (
            "finance_contracts",
            ["ACTIVE", "PAUSED", "ENDED", "CANCELLED"].as_slice(),
        ),
        "RECURRING_EXPENSE" => (
            "finance_recurrences",
            ["ACTIVE", "PAUSED", "ENDED", "CANCELLED"].as_slice(),
        ),
        "LOAN" => (
            "finance_loans",
            ["ACTIVE", "PAUSED", "PAID", "CANCELLED"].as_slice(),
        ),
        _ => return Err("Origem financeira inválida".into()),
    };
    if !valid.contains(&status) {
        return Err("Status da origem inválido".into());
    }
    let sql=format!("UPDATE {table} SET status=?2,resume_date=CASE WHEN ?2='ACTIVE' THEN ?3 ELSE resume_date END,updated_at=CURRENT_TIMESTAMP WHERE id=?1");
    if connection
        .execute(&sql, params![id, status, effective_date])
        .map_err(err)?
        == 0
    {
        return Err("Origem não encontrada".into());
    }
    if ["ENDED", "CANCELLED", "PAID"].contains(&status) {
        let predicate = match source_type {
            "CONTRACT" => "contract_id=?1",
            "RECURRING_EXPENSE" => "recurrence_id=?1",
            _ => {
                "loan_installment_id IN (SELECT id FROM finance_loan_installments WHERE loan_id=?1)"
            }
        };
        connection.execute(&format!("UPDATE finance_commitments SET status='CANCELLED',updated_at=CURRENT_TIMESTAMP WHERE {predicate} AND status='PENDING' AND due_date>=?2"),params![id,effective_date]).map_err(err)?;
    }
    Ok(())
}

pub fn generate_commitments(
    connection: &Connection,
    today: &str,
    horizon_months: u32,
) -> Result<FinanceCommitmentGenerationResult, String> {
    let today = parse_date(today)?;
    if horizon_months > 12 {
        return Err("A janela máxima é de 12 meses".into());
    }
    let window_start = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap();
    let next = window_start
        .checked_add_months(Months::new(horizon_months + 1))
        .ok_or("Janela de geração inválida")?;
    let window_end = next - Duration::days(1);
    let transaction = connection.unchecked_transaction().map_err(err)?;
    let mut result = FinanceCommitmentGenerationResult {
        created: 0,
        existing: 0,
        window_start: fmt(window_start),
        window_end: fmt(window_end),
    };
    for item in list_contracts(&transaction)?
        .into_iter()
        .filter(|x| x.status == "ACTIVE")
    {
        for due in dates(
            &item.frequency,
            &item.first_due_date,
            &item.start_date,
            item.end_date.as_deref(),
            item.resume_date.as_deref(),
            window_start,
            window_end,
        )? {
            insert_commitment(
                &transaction,
                &mut result,
                "CONTRACT",
                Some(&item.id),
                None,
                None,
                "INCOME",
                &item.scope,
                &item.category_id,
                item.amount_cents,
                false,
                due,
                &fmt(due),
            )?;
        }
    }
    for item in list_recurrences(&transaction)?
        .into_iter()
        .filter(|x| x.status == "ACTIVE")
    {
        for due in dates(
            &item.frequency,
            &item.first_due_date,
            &item.start_date,
            item.end_date.as_deref(),
            item.resume_date.as_deref(),
            window_start,
            window_end,
        )? {
            insert_commitment(
                &transaction,
                &mut result,
                "RECURRING_EXPENSE",
                None,
                Some(&item.id),
                None,
                "EXPENSE",
                &item.scope,
                &item.category_id,
                item.amount_cents,
                item.estimated,
                due,
                &fmt(due),
            )?;
        }
    }
    let mut statement=transaction.prepare("SELECT i.id,l.scope,l.category_id,i.amount_cents,i.due_date,i.installment_number FROM finance_loan_installments i JOIN finance_loans l ON l.id=i.loan_id WHERE l.status='ACTIVE' AND i.due_date BETWEEN ?1 AND ?2 AND (l.resume_date IS NULL OR i.due_date>=l.resume_date)").map_err(err)?;
    let rows = statement
        .query_map(params![fmt(window_start), fmt(window_end)], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, i64>(5)?,
            ))
        })
        .map_err(err)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(err)?;
    drop(statement);
    for (id, scope, category, amount, due, number) in rows {
        let parsed = parse_date(&due)?;
        insert_commitment(
            &transaction,
            &mut result,
            "LOAN_INSTALLMENT",
            None,
            None,
            Some(&id),
            "EXPENSE",
            &scope,
            &category,
            amount,
            false,
            parsed,
            &format!("installment-{number}"),
        )?;
    }
    transaction.commit().map_err(err)?;
    Ok(result)
}

#[allow(clippy::too_many_arguments)]
fn insert_commitment(
    connection: &Connection,
    result: &mut FinanceCommitmentGenerationResult,
    source_type: &str,
    contract: Option<&str>,
    recurrence: Option<&str>,
    installment: Option<&str>,
    direction: &str,
    scope: &str,
    category: &str,
    amount: i64,
    estimated: bool,
    due: NaiveDate,
    key: &str,
) -> Result<(), String> {
    let source = contract.or(recurrence).or(installment).unwrap();
    let id = safe_id(&format!("commitment-{source}-{key}"));
    let changed=connection.execute("INSERT OR IGNORE INTO finance_commitments(id,source_type,contract_id,recurrence_id,loan_installment_id,occurrence_key,direction,scope,category_id,expected_amount_cents,estimated,competence_date,due_date) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?12)",params![id,source_type,contract,recurrence,installment,key,direction,scope,category,amount,estimated,fmt(due)]).map_err(err)?;
    if changed == 1 {
        result.created += 1
    } else {
        result.existing += 1
    }
    Ok(())
}

pub fn list_commitments(
    connection: &Connection,
    filter: &FinanceCommitmentFilter,
) -> Result<Vec<FinanceCommitment>, String> {
    validate_commitment_filter(filter)?;
    let today = filter.today.as_deref().unwrap_or("9999-12-31");
    let mut items = query_commitments_with_today(connection, today)?;
    items.retain(|x| {
        filter
            .date_from
            .as_deref()
            .is_none_or(|v| x.due_date.as_str() >= v)
            && filter
                .date_to
                .as_deref()
                .is_none_or(|v| x.due_date.as_str() <= v)
            && filter
                .source_type
                .as_deref()
                .is_none_or(|v| x.source_type == v)
            && filter.scope.as_deref().is_none_or(|v| x.scope == v)
            && filter.direction.as_deref().is_none_or(|v| x.direction == v)
            && filter
                .status
                .as_deref()
                .is_none_or(|v| x.effective_status == v)
    });
    Ok(items)
}

fn query_commitments_with_today(
    connection: &Connection,
    today: &str,
) -> Result<Vec<FinanceCommitment>, String> {
    let mut s = connection.prepare(COMMITMENT_SELECT).map_err(err)?;
    let result = s
        .query_map([today], map_commitment)
        .map_err(err)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(err);
    result
}
const COMMITMENT_SELECT:&str="SELECT c.id,c.source_type,COALESCE(c.contract_id,c.recurrence_id,c.loan_installment_id),CASE c.source_type WHEN 'CONTRACT' THEN ct.name WHEN 'RECURRING_EXPENSE' THEN r.title ELSE l.name||' · parcela '||i.installment_number END,c.occurrence_key,c.direction,c.scope,c.category_id,k.name,c.expected_amount_cents,c.estimated,c.competence_date,c.due_date,c.status,CASE WHEN c.status='PENDING' AND c.due_date<?1 THEN 'OVERDUE' ELSE c.status END,c.transaction_id,c.settled_at,c.created_at,c.updated_at FROM finance_commitments c LEFT JOIN finance_contracts ct ON ct.id=c.contract_id LEFT JOIN finance_recurrences r ON r.id=c.recurrence_id LEFT JOIN finance_loan_installments i ON i.id=c.loan_installment_id LEFT JOIN finance_loans l ON l.id=i.loan_id JOIN financial_categories k ON k.id=c.category_id ORDER BY c.due_date,c.created_at";

pub fn get_commitment(
    connection: &Connection,
    id: &str,
    today: &str,
) -> Result<Option<FinanceCommitment>, String> {
    validate_id(id)?;
    validate_date(today)?;
    Ok(query_commitments_with_today(connection, today)?
        .into_iter()
        .find(|item| item.id == id))
}

pub fn adjust_commitment(
    connection: &Connection,
    input: &FinanceCommitmentAdjustmentInput,
    today: &str,
) -> Result<FinanceCommitment, String> {
    validate_id(&input.id)?;
    validate_date(&input.competence_date)?;
    validate_date(&input.due_date)?;
    if input.expected_amount_cents <= 0 {
        return Err("Valor previsto deve ser maior que zero".into());
    }
    if connection.execute("UPDATE finance_commitments SET expected_amount_cents=?2,competence_date=?3,due_date=?4,estimated=?5,updated_at=CURRENT_TIMESTAMP WHERE id=?1 AND status='PENDING'",params![input.id,input.expected_amount_cents,input.competence_date,input.due_date,input.estimated]).map_err(err)?==0{return Err("Somente compromissos pendentes podem ser ajustados".into());}
    get_commitment_by_id(connection, &input.id, today)
}

pub fn cancel_commitment(
    connection: &Connection,
    id: &str,
    today: &str,
) -> Result<FinanceCommitment, String> {
    if connection.execute("UPDATE finance_commitments SET status='CANCELLED',updated_at=CURRENT_TIMESTAMP WHERE id=?1 AND status='PENDING'",[id]).map_err(err)?==0{return Err("Somente compromissos pendentes podem ser cancelados".into());}
    get_commitment_by_id(connection, id, today)
}

pub fn settle_commitment(
    connection: &Connection,
    input: &FinanceCommitmentSettlementInput,
    today: &str,
) -> Result<FinanceCommitmentSettlementResult, String> {
    validate_id(&input.commitment_id)?;
    validate_id(&input.transaction_id)?;
    validate_date(&input.paid_at)?;
    if input.actual_amount_cents <= 0 {
        return Err("Valor efetivo deve ser maior que zero".into());
    }
    let transaction = connection.unchecked_transaction().map_err(err)?;
    let commitment = get_commitment_by_id(&transaction, &input.commitment_id, today)?;
    if commitment.status == "SETTLED" {
        let id = commitment.transaction_id.clone().unwrap();
        transaction.commit().map_err(err)?;
        return Ok(FinanceCommitmentSettlementResult {
            commitment,
            transaction_id: id,
            reused: true,
        });
    }
    if commitment.status != "PENDING" {
        return Err("Compromisso cancelado não pode ser baixado".into());
    }
    let tx_input = FinancialTransactionInput {
        id: input.transaction_id.clone(),
        transaction_type: commitment.direction.clone(),
        scope: commitment.scope.clone(),
        description: commitment.source_name.clone(),
        amount_cents: input.actual_amount_cents,
        category_id: commitment.category_id.clone(),
        competence_date: commitment.competence_date.clone(),
        due_date: Some(commitment.due_date.clone()),
        paid_at: Some(input.paid_at.clone()),
        status: "PAID".into(),
        notes: input.notes.clone(),
    };
    finance_repository::save_transaction(&transaction, &tx_input)?;
    if transaction.execute("UPDATE finance_commitments SET status='SETTLED',transaction_id=?2,settled_at=?3,updated_at=CURRENT_TIMESTAMP WHERE id=?1 AND status='PENDING' AND transaction_id IS NULL",params![input.commitment_id,input.transaction_id,input.paid_at]).map_err(err)?!=1{return Err("O compromisso já foi alterado; recarregue a tela".into());}
    transaction.commit().map_err(err)?;
    let commitment = get_commitment_by_id(connection, &input.commitment_id, today)?;
    Ok(FinanceCommitmentSettlementResult {
        commitment,
        transaction_id: input.transaction_id.clone(),
        reused: false,
    })
}

pub fn link_transaction(
    connection: &Connection,
    input: &FinanceCommitmentLinkInput,
    today: &str,
) -> Result<FinanceCommitmentSettlementResult, String> {
    let transaction = connection.unchecked_transaction().map_err(err)?;
    let commitment = get_commitment_by_id(&transaction, &input.commitment_id, today)?;
    if commitment.status == "SETTLED"
        && commitment.transaction_id.as_deref() == Some(&input.transaction_id)
    {
        transaction.commit().map_err(err)?;
        return Ok(FinanceCommitmentSettlementResult {
            commitment,
            transaction_id: input.transaction_id.clone(),
            reused: true,
        });
    }
    if commitment.status != "PENDING" {
        return Err("Somente compromissos pendentes podem receber vínculo".into());
    }
    let movement = finance_repository::get_transaction(&transaction, &input.transaction_id)?
        .ok_or("Lançamento não encontrado")?;
    if movement.status != "PAID"
        || movement.transaction_type != commitment.direction
        || movement.scope != commitment.scope
        || movement.category_id != commitment.category_id
    {
        return Err("O lançamento não é compatível com tipo, classificação, categoria e status do compromisso".into());
    }
    let paid = movement
        .paid_at
        .ok_or("Lançamento realizado sem data de caixa")?;
    transaction.execute("UPDATE finance_commitments SET status='SETTLED',transaction_id=?2,settled_at=?3,updated_at=CURRENT_TIMESTAMP WHERE id=?1 AND status='PENDING'",params![input.commitment_id,input.transaction_id,paid]).map_err(err)?;
    transaction.commit().map_err(err)?;
    let commitment = get_commitment_by_id(connection, &input.commitment_id, today)?;
    Ok(FinanceCommitmentSettlementResult {
        commitment,
        transaction_id: input.transaction_id.clone(),
        reused: false,
    })
}

pub fn commitment_summary(
    connection: &Connection,
    today: &str,
) -> Result<FinanceCommitmentSummary, String> {
    let today = parse_date(today)?;
    let end = today + Duration::days(30);
    let commitments = query_commitments_with_today(connection, &fmt(today))?;
    let mut out = FinanceCommitmentSummary::default();
    for c in commitments {
        if c.status == "PENDING" {
            if c.due_date < fmt(today) {
                out.overdue_count += 1;
                if c.direction == "INCOME" {
                    out.overdue_receivable_cents += c.expected_amount_cents
                } else {
                    out.overdue_payable_cents += c.expected_amount_cents
                }
            } else if c.due_date <= fmt(end) {
                if c.direction == "INCOME" {
                    out.receivable_next_30_cents += c.expected_amount_cents
                } else {
                    out.payable_next_30_cents += c.expected_amount_cents
                }
            }
            if c.source_type == "LOAN_INSTALLMENT" && c.due_date >= fmt(today) {
                out.future_installments_cents += c.expected_amount_cents
            }
        }
    }
    out.active_contracts = connection
        .query_row(
            "SELECT COUNT(*) FROM finance_contracts WHERE status='ACTIVE'",
            [],
            |r| r.get(0),
        )
        .map_err(err)?;
    out.contracted_monthly_equivalent_cents =
        monthly_equivalent(connection, "finance_contracts", None)?;
    out.professional_recurring_monthly_equivalent_cents =
        monthly_equivalent(connection, "finance_recurrences", Some("PROFESSIONAL"))?;
    out.informed_debt_balance_cents=connection.query_row("SELECT COALESCE(SUM(informed_balance_cents),0) FROM finance_loans WHERE status IN ('ACTIVE','PAUSED')",[],|r|r.get(0)).map_err(err)?;
    Ok(out)
}

fn monthly_equivalent(
    connection: &Connection,
    table: &str,
    scope: Option<&str>,
) -> Result<i64, String> {
    let sql = format!(
        "SELECT amount_cents,frequency FROM {table} WHERE status='ACTIVE'{}",
        if scope.is_some() { " AND scope=?1" } else { "" }
    );
    let mut s = connection.prepare(&sql).map_err(err)?;
    let rows = if let Some(value) = scope {
        s.query_map([value], |r| {
            Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?))
        })
        .map_err(err)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(err)?
    } else {
        s.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))
            .map_err(err)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(err)?
    };
    Ok(rows
        .into_iter()
        .map(|(amount, f)| match f.as_str() {
            "MONTHLY" => amount,
            "WEEKLY" => amount * 52 / 12,
            "YEARLY" => amount / 12,
            _ => 0,
        })
        .sum())
}

fn dates(
    frequency: &str,
    first: &str,
    start: &str,
    end: Option<&str>,
    resume: Option<&str>,
    window_start: NaiveDate,
    window_end: NaiveDate,
) -> Result<Vec<NaiveDate>, String> {
    let anchor = parse_date(first)?;
    let lower = [
        Some(parse_date(start)?),
        resume.map(parse_date).transpose()?,
        Some(window_start),
    ]
    .into_iter()
    .flatten()
    .max()
    .unwrap();
    let upper = end
        .map(parse_date)
        .transpose()?
        .map_or(window_end, |x| x.min(window_end));
    let mut out = Vec::new();
    match frequency {
        "MONTHLY" => {
            let mut month =
                NaiveDate::from_ymd_opt(window_start.year(), window_start.month(), 1).unwrap();
            while month <= upper {
                let d = clamped(month.year(), month.month(), anchor.day());
                if d >= anchor && d >= lower && d <= upper {
                    out.push(d)
                }
                month = month.checked_add_months(Months::new(1)).unwrap();
            }
        }
        "WEEKLY" => {
            let mut d = anchor;
            while d < lower {
                d += Duration::days(7)
            }
            while d <= upper {
                out.push(d);
                d += Duration::days(7)
            }
        }
        "YEARLY" => {
            for year in lower.year()..=upper.year() {
                let d = clamped(year, anchor.month(), anchor.day());
                if d >= anchor && d >= lower && d <= upper {
                    out.push(d)
                }
            }
        }
        "ONE_TIME" => {
            if anchor >= lower && anchor <= upper {
                out.push(anchor)
            }
        }
        _ => return Err("Periodicidade inválida".into()),
    }
    Ok(out)
}
fn clamped(year: i32, month: u32, day: u32) -> NaiveDate {
    let next = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap()
    };
    let last = (next - Duration::days(1)).day();
    NaiveDate::from_ymd_opt(year, month, day.min(last)).unwrap()
}

fn get_contract(c: &Connection, id: &str) -> Result<Option<FinanceContract>, String> {
    c.query_row("SELECT x.id,x.name,x.counterparty,x.description,x.scope,x.amount_cents,x.currency,x.frequency,x.start_date,x.end_date,x.first_due_date,x.status,x.category_id,k.name,x.notes,x.resume_date,x.created_at,x.updated_at FROM finance_contracts x JOIN financial_categories k ON k.id=x.category_id WHERE x.id=?1",[id],map_contract).optional().map_err(err)
}
fn get_recurrence(c: &Connection, id: &str) -> Result<Option<FinanceRecurrence>, String> {
    c.query_row("SELECT x.id,x.title,x.scope,x.amount_cents,x.estimated,x.frequency,x.start_date,x.end_date,x.first_due_date,x.status,x.category_id,k.name,x.notes,x.resume_date,x.created_at,x.updated_at FROM finance_recurrences x JOIN financial_categories k ON k.id=x.category_id WHERE x.id=?1",[id],map_recurrence).optional().map_err(err)
}
fn get_loan(c: &Connection, id: &str) -> Result<Option<FinanceLoan>, String> {
    c.query_row("SELECT x.id,x.name,x.creditor,x.purpose,x.scope,x.original_principal_cents,x.informed_balance_cents,x.balance_as_of,x.installment_count,x.interest_rate_basis_points,x.interest_rate_period,x.status,x.category_id,k.name,x.notes,x.resume_date,x.created_at,x.updated_at FROM finance_loans x JOIN financial_categories k ON k.id=x.category_id WHERE x.id=?1",[id],map_loan).optional().map_err(err)
}
fn get_commitment_by_id(
    c: &Connection,
    id: &str,
    today: &str,
) -> Result<FinanceCommitment, String> {
    query_commitments_with_today(c, today)?
        .into_iter()
        .find(|x| x.id == id)
        .ok_or_else(|| "Compromisso não encontrado".into())
}

fn map_contract(r: &Row) -> rusqlite::Result<FinanceContract> {
    Ok(FinanceContract {
        id: r.get(0)?,
        name: r.get(1)?,
        counterparty: r.get(2)?,
        description: r.get(3)?,
        scope: r.get(4)?,
        amount_cents: r.get(5)?,
        currency: r.get(6)?,
        frequency: r.get(7)?,
        start_date: r.get(8)?,
        end_date: r.get(9)?,
        first_due_date: r.get(10)?,
        status: r.get(11)?,
        category_id: r.get(12)?,
        category_name: r.get(13)?,
        notes: r.get(14)?,
        resume_date: r.get(15)?,
        created_at: r.get(16)?,
        updated_at: r.get(17)?,
    })
}
fn map_recurrence(r: &Row) -> rusqlite::Result<FinanceRecurrence> {
    Ok(FinanceRecurrence {
        id: r.get(0)?,
        title: r.get(1)?,
        scope: r.get(2)?,
        amount_cents: r.get(3)?,
        estimated: r.get(4)?,
        frequency: r.get(5)?,
        start_date: r.get(6)?,
        end_date: r.get(7)?,
        first_due_date: r.get(8)?,
        status: r.get(9)?,
        category_id: r.get(10)?,
        category_name: r.get(11)?,
        notes: r.get(12)?,
        resume_date: r.get(13)?,
        created_at: r.get(14)?,
        updated_at: r.get(15)?,
    })
}
fn map_loan(r: &Row) -> rusqlite::Result<FinanceLoan> {
    Ok(FinanceLoan {
        id: r.get(0)?,
        name: r.get(1)?,
        creditor: r.get(2)?,
        purpose: r.get(3)?,
        scope: r.get(4)?,
        original_principal_cents: r.get(5)?,
        informed_balance_cents: r.get(6)?,
        balance_as_of: r.get(7)?,
        installment_count: r.get(8)?,
        interest_rate_basis_points: r.get(9)?,
        interest_rate_period: r.get(10)?,
        status: r.get(11)?,
        category_id: r.get(12)?,
        category_name: r.get(13)?,
        notes: r.get(14)?,
        resume_date: r.get(15)?,
        created_at: r.get(16)?,
        updated_at: r.get(17)?,
    })
}
fn map_installment(r: &Row) -> rusqlite::Result<FinanceLoanInstallment> {
    Ok(FinanceLoanInstallment {
        id: r.get(0)?,
        loan_id: r.get(1)?,
        loan_name: r.get(2)?,
        installment_number: r.get(3)?,
        due_date: r.get(4)?,
        amount_cents: r.get(5)?,
        principal_cents: r.get(6)?,
        interest_cents: r.get(7)?,
        fees_cents: r.get(8)?,
        notes: r.get(9)?,
        commitment_id: r.get(10)?,
        commitment_status: r.get(11)?,
        created_at: r.get(12)?,
        updated_at: r.get(13)?,
    })
}
fn map_commitment(r: &Row) -> rusqlite::Result<FinanceCommitment> {
    Ok(FinanceCommitment {
        id: r.get(0)?,
        source_type: r.get(1)?,
        source_id: r.get(2)?,
        source_name: r.get(3)?,
        occurrence_key: r.get(4)?,
        direction: r.get(5)?,
        scope: r.get(6)?,
        category_id: r.get(7)?,
        category_name: r.get(8)?,
        expected_amount_cents: r.get(9)?,
        estimated: r.get(10)?,
        competence_date: r.get(11)?,
        due_date: r.get(12)?,
        status: r.get(13)?,
        effective_status: r.get(14)?,
        transaction_id: r.get(15)?,
        settled_at: r.get(16)?,
        created_at: r.get(17)?,
        updated_at: r.get(18)?,
    })
}
fn query_all<T, F>(c: &Connection, sql: &str, map: F) -> Result<Vec<T>, String>
where
    F: FnMut(&Row) -> rusqlite::Result<T>,
{
    let mut s = c.prepare(sql).map_err(err)?;
    let result = s
        .query_map([], map)
        .map_err(err)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(err);
    result
}
fn validate_origin(
    id: &str,
    name: &str,
    scope: &str,
    amount: i64,
    frequency: &str,
    start: &str,
    end: Option<&str>,
    due: &str,
) -> Result<(), String> {
    validate_id(id)?;
    required(name, "Nome")?;
    validate_scope(scope)?;
    if amount <= 0 {
        return Err("Valor deve ser maior que zero".into());
    }
    if !["MONTHLY", "WEEKLY", "YEARLY", "ONE_TIME"].contains(&frequency) {
        return Err("Periodicidade inválida".into());
    }
    let start = parse_date(start)?;
    let due = parse_date(due)?;
    if due < start {
        return Err("Primeiro vencimento não pode anteceder o início".into());
    }
    if let Some(end) = end {
        if parse_date(end)? < start {
            return Err("Fim não pode anteceder o início".into());
        }
    }
    Ok(())
}
fn validate_category(
    c: &Connection,
    id: &str,
    scope: &str,
    direction: &str,
    existing: Option<&str>,
) -> Result<(), String> {
    let cat =
        finance_repository::get_category(c, id)?.ok_or("Categoria financeira não encontrada")?;
    if cat.scope != scope {
        return Err("Categoria não pertence à classificação escolhida".into());
    }
    if !cat.enabled && existing != Some(id) {
        return Err("Categoria selecionada está desativada".into());
    }
    let expected = if direction == "INCOME" {
        "income"
    } else {
        "expense"
    };
    if id.starts_with("fin-") && !id.contains(expected) { /* custom categories remain valid */ }
    Ok(())
}
fn existing_contract_category(c: &Connection, id: &str) -> Result<Option<String>, String> {
    c.query_row(
        "SELECT category_id FROM finance_contracts WHERE id=?1",
        [id],
        |r| r.get(0),
    )
    .optional()
    .map_err(err)
}
fn existing_recurrence_category(c: &Connection, id: &str) -> Result<Option<String>, String> {
    c.query_row(
        "SELECT category_id FROM finance_recurrences WHERE id=?1",
        [id],
        |r| r.get(0),
    )
    .optional()
    .map_err(err)
}
fn existing_loan_category(c: &Connection, id: &str) -> Result<Option<String>, String> {
    c.query_row(
        "SELECT category_id FROM finance_loans WHERE id=?1",
        [id],
        |r| r.get(0),
    )
    .optional()
    .map_err(err)
}
fn validate_commitment_filter(f: &FinanceCommitmentFilter) -> Result<(), String> {
    for d in [
        f.date_from.as_deref(),
        f.date_to.as_deref(),
        f.today.as_deref(),
    ]
    .into_iter()
    .flatten()
    {
        validate_date(d)?;
    }
    if let (Some(a), Some(b)) = (f.date_from.as_deref(), f.date_to.as_deref()) {
        if a > b {
            return Err("Período inválido".into());
        }
    }
    if let Some(x) = f.source_type.as_deref() {
        if !["CONTRACT", "RECURRING_EXPENSE", "LOAN_INSTALLMENT"].contains(&x) {
            return Err("Origem inválida".into());
        }
    }
    if let Some(x) = f.scope.as_deref() {
        validate_scope(x)?;
    }
    if let Some(x) = f.direction.as_deref() {
        if !["INCOME", "EXPENSE"].contains(&x) {
            return Err("Direção inválida".into());
        }
    }
    if let Some(x) = f.status.as_deref() {
        if !["PENDING", "SETTLED", "OVERDUE", "CANCELLED"].contains(&x) {
            return Err("Status inválido".into());
        }
    }
    Ok(())
}
fn validate_id(v: &str) -> Result<(), String> {
    if !v.is_empty()
        && v.len() <= 100
        && v.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        Ok(())
    } else {
        Err("ID financeiro inválido".into())
    }
}
fn validate_scope(v: &str) -> Result<(), String> {
    if ["PERSONAL", "PROFESSIONAL"].contains(&v) {
        Ok(())
    } else {
        Err("Classificação financeira inválida".into())
    }
}
fn validate_date(v: &str) -> Result<(), String> {
    parse_date(v).map(|_| ())
}
fn parse_date(v: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(v, "%Y-%m-%d").map_err(|_| "Data inválida; use AAAA-MM-DD".into())
}
fn required<'a>(v: &'a str, label: &str) -> Result<&'a str, String> {
    let x = v.trim();
    if x.is_empty() {
        Err(format!("{label} é obrigatório"))
    } else {
        Ok(x)
    }
}
fn clean(v: Option<&str>) -> Option<String> {
    v.map(str::trim)
        .filter(|x| !x.is_empty())
        .map(str::to_string)
}
fn fmt(d: NaiveDate) -> String {
    d.format("%Y-%m-%d").to_string()
}
fn safe_id(value: &str) -> String {
    value
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .take(100)
        .collect()
}
fn err<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
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
    fn contract(id: &str, first: &str) -> FinanceContractInput {
        FinanceContractInput {
            id: id.into(),
            name: "Contrato".into(),
            counterparty: "Cliente".into(),
            description: None,
            scope: "PROFESSIONAL".into(),
            amount_cents: 10000,
            frequency: "MONTHLY".into(),
            start_date: "2024-01-01".into(),
            end_date: None,
            first_due_date: first.into(),
            category_id: "professional-digital-services".into(),
            notes: None,
        }
    }
    #[test]
    fn repeated_generation_is_idempotent_and_clamps_month_end() {
        let c = db();
        save_contract(&c, &contract("c1", "2024-01-31")).unwrap();
        let first = generate_commitments(&c, "2024-02-10", 1).unwrap();
        let second = generate_commitments(&c, "2024-02-10", 1).unwrap();
        assert_eq!(first.created, 2);
        assert_eq!(second.created, 0);
        let dates = list_commitments(
            &c,
            &FinanceCommitmentFilter {
                today: Some("2024-02-10".into()),
                ..Default::default()
            },
        )
        .unwrap()
        .into_iter()
        .map(|x| x.due_date)
        .collect::<Vec<_>>();
        assert_eq!(dates, vec!["2024-02-29", "2024-03-31"]);
    }
    #[test]
    fn settlement_is_atomic_and_idempotent() {
        let c = db();
        save_contract(&c, &contract("c2", "2024-02-15")).unwrap();
        generate_commitments(&c, "2024-02-01", 0).unwrap();
        let id = list_commitments(
            &c,
            &FinanceCommitmentFilter {
                today: Some("2024-02-01".into()),
                ..Default::default()
            },
        )
        .unwrap()[0]
            .id
            .clone();
        let input = FinanceCommitmentSettlementInput {
            commitment_id: id,
            transaction_id: "tx-c2".into(),
            actual_amount_cents: 9900,
            paid_at: "2024-02-16".into(),
            notes: None,
        };
        assert!(!settle_commitment(&c, &input, "2024-02-16").unwrap().reused);
        assert!(settle_commitment(&c, &input, "2024-02-16").unwrap().reused);
        assert_eq!(
            c.query_row("SELECT COUNT(*) FROM financial_transactions", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            1
        );
    }
    #[test]
    fn cancelling_linked_transaction_reopens_commitment() {
        let c = db();
        save_contract(&c, &contract("c3", "2024-02-15")).unwrap();
        generate_commitments(&c, "2024-02-01", 0).unwrap();
        let id = list_commitments(
            &c,
            &FinanceCommitmentFilter {
                today: Some("2024-02-01".into()),
                ..Default::default()
            },
        )
        .unwrap()[0]
            .id
            .clone();
        settle_commitment(
            &c,
            &FinanceCommitmentSettlementInput {
                commitment_id: id.clone(),
                transaction_id: "tx-c3".into(),
                actual_amount_cents: 10000,
                paid_at: "2024-02-15".into(),
                notes: None,
            },
            "2024-02-15",
        )
        .unwrap();
        finance_repository::cancel_transaction(&c, "tx-c3").unwrap();
        let reopened = get_commitment_by_id(&c, &id, "2024-02-15").unwrap();
        assert_eq!(reopened.status, "PENDING");
        assert!(reopened.transaction_id.is_none());
    }
    #[test]
    fn paused_contract_does_not_generate_new_commitments() {
        let c = db();
        save_contract(&c, &contract("c4", "2024-02-15")).unwrap();
        set_origin_status(&c, "CONTRACT", "c4", "PAUSED", "2024-02-01").unwrap();
        let result = generate_commitments(&c, "2024-02-01", 1).unwrap();
        assert_eq!(result.created, 0);
    }
    #[test]
    fn existing_paid_transaction_can_be_linked_once() {
        let c = db();
        save_contract(&c, &contract("c5", "2024-02-15")).unwrap();
        generate_commitments(&c, "2024-02-01", 0).unwrap();
        let commitment = list_commitments(
            &c,
            &FinanceCommitmentFilter {
                today: Some("2024-02-01".into()),
                ..Default::default()
            },
        )
        .unwrap()
        .remove(0);
        finance_repository::save_transaction(
            &c,
            &FinancialTransactionInput {
                id: "existing-c5".into(),
                transaction_type: "INCOME".into(),
                scope: "PROFESSIONAL".into(),
                description: "Recebimento".into(),
                amount_cents: 10000,
                category_id: "professional-digital-services".into(),
                competence_date: "2024-02-15".into(),
                due_date: Some("2024-02-15".into()),
                paid_at: Some("2024-02-16".into()),
                status: "PAID".into(),
                notes: None,
            },
        )
        .unwrap();
        let input = FinanceCommitmentLinkInput {
            commitment_id: commitment.id,
            transaction_id: "existing-c5".into(),
        };
        assert!(!link_transaction(&c, &input, "2024-02-16").unwrap().reused);
        assert!(link_transaction(&c, &input, "2024-02-16").unwrap().reused);
    }
}
