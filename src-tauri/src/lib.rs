//! EidosInsight - AI-driven personal research knowledge assistant
//!
//! Application entry point for the Tauri backend.

mod commands;
mod db;
mod errors;
mod models;
mod services;

#[cfg(test)]
mod test_e2e;

use commands::*;
use services::llm::{LlmRouter, OllamaAdapter, OllamaConfig};
use services::LlmState;
use services::scheduler;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Initialize database
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to resolve app data dir");
            std::fs::create_dir_all(&app_data_dir).ok();

            let db_path = app_data_dir.join("eidosinsight.db");
            let papers_dir = app_data_dir.join("papers");
            let exports_dir = app_data_dir.join("exports");
            let backups_dir = app_data_dir.join("backups");
            let logs_dir = app_data_dir.join("logs");
            let skills_dir = app_data_dir.join("skills");

            for dir in [&papers_dir, &exports_dir, &backups_dir, &logs_dir, &skills_dir] {
                std::fs::create_dir_all(dir).ok();
            }

            // Initialize database connection pool
            let db_manager = db::DatabaseManager::new(&db_path)?;
            db_manager.run_migrations()?;

            // Store state
            let db_arc = std::sync::Arc::new(std::sync::Mutex::new(db_manager));
            app.manage(db::DbState(db_arc.clone()));
            let logs_dir_clone = logs_dir.clone();
            app.manage(services::AppState {
                app_data_dir: app_data_dir.clone(),
                papers_dir,
                exports_dir,
                backups_dir,
                logs_dir,
                skills_dir,
            });

            // Initialize LLM router
            let ollama_config = OllamaConfig::default();
            let ollama_adapter = OllamaAdapter::new(ollama_config);
            let router = LlmRouter::new(ollama_adapter);
            app.manage(LlmState(std::sync::Arc::new(router)));

            // Initialize logging
            let log_file = logs_dir_clone.join("eidosinsight.log");
            let log_file = log_file.to_string_lossy().to_string();
            tauri::async_runtime::spawn(async move {
                use tracing_subscriber::{fmt, EnvFilter};
                let file = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&log_file)
                    .expect("Failed to open log file");
                fmt()
                    .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
                    .with_writer(std::sync::Mutex::new(file))
                    .with_ansi(false)
                    .init();
            });

            tracing::info!("EidosInsight started, data dir: {:?}", app_data_dir);

            // Start automation scheduler
            let app_handle = app.handle().clone();
            scheduler::start_scheduler(app_handle, db_arc);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Paper commands
            paper_cmd::import_paper,
            paper_cmd::import_pdf,
            paper_cmd::import_pdf_from_path,
            paper_cmd::batch_import_pdfs,
            paper_cmd::get_paper,
            paper_cmd::list_papers,
            paper_cmd::update_paper,
            paper_cmd::delete_paper,
            paper_cmd::move_paper_to_folder,
            paper_cmd::add_paper_tag,
            paper_cmd::remove_paper_tag,
            paper_cmd::read_pdf_file,
            // Search commands
            search_cmd::search_papers,
            search_cmd::search_fulltext,
            search_cmd::search_advanced,
            search_cmd::search_suggest,
            // Folder commands
            folder_cmd::create_folder,
            folder_cmd::list_folders,
            folder_cmd::update_folder,
            folder_cmd::delete_folder,
            // Tag commands
            tag_cmd::list_tags,
            tag_cmd::rename_tag,
            tag_cmd::delete_tag,
            // Note commands
            note_cmd::create_note,
            note_cmd::get_note,
            note_cmd::list_notes,
            note_cmd::update_note,
            note_cmd::delete_note,
            note_cmd::create_note_link,
            note_cmd::delete_note_link,
            note_cmd::get_note_with_links,
            note_cmd::list_outgoing_links,
            note_cmd::list_backlinks,
            note_cmd::search_notes,
            note_cmd::note_link_suggest,
            // Reading commands
            reading_cmd::get_reading_state,
            reading_cmd::update_reading_state,
            reading_cmd::list_reading_progress,
            // Export commands
            export_cmd::export_papers,
            // Settings commands
            settings_cmd::get_settings,
            settings_cmd::update_settings,
            settings_cmd::get_llm_config,
            settings_cmd::update_llm_config,
            settings_cmd::test_llm_connection,
            // Onboarding commands
            onboarding_cmd::is_first_launch,
            onboarding_cmd::complete_onboarding,
            onboarding_cmd::load_sample_data,
            // LLM commands
            llm_cmd::detect_ollama,
            llm_cmd::get_llm_status,
            llm_cmd::list_llm_models,
            llm_cmd::llm_complete,
            llm_cmd::llm_chat_stream,
            llm_cmd::save_api_key,
            llm_cmd::remove_api_key,
            llm_cmd::configure_llm_provider,
            llm_cmd::ai_wizard_demo,
            // Browser extension commands
            browser_cmd::import_from_browser,
            browser_cmd::get_extension_status,
            browser_cmd::register_native_host,
            // AI assistant commands
            ai_cmd::ai_explain_term,
            ai_cmd::ai_summarize,
            ai_cmd::ai_extract_insights,
            ai_cmd::ai_check_availability,
            // Automation commands
            automation_cmd::list_preset_tasks,
            automation_cmd::list_auto_tasks,
            automation_cmd::enable_task,
            automation_cmd::disable_task,
            automation_cmd::update_task_schedule,
            automation_cmd::update_task_config,
            automation_cmd::delete_task,
            automation_cmd::get_task_history,
            automation_cmd::run_task_now,
            automation_cmd::list_skills,
            automation_cmd::create_skill,
            automation_cmd::import_skill,
            automation_cmd::update_skill,
            automation_cmd::delete_skill,
            automation_cmd::invoke_skill,
            automation_cmd::schedule_to_cron,
            automation_cmd::schedule_description,
        ])
        .run(tauri::generate_context!())
        .expect("error while running EidosInsight");
}
