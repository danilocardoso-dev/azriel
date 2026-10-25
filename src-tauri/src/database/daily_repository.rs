use super::daily_models::*;
use rusqlite::{params, Connection, OptionalExtension};

const TASK_COLUMNS: &str = "id,title,description,status,priority,due_date,created_at,updated_at,completed_at";
const NOTE_COLUMNS: &str = "id,title,content,status,created_at,updated_at";
fn err(error: rusqlite::Error) -> String { error.to_string() }

pub fn list_tasks(connection: &Connection) -> Result<Vec<Task>, String> {
    query_tasks(connection, &format!("SELECT {TASK_COLUMNS} FROM tasks ORDER BY CASE priority WHEN 'critical' THEN 0 WHEN 'high' THEN 1 WHEN 'medium' THEN 2 ELSE 3 END, COALESCE(due_date,'9999-12-31'),created_at DESC"), [])
}
pub fn get_task(connection: &Connection, id: &str) -> Result<Option<Task>, String> {
    connection.query_row(&format!("SELECT {TASK_COLUMNS} FROM tasks WHERE id=?1"), [id], map_task).optional().map_err(err)
}
pub fn list_today_tasks(c: &Connection, today: &str) -> Result<Vec<Task>, String> {
    query_tasks(c, &format!("SELECT {TASK_COLUMNS} FROM tasks WHERE status NOT IN ('completed','cancelled') AND due_date=?1 ORDER BY title"), [today])
}
pub fn list_upcoming_tasks(c: &Connection, today: &str) -> Result<Vec<Task>, String> {
    query_tasks(c, &format!("SELECT {TASK_COLUMNS} FROM tasks WHERE status NOT IN ('completed','cancelled') AND due_date>?1 ORDER BY due_date,title"), [today])
}
pub fn list_inbox_tasks(c: &Connection) -> Result<Vec<Task>, String> { query_tasks(c, &format!("SELECT {TASK_COLUMNS} FROM tasks WHERE status='inbox' ORDER BY created_at DESC"), []) }
pub fn list_completed_tasks(c: &Connection) -> Result<Vec<Task>, String> { query_tasks(c, &format!("SELECT {TASK_COLUMNS} FROM tasks WHERE status='completed' ORDER BY completed_at DESC"), []) }
pub fn save_task(c: &Connection, input: &TaskInput) -> Result<Task, String> {
    validate_task(input)?;
    c.execute("INSERT INTO tasks(id,title,description,status,priority,due_date,completed_at) VALUES (?1,?2,?3,?4,?5,?6,CASE WHEN ?4='completed' THEN CURRENT_TIMESTAMP END) ON CONFLICT(id) DO UPDATE SET title=excluded.title,description=excluded.description,status=excluded.status,priority=excluded.priority,due_date=excluded.due_date,updated_at=CURRENT_TIMESTAMP,completed_at=CASE WHEN excluded.status='completed' THEN COALESCE(tasks.completed_at,CURRENT_TIMESTAMP) ELSE NULL END",params![input.id,input.title.trim(),input.description.trim(),input.status,input.priority,input.due_date]).map_err(err)?;
    get_task(c, &input.id)?.ok_or_else(|| "Tarefa não encontrada após salvar".into())
}
pub fn complete_task(c: &Connection, id: &str) -> Result<Task, String> {
    if c.execute("UPDATE tasks SET status='completed',completed_at=COALESCE(completed_at,CURRENT_TIMESTAMP),updated_at=CURRENT_TIMESTAMP WHERE id=?1",[id]).map_err(err)? == 0 { return Err("Tarefa não encontrada".into()); }
    get_task(c,id)?.ok_or_else(|| "Tarefa não encontrada".into())
}
pub fn delete_task(c: &Connection, id: &str) -> Result<(), String> { c.execute("DELETE FROM tasks WHERE id=?1",[id]).map_err(err)?; Ok(()) }
pub fn list_notes(c: &Connection, archived: bool) -> Result<Vec<Note>, String> {
    let sql=if archived { format!("SELECT {NOTE_COLUMNS} FROM notes ORDER BY updated_at DESC") } else { format!("SELECT {NOTE_COLUMNS} FROM notes WHERE status='active' ORDER BY updated_at DESC") };
    let mut s=c.prepare(&sql).map_err(err)?;
    let result = s.query_map([],map_note).map_err(err)?.collect::<Result<Vec<_>,_>>().map_err(err);
    result
}
pub fn get_note(c: &Connection,id:&str)->Result<Option<Note>,String>{c.query_row(&format!("SELECT {NOTE_COLUMNS} FROM notes WHERE id=?1"),[id],map_note).optional().map_err(err)}
pub fn save_note(c:&Connection,input:&NoteInput)->Result<Note,String>{
 validate_note(input)?; let title=input.title.as_deref().map(str::trim).filter(|v|!v.is_empty());
 c.execute("INSERT INTO notes(id,title,content,status) VALUES(?1,?2,?3,?4) ON CONFLICT(id) DO UPDATE SET title=excluded.title,content=excluded.content,status=excluded.status,updated_at=CURRENT_TIMESTAMP",params![input.id,title,input.content.trim(),input.status]).map_err(err)?;
 get_note(c,&input.id)?.ok_or_else(|| "Nota não encontrada após salvar".into())
}
pub fn archive_note(c:&Connection,id:&str)->Result<Note,String>{c.execute("UPDATE notes SET status='archived',updated_at=CURRENT_TIMESTAMP WHERE id=?1",[id]).map_err(err)?;get_note(c,id)?.ok_or_else(||"Nota não encontrada".into())}
pub fn delete_note(c:&Connection,id:&str)->Result<(),String>{c.execute("DELETE FROM notes WHERE id=?1",[id]).map_err(err)?;Ok(())}
pub fn counters(c:&Connection,today:&str)->Result<DailyCounters,String>{c.query_row("SELECT (SELECT COUNT(*) FROM tasks WHERE status NOT IN ('completed','cancelled')),(SELECT COUNT(*) FROM tasks WHERE status NOT IN ('completed','cancelled') AND due_date=?1),(SELECT COUNT(*) FROM tasks WHERE status NOT IN ('completed','cancelled') AND due_date<?1),(SELECT COUNT(*) FROM tasks WHERE status NOT IN ('completed','cancelled') AND priority IN ('high','critical')),(SELECT COUNT(*) FROM notes WHERE status='active'),(SELECT COUNT(*) FROM tasks WHERE status='completed')",[today],|r|Ok(DailyCounters{pending:r.get(0)?,today:r.get(1)?,overdue:r.get(2)?,priority:r.get(3)?,notes:r.get(4)?,completed:r.get(5)?})).map_err(err)}
fn query_tasks<P:rusqlite::Params>(c:&Connection,sql:&str,p:P)->Result<Vec<Task>,String>{
 let mut s=c.prepare(sql).map_err(err)?;
 let result=s.query_map(p,map_task).map_err(err)?.collect::<Result<Vec<_>,_>>().map_err(err);
 result
}
fn map_task(r:&rusqlite::Row<'_>)->rusqlite::Result<Task>{Ok(Task{id:r.get(0)?,title:r.get(1)?,description:r.get(2)?,status:r.get(3)?,priority:r.get(4)?,due_date:r.get(5)?,created_at:r.get(6)?,updated_at:r.get(7)?,completed_at:r.get(8)?})}
fn map_note(r:&rusqlite::Row<'_>)->rusqlite::Result<Note>{Ok(Note{id:r.get(0)?,title:r.get(1)?,content:r.get(2)?,status:r.get(3)?,created_at:r.get(4)?,updated_at:r.get(5)?})}
fn validate_task(i:&TaskInput)->Result<(),String>{if i.id.is_empty()||i.title.trim().is_empty(){return Err("ID e título são obrigatórios".into())}if !["inbox","pending","in_progress","completed","cancelled"].contains(&i.status.as_str())||!["low","medium","high","critical"].contains(&i.priority.as_str()){return Err("Status ou prioridade inválidos".into())}Ok(())}
fn validate_note(i:&NoteInput)->Result<(),String>{if i.id.is_empty()||i.content.trim().is_empty()||!["active","archived"].contains(&i.status.as_str()){return Err("Nota inválida".into())}Ok(())}
