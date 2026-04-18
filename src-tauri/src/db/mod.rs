//! Database module - SQLite with FTS5

mod manager;
pub mod migrations;
mod repos;

pub use manager::{DatabaseManager, DbState};
pub use repos::*;
