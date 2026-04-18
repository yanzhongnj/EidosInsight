//! Automation scheduler service
//!
//! Runs as a background tokio task. Wakes up every minute, checks enabled
//! auto_tasks whose next_run <= now, executes them, and records results.
//! Natural language schedule → cron conversion lives here too.

use std::sync::Arc;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};

use crate::db::DatabaseManager;
use crate::db::TaskRepo;
use crate::errors::Result;
use crate::models::AutoTask;

// Re-export for commands
pub use crate::models::ScheduleConfig;

/// Convert a human-readable ScheduleConfig to a cron expression.
///
/// The user never sees the cron; this is purely internal.
pub fn schedule_to_cron(cfg: &ScheduleConfig) -> String {
    match cfg.frequency.as_str() {
        "daily" => {
            let (h, m) = resolve_time(&cfg.time, cfg.custom_hour, cfg.custom_minute);
            format!("{} {} * * *", m, h)
        }
        "weekly" => {
            let (h, m) = resolve_time(&cfg.time, cfg.custom_hour, cfg.custom_minute);
            let days = cfg
                .weekdays
                .as_ref()
                .map(|w| w.iter().map(|d| d.to_string()).collect::<Vec<_>>().join(","))
                .unwrap_or_else(|| "1".to_string()); // default Monday
            format!("{} {} * * {}", m, h, days)
        }
        "custom" => {
            if let Some(interval) = cfg.custom_interval_hours {
                if interval > 0 {
                    format!("0 */{} * * *", interval)
                } else {
                    // Every 30 minutes (for Zotero sync)
                    "*/30 * * * *".to_string()
                }
            } else {
                let h = cfg.custom_hour.unwrap_or(8);
                let m = cfg.custom_minute.unwrap_or(0);
                format!("{} {} * * *", m, h)
            }
        }
        _ => "0 8 * * *".to_string(), // fallback: 8am daily
    }
}

fn resolve_time(
    time_label: &str,
    custom_hour: Option<u8>,
    custom_minute: Option<u8>,
) -> (u8, u8) {
    match time_label {
        "morning_8" => (8, 0),
        "noon_12" => (12, 0),
        "evening_8" => (20, 0),
        "custom" => (
            custom_hour.unwrap_or(8),
            custom_minute.unwrap_or(0),
        ),
        _ => (8, 0),
    }
}

/// Human-readable description of a schedule (for frontend display)
pub fn schedule_description(cfg: &ScheduleConfig) -> String {
    match cfg.frequency.as_str() {
        "daily" => {
            let (h, m) = resolve_time(&cfg.time, cfg.custom_hour, cfg.custom_minute);
            format!("每天 {:02}:{:02}", h, m)
        }
        "weekly" => {
            let (h, m) = resolve_time(&cfg.time, cfg.custom_hour, cfg.custom_minute);
            let day_names = ["日", "一", "二", "三", "四", "五", "六"];
            let days = cfg
                .weekdays
                .as_ref()
                .map(|w| {
                    w.iter()
                        .filter_map(|d| day_names.get(*d as usize).copied())
                        .collect::<Vec<_>>()
                        .join("、")
                })
                .unwrap_or_else(|| "一".to_string());
            format!("每周{} {:02}:{:02}", days, h, m)
        }
        "custom" => {
            if let Some(interval) = cfg.custom_interval_hours {
                if interval > 0 {
                    format!("每隔{}小时", interval)
                } else {
                    "每30分钟".to_string()
                }
            } else {
                let h = cfg.custom_hour.unwrap_or(8);
                let m = cfg.custom_minute.unwrap_or(0);
                format!("每天 {:02}:{:02}", h, m)
            }
        }
        _ => "每天 08:00".to_string(),
    }
}

/// Compute next_run timestamp from a cron expression.
/// Returns None if the cron expression is invalid.
pub fn next_run_from_cron(cron_expr: &str) -> Option<String> {
    use cron::Schedule;
    use std::str::FromStr;

    // The `cron` crate expects 7-field format: sec min hour dom month dow year
    // Our expressions are 5-field: min hour dom month dow
    // Prepend "0 " for seconds, append " *" for year
    let full_cron = format!("0 {} *", cron_expr);
    let sched = Schedule::from_str(&full_cron).ok()?;
    let next = sched.upcoming(chrono::Utc).next()?;
    Some(next.to_rfc3339())
}

/// The scheduler background loop.
/// Spawns a tokio task that wakes every 60s and checks for due tasks.
pub fn start_scheduler(app_handle: AppHandle, db: Arc<Mutex<DatabaseManager>>) {
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;

            if let Err(e) = tick(&app_handle, &db) {
                tracing::error!("Scheduler tick error: {}", e);
            }
        }
    });
}

fn tick(app_handle: &AppHandle, db: &Arc<Mutex<DatabaseManager>>) -> Result<()> {
    let now = chrono::Utc::now().to_rfc3339();

    let due_tasks: Vec<AutoTask> = {
        let db_manager = db.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
        let conn = db_manager.conn();
        let enabled = TaskRepo::list_enabled(conn)?;

        enabled
            .into_iter()
            .filter(|t| {
                t.next_run
                    .as_ref()
                    .map(|nr| nr <= &now)
                    .unwrap_or(true) // No next_run = should run now
            })
            .collect()
    };

    for task in due_tasks {
        tracing::info!("Executing scheduled task: {} ({})", task.name, task.id);

        let start = std::time::Instant::now();
        let execution_id = {
            let db_manager = db.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
            let conn = db_manager.conn();
            let execution = TaskRepo::create_execution(conn, &task.id)?;
            execution.id
        };

        // Execute the task (blocking for now; tool-chain is M4)
        let result = execute_task(task.id.clone(), &task);
        let duration_ms = start.elapsed().as_millis() as i64;

        {
            let db_manager = db.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
            let conn = db_manager.conn();

            match result {
                Ok(output) => {
                    TaskRepo::finish_execution(
                        conn,
                        &execution_id,
                        "success",
                        Some(&output),
                        None,
                        duration_ms,
                    )?;
                }
                Err(e) => {
                    TaskRepo::finish_execution(
                        conn,
                        &execution_id,
                        "failed",
                        None,
                        Some(&e.to_string()),
                        duration_ms,
                    )?;
                }
            }

            // Update last_run and compute next_run
            let next = task
                .schedule_cron
                .as_ref()
                .and_then(|cron| next_run_from_cron(cron));
            TaskRepo::update_last_run(
                conn,
                &task.id,
                &now,
                next.as_deref(),
            )?;
        }

        // Emit event to frontend
        let _ = app_handle.emit("task-executed", &task.id);
    }

    Ok(())
}

/// Execute a single task.
/// For now this is a placeholder that returns a summary string.
/// Full tool-chain execution will be added in M4/M5.
fn execute_task(_task_id: String, task: &AutoTask) -> Result<String> {
    // Parse the definition JSON to get steps
    let definition: serde_json::Value =
        serde_json::from_str(&task.definition).unwrap_or(serde_json::Value::Null);

    let step_count = definition
        .get("steps")
        .and_then(|s| s.as_array())
        .map(|a| a.len())
        .unwrap_or(0);

    // For M3, we emit a notification summary
    let config: serde_json::Value =
        serde_json::from_str(task.config.as_deref().unwrap_or("{}"))
            .unwrap_or(serde_json::Value::Null);

    let keywords = config
        .get("keywords")
        .and_then(|k| k.as_str())
        .unwrap_or("");

    let output_template = definition
        .get("output")
        .and_then(|o| o.get("template"))
        .and_then(|t| t.as_str())
        .unwrap_or("任务执行完成");

    let output = output_template
        .replace("{{keywords}}", keywords)
        .replace("{{brief.results}}", "（摘要结果将在 M4 完整实现）")
        .replace("{{search.results}}", "（搜索结果将在 M4 完整实现）")
        .replace("{{check.count}}", "3");

    Ok(format!(
        "任务「{}」执行完成（{}步），输出：{}",
        task.name, step_count, output
    ))
}
