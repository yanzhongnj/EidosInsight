//! AI result repository

use rusqlite::{params, Connection};
use crate::errors::Result;
use crate::models::*;

pub struct AiResultRepo;

impl AiResultRepo {
    pub fn insert(conn: &Connection, result: &AiResult) -> Result<AiResult> {
        conn.execute(
            "INSERT INTO ai_results (id, result_type, collection_id, title, content, structured_data, model_used, paper_ids, dimensions, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                result.id, result.result_type, result.collection_id, result.title,
                result.content, result.structured_data, result.model_used,
                result.paper_ids, result.dimensions, result.created_at
            ],
        )?;
        Self::get_by_id(conn, &result.id)
    }

    pub fn get_by_id(conn: &Connection, id: &str) -> Result<AiResult> {
        conn.query_row(
            "SELECT id, result_type, collection_id, title, content, structured_data, model_used, paper_ids, dimensions, created_at
             FROM ai_results WHERE id = ?1",
            params![id],
            |row| Ok(AiResult {
                id: row.get(0)?,
                result_type: row.get(1)?,
                collection_id: row.get(2)?,
                title: row.get(3)?,
                content: row.get(4)?,
                structured_data: row.get(5)?,
                model_used: row.get(6)?,
                paper_ids: row.get(7)?,
                dimensions: row.get(8)?,
                created_at: row.get(9)?,
            }),
        ).map_err(|e| crate::errors::AppError::Internal(e.to_string()))
    }

    pub fn list_by_type(conn: &Connection, result_type: &str) -> Result<Vec<AiResult>> {
        let mut stmt = conn.prepare(
            "SELECT id, result_type, collection_id, title, content, structured_data, model_used, paper_ids, dimensions, created_at
             FROM ai_results WHERE result_type = ?1 ORDER BY created_at DESC"
        )?;
        let results = stmt.query_map(params![result_type], |row| {
            Ok(AiResult {
                id: row.get(0)?,
                result_type: row.get(1)?,
                collection_id: row.get(2)?,
                title: row.get(3)?,
                content: row.get(4)?,
                structured_data: row.get(5)?,
                model_used: row.get(6)?,
                paper_ids: row.get(7)?,
                dimensions: row.get(8)?,
                created_at: row.get(9)?,
            })
        })?.filter_map(|r| r.ok()).collect();
        Ok(results)
    }

    pub fn list_by_collection(conn: &Connection, collection_id: &str) -> Result<Vec<AiResult>> {
        let mut stmt = conn.prepare(
            "SELECT id, result_type, collection_id, title, content, structured_data, model_used, paper_ids, dimensions, created_at
             FROM ai_results WHERE collection_id = ?1 ORDER BY created_at DESC"
        )?;
        let results = stmt.query_map(params![collection_id], |row| {
            Ok(AiResult {
                id: row.get(0)?,
                result_type: row.get(1)?,
                collection_id: row.get(2)?,
                title: row.get(3)?,
                content: row.get(4)?,
                structured_data: row.get(5)?,
                model_used: row.get(6)?,
                paper_ids: row.get(7)?,
                dimensions: row.get(8)?,
                created_at: row.get(9)?,
            })
        })?.filter_map(|r| r.ok()).collect();
        Ok(results)
    }

    pub fn list_all(conn: &Connection) -> Result<Vec<AiResult>> {
        let mut stmt = conn.prepare(
            "SELECT id, result_type, collection_id, title, content, structured_data, model_used, paper_ids, dimensions, created_at
             FROM ai_results ORDER BY created_at DESC"
        )?;
        let results = stmt.query_map([], |row| {
            Ok(AiResult {
                id: row.get(0)?,
                result_type: row.get(1)?,
                collection_id: row.get(2)?,
                title: row.get(3)?,
                content: row.get(4)?,
                structured_data: row.get(5)?,
                model_used: row.get(6)?,
                paper_ids: row.get(7)?,
                dimensions: row.get(8)?,
                created_at: row.get(9)?,
            })
        })?.filter_map(|r| r.ok()).collect();
        Ok(results)
    }

    pub fn delete(conn: &Connection, id: &str) -> Result<()> {
        conn.execute("DELETE FROM ai_results WHERE id = ?1", params![id])?;
        Ok(())
    }
}
