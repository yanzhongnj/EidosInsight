//! Settings repository

use rusqlite::{params, Connection};
use crate::errors::Result;
use crate::models::AppSettings;

pub struct SettingsRepo;

impl SettingsRepo {
    pub fn get(conn: &Connection) -> Result<AppSettings> {
        let mut settings = AppSettings::default();

        let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
        let rows: Vec<(String, String)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .filter_map(|r| r.ok())
            .collect();

        for (key, value) in rows {
            match key.as_str() {
                "theme" => settings.theme = value,
                "language" => settings.language = value,
                "auto_process_papers" => settings.auto_process_papers = value == "true",
                "default_llm_provider" => settings.default_llm_provider = value,
                "check_updates_on_start" => settings.check_updates_on_start = value == "true",
                "paper_storage_path" => settings.paper_storage_path = Some(value),
                "max_backup_count" => settings.max_backup_count = value.parse().unwrap_or(5),
                "export_format" => settings.export_format = value,
                _ => {}
            }
        }

        Ok(settings)
    }

    pub fn set(conn: &Connection, key: &str, value: &str) -> Result<()> {
        conn.execute(
            "INSERT INTO settings (key, value, updated_at) VALUES (?1, ?2, datetime('now'))
             ON CONFLICT(key) DO UPDATE SET value = ?2, updated_at = datetime('now')",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_value(conn: &Connection, key: &str) -> Result<Option<String>> {
        let result = conn.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get(0),
        );
        match result {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn update_settings(conn: &Connection, settings: &AppSettings) -> Result<()> {
        Self::set(conn, "theme", &settings.theme)?;
        Self::set(conn, "language", &settings.language)?;
        Self::set(conn, "auto_process_papers", if settings.auto_process_papers { "true" } else { "false" })?;
        Self::set(conn, "default_llm_provider", &settings.default_llm_provider)?;
        Self::set(conn, "check_updates_on_start", if settings.check_updates_on_start { "true" } else { "false" })?;
        if let Some(ref path) = settings.paper_storage_path {
            Self::set(conn, "paper_storage_path", path)?;
        }
        Self::set(conn, "max_backup_count", &settings.max_backup_count.to_string())?;
        Self::set(conn, "export_format", &settings.export_format)?;
        Ok(())
    }
}
