//! Models for incremental crawl / file watching

use serde::{Deserialize, Serialize};

/// A directory being monitored for new/modified documents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchDir {
    pub id: String,
    /// Absolute path to the monitored directory
    pub path: String,
    /// Whether this watch directory is active
    pub is_enabled: bool,
    /// File extensions to include (comma-separated, e.g. "pdf,txt,docx")
    pub file_filter: Option<String>,
    /// Auto-assign folder ID for imported documents
    pub folder_id: Option<String>,
    /// Recurse into subdirectories
    pub recursive: bool,
    /// Scan interval in minutes (0 = use default from scheduler)
    pub scan_interval_min: u32,
    /// Last scan timestamp
    pub last_scan: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWatchDirInput {
    pub path: String,
    pub is_enabled: Option<bool>,
    pub file_filter: Option<String>,
    pub folder_id: Option<String>,
    pub recursive: Option<bool>,
    pub scan_interval_min: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateWatchDirInput {
    pub id: String,
    pub path: Option<String>,
    pub is_enabled: Option<bool>,
    pub file_filter: Option<String>,
    pub folder_id: Option<String>,
    pub recursive: Option<bool>,
    pub scan_interval_min: Option<u32>,
}

/// Fingerprint of a file for change detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileFingerprint {
    pub id: String,
    /// Absolute path of the source file
    pub file_path: String,
    /// The watch_dir this file was discovered from
    pub watch_dir_id: String,
    /// File size in bytes
    pub file_size: i64,
    /// Last modified time as Unix timestamp
    pub modified_time: i64,
    /// SHA-256 hash of the file content (for robust change detection)
    pub content_hash: Option<String>,
    /// Whether this file has been imported into the papers table
    pub is_imported: bool,
    /// The paper ID if imported
    pub paper_id: Option<String>,
    /// First discovered timestamp
    pub discovered_at: String,
    /// Last checked timestamp
    pub last_checked: String,
}

/// Result of a crawl scan — describes what was found
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlScanResult {
    /// Number of new files found
    pub new_files: u32,
    /// Number of modified files detected
    pub modified_files: u32,
    /// Number of files successfully imported
    pub imported: u32,
    /// Number of files that failed to import
    pub failed: u32,
    /// Total files scanned
    pub total_scanned: u32,
    /// Error messages from failed imports
    pub errors: Vec<String>,
}
