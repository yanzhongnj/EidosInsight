//! Automation task and skill Tauri command handlers

use tauri::State;

use crate::db::DbState;
use crate::errors::{AppError, Result};
use crate::models::*;
use crate::services::scheduler;

type DbStateRef<'a> = State<'a, DbState>;

// ── Preset Templates ────────────────────────────────────────────────────────

/// List preset task templates
#[tauri::command]
pub fn list_preset_tasks() -> Vec<PresetTemplate> {
    vec![
        PresetTemplate {
            preset_id: "daily_trending".into(),
            name: "每日技术热点".into(),
            description: "搜索关注领域最新论文，批量生成摘要，每天推送".into(),
            definition: r#"{"steps":[{"id":"search","tool":"paper.search","input":{"query":"{{keywords}}","sort":"trending","time_range":"day","limit":10}},{"id":"brief","tool":"paper.brief","input":{"papers":"{{search.results}}"}}],"output":{"type":"notification","template":"今日{{keywords}}热点：\n{{brief.results}}"}}"#.into(),
            default_cron: "0 8 * * *".into(),
            config_fields: vec![
                PresetConfigField {
                    key: "keywords".into(),
                    label: "关注关键词".into(),
                    field_type: "text".into(),
                    required: true,
                    default_value: Some("".into()),
                    options: None,
                },
            ],
        },
        PresetTemplate {
            preset_id: "weekly_trending".into(),
            name: "每周热点追踪".into(),
            description: "生成周度热点报告，每周推送".into(),
            definition: r#"{"steps":[{"id":"search","tool":"paper.search","input":{"query":"{{keywords}}","sort":"trending","time_range":"week","limit":10}},{"id":"brief","tool":"paper.brief","input":{"papers":"{{search.results}}"}}],"output":{"type":"notification","template":"本周{{keywords}}热点：\n{{brief.results}}"}}"#.into(),
            default_cron: "0 9 * * 1".into(),
            config_fields: vec![
                PresetConfigField {
                    key: "keywords".into(),
                    label: "关注关键词".into(),
                    field_type: "text".into(),
                    required: true,
                    default_value: Some("".into()),
                    options: None,
                },
            ],
        },
        PresetTemplate {
            preset_id: "new_paper_sub".into(),
            name: "新论文订阅".into(),
            description: "搜索指定关键词新论文，发现时通知".into(),
            definition: r#"{"steps":[{"id":"search","tool":"paper.search","input":{"query":"{{keywords}}","sort":"newest","time_range":"day","limit":5}}],"output":{"type":"notification","template":"新论文通知：\n{{search.results}}"}}"#.into(),
            default_cron: "0 10,16 * * *".into(),
            config_fields: vec![
                PresetConfigField {
                    key: "keywords".into(),
                    label: "关注关键词/作者".into(),
                    field_type: "text".into(),
                    required: true,
                    default_value: Some("".into()),
                    options: None,
                },
            ],
        },
        PresetTemplate {
            preset_id: "daily_reading".into(),
            name: "每日阅读提醒".into(),
            description: "检查阅读进度，推送未读清单".into(),
            definition: r#"{"steps":[{"id":"check","tool":"reading.unread","input":{"limit":5}}],"output":{"type":"notification","template":"你有{{check.count}}篇未读论文"}}"#.into(),
            default_cron: "0 20 * * *".into(),
            config_fields: vec![],
        },
        PresetTemplate {
            preset_id: "citation_refresh".into(),
            name: "引用数据刷新".into(),
            description: "定时刷新引用缓存数据".into(),
            definition: r#"{"steps":[{"id":"refresh","tool":"citation.refresh","input":{}}],"output":{"type":"silent"}}"#.into(),
            default_cron: "0 3 * * *".into(),
            config_fields: vec![],
        },
        PresetTemplate {
            preset_id: "zotero_sync".into(),
            name: "Zotero增量同步".into(),
            description: "与Zotero同步文献库".into(),
            definition: r#"{"steps":[{"id":"sync","tool":"zotero.sync","input":{}}],"output":{"type":"silent"}}"#.into(),
            default_cron: "*/30 * * * *".into(),
            config_fields: vec![
                PresetConfigField {
                    key: "zotero_data_dir".into(),
                    label: "Zotero数据目录".into(),
                    field_type: "text".into(),
                    required: false,
                    default_value: None,
                    options: None,
                },
            ],
        },
    ]
}

// ── AutoTask Commands ───────────────────────────────────────────────────────

#[tauri::command]
pub fn list_auto_tasks(db: DbStateRef<'_>) -> Result<Vec<AutoTask>> {
    let db_manager = db.0.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    crate::db::TaskRepo::list_all(db_manager.conn())
}

#[tauri::command]
pub fn enable_task(db: DbStateRef<'_>, preset_id: Option<String>, name: String, definition: String, schedule: String, schedule_cron: Option<String>, config: Option<String>) -> Result<AutoTask> {
    let db_manager = db.0.lock().map_err(|e| AppError::Internal(e.to_string()))?;

    // Parse schedule to cron if not provided
    let cron = match schedule_cron {
        Some(c) => c,
        None => {
            let sched: ScheduleConfig = serde_json::from_str(&schedule)
                .map_err(|e| AppError::Internal(format!("Invalid schedule config: {}", e)))?;
            scheduler::schedule_to_cron(&sched)
        }
    };

    // Compute next_run
    let next_run = scheduler::next_run_from_cron(&cron);

    let input = CreateAutoTask {
        name,
        task_type: if preset_id.is_some() { "preset".into() } else { "custom".into() },
        preset_id,
        definition,
        schedule,
        schedule_cron: Some(cron),
        config,
    };

    let task = crate::db::TaskRepo::create(db_manager.conn(), &input)?;

    // Update next_run
    if let Some(nr) = next_run {
        crate::db::TaskRepo::update_last_run(db_manager.conn(), &task.id, "", Some(&nr))?;
        crate::db::TaskRepo::enable(db_manager.conn(), &task.id)?;
    }

    crate::db::TaskRepo::get_by_id(db_manager.conn(), &task.id)
}

#[tauri::command]
pub fn disable_task(db: DbStateRef<'_>, id: String) -> Result<()> {
    let db_manager = db.0.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    crate::db::TaskRepo::disable(db_manager.conn(), &id)
}

#[tauri::command]
pub fn update_task_schedule(db: DbStateRef<'_>, id: String, schedule: String, schedule_cron: Option<String>) -> Result<()> {
    let db_manager = db.0.lock().map_err(|e| AppError::Internal(e.to_string()))?;

    let cron = match schedule_cron {
        Some(c) => c,
        None => {
            let sched: ScheduleConfig = serde_json::from_str(&schedule)
                .map_err(|e| AppError::Internal(format!("Invalid schedule config: {}", e)))?;
            scheduler::schedule_to_cron(&sched)
        }
    };

    let next_run = scheduler::next_run_from_cron(&cron);

    let update = UpdateAutoTask {
        name: None,
        definition: None,
        schedule: Some(schedule),
        schedule_cron: Some(cron),
        is_enabled: None,
        config: None,
    };

    crate::db::TaskRepo::update(db_manager.conn(), &id, &update)?;

    // Update next_run
    if let Some(nr) = next_run {
        crate::db::TaskRepo::update_last_run(db_manager.conn(), &id, "", Some(&nr))?;
    }

    Ok(())
}

#[tauri::command]
pub fn update_task_config(db: DbStateRef<'_>, id: String, config: String) -> Result<()> {
    let db_manager = db.0.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    let update = UpdateAutoTask {
        name: None,
        definition: None,
        schedule: None,
        schedule_cron: None,
        is_enabled: None,
        config: Some(config),
    };
    crate::db::TaskRepo::update(db_manager.conn(), &id, &update)
}

#[tauri::command]
pub fn delete_task(db: DbStateRef<'_>, id: String) -> Result<()> {
    let db_manager = db.0.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    crate::db::TaskRepo::delete(db_manager.conn(), &id)
}

#[tauri::command]
pub fn get_task_history(db: DbStateRef<'_>, task_id: String, limit: Option<u32>) -> Result<Vec<TaskExecution>> {
    let db_manager = db.0.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    crate::db::TaskRepo::list_executions(db_manager.conn(), &task_id, limit.unwrap_or(20))
}

#[tauri::command]
pub fn run_task_now(db: DbStateRef<'_>, id: String) -> Result<TaskExecution> {
    let db_manager = db.0.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    let task = crate::db::TaskRepo::get_by_id(db_manager.conn(), &id)?;
    let execution = crate::db::TaskRepo::create_execution(db_manager.conn(), &id)?;

    // For M3, just record a placeholder execution result
    let start = std::time::Instant::now();
    let config: serde_json::Value =
        serde_json::from_str(task.config.as_deref().unwrap_or("{}"))
            .unwrap_or(serde_json::Value::Null);
    let keywords = config.get("keywords").and_then(|k| k.as_str()).unwrap_or("");
    let output = format!("手动执行任务「{}」完成。关键词：{}", task.name, keywords);
    let duration_ms = start.elapsed().as_millis() as i64;

    crate::db::TaskRepo::finish_execution(
        db_manager.conn(),
        &execution.id,
        "success",
        Some(&output),
        None,
        duration_ms,
    )?;

    let now = chrono::Utc::now().to_rfc3339();
    let next = task.schedule_cron.as_ref().and_then(|c| scheduler::next_run_from_cron(c));
    crate::db::TaskRepo::update_last_run(db_manager.conn(), &id, &now, next.as_deref())?;

    crate::db::TaskRepo::list_executions(db_manager.conn(), &id, 1)?
        .into_iter()
        .next()
        .ok_or(AppError::Internal("Execution not found after insert".into()))
}

// ── Skill Commands ──────────────────────────────────────────────────────────

#[tauri::command]
pub fn list_skills(db: DbStateRef<'_>) -> Result<Vec<Skill>> {
    let db_manager = db.0.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    crate::db::SkillRepo::list_all(db_manager.conn())
}

#[tauri::command]
pub fn create_skill(db: DbStateRef<'_>, name: String, definition: String, source: String, trigger: Option<String>) -> Result<Skill> {
    let db_manager = db.0.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    let input = CreateSkill { name, definition, source, trigger };
    crate::db::SkillRepo::create(db_manager.conn(), &input)
}

#[tauri::command]
pub fn import_skill(db: DbStateRef<'_>, markdown: String) -> Result<Skill> {
    let db_manager = db.0.lock().map_err(|e| AppError::Internal(e.to_string()))?;

    // Parse YAML frontmatter to extract name and trigger
    let (name, trigger) = if markdown.starts_with("---") {
        let end = markdown[3..].find("---").map(|i| i + 3);
        match end {
            Some(end_idx) => {
                let fm = &markdown[3..end_idx];
                let name = fm.lines()
                    .find(|l| l.starts_with("name:"))
                    .and_then(|l| l.split(':').nth(1))
                    .map(|v| v.trim().trim_matches('"').trim_matches('\'').to_string())
                    .unwrap_or_else(|| "Imported Skill".to_string());
                let trigger = fm.lines()
                    .find(|l| l.starts_with("trigger:"))
                    .and_then(|l| l.split(':').nth(1))
                    .map(|v| v.trim().trim_matches('"').trim_matches('\'').to_string());
                (name, trigger)
            }
            None => ("Imported Skill".to_string(), None),
        }
    } else {
        ("Imported Skill".to_string(), None)
    };

    let input = CreateSkill {
        name,
        definition: markdown,
        source: "community".to_string(),
        trigger,
    };
    crate::db::SkillRepo::create(db_manager.conn(), &input)
}

#[tauri::command]
pub fn update_skill(db: DbStateRef<'_>, id: String, name: Option<String>, definition: Option<String>, trigger: Option<String>, is_active: Option<bool>) -> Result<()> {
    let db_manager = db.0.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    let input = UpdateSkill { name, definition, trigger, is_active };
    crate::db::SkillRepo::update(db_manager.conn(), &id, &input)
}

#[tauri::command]
pub fn delete_skill(db: DbStateRef<'_>, id: String) -> Result<()> {
    let db_manager = db.0.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    crate::db::SkillRepo::delete(db_manager.conn(), &id)
}

#[tauri::command]
pub fn invoke_skill(db: DbStateRef<'_>, skill_id: String, params: Option<std::collections::HashMap<String, serde_json::Value>>) -> Result<SkillResult> {
    let db_manager = db.0.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    let skill = crate::db::SkillRepo::get_by_id(db_manager.conn(), &skill_id)?;

    let start = std::time::Instant::now();

    // For M3: placeholder execution
    // Full tool-chain execution will be in M4/M5
    let params_desc = params
        .map(|p| {
            p.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default();

    let output = if params_desc.is_empty() {
        format!("技能「{}」执行完成", skill.name)
    } else {
        format!("技能「{}」执行完成，参数：{}", skill.name, params_desc)
    };

    Ok(SkillResult {
        success: true,
        output,
        duration_ms: start.elapsed().as_millis() as u64,
        error: None,
    })
}

/// Convert a human-readable schedule config to a cron expression
#[tauri::command]
pub fn schedule_to_cron(schedule: String) -> Result<String> {
    let cfg: ScheduleConfig = serde_json::from_str(&schedule)
        .map_err(|e| AppError::Internal(format!("Invalid schedule config: {}", e)))?;
    Ok(scheduler::schedule_to_cron(&cfg))
}

/// Get a human-readable description of a schedule
#[tauri::command]
pub fn schedule_description(schedule: String) -> Result<String> {
    let cfg: ScheduleConfig = serde_json::from_str(&schedule)
        .map_err(|e| AppError::Internal(format!("Invalid schedule config: {}", e)))?;
    Ok(scheduler::schedule_description(&cfg))
}
