use super::study_review_models::*;
use super::study_review_scheduler::{schedule, SCHEDULER_VERSION};
use chrono::{DateTime, Local, Utc};
use rusqlite::{params, Connection, OptionalExtension, Row};

const CARD_SELECT: &str = "SELECT card.id,card.front,card.back,card.status,
            card.roadmap_id,roadmap.name,card.stage_id,stage.name,card.topic_id,topic.name,
            card.activity_id,activity.title,card.notebook_id,notebook.title,card.note_id,note.title,
            review.due_at,review.last_reviewed_at,review.review_count,review.correct_count,
            review.incorrect_count,review.current_interval_seconds,review.scheduler_version,
            card.created_at,card.updated_at
     FROM study_cards card
     JOIN study_review_states review ON review.card_id=card.id
     LEFT JOIN study_roadmaps roadmap ON roadmap.id=card.roadmap_id
     LEFT JOIN roadmap_stages stage ON stage.id=card.stage_id
     LEFT JOIN roadmap_topics topic ON topic.id=card.topic_id
     LEFT JOIN roadmap_activities activity ON activity.id=card.activity_id
     LEFT JOIN study_notebooks notebook ON notebook.id=card.notebook_id
     LEFT JOIN study_notes note ON note.id=card.note_id";

#[derive(Default, Clone)]
struct CardContext {
    roadmap_id: Option<String>,
    stage_id: Option<String>,
    topic_id: Option<String>,
    activity_id: Option<String>,
    notebook_id: Option<String>,
    note_id: Option<String>,
}

pub fn save_card(connection: &Connection, input: &StudyCardInput) -> Result<StudyCard, String> {
    save_card_at(connection, input, Utc::now())
}

pub fn save_cards(
    connection: &mut Connection,
    inputs: &[StudyCardInput],
) -> Result<Vec<StudyCard>, String> {
    if inputs.is_empty() || inputs.len() > 10 {
        return Err("Selecione entre 1 e 10 cards para salvar".into());
    }
    let mut identifiers = std::collections::HashSet::new();
    if inputs
        .iter()
        .any(|input| !identifiers.insert(input.id.trim().to_string()))
    {
        return Err("A lista contém identificadores de card duplicados".into());
    }
    let transaction = connection.transaction().map_err(err)?;
    let now = Utc::now();
    let mut saved = Vec::with_capacity(inputs.len());
    for input in inputs {
        saved.push(save_card_at(&transaction, input, now)?);
    }
    transaction.commit().map_err(err)?;
    Ok(saved)
}

fn save_card_at(
    connection: &Connection,
    input: &StudyCardInput,
    now: DateTime<Utc>,
) -> Result<StudyCard, String> {
    validate_id(&input.id)?;
    let front = input.front.trim();
    let back = input.back.trim();
    if front.is_empty() || front.chars().count() > 4_000 {
        return Err("A pergunta deve ter entre 1 e 4000 caracteres".into());
    }
    if back.is_empty() || back.chars().count() > 12_000 {
        return Err("A resposta deve ter entre 1 e 12000 caracteres".into());
    }
    let context = resolve_context(connection, input)?;
    let timestamp = now.to_rfc3339();
    connection.execute(
        "INSERT INTO study_cards(
           id,front,back,status,roadmap_id,stage_id,topic_id,activity_id,notebook_id,note_id,created_at,updated_at
         ) VALUES (?1,?2,?3,'ACTIVE',?4,?5,?6,?7,?8,?9,?10,?10)
         ON CONFLICT(id) DO UPDATE SET
           front=excluded.front,back=excluded.back,roadmap_id=excluded.roadmap_id,
           stage_id=excluded.stage_id,topic_id=excluded.topic_id,activity_id=excluded.activity_id,
           notebook_id=excluded.notebook_id,note_id=excluded.note_id,updated_at=excluded.updated_at",
        params![
            input.id,
            front,
            back,
            context.roadmap_id,
            context.stage_id,
            context.topic_id,
            context.activity_id,
            context.notebook_id,
            context.note_id,
            timestamp,
        ],
    ).map_err(err)?;
    connection
        .execute(
            "INSERT OR IGNORE INTO study_review_states(
           card_id,due_at,last_reviewed_at,review_count,correct_count,incorrect_count,
           current_interval_seconds,scheduler_version,updated_at
         ) VALUES (?1,?2,NULL,0,0,0,0,?3,?2)",
            params![input.id, timestamp, SCHEDULER_VERSION],
        )
        .map_err(err)?;
    if input.source_material_ids.len() > 10 {
        return Err("Um card pode receber no máximo 10 materiais de origem".into());
    }
    for material_id in &input.source_material_ids {
        validate_id(material_id)?;
        let material_exists = connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM study_materials WHERE id=?1 AND removed_at IS NULL)",
            [material_id],
            |row| row.get::<_, i64>(0),
        ).map_err(err)? == 1;
        if !material_exists { return Err("Material de origem não encontrado na biblioteca".into()); }
        let relation_id = format!("material-relation-{material_id}-card-{}", input.id);
        connection.execute(
            "INSERT OR IGNORE INTO study_material_relations(id,material_id,relation_type,relation_id,created_at) VALUES (?1,?2,'CARD',?3,?4)",
            params![relation_id, material_id, input.id, timestamp],
        ).map_err(err)?;
    }
    get_card(connection, &input.id)?.ok_or_else(|| "Card não encontrado após salvar".into())
}

pub fn get_card(connection: &Connection, id: &str) -> Result<Option<StudyCard>, String> {
    validate_id(id)?;
    connection
        .query_row(&format!("{CARD_SELECT} WHERE card.id=?1"), [id], map_card)
        .optional()
        .map_err(err)
}

pub fn list_cards(
    connection: &Connection,
    input: &StudyCardListInput,
) -> Result<Vec<StudyCard>, String> {
    validate_optional_id(input.roadmap_id.as_deref())?;
    validate_optional_id(input.activity_id.as_deref())?;
    validate_optional_id(input.note_id.as_deref())?;
    let status = input
        .status
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    if let Some(value) = status {
        validate_status(value)?;
    }
    let filter = input
        .filter
        .as_deref()
        .unwrap_or("ALL")
        .trim()
        .to_uppercase();
    if !["ALL", "DUE", "NEW", "SUSPENDED", "ARCHIVED"].contains(&filter.as_str()) {
        return Err("Filtro de cards inválido".into());
    }
    let pattern = input
        .query
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| format!("%{}%", escape_like(value)));
    let now = Utc::now().to_rfc3339();
    let mut statement = connection.prepare(&format!(
        "{CARD_SELECT}
         WHERE (?1 IS NULL OR card.status=?1)
           AND (?2 IS NULL OR card.front LIKE ?2 ESCAPE '\\' OR card.back LIKE ?2 ESCAPE '\\')
           AND (?3 IS NULL OR card.roadmap_id=?3)
           AND (?4 IS NULL OR card.activity_id=?4)
           AND (?5 IS NULL OR card.note_id=?5)
           AND (
             ?6='ALL'
             OR (?6='DUE' AND card.status='ACTIVE' AND review.review_count>0 AND date(review.due_at,'localtime')<=date(?7,'localtime'))
             OR (?6='NEW' AND card.status='ACTIVE' AND review.review_count=0)
             OR (?6='SUSPENDED' AND card.status='SUSPENDED')
             OR (?6='ARCHIVED' AND card.status='ARCHIVED')
           )
         ORDER BY card.updated_at DESC,card.id DESC LIMIT ?8 OFFSET ?9"
    )).map_err(err)?;
    let rows = statement
        .query_map(
            params![
                status,
                pattern,
                input.roadmap_id,
                input.activity_id,
                input.note_id,
                filter,
                now,
                input.limit.clamp(1, 100),
                input.offset.max(0),
            ],
            map_card,
        )
        .map_err(err)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(err)
}

pub fn set_card_status(
    connection: &Connection,
    id: &str,
    status: &str,
) -> Result<StudyCard, String> {
    validate_id(id)?;
    validate_status(status)?;
    let changed = connection
        .execute(
            "UPDATE study_cards SET status=?1,updated_at=?2 WHERE id=?3",
            params![status, Utc::now().to_rfc3339(), id],
        )
        .map_err(err)?;
    if changed == 0 {
        return Err("Card de estudo não encontrado".into());
    }
    get_card(connection, id)?.ok_or_else(|| "Card não encontrado após atualizar".into())
}

pub fn delete_card(connection: &Connection, id: &str) -> Result<(), String> {
    validate_id(id)?;
    let referenced: bool = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM study_review_session_items WHERE card_id=?1)",
            [id],
            |row| row.get(0),
        )
        .map_err(err)?;
    if referenced {
        return Err(
            "Este card já pertence ao histórico de revisão. Arquive-o para preservar a auditoria"
                .into(),
        );
    }
    let changed = connection
        .execute("DELETE FROM study_cards WHERE id=?1", [id])
        .map_err(err)?;
    if changed == 0 {
        return Err("Card de estudo não encontrado".into());
    }
    Ok(())
}

pub fn dashboard_summary(connection: &Connection) -> Result<StudyReviewDashboardSummary, String> {
    dashboard_summary_at(connection, Utc::now())
}

fn dashboard_summary_at(
    connection: &Connection,
    now: DateTime<Utc>,
) -> Result<StudyReviewDashboardSummary, String> {
    let timestamp = now.to_rfc3339();
    let (overdue, due_today, new_cards) = connection.query_row(
        "SELECT
           COALESCE(SUM(CASE WHEN card.status='ACTIVE' AND review.review_count>0 AND date(review.due_at,'localtime')<date(?1,'localtime') THEN 1 ELSE 0 END),0),
           COALESCE(SUM(CASE WHEN card.status='ACTIVE' AND review.review_count>0 AND date(review.due_at,'localtime')=date(?1,'localtime') THEN 1 ELSE 0 END),0),
           COALESCE(SUM(CASE WHEN card.status='ACTIVE' AND review.review_count=0 THEN 1 ELSE 0 END),0)
         FROM study_cards card JOIN study_review_states review ON review.card_id=card.id",
        [&timestamp],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    ).map_err(err)?;
    let reviewed_today = connection.query_row(
        "SELECT COUNT(*) FROM study_review_events WHERE date(reviewed_at,'localtime')=date(?1,'localtime')",
        [&timestamp],
        |row| row.get(0),
    ).map_err(err)?;
    let last_reviewed_at = connection
        .query_row(
            "SELECT MAX(reviewed_at) FROM study_review_events",
            [],
            |row| row.get(0),
        )
        .map_err(err)?;
    Ok(StudyReviewDashboardSummary {
        overdue,
        due_today,
        new_cards,
        reviewed_today,
        last_reviewed_at,
    })
}

pub fn get_review_queue(
    connection: &Connection,
    mode: &str,
) -> Result<Vec<StudyReviewQueueItem>, String> {
    get_review_queue_at(connection, mode, Utc::now())
}

fn get_review_queue_at(
    connection: &Connection,
    mode: &str,
    now: DateTime<Utc>,
) -> Result<Vec<StudyReviewQueueItem>, String> {
    let (limit, overdue_only) = queue_mode(mode)?;
    let timestamp = now.to_rfc3339();
    let mut statement = connection.prepare(&format!(
        "{CARD_SELECT}
         WHERE card.status='ACTIVE'
           AND (
             (?1=1 AND review.review_count>0 AND date(review.due_at,'localtime')<date(?2,'localtime'))
             OR
             (?1=0 AND (review.review_count=0 OR date(review.due_at,'localtime')<=date(?2,'localtime')))
           )
         ORDER BY
           CASE
             WHEN review.review_count>0 AND date(review.due_at,'localtime')<date(?2,'localtime') THEN 0
             WHEN review.review_count>0 THEN 1
             ELSE 2
           END,
           review.due_at ASC,card.created_at ASC,card.id ASC
         LIMIT ?3"
    )).map_err(err)?;
    let rows = statement
        .query_map(params![overdue_only, timestamp, limit], map_card)
        .map_err(err)?;
    rows.enumerate()
        .map(|(index, row)| {
            let card = row.map_err(err)?;
            Ok(StudyReviewQueueItem {
                item_id: None,
                order: index as i64 + 1,
                category: queue_category(&card, now),
                card,
            })
        })
        .collect()
}

pub fn start_review_session(
    connection: &mut Connection,
    input: &StartStudyReviewSessionInput,
) -> Result<StudyReviewSession, String> {
    start_review_session_at(connection, input, Utc::now())
}

fn start_review_session_at(
    connection: &mut Connection,
    input: &StartStudyReviewSessionInput,
    now: DateTime<Utc>,
) -> Result<StudyReviewSession, String> {
    validate_id(&input.id)?;
    validate_optional_id(input.study_session_id.as_deref())?;
    if let Some(study_session_id) = input.study_session_id.as_deref() {
        let exists: bool = connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM study_sessions WHERE id=?1)",
                [study_session_id],
                |row| row.get(0),
            )
            .map_err(err)?;
        if !exists {
            return Err("Sessão de foco vinculada não encontrada".into());
        }
    }
    let queue = get_review_queue_at(connection, &input.mode, now)?;
    if queue.is_empty() {
        return Err("Não existem cards disponíveis para esta fila".into());
    }
    let timestamp = now.to_rfc3339();
    let transaction = connection.transaction().map_err(err)?;
    transaction.execute(
        "INSERT INTO study_review_sessions(id,started_at,ended_at,status,planned_cards,reviewed_cards,study_session_id,open_slot,created_at)
         VALUES (?1,?2,NULL,'ACTIVE',?3,0,?4,1,?2)",
        params![input.id, timestamp, queue.len() as i64, input.study_session_id],
    ).map_err(|error| if error.to_string().contains("idx_study_review_sessions_single_open") { "Já existe uma sessão de revisão ativa".into() } else { err(error) })?;
    for item in queue {
        let item_id = format!("{}:{:03}", input.id, item.order);
        transaction.execute(
            "INSERT INTO study_review_session_items(id,review_session_id,card_id,item_order,status)
             VALUES (?1,?2,?3,?4,'PENDING')",
            params![item_id, input.id, item.card.id, item.order],
        ).map_err(err)?;
    }
    transaction.commit().map_err(err)?;
    load_review_session(connection, &input.id)?
        .ok_or_else(|| "Sessão de revisão não encontrada após iniciar".into())
}

pub fn get_active_review_session(
    connection: &Connection,
) -> Result<Option<StudyReviewSession>, String> {
    let id = connection
        .query_row(
            "SELECT id FROM study_review_sessions WHERE open_slot=1 LIMIT 1",
            [],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(err)?;
    id.map(|value| load_review_session(connection, &value))
        .transpose()
        .map(|value| value.flatten())
}

pub fn submit_review_result(
    connection: &mut Connection,
    input: &SubmitStudyReviewResultInput,
) -> Result<StudyReviewResult, String> {
    submit_review_result_at(connection, input, Utc::now())
}

fn submit_review_result_at(
    connection: &mut Connection,
    input: &SubmitStudyReviewResultInput,
    now: DateTime<Utc>,
) -> Result<StudyReviewResult, String> {
    validate_id(&input.id)?;
    validate_id(&input.review_session_id)?;
    if input.session_item_id.trim().is_empty() || input.session_item_id.len() > 160 {
        return Err("Item de revisão inválido".into());
    }
    if let Some(value) = input.response_time_ms {
        if !(0..=86_400_000).contains(&value) {
            return Err("Tempo de resposta inválido".into());
        }
    }
    let result = input.result.trim().to_uppercase();
    let transaction = connection.transaction().map_err(err)?;
    let session_status = transaction
        .query_row(
            "SELECT status FROM study_review_sessions WHERE id=?1",
            [&input.review_session_id],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(err)?
        .ok_or_else(|| "Sessão de revisão não encontrada".to_string())?;
    if session_status != "ACTIVE" {
        return Err("A sessão de revisão já foi encerrada".into());
    }
    let current_item = transaction
        .query_row(
            "SELECT id,card_id FROM study_review_session_items
         WHERE review_session_id=?1 AND status='PENDING' ORDER BY item_order LIMIT 1",
            [&input.review_session_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )
        .optional()
        .map_err(err)?
        .ok_or_else(|| "A sessão não possui cards pendentes".to_string())?;
    if current_item.0 != input.session_item_id {
        return Err("O card informado não é o próximo item da fila".into());
    }
    let state = transaction
        .query_row(
            "SELECT due_at,current_interval_seconds FROM study_review_states WHERE card_id=?1",
            [&current_item.1],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)),
        )
        .map_err(err)?;
    let outcome = schedule(state.1, &result, now)?;
    let reviewed_at = now.to_rfc3339();
    let next_due_at = outcome.next_due_at.to_rfc3339();
    transaction.execute(
        "INSERT INTO study_review_events(
           id,card_id,review_session_id,session_item_id,reviewed_at,result,response_time_ms,
           previous_due_at,next_due_at,previous_interval_seconds,next_interval_seconds,scheduler_version
         ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
        params![input.id,current_item.1,input.review_session_id,input.session_item_id,reviewed_at,result,input.response_time_ms,state.0,next_due_at,state.1,outcome.next_interval_seconds,SCHEDULER_VERSION],
    ).map_err(|error| if error.to_string().contains("UNIQUE constraint failed: study_review_events.session_item_id") { "Este card já foi avaliado nesta sessão".into() } else { err(error) })?;
    transaction
        .execute(
            "UPDATE study_review_states SET
           due_at=?1,last_reviewed_at=?2,review_count=review_count+1,
           correct_count=correct_count+?3,incorrect_count=incorrect_count+?4,
           current_interval_seconds=?5,scheduler_version=?6,updated_at=?2
         WHERE card_id=?7",
            params![
                next_due_at,
                reviewed_at,
                outcome.correct_increment,
                outcome.incorrect_increment,
                outcome.next_interval_seconds,
                SCHEDULER_VERSION,
                current_item.1
            ],
        )
        .map_err(err)?;
    let changed = transaction.execute(
        "UPDATE study_review_session_items SET status='REVIEWED',reviewed_at=?1 WHERE id=?2 AND status='PENDING'",
        params![reviewed_at, input.session_item_id],
    ).map_err(err)?;
    if changed != 1 {
        return Err("Este card já foi processado".into());
    }
    transaction
        .execute(
            "UPDATE study_review_sessions SET reviewed_cards=reviewed_cards+1 WHERE id=?1",
            [&input.review_session_id],
        )
        .map_err(err)?;
    let pending: i64 = transaction.query_row(
        "SELECT COUNT(*) FROM study_review_session_items WHERE review_session_id=?1 AND status='PENDING'",
        [&input.review_session_id],
        |row| row.get(0),
    ).map_err(err)?;
    if pending == 0 {
        transaction.execute(
            "UPDATE study_review_sessions SET status='COMPLETED',ended_at=?1,open_slot=NULL WHERE id=?2",
            params![reviewed_at, input.review_session_id],
        ).map_err(err)?;
    }
    transaction.commit().map_err(err)?;
    Ok(StudyReviewResult {
        session: load_review_session(connection, &input.review_session_id)?
            .ok_or_else(|| "Sessão de revisão não encontrada após avaliação".to_string())?,
        next_due_at,
        next_interval_seconds: outcome.next_interval_seconds,
    })
}

pub fn cancel_review_session(
    connection: &mut Connection,
    id: &str,
) -> Result<StudyReviewSession, String> {
    validate_id(id)?;
    let timestamp = Utc::now().to_rfc3339();
    let transaction = connection.transaction().map_err(err)?;
    let changed = transaction.execute(
        "UPDATE study_review_sessions SET status='CANCELLED',ended_at=?1,open_slot=NULL WHERE id=?2 AND status='ACTIVE'",
        params![timestamp, id],
    ).map_err(err)?;
    if changed == 0 {
        return Err("Sessão de revisão ativa não encontrada".into());
    }
    transaction.execute(
        "UPDATE study_review_session_items SET status='SKIPPED' WHERE review_session_id=?1 AND status='PENDING'",
        [id],
    ).map_err(err)?;
    transaction.commit().map_err(err)?;
    load_review_session(connection, id)?
        .ok_or_else(|| "Sessão de revisão não encontrada após cancelar".into())
}

pub fn complete_review_session(
    connection: &mut Connection,
    id: &str,
) -> Result<StudyReviewSession, String> {
    validate_id(id)?;
    let pending: i64 = connection.query_row(
        "SELECT COUNT(*) FROM study_review_session_items WHERE review_session_id=?1 AND status='PENDING'",
        [id],
        |row| row.get(0),
    ).map_err(err)?;
    if pending > 0 {
        return Err("Avalie todos os cards ou cancele a sessão".into());
    }
    let timestamp = Utc::now().to_rfc3339();
    connection.execute(
        "UPDATE study_review_sessions SET status='COMPLETED',ended_at=COALESCE(ended_at,?1),open_slot=NULL WHERE id=?2 AND status='ACTIVE'",
        params![timestamp, id],
    ).map_err(err)?;
    load_review_session(connection, id)?.ok_or_else(|| "Sessão de revisão não encontrada".into())
}

pub fn get_review_summary(connection: &Connection, id: &str) -> Result<StudyReviewSummary, String> {
    validate_id(id)?;
    let session = connection.query_row(
        "SELECT status,reviewed_cards,started_at,ended_at FROM study_review_sessions WHERE id=?1",
        [id],
        |row| Ok((row.get::<_, String>(0)?,row.get::<_, i64>(1)?,row.get::<_, String>(2)?,row.get::<_, Option<String>>(3)?)),
    ).optional().map_err(err)?.ok_or_else(|| "Sessão de revisão não encontrada".to_string())?;
    let counts = connection
        .query_row(
            "SELECT
           COALESCE(SUM(result='AGAIN'),0),COALESCE(SUM(result='HARD'),0),
           COALESCE(SUM(result='GOOD'),0),COALESCE(SUM(result='EASY'),0)
         FROM study_review_events WHERE review_session_id=?1",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .map_err(err)?;
    let next_review_at = connection
        .query_row(
            "SELECT MIN(review.due_at) FROM study_review_events event
         JOIN study_review_states review ON review.card_id=event.card_id
         WHERE event.review_session_id=?1",
            [id],
            |row| row.get(0),
        )
        .map_err(err)?;
    let start = DateTime::parse_from_rfc3339(&session.2).map_err(|error| error.to_string())?;
    let end = session
        .3
        .as_deref()
        .map(DateTime::parse_from_rfc3339)
        .transpose()
        .map_err(|error| error.to_string())?
        .unwrap_or_else(|| Utc::now().into());
    Ok(StudyReviewSummary {
        session_id: id.to_string(),
        status: session.0,
        reviewed_cards: session.1,
        again: counts.0,
        hard: counts.1,
        good: counts.2,
        easy: counts.3,
        duration_seconds: (end - start).num_seconds().max(0),
        next_review_at,
    })
}

pub fn list_review_sessions(
    connection: &Connection,
    input: &StudyReviewSessionListInput,
) -> Result<Vec<StudyReviewSession>, String> {
    let mut statement = connection.prepare(
        "SELECT id FROM study_review_sessions ORDER BY started_at DESC,id DESC LIMIT ?1 OFFSET ?2"
    ).map_err(err)?;
    let ids = statement
        .query_map(
            params![input.limit.clamp(1, 100), input.offset.max(0)],
            |row| row.get::<_, String>(0),
        )
        .map_err(err)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(err)?;
    ids.into_iter()
        .map(|id| {
            load_review_session(connection, &id)?
                .ok_or_else(|| "Sessão de revisão não encontrada".into())
        })
        .collect()
}

fn load_review_session(
    connection: &Connection,
    id: &str,
) -> Result<Option<StudyReviewSession>, String> {
    let base = connection.query_row(
        "SELECT id,started_at,ended_at,status,planned_cards,reviewed_cards,study_session_id,created_at
         FROM study_review_sessions WHERE id=?1",
        [id],
        |row| Ok((row.get::<_,String>(0)?,row.get::<_,String>(1)?,row.get::<_,Option<String>>(2)?,row.get::<_,String>(3)?,row.get::<_,i64>(4)?,row.get::<_,i64>(5)?,row.get::<_,Option<String>>(6)?,row.get::<_,String>(7)?)),
    ).optional().map_err(err)?;
    let Some(base) = base else {
        return Ok(None);
    };
    let context_label = connection.query_row(
        "SELECT CASE
           WHEN COUNT(DISTINCT COALESCE(roadmap.name,notebook.title,'REVISÃO LIVRE')) > 1 THEN 'MÚLTIPLOS CONTEXTOS'
           ELSE COALESCE(MAX(COALESCE(roadmap.name,notebook.title)),'REVISÃO LIVRE') END
         FROM study_review_session_items item
         JOIN study_cards card ON card.id=item.card_id
         LEFT JOIN study_roadmaps roadmap ON roadmap.id=card.roadmap_id
         LEFT JOIN study_notebooks notebook ON notebook.id=card.notebook_id
         WHERE item.review_session_id=?1",
        [id],
        |row| row.get(0),
    ).map_err(err)?;
    let current_item = load_current_item(connection, id)?;
    Ok(Some(StudyReviewSession {
        id: base.0,
        started_at: base.1,
        ended_at: base.2,
        status: base.3,
        planned_cards: base.4,
        reviewed_cards: base.5,
        study_session_id: base.6,
        context_label,
        current_item,
        created_at: base.7,
    }))
}

fn load_current_item(
    connection: &Connection,
    session_id: &str,
) -> Result<Option<StudyReviewQueueItem>, String> {
    let item = connection
        .query_row(
            "SELECT id,card_id,item_order FROM study_review_session_items
         WHERE review_session_id=?1 AND status='PENDING' ORDER BY item_order LIMIT 1",
            [session_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            },
        )
        .optional()
        .map_err(err)?;
    let Some((item_id, card_id, order)) = item else {
        return Ok(None);
    };
    let card =
        get_card(connection, &card_id)?.ok_or_else(|| "Card da fila não encontrado".to_string())?;
    Ok(Some(StudyReviewQueueItem {
        item_id: Some(item_id),
        order,
        category: queue_category(&card, Utc::now()),
        card,
    }))
}

fn queue_mode(mode: &str) -> Result<(i64, i64), String> {
    match mode.trim().to_uppercase().as_str() {
        "TEN" => Ok((10, 0)),
        "TWENTY" => Ok((20, 0)),
        "ALL_OVERDUE" => Ok((10_000, 1)),
        _ => Err("Modo da fila de revisão inválido".into()),
    }
}

fn queue_category(card: &StudyCard, now: DateTime<Utc>) -> String {
    if card.review_count == 0 {
        return "NEW".into();
    }
    let due = DateTime::parse_from_rfc3339(&card.due_at).map(|value| value.with_timezone(&Local));
    match due {
        Ok(value) if value.date_naive() < now.with_timezone(&Local).date_naive() => {
            "OVERDUE".into()
        }
        _ => "TODAY".into(),
    }
}

fn resolve_context(connection: &Connection, input: &StudyCardInput) -> Result<CardContext, String> {
    for value in [
        &input.roadmap_id,
        &input.stage_id,
        &input.topic_id,
        &input.activity_id,
        &input.notebook_id,
        &input.note_id,
    ] {
        validate_optional_id(value.as_deref())?;
    }
    let mut context = CardContext {
        roadmap_id: input.roadmap_id.clone(),
        stage_id: input.stage_id.clone(),
        topic_id: input.topic_id.clone(),
        activity_id: input.activity_id.clone(),
        notebook_id: input.notebook_id.clone(),
        note_id: input.note_id.clone(),
    };
    if let Some(note_id) = input.note_id.as_deref() {
        let actual = connection.query_row(
            "SELECT roadmap_id,stage_id,topic_id,activity_id,notebook_id,id FROM study_notes WHERE id=?1",
            [note_id],
            |row| Ok(CardContext { roadmap_id:row.get(0)?,stage_id:row.get(1)?,topic_id:row.get(2)?,activity_id:row.get(3)?,notebook_id:row.get(4)?,note_id:Some(row.get(5)?) }),
        ).optional().map_err(err)?.ok_or_else(|| "Nota vinculada não encontrada".to_string())?;
        merge_context(&mut context, actual, "nota")?;
    }
    if let Some(activity_id) = context.activity_id.clone() {
        let actual = connection
            .query_row(
                "SELECT roadmap.id,stage.id,topic.id,activity.id FROM roadmap_activities activity
             JOIN roadmap_topics topic ON topic.id=activity.topic_id
             JOIN roadmap_stages stage ON stage.id=topic.stage_id
             JOIN study_roadmaps roadmap ON roadmap.id=stage.roadmap_id WHERE activity.id=?1",
                [activity_id],
                |row| {
                    Ok(CardContext {
                        roadmap_id: Some(row.get(0)?),
                        stage_id: Some(row.get(1)?),
                        topic_id: Some(row.get(2)?),
                        activity_id: Some(row.get(3)?),
                        ..Default::default()
                    })
                },
            )
            .optional()
            .map_err(err)?
            .ok_or_else(|| "Atividade vinculada não encontrada".to_string())?;
        merge_context(&mut context, actual, "atividade")?;
    } else if let Some(topic_id) = context.topic_id.clone() {
        let actual = connection
            .query_row(
                "SELECT roadmap.id,stage.id,topic.id FROM roadmap_topics topic
             JOIN roadmap_stages stage ON stage.id=topic.stage_id
             JOIN study_roadmaps roadmap ON roadmap.id=stage.roadmap_id WHERE topic.id=?1",
                [topic_id],
                |row| {
                    Ok(CardContext {
                        roadmap_id: Some(row.get(0)?),
                        stage_id: Some(row.get(1)?),
                        topic_id: Some(row.get(2)?),
                        ..Default::default()
                    })
                },
            )
            .optional()
            .map_err(err)?
            .ok_or_else(|| "Tópico vinculado não encontrado".to_string())?;
        merge_context(&mut context, actual, "tópico")?;
    } else if let Some(stage_id) = context.stage_id.clone() {
        let actual = connection.query_row(
            "SELECT roadmap.id,stage.id FROM roadmap_stages stage JOIN study_roadmaps roadmap ON roadmap.id=stage.roadmap_id WHERE stage.id=?1",
            [stage_id],
            |row| Ok(CardContext { roadmap_id:Some(row.get(0)?),stage_id:Some(row.get(1)?),..Default::default() }),
        ).optional().map_err(err)?.ok_or_else(|| "Etapa vinculada não encontrada".to_string())?;
        merge_context(&mut context, actual, "etapa")?;
    } else if let Some(roadmap_id) = context.roadmap_id.as_deref() {
        let exists: bool = connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM study_roadmaps WHERE id=?1)",
                [roadmap_id],
                |row| row.get(0),
            )
            .map_err(err)?;
        if !exists {
            return Err("Roadmap vinculado não encontrado".into());
        }
    }
    if let Some(notebook_id) = context.notebook_id.as_deref() {
        let exists: bool = connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM study_notebooks WHERE id=?1)",
                [notebook_id],
                |row| row.get(0),
            )
            .map_err(err)?;
        if !exists {
            return Err("Caderno vinculado não encontrado".into());
        }
    }
    Ok(context)
}

fn merge_context(
    current: &mut CardContext,
    actual: CardContext,
    source: &str,
) -> Result<(), String> {
    merge_id(
        &mut current.roadmap_id,
        actual.roadmap_id,
        "roadmap",
        source,
    )?;
    merge_id(&mut current.stage_id, actual.stage_id, "etapa", source)?;
    merge_id(&mut current.topic_id, actual.topic_id, "tópico", source)?;
    merge_id(
        &mut current.activity_id,
        actual.activity_id,
        "atividade",
        source,
    )?;
    merge_id(
        &mut current.notebook_id,
        actual.notebook_id,
        "caderno",
        source,
    )?;
    merge_id(&mut current.note_id, actual.note_id, "nota", source)
}

fn merge_id(
    current: &mut Option<String>,
    actual: Option<String>,
    label: &str,
    source: &str,
) -> Result<(), String> {
    match (&current, &actual) {
        (Some(expected), Some(found)) if expected != found => {
            Err(format!("O {label} não pertence à {source} informada"))
        }
        (None, Some(found)) => {
            *current = Some(found.clone());
            Ok(())
        }
        _ => Ok(()),
    }
}

fn map_card(row: &Row<'_>) -> rusqlite::Result<StudyCard> {
    Ok(StudyCard {
        id: row.get(0)?,
        front: row.get(1)?,
        back: row.get(2)?,
        status: row.get(3)?,
        roadmap_id: row.get(4)?,
        roadmap_name: row.get(5)?,
        stage_id: row.get(6)?,
        stage_name: row.get(7)?,
        topic_id: row.get(8)?,
        topic_name: row.get(9)?,
        activity_id: row.get(10)?,
        activity_title: row.get(11)?,
        notebook_id: row.get(12)?,
        notebook_title: row.get(13)?,
        note_id: row.get(14)?,
        note_title: row.get(15)?,
        due_at: row.get(16)?,
        last_reviewed_at: row.get(17)?,
        review_count: row.get(18)?,
        correct_count: row.get(19)?,
        incorrect_count: row.get(20)?,
        current_interval_seconds: row.get(21)?,
        scheduler_version: row.get(22)?,
        created_at: row.get(23)?,
        updated_at: row.get(24)?,
    })
}

fn validate_status(value: &str) -> Result<(), String> {
    if ["ACTIVE", "SUSPENDED", "ARCHIVED"].contains(&value) {
        Ok(())
    } else {
        Err("Status do card inválido".into())
    }
}

fn validate_id(value: &str) -> Result<(), String> {
    let value = value.trim();
    if value.is_empty() || value.len() > 120 {
        Err("Identificador inválido".into())
    } else {
        Ok(())
    }
}

fn validate_optional_id(value: Option<&str>) -> Result<(), String> {
    value.map(validate_id).unwrap_or(Ok(()))
}

fn escape_like(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

fn err(error: rusqlite::Error) -> String {
    error.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone};

    fn database() -> Connection {
        let mut connection = Connection::open_in_memory().unwrap();
        super::super::initialize(&mut connection).unwrap();
        connection
    }

    fn card(id: &str) -> StudyCardInput {
        StudyCardInput {
            id: id.into(),
            front: format!("Pergunta {id}"),
            back: format!("Resposta {id}"),
            roadmap_id: None,
            stage_id: None,
            topic_id: None,
            activity_id: None,
            notebook_id: None,
            note_id: None,
            source_material_ids: vec![],
        }
    }

    fn clock() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 9, 25, 12, 0, 0).unwrap()
    }

    #[test]
    fn card_create_edit_status_search_and_safe_delete() {
        let connection = database();
        let mut input = card("card-1");
        let saved = save_card_at(&connection, &input, clock()).unwrap();
        assert_eq!(saved.review_count, 0);
        input.front = "Pergunta editada".into();
        assert_eq!(
            save_card_at(&connection, &input, clock()).unwrap().front,
            "Pergunta editada"
        );
        assert_eq!(
            set_card_status(&connection, "card-1", "SUSPENDED")
                .unwrap()
                .status,
            "SUSPENDED"
        );
        assert_eq!(
            list_cards(
                &connection,
                &StudyCardListInput {
                    query: Some("editada".into()),
                    status: None,
                    filter: None,
                    roadmap_id: None,
                    activity_id: None,
                    note_id: None,
                    limit: 50,
                    offset: 0
                }
            )
            .unwrap()
            .len(),
            1
        );
        delete_card(&connection, "card-1").unwrap();
        assert!(get_card(&connection, "card-1").unwrap().is_none());
    }

    #[test]
    fn generated_card_batch_is_atomic_idempotent_and_keeps_scheduler_new() {
        let mut connection = database();
        let inputs = vec![card("ai-card-1"), card("ai-card-2")];
        let saved = save_cards(&mut connection, &inputs).unwrap();
        assert_eq!(saved.len(), 2);
        assert!(saved.iter().all(|item| item.review_count == 0));
        assert_eq!(save_cards(&mut connection, &inputs).unwrap().len(), 2);
        let count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM study_cards WHERE id LIKE 'ai-card-%'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 2);

        let mut invalid = card("ai-card-3");
        invalid.front.clear();
        assert!(save_cards(&mut connection, &[card("ai-card-4"), invalid]).is_err());
        let rolled_back: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM study_cards WHERE id IN ('ai-card-3','ai-card-4')",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(rolled_back, 0);
    }

    #[test]
    fn queue_prioritizes_overdue_then_today_then_new() {
        let connection = database();
        for id in ["new", "today", "overdue"] {
            save_card_at(&connection, &card(id), clock()).unwrap();
        }
        connection
            .execute(
                "UPDATE study_review_states SET review_count=1,due_at=?1 WHERE card_id='today'",
                [(clock() + Duration::hours(2)).to_rfc3339()],
            )
            .unwrap();
        connection
            .execute(
                "UPDATE study_review_states SET review_count=1,due_at=?1 WHERE card_id='overdue'",
                [(clock() - Duration::days(2)).to_rfc3339()],
            )
            .unwrap();
        let queue = get_review_queue_at(&connection, "TEN", clock()).unwrap();
        assert_eq!(
            queue
                .iter()
                .map(|item| item.card.id.as_str())
                .collect::<Vec<_>>(),
            vec!["overdue", "today", "new"]
        );
        assert_eq!(
            queue
                .iter()
                .map(|item| item.category.as_str())
                .collect::<Vec<_>>(),
            vec!["OVERDUE", "TODAY", "NEW"]
        );
    }

    #[test]
    fn submit_is_atomic_and_duplicate_item_is_rejected() {
        let mut connection = database();
        save_card_at(&connection, &card("card-1"), clock()).unwrap();
        let session = start_review_session_at(
            &mut connection,
            &StartStudyReviewSessionInput {
                id: "review-1".into(),
                mode: "TEN".into(),
                study_session_id: None,
            },
            clock(),
        )
        .unwrap();
        let item = session.current_item.unwrap().item_id.unwrap();
        let input = SubmitStudyReviewResultInput {
            id: "event-1".into(),
            review_session_id: "review-1".into(),
            session_item_id: item,
            result: "GOOD".into(),
            response_time_ms: Some(900),
        };
        let result = submit_review_result_at(&mut connection, &input, clock()).unwrap();
        assert_eq!(result.session.status, "COMPLETED");
        assert_eq!(
            get_card(&connection, "card-1")
                .unwrap()
                .unwrap()
                .correct_count,
            1
        );
        assert!(submit_review_result_at(&mut connection, &input, clock()).is_err());
        assert_eq!(
            connection
                .query_row("SELECT COUNT(*) FROM study_review_events", [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            1
        );
    }

    #[test]
    fn submit_rolls_back_event_when_state_update_fails() {
        let mut connection = database();
        save_card_at(&connection, &card("card-1"), clock()).unwrap();
        let session = start_review_session_at(
            &mut connection,
            &StartStudyReviewSessionInput {
                id: "review-1".into(),
                mode: "TEN".into(),
                study_session_id: None,
            },
            clock(),
        )
        .unwrap();
        let item = session.current_item.unwrap().item_id.unwrap();
        connection.execute_batch("CREATE TRIGGER fail_review_state BEFORE UPDATE ON study_review_states BEGIN SELECT RAISE(ABORT,'forced failure'); END;").unwrap();
        let input = SubmitStudyReviewResultInput {
            id: "event-1".into(),
            review_session_id: "review-1".into(),
            session_item_id: item,
            result: "GOOD".into(),
            response_time_ms: None,
        };
        assert!(submit_review_result_at(&mut connection, &input, clock()).is_err());
        assert_eq!(
            connection
                .query_row("SELECT COUNT(*) FROM study_review_events", [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            0
        );
        assert_eq!(
            connection
                .query_row(
                    "SELECT reviewed_cards FROM study_review_sessions WHERE id='review-1'",
                    [],
                    |row| row.get::<_, i64>(0)
                )
                .unwrap(),
            0
        );
    }

    #[test]
    fn note_context_review_history_and_card_edit_remain_consistent() {
        let mut connection = database();
        connection.execute("INSERT INTO study_roadmaps(id,name,status) VALUES ('review-roadmap','Roadmap Review','active')", []).unwrap();
        connection.execute("INSERT INTO roadmap_stages(id,roadmap_id,name,stage_order) VALUES ('review-stage','review-roadmap','Etapa',1)", []).unwrap();
        connection.execute("INSERT INTO roadmap_topics(id,stage_id,name,topic_order) VALUES ('review-topic','review-stage','Tópico',1)", []).unwrap();
        connection.execute("INSERT INTO roadmap_activities(id,topic_id,title,activity_type,status,activity_order) VALUES ('review-activity','review-topic','Atividade','LESSON','pending',1)", []).unwrap();
        connection.execute("INSERT INTO study_notebooks(id,title,status,created_at,updated_at) VALUES ('review-notebook','Caderno','ACTIVE',?1,?1)", [clock().to_rfc3339()]).unwrap();
        connection.execute("INSERT INTO study_notes(id,notebook_id,title,content,roadmap_id,stage_id,topic_id,activity_id,created_at,updated_at) VALUES ('review-note','review-notebook','Nota','Conteúdo','review-roadmap','review-stage','review-topic','review-activity',?1,?1)", [clock().to_rfc3339()]).unwrap();
        let mut input = card("context-card");
        input.note_id = Some("review-note".into());
        let saved = save_card_at(&connection, &input, clock()).unwrap();
        assert_eq!(saved.roadmap_id.as_deref(), Some("review-roadmap"));
        assert_eq!(saved.notebook_id.as_deref(), Some("review-notebook"));

        let session = start_review_session_at(
            &mut connection,
            &StartStudyReviewSessionInput {
                id: "context-review".into(),
                mode: "TEN".into(),
                study_session_id: None,
            },
            clock(),
        )
        .unwrap();
        submit_review_result_at(
            &mut connection,
            &SubmitStudyReviewResultInput {
                id: "context-event".into(),
                review_session_id: session.id,
                session_item_id: session.current_item.unwrap().item_id.unwrap(),
                result: "GOOD".into(),
                response_time_ms: Some(1_200),
            },
            clock(),
        )
        .unwrap();
        input.front = "Pergunta preservando histórico".into();
        let edited = save_card_at(&connection, &input, clock() + Duration::minutes(1)).unwrap();
        assert_eq!(edited.review_count, 1);
        assert_eq!(
            get_review_summary(&connection, "context-review")
                .unwrap()
                .good,
            1
        );
        assert!(delete_card(&connection, "context-card").is_err());
        assert_eq!(
            set_card_status(&connection, "context-card", "ARCHIVED")
                .unwrap()
                .status,
            "ARCHIVED"
        );
        assert!(
            get_review_queue_at(&connection, "TEN", clock() + Duration::days(10))
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            set_card_status(&connection, "context-card", "ACTIVE")
                .unwrap()
                .status,
            "ACTIVE"
        );
    }
}
