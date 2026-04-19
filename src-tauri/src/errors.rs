//! Unified error type for EidosInsight

use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Paper not found: {id}")]
    PaperNotFound { id: String },

    #[error("Folder not found: {id}")]
    FolderNotFound { id: String },

    #[error("Note not found: {id}")]
    NoteNotFound { id: String },

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("PDF parse error: {0}")]
    PdfParse(String),

    #[error("Document parse error: {0}")]
    DocumentParse(String),

    #[error("LLM error: {0}")]
    Llm(String),

    #[error("Keyring error: {0}")]
    Keyring(String),

    #[error("{0}")]
    Internal(String),
}

// Serialize as a simple string message for the frontend
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
