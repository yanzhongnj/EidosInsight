//! Export command handlers

use crate::db::{DbState, PaperRepo};
use crate::errors::Result;
use crate::models::PaperFilter;
use crate::services::AppState;
use tauri::State;

#[tauri::command]
pub fn export_papers(
    format: String,
    filter: Option<PaperFilter>,
    db: State<'_, DbState>,
    app_state: State<'_, AppState>,
) -> Result<String> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    let filter = filter.unwrap_or_default();
    let result = PaperRepo::list(db.conn(), &filter)?;

    let content = match format.as_str() {
        "bibtex" => export_bibtex(&result.papers),
        "csv" => export_csv(&result.papers),
        "markdown" => export_markdown(&result.papers),
        "apa" => export_apa(&result.papers),
        _ => return Err(crate::errors::AppError::InvalidInput(format!("Unsupported export format: {}", format))),
    };

    // Save to file
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let filename = format!("{}_{}.{}", "export", timestamp, format);
    let filepath = app_state.exports_dir.join(&filename);
    std::fs::write(&filepath, &content)?;

    tracing::info!(format = %format, count = result.papers.len(), path = %filepath.display(), "Papers exported");
    Ok(filepath.to_string_lossy().to_string())
}

fn export_bibtex(papers: &[crate::models::Paper]) -> String {
    papers.iter().map(|p| {
        let key = p.doi.as_deref().unwrap_or(&p.id);
        format!(
            "@article{{{key},\n  title = {{{}}},\n  author = {{{}}},\n  journal = {{{}}},\n  year = {{{}}},\n  doi = {{{}}}\n}}",
            p.title,
            p.authors.as_deref().unwrap_or(""),
            p.journal.as_deref().unwrap_or(""),
            p.publication_date.as_deref().unwrap_or(""),
            p.doi.as_deref().unwrap_or(""),
        )
    }).collect::<Vec<_>>().join("\n\n")
}

fn export_csv(papers: &[crate::models::Paper]) -> String {
    let mut csv = String::from("Title,Authors,Journal,Year,DOI,Reading Status\n");
    for p in papers {
        csv.push_str(&format!(
            "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"\n",
            p.title.replace('"', "\"\""),
            p.authors.as_deref().unwrap_or("").replace('"', "\"\""),
            p.journal.as_deref().unwrap_or("").replace('"', "\"\""),
            p.publication_date.as_deref().unwrap_or(""),
            p.doi.as_deref().unwrap_or(""),
            p.reading_status,
        ));
    }
    csv
}

fn export_markdown(papers: &[crate::models::Paper]) -> String {
    let mut md = String::from("# Exported Papers\n\n");
    for p in papers {
        md.push_str(&format!("## {}\n", p.title));
        if let Some(ref authors) = p.authors { md.push_str(&format!("**Authors:** {}\n\n", authors)); }
        if let Some(ref journal) = p.journal { md.push_str(&format!("**Journal:** {}\n\n", journal)); }
        if let Some(ref doi) = p.doi { md.push_str(&format!("**DOI:** {}\n\n", doi)); }
        if let Some(ref abstract_text) = p.abstract_text { md.push_str(&format!("### Abstract\n{}\n\n", abstract_text)); }
        md.push_str("---\n\n");
    }
    md
}

fn export_apa(papers: &[crate::models::Paper]) -> String {
    papers.iter().map(|p| {
        let authors = p.authors.as_deref().unwrap_or("Unknown");
        let year = p.publication_date.as_deref().unwrap_or("n.d.");
        let title = &p.title;
        let journal = p.journal.as_deref().unwrap_or("");
        if journal.is_empty() {
            format!("{}, {} ({})", authors, title, year)
        } else {
            format!("{}, {} ({}, {})", authors, title, year, journal)
        }
    }).collect::<Vec<_>>().join("\n\n")
}
