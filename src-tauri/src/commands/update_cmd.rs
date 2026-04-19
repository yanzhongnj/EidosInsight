//! Auto-update command — GitHub Releases API based update checker.
//!
//! This implementation does NOT use tauri-plugin-updater (which requires
//! Ed25519 signing keys). Instead, it queries the GitHub Releases API
//! to check for new versions and returns update info to the frontend.
//!
//! Future: migrate to tauri-plugin-updater once signing keys are set up.

use serde::{Deserialize, Serialize};

use crate::errors::AppError;

// ── Types ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateCheckResult {
    pub has_update: bool,
    pub current_version: String,
    pub latest_version: String,
    pub published_at: Option<String>,
    pub release_notes: Option<String>,
    pub download_url: Option<String>,
    pub html_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    name: Option<String>,
    body: Option<String>,
    published_at: Option<String>,
    html_url: Option<String>,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

// ── Constants ────────────────────────────────────────────────────────────────

const GITHUB_REPO: &str = "yanzhongnj/EidosInsight";
const USER_AGENT: &str = "EidosInsight-UpdateChecker/0.1.0";

// ── Commands ─────────────────────────────────────────────────────────────────

/// Check for updates via GitHub Releases API.
/// Returns update info if a newer version is available.
#[tauri::command]
pub async fn check_for_updates() -> Result<UpdateCheckResult, AppError> {
    let current = env!("CARGO_PKG_VERSION").to_string();

    let url = format!("https://api.github.com/repos/{}/releases/latest", GITHUB_REPO);

    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| AppError::Internal(format!("HTTP client error: {}", e)))?;

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("GitHub API request failed: {}", e)))?;

    if !resp.status().is_success() {
        // If 404, no releases yet — that's fine, not an error
        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(UpdateCheckResult {
                has_update: false,
                current_version: current.clone(),
                latest_version: current,
                published_at: None,
                release_notes: None,
                download_url: None,
                html_url: None,
            });
        }
        return Err(AppError::Internal(format!(
            "GitHub API returned status {}",
            resp.status()
        )));
    }

    let release: GitHubRelease = resp
        .json()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to parse GitHub release: {}", e)))?;

    // Normalize version strings (strip leading 'v')
    let latest_raw = release.tag_name.trim_start_matches('v').to_string();
    let latest = latest_raw.clone();

    let has_update = is_newer_version(&current, &latest);

    // Find the NSIS installer asset for Windows
    let download_url = release
        .assets
        .iter()
        .find(|a| a.name.ends_with("-setup.exe") || a.name.ends_with(".exe"))
        .map(|a| a.browser_download_url.clone());

    Ok(UpdateCheckResult {
        has_update,
        current_version: current,
        latest_version: latest,
        published_at: release.published_at,
        release_notes: release.body,
        download_url,
        html_url: release.html_url,
    })
}

/// Get the current app version.
#[tauri::command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Compare semantic versions: returns true if `latest` > `current`.
/// Supports versions like "0.1.0", "1.2.3-beta", etc.
fn is_newer_version(current: &str, latest: &str) -> bool {
    let parse_parts = |v: &str| -> Vec<u32> {
        v.split('-')
            .next()
            .unwrap_or("0")
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect()
    };

    let cur_parts = parse_parts(current);
    let new_parts = parse_parts(latest);

    for i in 0..new_parts.len().max(cur_parts.len()) {
        let c = cur_parts.get(i).unwrap_or(&0);
        let n = new_parts.get(i).unwrap_or(&0);
        if n > c {
            return true;
        }
        if n < c {
            return false;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_newer_version() {
        assert!(is_newer_version("0.1.0", "0.2.0"));
        assert!(is_newer_version("0.1.0", "1.0.0"));
        assert!(is_newer_version("0.1.0", "0.1.1"));
        assert!(!is_newer_version("0.2.0", "0.1.0"));
        assert!(!is_newer_version("0.1.0", "0.1.0"));
        assert!(is_newer_version("0.1.0", "0.1.0-beta.2") == false); // pre-release treated same
    }
}
