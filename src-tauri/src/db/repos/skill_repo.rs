//! Repository for skills

use rusqlite::{params, Connection};
use crate::errors::Result;
use crate::models::{Skill, CreateSkill, UpdateSkill};

pub struct SkillRepo;

impl SkillRepo {
    pub fn create(conn: &Connection, input: &CreateSkill) -> Result<Skill> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO skills (id, name, definition, source, version, trigger, is_active, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, '1.0.0', ?5, 1, ?6, ?6)",
            params![id, input.name, input.definition, input.source, input.trigger, now],
        )?;

        Self::get_by_id(conn, &id)
    }

    pub fn get_by_id(conn: &Connection, id: &str) -> Result<Skill> {
        let mut stmt = conn.prepare(
            "SELECT id, name, definition, source, version, trigger, is_active, created_at, updated_at
             FROM skills WHERE id = ?1"
        )?;
        Ok(stmt.query_row(params![id], |row| {
            Ok(Skill {
                id: row.get(0)?,
                name: row.get(1)?,
                definition: row.get(2)?,
                source: row.get(3)?,
                version: row.get(4)?,
                trigger: row.get(5)?,
                is_active: row.get::<_, i32>(6)? != 0,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })?)
    }

    pub fn list_all(conn: &Connection) -> Result<Vec<Skill>> {
        let mut stmt = conn.prepare(
            "SELECT id, name, definition, source, version, trigger, is_active, created_at, updated_at
             FROM skills ORDER BY created_at"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Skill {
                id: row.get(0)?,
                name: row.get(1)?,
                definition: row.get(2)?,
                source: row.get(3)?,
                version: row.get(4)?,
                trigger: row.get(5)?,
                is_active: row.get::<_, i32>(6)? != 0,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn list_active(conn: &Connection) -> Result<Vec<Skill>> {
        let mut stmt = conn.prepare(
            "SELECT id, name, definition, source, version, trigger, is_active, created_at, updated_at
             FROM skills WHERE is_active = 1 ORDER BY name"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Skill {
                id: row.get(0)?,
                name: row.get(1)?,
                definition: row.get(2)?,
                source: row.get(3)?,
                version: row.get(4)?,
                trigger: row.get(5)?,
                is_active: row.get::<_, i32>(6)? != 0,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn find_by_trigger(conn: &Connection, trigger: &str) -> Result<Option<Skill>> {
        let mut stmt = conn.prepare(
            "SELECT id, name, definition, source, version, trigger, is_active, created_at, updated_at
             FROM skills WHERE trigger = ?1 AND is_active = 1"
        )?;
        let result = stmt.query_row(params![trigger], |row| {
            Ok(Skill {
                id: row.get(0)?,
                name: row.get(1)?,
                definition: row.get(2)?,
                source: row.get(3)?,
                version: row.get(4)?,
                trigger: row.get(5)?,
                is_active: row.get::<_, i32>(6)? != 0,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        });
        match result {
            Ok(skill) => Ok(Some(skill)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn update(conn: &Connection, id: &str, input: &UpdateSkill) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        let mut sql_parts = Vec::new();
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(ref v) = input.name {
            sql_parts.push("name = ?");
            param_values.push(Box::new(v.clone()));
        }
        if let Some(ref v) = input.definition {
            sql_parts.push("definition = ?");
            param_values.push(Box::new(v.clone()));
        }
        if let Some(ref v) = input.trigger {
            sql_parts.push("trigger = ?");
            param_values.push(Box::new(v.clone()));
        }
        if let Some(v) = input.is_active {
            sql_parts.push("is_active = ?");
            param_values.push(Box::new(v as i32));
        }

        if sql_parts.is_empty() {
            return Ok(());
        }

        sql_parts.push("updated_at = ?");
        param_values.push(Box::new(now));

        let sql = format!("UPDATE skills SET {} WHERE id = ?", sql_parts.join(", "));
        param_values.push(Box::new(id.to_string()));

        let param_refs: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
        conn.execute(&sql, param_refs.as_slice())?;
        Ok(())
    }

    pub fn delete(conn: &Connection, id: &str) -> Result<()> {
        conn.execute("DELETE FROM skills WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Load a skill from a Markdown file in the skills directory
    pub fn load_from_file(conn: &Connection, path: &std::path::Path) -> Result<Option<Skill>> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;

        // Parse YAML frontmatter
        let (frontmatter, _body) = if content.starts_with("---") {
            let end = content[3..].find("---").map(|i| i + 3);
            match end {
                Some(end_idx) => {
                    let fm = &content[3..end_idx];
                    let body = content[end_idx + 3..].trim();
                    (fm.to_string(), body.to_string())
                }
                None => return Ok(None),
            }
        } else {
            return Ok(None);
        };

        // Extract name from frontmatter
        let name = frontmatter
            .lines()
            .find(|l| l.starts_with("name:"))
            .and_then(|l| l.split(':').nth(1))
            .map(|v| v.trim().trim_matches('"').trim_matches('\''))
            .unwrap_or("Unknown Skill")
            .to_string();

        let trigger = frontmatter
            .lines()
            .find(|l| l.starts_with("trigger:"))
            .and_then(|l| l.split(':').nth(1))
            .map(|v| v.trim().trim_matches('"').trim_matches('\'').to_string());

        // Check if already imported (by trigger or name)
        if let Some(ref trig) = trigger {
            if let Ok(Some(_)) = Self::find_by_trigger(conn, trig) {
                return Ok(None); // Already exists
            }
        }

        let input = CreateSkill {
            name,
            definition: content,
            source: "builtin".to_string(),
            trigger,
        };

        Ok(Some(Self::create(conn, &input)?))
    }
}
