use super::study_models::*;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension, Row};
use std::collections::HashMap;

#[derive(Default)]
struct StudyContext {
    roadmap_id: Option<String>,
    stage_id: Option<String>,
    topic_id: Option<String>,
    activity_id: Option<String>,
    roadmap_name: String,
    stage_name: String,
    topic_name: String,
    activity_title: String,
}

pub fn start_session(
    connection: &mut Connection,
    input: &StartStudySessionInput,
) -> Result<StudySession, String> {
    start_session_at(connection, input, Utc::now())
}

pub fn pause_session(connection: &Connection, id: &str) -> Result<StudySession, String> {
    pause_session_at(connection, id, Utc::now())
}

pub fn resume_session(connection: &Connection, id: &str) -> Result<StudySession, String> {
    resume_session_at(connection, id, Utc::now())
}

pub fn complete_session(connection: &Connection, id: &str) -> Result<StudySession, String> {
    finish_session_at(connection, id, "COMPLETED", Utc::now())
}

pub fn cancel_session(connection: &Connection, id: &str) -> Result<StudySession, String> {
    finish_session_at(connection, id, "CANCELLED", Utc::now())
}

pub fn get_active_session(connection: &Connection) -> Result<Option<StudySession>, String> {
    get_active_session_at(connection, Utc::now())
}

pub fn list_sessions(
    connection: &Connection,
    input: &StudySessionListInput,
) -> Result<Vec<StudySession>, String> {
    if let Some(value) = input.date_from.as_deref() {
        validate_date(value)?;
    }
    if let Some(value) = input.date_to.as_deref() {
        validate_date(value)?;
    }
    if let (Some(from), Some(to)) = (&input.date_from, &input.date_to) {
        if from > to {
            return Err("Período do histórico é inválido".into());
        }
    }
    let limit = input.limit.clamp(1, 100);
    let offset = input.offset.max(0);
    let mut statement = connection.prepare(
        "SELECT id,roadmap_id,stage_id,topic_id,activity_id,roadmap_name,stage_name,topic_name,activity_title,planned_focus_minutes,actual_focus_seconds,break_seconds,status,started_at,running_since,paused_at,ended_at,created_at,updated_at,(SELECT COUNT(*) FROM study_notes linked_note WHERE linked_note.study_session_id=study_sessions.id)
         FROM study_sessions
         WHERE (?1 IS NULL OR roadmap_id=?1)
           AND (?2 IS NULL OR date(started_at,'localtime')>=?2)
           AND (?3 IS NULL OR date(started_at,'localtime')<=?3)
         ORDER BY started_at DESC,id DESC LIMIT ?4 OFFSET ?5",
    ).map_err(err)?;
    let now = Utc::now();
    let rows = statement
        .query_map(
            params![
                input.roadmap_id,
                input.date_from,
                input.date_to,
                limit,
                offset
            ],
            map_session,
        )
        .map_err(err)?;
    rows.collect::<Result<Vec<_>, _>>()
        .map(|items| {
            items
                .into_iter()
                .map(|item| effective_session(item, now))
                .collect()
        })
        .map_err(err)
}

pub fn today_summary(connection: &Connection) -> Result<StudyTodaySummary, String> {
    today_summary_at(connection, Utc::now())
}

pub fn get_settings(connection: &Connection) -> Result<StudySettings, String> {
    connection
        .query_row(
            "SELECT focus_minutes,short_break_minutes,updated_at FROM study_settings WHERE id=1",
            [],
            |row| {
                Ok(StudySettings {
                    focus_minutes: row.get(0)?,
                    short_break_minutes: row.get(1)?,
                    updated_at: row.get(2)?,
                })
            },
        )
        .map_err(err)
}

pub fn update_settings(
    connection: &Connection,
    input: &StudySettingsInput,
) -> Result<StudySettings, String> {
    if !(1..=240).contains(&input.focus_minutes) {
        return Err("A duração de foco deve ficar entre 1 e 240 minutos".into());
    }
    if !(1..=60).contains(&input.short_break_minutes) {
        return Err("A pausa curta deve ficar entre 1 e 60 minutos".into());
    }
    connection.execute(
        "UPDATE study_settings SET focus_minutes=?1,short_break_minutes=?2,updated_at=CURRENT_TIMESTAMP WHERE id=1",
        params![input.focus_minutes, input.short_break_minutes],
    ).map_err(err)?;
    get_settings(connection)
}

fn start_session_at(
    connection: &mut Connection,
    input: &StartStudySessionInput,
    now: DateTime<Utc>,
) -> Result<StudySession, String> {
    validate_id(&input.id)?;
    if !(1..=240).contains(&input.planned_focus_minutes) {
        return Err("A duração planejada deve ficar entre 1 e 240 minutos".into());
    }
    let context = resolve_context(connection, input)?;
    let timestamp = now.to_rfc3339();
    let transaction = connection.transaction().map_err(err)?;
    let open: bool = transaction
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM study_sessions WHERE open_slot=1)",
            [],
            |row| row.get(0),
        )
        .map_err(err)?;
    if open {
        return Err("Já existe uma sessão de estudo ativa ou pausada".into());
    }
    transaction.execute(
        "INSERT INTO study_sessions(id,roadmap_id,stage_id,topic_id,activity_id,roadmap_name,stage_name,topic_name,activity_title,planned_focus_minutes,actual_focus_seconds,break_seconds,status,started_at,running_since,paused_at,ended_at,open_slot,created_at,updated_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,0,0,'ACTIVE',?11,?11,NULL,NULL,1,?11,?11)",
        params![input.id,context.roadmap_id,context.stage_id,context.topic_id,context.activity_id,context.roadmap_name,context.stage_name,context.topic_name,context.activity_title,input.planned_focus_minutes,timestamp],
    ).map_err(|error| if error.to_string().contains("idx_study_sessions_single_open") { "Já existe uma sessão de estudo ativa ou pausada".into() } else { err(error) })?;
    transaction.commit().map_err(err)?;
    load_session_at(connection, &input.id, now)?
        .ok_or_else(|| "Sessão não encontrada após iniciar".into())
}

fn pause_session_at(
    connection: &Connection,
    id: &str,
    now: DateTime<Utc>,
) -> Result<StudySession, String> {
    validate_id(id)?;
    let current = load_raw_session(connection, id)?
        .ok_or_else(|| "Sessão de estudo não encontrada".to_string())?;
    if current.status != "ACTIVE" {
        return Err("Somente uma sessão ativa pode ser pausada".into());
    }
    let elapsed = seconds_between(current.running_since.as_deref(), now)?;
    let timestamp = now.to_rfc3339();
    connection.execute(
        "UPDATE study_sessions SET status='PAUSED',actual_focus_seconds=actual_focus_seconds+?1,running_since=NULL,paused_at=?2,updated_at=?2 WHERE id=?3 AND status='ACTIVE'",
        params![elapsed,timestamp,id],
    ).map_err(err)?;
    load_session_at(connection, id, now)?.ok_or_else(|| "Sessão não encontrada após pausar".into())
}

fn resume_session_at(
    connection: &Connection,
    id: &str,
    now: DateTime<Utc>,
) -> Result<StudySession, String> {
    validate_id(id)?;
    let current = load_raw_session(connection, id)?
        .ok_or_else(|| "Sessão de estudo não encontrada".to_string())?;
    if current.status != "PAUSED" {
        return Err("Somente uma sessão pausada pode ser retomada".into());
    }
    let paused = seconds_between(current.paused_at.as_deref(), now)?;
    let timestamp = now.to_rfc3339();
    connection.execute(
        "UPDATE study_sessions SET status='ACTIVE',break_seconds=break_seconds+?1,running_since=?2,paused_at=NULL,updated_at=?2 WHERE id=?3 AND status='PAUSED'",
        params![paused,timestamp,id],
    ).map_err(err)?;
    load_session_at(connection, id, now)?.ok_or_else(|| "Sessão não encontrada após retomar".into())
}

fn finish_session_at(
    connection: &Connection,
    id: &str,
    status: &str,
    now: DateTime<Utc>,
) -> Result<StudySession, String> {
    validate_id(id)?;
    let current = load_raw_session(connection, id)?
        .ok_or_else(|| "Sessão de estudo não encontrada".to_string())?;
    if !["ACTIVE", "PAUSED"].contains(&current.status.as_str()) {
        return Err("A sessão já foi encerrada".into());
    }
    let focus_increment = if current.status == "ACTIVE" {
        seconds_between(current.running_since.as_deref(), now)?
    } else {
        0
    };
    let break_increment = if current.status == "PAUSED" {
        seconds_between(current.paused_at.as_deref(), now)?
    } else {
        0
    };
    let timestamp = now.to_rfc3339();
    connection.execute(
        "UPDATE study_sessions SET status=?1,actual_focus_seconds=actual_focus_seconds+?2,break_seconds=break_seconds+?3,running_since=NULL,paused_at=NULL,ended_at=?4,open_slot=NULL,updated_at=?4 WHERE id=?5 AND status IN ('ACTIVE','PAUSED')",
        params![status,focus_increment,break_increment,timestamp,id],
    ).map_err(err)?;
    load_session_at(connection, id, now)?
        .ok_or_else(|| "Sessão não encontrada após encerrar".into())
}

fn get_active_session_at(
    connection: &Connection,
    now: DateTime<Utc>,
) -> Result<Option<StudySession>, String> {
    connection.query_row(
        "SELECT id,roadmap_id,stage_id,topic_id,activity_id,roadmap_name,stage_name,topic_name,activity_title,planned_focus_minutes,actual_focus_seconds,break_seconds,status,started_at,running_since,paused_at,ended_at,created_at,updated_at,(SELECT COUNT(*) FROM study_notes linked_note WHERE linked_note.study_session_id=study_sessions.id) FROM study_sessions WHERE open_slot=1 LIMIT 1",
        [], map_session,
    ).optional().map(|item| item.map(|session| effective_session(session, now))).map_err(err)
}

fn today_summary_at(
    connection: &Connection,
    now: DateTime<Utc>,
) -> Result<StudyTodaySummary, String> {
    let mut statement = connection.prepare(
        "SELECT id,roadmap_id,stage_id,topic_id,activity_id,roadmap_name,stage_name,topic_name,activity_title,planned_focus_minutes,actual_focus_seconds,break_seconds,status,started_at,running_since,paused_at,ended_at,created_at,updated_at,(SELECT COUNT(*) FROM study_notes linked_note WHERE linked_note.study_session_id=study_sessions.id)
         FROM study_sessions WHERE date(started_at,'localtime')=date('now','localtime') ORDER BY started_at DESC,id DESC",
    ).map_err(err)?;
    let rows = statement.query_map([], map_session).map_err(err)?;
    let sessions = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(err)?
        .into_iter()
        .map(|session| effective_session(session, now))
        .collect::<Vec<_>>();
    let counted = sessions
        .iter()
        .filter(|session| session.status != "CANCELLED")
        .collect::<Vec<_>>();
    let focus_seconds = counted
        .iter()
        .map(|session| session.current_focus_seconds)
        .sum();
    let completed_sessions = sessions
        .iter()
        .filter(|session| session.status == "COMPLETED")
        .count() as i64;
    let completed_activities = connection.query_row(
        "SELECT COUNT(*) FROM roadmap_activities WHERE completed_at IS NOT NULL AND date(completed_at,'localtime')=date('now','localtime')",
        [], |row| row.get(0),
    ).map_err(err)?;
    let mut by_roadmap: HashMap<(Option<String>, String), i64> = HashMap::new();
    for session in counted {
        if session.roadmap_id.is_none() && session.roadmap_name.is_empty() {
            continue;
        }
        *by_roadmap
            .entry((session.roadmap_id.clone(), session.roadmap_name.clone()))
            .or_default() += session.current_focus_seconds;
    }
    let most_studied_roadmap = by_roadmap
        .into_iter()
        .max_by_key(|(_, seconds)| *seconds)
        .map(
            |((roadmap_id, roadmap_name), focus_seconds)| StudyRoadmapFocus {
                roadmap_id,
                roadmap_name,
                focus_seconds,
            },
        );
    let active_session = get_active_session_at(connection, now)?;
    let last_session = sessions
        .first()
        .cloned()
        .or_else(|| load_last_session(connection, now).ok().flatten());
    Ok(StudyTodaySummary {
        focus_seconds,
        completed_sessions,
        completed_activities,
        most_studied_roadmap,
        active_session,
        last_session,
    })
}

fn resolve_context(
    connection: &Connection,
    input: &StartStudySessionInput,
) -> Result<StudyContext, String> {
    if let Some(activity_id) = input.activity_id.as_deref() {
        validate_id(activity_id)?;
        let context = connection.query_row(
            "SELECT roadmap.id,roadmap.name,stage.id,stage.name,topic.id,topic.name,activity.id,activity.title
             FROM roadmap_activities activity
             JOIN roadmap_topics topic ON topic.id=activity.topic_id
             JOIN roadmap_stages stage ON stage.id=topic.stage_id
             JOIN study_roadmaps roadmap ON roadmap.id=stage.roadmap_id
             WHERE activity.id=?1",
            [activity_id],
            |row| Ok(StudyContext { roadmap_id: Some(row.get(0)?), roadmap_name: row.get(1)?, stage_id: Some(row.get(2)?), stage_name: row.get(3)?, topic_id: Some(row.get(4)?), topic_name: row.get(5)?, activity_id: Some(row.get(6)?), activity_title: row.get(7)? }),
        ).optional().map_err(err)?.ok_or_else(|| "Atividade de estudo não encontrada".to_string())?;
        if input
            .roadmap_id
            .as_deref()
            .is_some_and(|id| Some(id) != context.roadmap_id.as_deref())
        {
            return Err("A atividade não pertence ao roadmap informado".into());
        }
        return Ok(context);
    }
    if let Some(roadmap_id) = input.roadmap_id.as_deref() {
        validate_id(roadmap_id)?;
        return connection
            .query_row(
                "SELECT id,name FROM study_roadmaps WHERE id=?1",
                [roadmap_id],
                |row| {
                    Ok(StudyContext {
                        roadmap_id: Some(row.get(0)?),
                        roadmap_name: row.get(1)?,
                        ..StudyContext::default()
                    })
                },
            )
            .optional()
            .map_err(err)?
            .ok_or_else(|| "Roadmap de estudo não encontrado".into());
    }
    Ok(StudyContext::default())
}

fn load_raw_session(connection: &Connection, id: &str) -> Result<Option<StudySession>, String> {
    connection.query_row(
        "SELECT id,roadmap_id,stage_id,topic_id,activity_id,roadmap_name,stage_name,topic_name,activity_title,planned_focus_minutes,actual_focus_seconds,break_seconds,status,started_at,running_since,paused_at,ended_at,created_at,updated_at,(SELECT COUNT(*) FROM study_notes linked_note WHERE linked_note.study_session_id=study_sessions.id) FROM study_sessions WHERE id=?1",
        [id], map_session,
    ).optional().map_err(err)
}

fn load_session_at(
    connection: &Connection,
    id: &str,
    now: DateTime<Utc>,
) -> Result<Option<StudySession>, String> {
    Ok(load_raw_session(connection, id)?.map(|session| effective_session(session, now)))
}

fn load_last_session(
    connection: &Connection,
    now: DateTime<Utc>,
) -> Result<Option<StudySession>, String> {
    connection.query_row(
        "SELECT id,roadmap_id,stage_id,topic_id,activity_id,roadmap_name,stage_name,topic_name,activity_title,planned_focus_minutes,actual_focus_seconds,break_seconds,status,started_at,running_since,paused_at,ended_at,created_at,updated_at,(SELECT COUNT(*) FROM study_notes linked_note WHERE linked_note.study_session_id=study_sessions.id) FROM study_sessions ORDER BY started_at DESC,id DESC LIMIT 1",
        [], map_session,
    ).optional().map(|item| item.map(|session| effective_session(session, now))).map_err(err)
}

fn effective_session(mut session: StudySession, now: DateTime<Utc>) -> StudySession {
    session.current_focus_seconds = session.actual_focus_seconds;
    session.observed_at = now.to_rfc3339();
    if session.status == "ACTIVE" {
        if let Ok(elapsed) = seconds_between(session.running_since.as_deref(), now) {
            session.current_focus_seconds += elapsed;
        }
    }
    session
}

fn seconds_between(value: Option<&str>, now: DateTime<Utc>) -> Result<i64, String> {
    let value = value.ok_or_else(|| "Timestamp da sessão está ausente".to_string())?;
    let start = DateTime::parse_from_rfc3339(value)
        .map_err(|_| "Timestamp da sessão é inválido".to_string())?
        .with_timezone(&Utc);
    Ok((now - start).num_seconds().max(0))
}

fn map_session(row: &Row<'_>) -> rusqlite::Result<StudySession> {
    Ok(StudySession {
        id: row.get(0)?,
        roadmap_id: row.get(1)?,
        stage_id: row.get(2)?,
        topic_id: row.get(3)?,
        activity_id: row.get(4)?,
        roadmap_name: row.get(5)?,
        stage_name: row.get(6)?,
        topic_name: row.get(7)?,
        activity_title: row.get(8)?,
        planned_focus_minutes: row.get(9)?,
        actual_focus_seconds: row.get(10)?,
        current_focus_seconds: row.get(10)?,
        linked_note_count: row.get(19)?,
        observed_at: String::new(),
        break_seconds: row.get(11)?,
        status: row.get(12)?,
        started_at: row.get(13)?,
        running_since: row.get(14)?,
        paused_at: row.get(15)?,
        ended_at: row.get(16)?,
        created_at: row.get(17)?,
        updated_at: row.get(18)?,
    })
}

fn validate_id(value: &str) -> Result<(), String> {
    if !value.is_empty()
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || character == '-' || character == '_'
        })
    {
        Ok(())
    } else {
        Err("ID da sessão é inválido".into())
    }
}

fn validate_date(value: &str) -> Result<(), String> {
    chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map(|_| ())
        .map_err(|_| "Data inválida; use AAAA-MM-DD".into())
}

fn err(error: impl std::fmt::Display) -> String {
    error.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn database() -> Connection {
        let mut connection = Connection::open_in_memory().unwrap();
        crate::database::initialize(&mut connection).unwrap();
        connection.execute("INSERT INTO study_roadmaps(id,name,status) VALUES ('roadmap-study','Roadmap Study','active')", []).unwrap();
        connection.execute("INSERT INTO roadmap_stages(id,roadmap_id,name,stage_order) VALUES ('stage-study','roadmap-study','Fundamentos',1)", []).unwrap();
        connection.execute("INSERT INTO roadmap_topics(id,stage_id,name,topic_order) VALUES ('topic-study','stage-study','Conceitos',1)", []).unwrap();
        connection.execute("INSERT INTO roadmap_activities(id,topic_id,title,activity_type,status,activity_order) VALUES ('activity-study','topic-study','Atividade prática','EXERCISE','pending',1)", []).unwrap();
        connection
    }

    fn input(id: &str) -> StartStudySessionInput {
        StartStudySessionInput {
            id: id.into(),
            roadmap_id: Some("roadmap-study".into()),
            activity_id: Some("activity-study".into()),
            planned_focus_minutes: 25,
        }
    }

    #[test]
    fn start_pause_resume_and_complete_are_timestamp_driven() {
        let mut connection = database();
        let start = Utc::now();
        start_session_at(&mut connection, &input("session-one"), start).unwrap();
        let paused =
            pause_session_at(&connection, "session-one", start + Duration::seconds(600)).unwrap();
        assert_eq!(paused.actual_focus_seconds, 600);
        assert_eq!(paused.status, "PAUSED");
        let resumed =
            resume_session_at(&connection, "session-one", start + Duration::seconds(900)).unwrap();
        assert_eq!(resumed.break_seconds, 300);
        let completed = finish_session_at(
            &connection,
            "session-one",
            "COMPLETED",
            start + Duration::seconds(1500),
        )
        .unwrap();
        assert_eq!(completed.actual_focus_seconds, 1200);
        assert_eq!(completed.break_seconds, 300);
        assert_eq!(completed.status, "COMPLETED");
    }

    #[test]
    fn only_one_open_session_is_allowed_and_recovery_keeps_elapsed_time() {
        let mut connection = database();
        let start = Utc::now();
        start_session_at(&mut connection, &input("session-one"), start).unwrap();
        assert!(
            start_session_at(&mut connection, &input("session-two"), start)
                .unwrap_err()
                .contains("Já existe")
        );
        let recovered = get_active_session_at(&connection, start + Duration::seconds(90))
            .unwrap()
            .unwrap();
        assert_eq!(recovered.actual_focus_seconds, 0);
        assert_eq!(recovered.current_focus_seconds, 90);
    }

    #[test]
    fn invalid_context_is_rejected_and_cancelled_time_is_not_counted_today() {
        let mut connection = database();
        let mut invalid = input("bad-context");
        invalid.activity_id = Some("missing".into());
        assert!(start_session_at(&mut connection, &invalid, Utc::now())
            .unwrap_err()
            .contains("não encontrada"));
        let start = Utc::now();
        start_session_at(&mut connection, &input("cancelled-session"), start).unwrap();
        finish_session_at(
            &connection,
            "cancelled-session",
            "CANCELLED",
            start + Duration::seconds(120),
        )
        .unwrap();
        let summary = today_summary_at(&connection, start + Duration::seconds(120)).unwrap();
        assert_eq!(summary.focus_seconds, 0);
        assert_eq!(summary.completed_sessions, 0);
    }
}
