//! Repository for auto_tasks and task_executions

use rusqlite::{params, Connection};
use crate::errors::Result;
use crate::models::{AutoTask, CreateAutoTask, UpdateAutoTask, TaskExecution};

pub struct TaskRepo;

impl TaskRepo {
    // ── AutoTask CRUD ───────────────────────────────────────────────────────

    pub fn create(conn: &Connection, input: &CreateAutoTask) -> Result<AutoTask> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let schedule_cron = input.schedule_cron.clone().unwrap_or_default();

        conn.execute(
            "INSERT INTO auto_tasks (id, name, task_type, preset_id, definition, schedule, schedule_cron, is_enabled, config, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1, ?8, ?9, ?9)",
            params![id, input.name, input.task_type, input.preset_id, input.definition, input.schedule, schedule_cron, input.config, now],
        )?;

        Self::get_by_id(conn, &id)
    }

    pub fn get_by_id(conn: &Connection, id: &str) -> Result<AutoTask> {
        let mut stmt = conn.prepare(
            "SELECT id, name, task_type, preset_id, definition, schedule, schedule_cron,
                    is_enabled, last_run, next_run, config, created_at, updated_at
             FROM auto_tasks WHERE id = ?1"
        )?;
        Ok(stmt.query_row(params![id], |row| {
            Ok(AutoTask {
                id: row.get(0)?,
                name: row.get(1)?,
                task_type: row.get(2)?,
                preset_id: row.get(3)?,
                definition: row.get(4)?,
                schedule: row.get(5)?,
                schedule_cron: row.get(6)?,
                is_enabled: row.get::<_, i32>(7)? != 0,
                last_run: row.get(8)?,
                next_run: row.get(9)?,
                config: row.get(10)?,
                created_at: row.get(11)?,
                updated_at: row.get(12)?,
            })
        })?)
    }

    pub fn list_all(conn: &Connection) -> Result<Vec<AutoTask>> {
        let mut stmt = conn.prepare(
            "SELECT id, name, task_type, preset_id, definition, schedule, schedule_cron,
                    is_enabled, last_run, next_run, config, created_at, updated_at
             FROM auto_tasks ORDER BY created_at"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(AutoTask {
                id: row.get(0)?,
                name: row.get(1)?,
                task_type: row.get(2)?,
                preset_id: row.get(3)?,
                definition: row.get(4)?,
                schedule: row.get(5)?,
                schedule_cron: row.get(6)?,
                is_enabled: row.get::<_, i32>(7)? != 0,
                last_run: row.get(8)?,
                next_run: row.get(9)?,
                config: row.get(10)?,
                created_at: row.get(11)?,
                updated_at: row.get(12)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn list_enabled(conn: &Connection) -> Result<Vec<AutoTask>> {
        let mut stmt = conn.prepare(
            "SELECT id, name, task_type, preset_id, definition, schedule, schedule_cron,
                    is_enabled, last_run, next_run, config, created_at, updated_at
             FROM auto_tasks WHERE is_enabled = 1 ORDER BY next_run"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(AutoTask {
                id: row.get(0)?,
                name: row.get(1)?,
                task_type: row.get(2)?,
                preset_id: row.get(3)?,
                definition: row.get(4)?,
                schedule: row.get(5)?,
                schedule_cron: row.get(6)?,
                is_enabled: row.get::<_, i32>(7)? != 0,
                last_run: row.get(8)?,
                next_run: row.get(9)?,
                config: row.get(10)?,
                created_at: row.get(11)?,
                updated_at: row.get(12)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn update(conn: &Connection, id: &str, input: &UpdateAutoTask) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        let mut sql_parts = Vec::new();
        let mut params_vec: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(ref v) = input.name {
            sql_parts.push("name = ?");
            params_vec.push(Box::new(v.clone()));
        }
        if let Some(ref v) = input.definition {
            sql_parts.push("definition = ?");
            params_vec.push(Box::new(v.clone()));
        }
        if let Some(ref v) = input.schedule {
            sql_parts.push("schedule = ?");
            params_vec.push(Box::new(v.clone()));
        }
        if let Some(ref v) = input.schedule_cron {
            sql_parts.push("schedule_cron = ?");
            params_vec.push(Box::new(v.clone()));
        }
        if let Some(v) = input.is_enabled {
            sql_parts.push("is_enabled = ?");
            params_vec.push(Box::new(v as i32));
        }
        if let Some(ref v) = input.config {
            sql_parts.push("config = ?");
            params_vec.push(Box::new(v.clone()));
        }

        if sql_parts.is_empty() {
            return Ok(());
        }

        sql_parts.push("updated_at = ?");
        params_vec.push(Box::new(now));

        let sql = format!("UPDATE auto_tasks SET {} WHERE id = ?", sql_parts.join(", "));
        params_vec.push(Box::new(id.to_string()));

        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
        conn.execute(&sql, param_refs.as_slice())?;
        Ok(())
    }

    pub fn delete(conn: &Connection, id: &str) -> Result<()> {
        conn.execute("DELETE FROM auto_tasks WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn update_last_run(conn: &Connection, id: &str, last_run: &str, next_run: Option<&str>) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        if let Some(nr) = next_run {
            conn.execute(
                "UPDATE auto_tasks SET last_run = ?1, next_run = ?2, updated_at = ?3 WHERE id = ?4",
                params![last_run, nr, now, id],
            )?;
        } else {
            conn.execute(
                "UPDATE auto_tasks SET last_run = ?1, updated_at = ?2 WHERE id = ?3",
                params![last_run, now, id],
            )?;
        }
        Ok(())
    }

    pub fn enable(conn: &Connection, id: &str) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE auto_tasks SET is_enabled = 1, updated_at = ?1 WHERE id = ?2",
            params![now, id],
        )?;
        Ok(())
    }

    pub fn disable(conn: &Connection, id: &str) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE auto_tasks SET is_enabled = 0, updated_at = ?1 WHERE id = ?2",
            params![now, id],
        )?;
        Ok(())
    }

    // ── TaskExecution CRUD ──────────────────────────────────────────────────

    pub fn create_execution(conn: &Connection, task_id: &str) -> Result<TaskExecution> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO task_executions (id, task_id, status, started_at) VALUES (?1, ?2, 'running', ?3)",
            params![id, task_id, now],
        )?;
        Ok(TaskExecution {
            id,
            task_id: task_id.to_string(),
            status: "running".to_string(),
            started_at: now,
            finished_at: None,
            result: None,
            error: None,
            duration_ms: None,
        })
    }

    pub fn finish_execution(
        conn: &Connection,
        id: &str,
        status: &str,
        result: Option<&str>,
        error: Option<&str>,
        duration_ms: i64,
    ) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE task_executions SET status = ?1, finished_at = ?2, result = ?3, error = ?4, duration_ms = ?5 WHERE id = ?6",
            params![status, now, result, error, duration_ms, id],
        )?;
        Ok(())
    }

    pub fn list_executions(conn: &Connection, task_id: &str, limit: u32) -> Result<Vec<TaskExecution>> {
        let mut stmt = conn.prepare(
            "SELECT id, task_id, status, started_at, finished_at, result, error, duration_ms
             FROM task_executions WHERE task_id = ?1 ORDER BY started_at DESC LIMIT ?2"
        )?;
        let rows = stmt.query_map(params![task_id, limit], |row| {
            Ok(TaskExecution {
                id: row.get(0)?,
                task_id: row.get(1)?,
                status: row.get(2)?,
                started_at: row.get(3)?,
                finished_at: row.get(4)?,
                result: row.get(5)?,
                error: row.get(6)?,
                duration_ms: row.get(7)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn get_latest_execution(conn: &Connection, task_id: &str) -> Result<Option<TaskExecution>> {
        let mut stmt = conn.prepare(
            "SELECT id, task_id, status, started_at, finished_at, result, error, duration_ms
             FROM task_executions WHERE task_id = ?1 ORDER BY started_at DESC LIMIT 1"
        )?;
        let result = stmt.query_row(params![task_id], |row| {
            Ok(TaskExecution {
                id: row.get(0)?,
                task_id: row.get(1)?,
                status: row.get(2)?,
                started_at: row.get(3)?,
                finished_at: row.get(4)?,
                result: row.get(5)?,
                error: row.get(6)?,
                duration_ms: row.get(7)?,
            })
        });
        match result {
            Ok(exec) => Ok(Some(exec)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}
