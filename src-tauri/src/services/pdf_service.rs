//! PDF parsing and metadata extraction service

use lopdf::Document;
use std::path::Path;
use crate::errors::{AppError, Result};

/// Metadata extracted from a PDF file
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PdfMetadata {
    pub title: Option<String>,
    pub authors: Option<String>,
    pub abstract_text: Option<String>,
    pub journal: Option<String>,
    pub volume: Option<String>,
    pub issue: Option<String>,
    pub pages: Option<String>,
    pub doi: Option<String>,
    pub publication_date: Option<String>,
    pub page_count: u32,
}

/// Extract metadata from a PDF file at the given path
pub fn extract_metadata(pdf_path: &Path) -> Result<PdfMetadata> {
    let doc = Document::load(pdf_path).map_err(|e| {
        AppError::PdfParse(format!("Failed to load PDF '{}': {}", pdf_path.display(), e))
    })?;

    let page_count = doc.get_pages().len() as u32;

    // Extract metadata from the PDF Info dictionary via trailer
    let title = get_metadata_string(&doc, b"Title");
    let authors = get_metadata_string(&doc, b"Author");
    let _keywords = get_metadata_string(&doc, b"Keywords");
    let subject = get_metadata_string(&doc, b"Subject");
    let creation_date = get_metadata_string(&doc, b"CreationDate");

    // Try to extract DOI from metadata or first page text
    let doi = extract_doi(&doc);

    // Use Subject as abstract if available, otherwise try to extract from first page
    let abstract_text = if subject.is_some() {
        subject
    } else {
        extract_abstract_from_text(&doc)
    };

    // Pages range string
    let pages = if page_count > 0 {
        Some(format!("1-{}", page_count))
    } else {
        None
    };

    // Clean up title - if it's empty or looks like a filename, try first page text
    let title = match title {
        Some(ref t) if !t.trim().is_empty() && t.len() > 3 => Some(t.clone()),
        _ => {
            // Try to extract title from first page text
            extract_title_from_text(&doc)
        }
    };

    Ok(PdfMetadata {
        title,
        authors,
        abstract_text,
        journal: None,
        volume: None,
        issue: None,
        pages,
        doi,
        publication_date: creation_date,
        page_count,
    })
}

/// Extract first N pages of text content from PDF
pub fn extract_text(pdf_path: &Path, max_pages: Option<u32>) -> Result<String> {
    let doc = Document::load(pdf_path).map_err(|e| {
        AppError::PdfParse(format!("Failed to load PDF '{}': {}", pdf_path.display(), e))
    })?;

    let pages = doc.get_pages();
    let limit = max_pages.unwrap_or(u32::MAX);
    let mut text_parts: Vec<String> = Vec::new();

    for (i, (page_num, _)) in pages.iter().enumerate() {
        if i as u32 >= limit {
            break;
        }
        if let Ok(page_text) = doc.extract_text(&[*page_num]) {
            text_parts.push(page_text);
        }
    }

    Ok(text_parts.join("\n\n"))
}

/// Get page count of a PDF file
pub fn get_page_count(pdf_path: &Path) -> Result<u32> {
    let doc = Document::load(pdf_path).map_err(|e| {
        AppError::PdfParse(format!("Failed to load PDF '{}': {}", pdf_path.display(), e))
    })?;
    Ok(doc.get_pages().len() as u32)
}

// --- Internal helpers ---

/// Extract a metadata string from the PDF Info dictionary
fn get_metadata_string(doc: &Document, key: &'static [u8]) -> Option<String> {
    // Navigate the trailer -> Info reference to find the Info dictionary
    let info_ref = doc.trailer.get(b"Info").ok()?;

    if let lopdf::Object::Reference(info_id) = info_ref {
        if let Some(info_obj) = doc.objects.get(info_id) {
            if let lopdf::Object::Dictionary(dict) = info_obj {
                if let Some(value) = dict.get(key).ok() {
                    return object_to_string(value);
                }
            }
        }
    }

    None
}

/// Convert a PDF object to a String
fn object_to_string(obj: &lopdf::Object) -> Option<String> {
    match obj {
        lopdf::Object::String(bytes, _) => {
            // Try UTF-8 first, then lossy conversion
            match String::from_utf8(bytes.clone()) {
                Ok(s) => Some(s),
                Err(_) => Some(String::from_utf8_lossy(bytes).into_owned()),
            }
        }
        lopdf::Object::Name(bytes) => {
            String::from_utf8(bytes.clone()).ok()
        }
        _ => None,
    }
    .map(|s| s.trim().to_string())
    .filter(|s| !s.is_empty())
}

fn extract_doi(doc: &Document) -> Option<String> {
    // Try metadata first
    if let Some(doi_str) = get_metadata_string(doc, b"DOI") {
        return Some(doi_str);
    }

    // Try to find DOI pattern in first page text
    let pages: Vec<u32> = doc.get_pages().keys().take(1).cloned().collect();
    if pages.is_empty() {
        return None;
    }

    if let Ok(text) = doc.extract_text(&pages) {
        let doi_pattern = regex::Regex::new(r"10\.\d{4,9}/[^\s\]'']+").ok()?;
        if let Some(captures) = doi_pattern.find(&text) {
            let doi = captures.as_str().trim_end_matches('.').to_string();
            return Some(doi);
        }
    }

    None
}

fn extract_abstract_from_text(doc: &Document) -> Option<String> {
    // Try first 2 pages for abstract
    let pages: Vec<u32> = doc.get_pages().keys().take(2).cloned().collect();
    if pages.is_empty() {
        return None;
    }

    if let Ok(text) = doc.extract_text(&pages) {
        // Look for "Abstract" section - common patterns in academic papers
        if let Ok(pattern) = regex::Regex::new(
            r"(?is)abstract[:\s]*\n?(.*?)(?:\n\s*(?:introduction|keywords|1\.|1\s|introduction|1\s+introduction)|$)"
        ) {
            if let Some(captures) = pattern.captures(&text) {
                if let Some(abstract_match) = captures.get(1) {
                    let cleaned = abstract_match.as_str().trim().to_string();
                    if cleaned.len() > 30 && cleaned.len() < 5000 {
                        return Some(cleaned);
                    }
                }
            }
        }
    }

    None
}

fn extract_title_from_text(doc: &Document) -> Option<String> {
    // Try first page only
    let pages: Vec<u32> = doc.get_pages().keys().take(1).cloned().collect();
    if pages.is_empty() {
        return None;
    }

    if let Ok(text) = doc.extract_text(&pages) {
        // The title is typically in the first few lines
        let lines: Vec<&str> = text.lines()
            .filter(|l| !l.trim().is_empty())
            .collect();

        // Heuristic: title is usually the first non-empty line that's reasonably short
        for line in &lines {
            let trimmed = line.trim();
            if trimmed.len() > 10 && trimmed.len() < 300 {
                return Some(trimmed.to_string());
            }
        }

        // Fallback: use the first non-empty line
        if let Some(first) = lines.first() {
            let trimmed = first.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }

    None
}
