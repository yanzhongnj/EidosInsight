//! Tag repository

use rusqlite::{params, Connection};
use crate::errors::Result;
use crate::models::*;

pub struct TagRepo;

impl TagRepo {
    pub fn find_or_create(conn: &Connection, name: &str, color: Option<&str>) -> Result<Tag> {
        // Try to find existing tag
        let existing = conn.query_row(
            "SELECT id, name, color, usage_count, date_created FROM tags WHERE name = ?1",
            params![name],
            |row| Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                usage_count: row.get(3)?,
                date_created: row.get(4)?,
            }),
        );

        if let Ok(tag) = existing {
            return Ok(tag);
        }

        // Create new tag
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().naive_utc().format("%Y-%m-%d %H:%M:%S").to_string();
        conn.execute(
            "INSERT INTO tags (id, name, color, date_created) VALUES (?1, ?2, ?3, ?4)",
            params![id, name, color, now],
        )?;

        Ok(Tag {
            id,
            name: name.to_string(),
            color: color.map(|c| c.to_string()),
            usage_count: 0,
            date_created: now,
        })
    }

    pub fn list(conn: &Connection) -> Result<Vec<Tag>> {
        let mut stmt = conn.prepare(
            "SELECT id, name, color, usage_count, date_created FROM tags ORDER BY usage_count DESC, name ASC"
        )?;
        let tags = stmt.query_map([], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                usage_count: row.get(3)?,
                date_created: row.get(4)?,
            })
        })?.filter_map(|r| r.ok()).collect();
        Ok(tags)
    }

    pub fn add_paper_tag(conn: &Connection, paper_id: &str, tag_id: &str) -> Result<()> {
        let now = chrono::Utc::now().naive_utc().format("%Y-%m-%d %H:%M:%S").to_string();
        conn.execute(
            "INSERT OR IGNORE INTO paper_tags (paper_id, tag_id, date_added) VALUES (?1, ?2, ?3)",
            params![paper_id, tag_id, now],
        )?;
        conn.execute("UPDATE tags SET usage_count = usage_count + 1 WHERE id = ?1", params![tag_id])?;
        Ok(())
    }

    pub fn remove_paper_tag(conn: &Connection, paper_id: &str, tag_id: &str) -> Result<()> {
        conn.execute("DELETE FROM paper_tags WHERE paper_id = ?1 AND tag_id = ?2", params![paper_id, tag_id])?;
        conn.execute("UPDATE tags SET usage_count = MAX(0, usage_count - 1) WHERE id = ?1", params![tag_id])?;
        Ok(())
    }

    pub fn get_tags_for_paper(conn: &Connection, paper_id: &str) -> Result<Vec<Tag>> {
        let mut stmt = conn.prepare(
            "SELECT t.id, t.name, t.color, t.usage_count, t.date_created
             FROM tags t INNER JOIN paper_tags pt ON t.id = pt.tag_id
             WHERE pt.paper_id = ?1 ORDER BY t.name"
        )?;
        let tags = stmt.query_map(params![paper_id], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                usage_count: row.get(3)?,
                date_created: row.get(4)?,
            })
        })?.filter_map(|r| r.ok()).collect();
        Ok(tags)
    }

    pub fn rename(conn: &Connection, id: &str, new_name: &str) -> Result<()> {
        conn.execute("UPDATE tags SET name = ?1 WHERE id = ?2", params![new_name, id])?;
        Ok(())
    }

    pub fn delete(conn: &Connection, id: &str) -> Result<()> {
        conn.execute("DELETE FROM paper_tags WHERE tag_id = ?1", params![id])?;
        conn.execute("DELETE FROM tags WHERE id = ?1", params![id])?;
        Ok(())
    }
}
