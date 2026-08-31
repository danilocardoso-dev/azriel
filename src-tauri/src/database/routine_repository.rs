use super::{
    automation_repository,
    routine_models::{Routine, RoutineHistory, RoutineInput, RoutineStep},
};
use crate::automation_policy;
use crate::automation_policy::{ActionRequest, ActionSource, PolicyEngine};
use rusqlite::{params, Connection, OptionalExtension};

const MAX_STEPS: usize = 20;
const MAX_TOTAL_DELAY_MS: i64 = 60_000;

pub fn list_routines(connection: &Connection) -> Result<Vec<Routine>, String> {
    let mut statement = connection
        .prepare("SELECT id FROM routines ORDER BY enabled DESC,name COLLATE NOCASE")
        .map_err(err)?;
    let ids = statement
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(err)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(err)?;
    ids.into_iter()
        .map(|id| get_routine(connection, &id)?.ok_or_else(|| "Rotina não encontrada".into()))
        .collect()
}

pub fn get_routine(connection: &Connection, id: &str) -> Result<Option<Routine>, String> {
    let header = connection
        .query_row(
            "SELECT id,name,description,enabled,confirmation_required,revision,created_at,updated_at FROM routines WHERE id=?1",
            [id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, bool>(3)?,
                    row.get::<_, bool>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                ))
            },
        )
        .optional()
        .map_err(err)?;
    let Some((
        id,
        name,
        description,
        enabled,
        confirmation_required,
        revision,
        created_at,
        updated_at,
    )) = header
    else {
        return Ok(None);
    };
    Ok(Some(Routine {
        steps: list_steps(connection, &id)?,
        id,
        name,
        description,
        enabled,
        confirmation_required,
        revision,
        created_at,
        updated_at,
    }))
}

pub fn save_routine(connection: &mut Connection, input: &RoutineInput) -> Result<Routine, String> {
    validate_input(input)?;
    for step in &input.steps {
        let action = automation_policy::action(&step.action_id)
            .ok_or_else(|| format!("Passo {}: ação não registrada", step.order))?;
        PolicyEngine::authorize(
            connection,
            &ActionRequest {
                action_id: step.action_id.clone(),
                source: ActionSource::Ui,
                target_id: Some(step.target_id.clone()),
            },
            action,
        )
        .map_err(|error| format!("Passo {}: {}", step.order, error.message))?;
    }
    let transaction = connection.transaction().map_err(err)?;
    transaction
        .execute(
            "INSERT INTO routines(id,name,description,enabled,confirmation_required) VALUES (?1,?2,?3,?4,?5)
             ON CONFLICT(id) DO UPDATE SET name=excluded.name,description=excluded.description,enabled=excluded.enabled,confirmation_required=excluded.confirmation_required,revision=routines.revision+1,updated_at=CURRENT_TIMESTAMP",
            params![input.id, input.name.trim(), input.description.trim(), input.enabled, input.confirmation_required],
        )
        .map_err(|error| unique_error(error, "Já existe uma rotina com este nome"))?;
    transaction
        .execute("DELETE FROM routine_steps WHERE routine_id=?1", [&input.id])
        .map_err(err)?;
    for step in &input.steps {
        transaction
            .execute(
                "INSERT INTO routine_steps(id,routine_id,step_order,action_id,target_type,target_id,delay_ms,enabled) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
                params![step.id, input.id, step.order, step.action_id, step.target_type, step.target_id, step.delay_ms, step.enabled],
            )
            .map_err(err)?;
    }
    transaction.commit().map_err(err)?;
    get_routine(connection, &input.id)?.ok_or_else(|| "Rotina não encontrada após salvar".into())
}

pub fn delete_routine(connection: &Connection, id: &str) -> Result<(), String> {
    connection
        .execute("DELETE FROM routines WHERE id=?1", [id])
        .map_err(err)?;
    Ok(())
}

pub fn start_history(
    connection: &Connection,
    routine: &Routine,
    source: &str,
    confirmation_required: bool,
) -> Result<i64, String> {
    let status = if confirmation_required {
        "waiting_confirmation"
    } else {
        "executing"
    };
    connection
        .execute(
            "INSERT INTO routine_history(routine_id,routine_name,routine_revision,source,status,confirmation_required,total_steps) VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![routine.id, routine.name, routine.revision, source, status, confirmation_required, routine.steps.iter().filter(|step| step.enabled).count() as i64],
        )
        .map_err(err)?;
    Ok(connection.last_insert_rowid())
}

pub fn get_history(connection: &Connection, id: i64) -> Result<Option<RoutineHistory>, String> {
    connection.query_row(
        "SELECT id,routine_id,routine_name,routine_revision,source,status,confirmation_required,confirmed,total_steps,completed_steps,failed_step,error,started_at,completed_at FROM routine_history WHERE id=?1",
        [id],
        |row| Ok(RoutineHistory {
            id: row.get(0)?, routine_id: row.get(1)?, routine_name: row.get(2)?, routine_revision: row.get(3)?, source: row.get(4)?, status: row.get(5)?, confirmation_required: row.get(6)?, confirmed: row.get(7)?, total_steps: row.get(8)?, completed_steps: row.get(9)?, failed_step: row.get(10)?, error: row.get(11)?, started_at: row.get(12)?, completed_at: row.get(13)?,
        }),
    ).optional().map_err(err)
}

pub fn begin_confirmed_execution(
    connection: &Connection,
    id: i64,
    revision: i64,
) -> Result<bool, String> {
    connection.execute(
        "UPDATE routine_history SET status='executing',confirmed=1 WHERE id=?1 AND status='waiting_confirmation' AND routine_revision=?2",
        params![id, revision],
    ).map(|changed| changed == 1).map_err(err)
}

pub fn cancel_waiting(connection: &Connection, id: i64) -> Result<bool, String> {
    connection.execute(
        "UPDATE routine_history SET status='cancelled',completed_at=CURRENT_TIMESTAMP,error='Execução cancelada pelo operador' WHERE id=?1 AND status='waiting_confirmation'",
        [id],
    ).map(|changed| changed == 1).map_err(err)
}

pub fn cancel_stale_waiting(connection: &Connection) -> Result<usize, String> {
    connection.execute(
        "UPDATE routine_history SET status='cancelled',completed_at=CURRENT_TIMESTAMP,error='Confirmação expirada após reinício do Azriel' WHERE status='waiting_confirmation'",
        [],
    ).map_err(err)
}

pub fn update_progress(
    connection: &Connection,
    id: i64,
    completed_steps: i64,
) -> Result<(), String> {
    connection
        .execute(
            "UPDATE routine_history SET completed_steps=?1 WHERE id=?2 AND status='executing'",
            params![completed_steps, id],
        )
        .map_err(err)?;
    Ok(())
}

pub fn finish_history(
    connection: &Connection,
    id: i64,
    success: bool,
    completed_steps: i64,
    failed_step: Option<i64>,
    error_message: Option<&str>,
) -> Result<(), String> {
    connection.execute(
        "UPDATE routine_history SET status=?1,completed_steps=?2,failed_step=?3,error=?4,completed_at=CURRENT_TIMESTAMP WHERE id=?5",
        params![if success { "completed" } else { "failed" }, completed_steps, failed_step, error_message, id],
    ).map_err(err)?;
    Ok(())
}

pub fn list_history(connection: &Connection, limit: i64) -> Result<Vec<RoutineHistory>, String> {
    let mut statement = connection.prepare(
        "SELECT id,routine_id,routine_name,routine_revision,source,status,confirmation_required,confirmed,total_steps,completed_steps,failed_step,error,started_at,completed_at FROM routine_history ORDER BY id DESC LIMIT ?1"
    ).map_err(err)?;
    let rows = statement
        .query_map([limit.clamp(1, 500)], |row| {
            Ok(RoutineHistory {
                id: row.get(0)?,
                routine_id: row.get(1)?,
                routine_name: row.get(2)?,
                routine_revision: row.get(3)?,
                source: row.get(4)?,
                status: row.get(5)?,
                confirmation_required: row.get(6)?,
                confirmed: row.get(7)?,
                total_steps: row.get(8)?,
                completed_steps: row.get(9)?,
                failed_step: row.get(10)?,
                error: row.get(11)?,
                started_at: row.get(12)?,
                completed_at: row.get(13)?,
            })
        })
        .map_err(err)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(err)
}

fn list_steps(connection: &Connection, routine_id: &str) -> Result<Vec<RoutineStep>, String> {
    let mut statement = connection.prepare(
        "SELECT id,step_order,action_id,target_type,target_id,delay_ms,enabled FROM routine_steps WHERE routine_id=?1 ORDER BY step_order"
    ).map_err(err)?;
    let rows = statement
        .query_map([routine_id], |row| {
            Ok(RoutineStep {
                id: row.get(0)?,
                order: row.get(1)?,
                action_id: row.get(2)?,
                target_type: row.get(3)?,
                target_id: row.get(4)?,
                delay_ms: row.get(5)?,
                enabled: row.get(6)?,
            })
        })
        .map_err(err)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(err)
}

fn validate_input(input: &RoutineInput) -> Result<(), String> {
    automation_repository::validate_id(&input.id, "rotina")?;
    let name = input.name.trim();
    if name.is_empty() || name.chars().count() > 120 {
        return Err("O nome da rotina deve ter entre 1 e 120 caracteres".into());
    }
    if input.description.chars().count() > 500 {
        return Err("A descrição da rotina deve ter no máximo 500 caracteres".into());
    }
    if input.steps.is_empty() || input.steps.len() > MAX_STEPS {
        return Err(format!(
            "A rotina deve possuir entre 1 e {MAX_STEPS} passos"
        ));
    }
    if !input.steps.iter().any(|step| step.enabled) {
        return Err("A rotina precisa possuir ao menos um passo ativo".into());
    }
    let mut total_delay = 0;
    for (index, step) in input.steps.iter().enumerate() {
        automation_repository::validate_id(&step.id, "passo")?;
        automation_repository::validate_id(&step.target_id, "alvo")?;
        let action = automation_policy::action(&step.action_id)
            .ok_or_else(|| format!("Passo {}: ação não registrada", index + 1))?;
        if action.target_type != step.target_type {
            return Err(format!(
                "Passo {}: o tipo de alvo não corresponde à ação",
                index + 1
            ));
        }
        if step.order != (index + 1) as i64 {
            return Err("A ordem dos passos deve ser contínua e começar em 1".into());
        }
        if !(0..=10_000).contains(&step.delay_ms) {
            return Err(format!(
                "Passo {}: o intervalo deve estar entre 0 e 10000 ms",
                index + 1
            ));
        }
        total_delay += step.delay_ms;
    }
    if total_delay > MAX_TOTAL_DELAY_MS {
        return Err("A soma dos intervalos da rotina não pode ultrapassar 60000 ms".into());
    }
    Ok(())
}

fn unique_error(error: rusqlite::Error, message: &str) -> String {
    match error {
        rusqlite::Error::SqliteFailure(_, Some(detail))
            if detail.contains("UNIQUE constraint failed") =>
        {
            message.into()
        }
        other => other.to_string(),
    }
}

fn err(error: rusqlite::Error) -> String {
    error.to_string()
}
