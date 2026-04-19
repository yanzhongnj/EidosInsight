//! Multi-format document parsing service
//!
//! Provides a unified `DocumentParser` trait and implementations for each
//! supported file type: PDF, TXT, DOCX, XLSX, CSV, HTML.

use std::path::Path;

use crate::errors::{AppError, Result};
use crate::models::FileType;
use calamine::Reader;

/// Metadata and content extracted from a document file
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DocumentContent {
    pub title: Option<String>,
    pub authors: Option<String>,
    pub abstract_text: Option<String>,
    pub journal: Option<String>,
    pub publication_date: Option<String>,
    pub pages: Option<String>,
    pub doi: Option<String>,
    /// Full text content of the document (used for FTS indexing)
    pub text_content: Option<String>,
    /// Page count (PDF only; other formats set to 0)
    pub page_count: u32,
}

/// Parse a document file and extract metadata + content
pub fn parse_document(file_path: &Path, file_type: &FileType) -> Result<DocumentContent> {
    match file_type {
        FileType::Pdf => parse_pdf(file_path),
        FileType::Txt => parse_txt(file_path),
        FileType::Docx => parse_docx(file_path),
        FileType::Xlsx => parse_xlsx(file_path),
        FileType::Csv => parse_csv(file_path),
        FileType::Html => parse_html(file_path),
    }
}

/// Determine the title from the file name (fallback for formats without metadata)
fn title_from_filename(path: &Path) -> Option<String> {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
}

// ── PDF ──────────────────────────────────────────────────────────────────────

fn parse_pdf(path: &Path) -> Result<DocumentContent> {
    let metadata = crate::services::pdf_service::extract_metadata(path).unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to extract PDF metadata, using defaults");
        crate::services::pdf_service::PdfMetadata {
            title: title_from_filename(path),
            authors: None,
            abstract_text: None,
            journal: None,
            volume: None,
            issue: None,
            pages: None,
            doi: None,
            publication_date: None,
            page_count: 0,
        }
    });

    // Extract text content for full-text search
    let text_content = crate::services::pdf_service::extract_text(path, None)
        .ok()
        .filter(|t| !t.trim().is_empty());

    Ok(DocumentContent {
        title: metadata.title,
        authors: metadata.authors,
        abstract_text: metadata.abstract_text,
        journal: metadata.journal,
        publication_date: metadata.publication_date,
        pages: metadata.pages,
        doi: metadata.doi,
        text_content,
        page_count: metadata.page_count,
    })
}

// ── TXT ──────────────────────────────────────────────────────────────────────

fn parse_txt(path: &Path) -> Result<DocumentContent> {
    let bytes = std::fs::read(path).map_err(|e| {
        AppError::DocumentParse(format!("Failed to read TXT file '{}': {}", path.display(), e))
    })?;

    // Try UTF-8 first, then try CJK encodings, then fall back to lossy decoding
    let text = if let Ok(s) = String::from_utf8(bytes.clone()) {
        s
    } else if let Some(decoded) = try_decode_cjk_bytes(&bytes) {
        decoded
    } else {
        String::from_utf8_lossy(&bytes).into_owned()
    };

    let title = extract_title_from_text(&text)
        .or_else(|| title_from_filename(path));

    let abstract_text = extract_abstract_from_text(&text);

    Ok(DocumentContent {
        title,
        authors: None,
        abstract_text,
        journal: None,
        publication_date: None,
        pages: None,
        doi: None,
        text_content: Some(text),
        page_count: 0,
    })
}

// ── DOCX (Word) ──────────────────────────────────────────────────────────────
// DOCX is a ZIP archive containing XML files. We parse it manually using
// the `regex` crate to extract text, avoiding complex docx-rs API issues.

fn parse_docx(path: &Path) -> Result<DocumentContent> {
    let bytes = std::fs::read(path).map_err(|e| {
        AppError::DocumentParse(format!("Failed to read DOCX file '{}': {}", path.display(), e))
    })?;

    let reader = std::io::Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(reader)
        .map_err(|e| AppError::DocumentParse(format!("Failed to open DOCX as ZIP '{}': {}", path.display(), e)))?;

    // Extract core properties (docProps/core.xml)
    let mut title = None;
    let mut authors = None;

    if let Ok(core_xml) = archive.by_name("docProps/core.xml") {
        let core_content = std::io::read_to_string(core_xml).unwrap_or_default();
        title = extract_xml_tag(&core_content, "dc:title");
        authors = extract_xml_tag(&core_content, "dc:creator");
    }

    // Extract text content (word/document.xml)
    let mut text_parts: Vec<String> = Vec::new();

    if let Ok(doc_xml) = archive.by_name("word/document.xml") {
        let doc_content = std::io::read_to_string(doc_xml).unwrap_or_default();

        // Extract all <w:t> text elements
        let text_re = regex::Regex::new(r"<w:t[^>]*>([^<]*)</w:t>").ok();
        if let Some(re) = text_re {
            let mut current_para = String::new();
            for cap in re.captures_iter(&doc_content) {
                if let Some(text) = cap.get(1) {
                    current_para.push_str(text.as_str());
                }
            }
            // Split by paragraph boundaries (</w:p> tags)
            let para_re = regex::Regex::new(r"</w:p>").ok();
            if let Some(p_re) = para_re {
                let segments: Vec<&str> = p_re.split(&doc_content).collect();
                for segment in &segments {
                    let mut para_text = String::new();
                    for cap in re.captures_iter(segment) {
                        if let Some(text) = cap.get(1) {
                            para_text.push_str(text.as_str());
                        }
                    }
                    if !para_text.trim().is_empty() {
                        text_parts.push(para_text);
                    }
                }
            } else {
                if !current_para.trim().is_empty() {
                    text_parts.push(current_para);
                }
            }
        }
    }

    let full_text = text_parts.join("\n");

    title = title.or_else(|| extract_title_from_text(&full_text))
        .or_else(|| title_from_filename(path));

    let abstract_text = extract_abstract_from_text(&full_text);

    Ok(DocumentContent {
        title,
        authors,
        abstract_text,
        journal: None,
        publication_date: None,
        pages: None,
        doi: None,
        text_content: if full_text.trim().is_empty() { None } else { Some(full_text) },
        page_count: 0,
    })
}

/// Extract the content of an XML tag (e.g., <dc:title>Title Here</dc:title>)
fn extract_xml_tag(xml: &str, tag: &str) -> Option<String> {
    let pattern = format!("<{}[^>]*>([^<]*)</{}>", tag, tag);
    regex::Regex::new(&pattern).ok()
        .and_then(|re| re.captures(xml))
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().trim().to_string())
        .filter(|s| !s.is_empty())
}

// ── XLSX (Excel) ─────────────────────────────────────────────────────────────

fn parse_xlsx(path: &Path) -> Result<DocumentContent> {
    let mut workbook: calamine::Sheets<_> = calamine::open_workbook(path).map_err(|e| {
        AppError::DocumentParse(format!("Failed to open XLSX '{}': {}", path.display(), e))
    })?;

    let mut all_text = String::new();
    let sheet_names = workbook.sheet_names().to_vec();

    for sheet_name in &sheet_names {
        if !all_text.is_empty() {
            all_text.push_str("\n\n");
        }
        all_text.push_str(&format!("[Sheet: {}]\n", sheet_name));

        if let Ok(range) = workbook.worksheet_range(sheet_name) {
            for row in range.rows() {
                let row_text: Vec<String> = row.iter()
                    .filter_map(|cell| {
                        let s = cell.to_string();
                        if s.trim().is_empty() { None } else { Some(s) }
                    })
                    .collect();
                if !row_text.is_empty() {
                    all_text.push_str(&row_text.join("\t"));
                    all_text.push('\n');
                }
            }
        }
    }

    let title = title_from_filename(path);

    Ok(DocumentContent {
        title,
        authors: None,
        abstract_text: None,
        journal: None,
        publication_date: None,
        pages: None,
        doi: None,
        text_content: if all_text.trim().is_empty() { None } else { Some(all_text) },
        page_count: 0,
    })
}

// ── CSV ──────────────────────────────────────────────────────────────────────

fn parse_csv(path: &Path) -> Result<DocumentContent> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        AppError::DocumentParse(format!("Failed to read CSV file '{}': {}", path.display(), e))
    })?;

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(content.as_bytes());

    // Extract headers as part of content
    let headers = reader.headers()
        .map(|h| h.iter().collect::<Vec<_>>().join("\t"))
        .unwrap_or_default();

    let mut rows: Vec<String> = Vec::new();
    if !headers.is_empty() {
        rows.push(format!("[Headers: {}]", headers));
    }

    for record in reader.records() {
        match record {
            Ok(r) => {
                let row_text: Vec<String> = r.iter().map(|s| s.to_string()).collect();
                rows.push(row_text.join("\t"));
            }
            Err(e) => {
                tracing::warn!(error = %e, "Skipping malformed CSV row");
            }
        }
    }

    let full_text = rows.join("\n");
    let title = title_from_filename(path);

    Ok(DocumentContent {
        title,
        authors: None,
        abstract_text: None,
        journal: None,
        publication_date: None,
        pages: None,
        doi: None,
        text_content: if full_text.trim().is_empty() { None } else { Some(full_text) },
        page_count: 0,
    })
}

// ── HTML ─────────────────────────────────────────────────────────────────────

fn parse_html(path: &Path) -> Result<DocumentContent> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        AppError::DocumentParse(format!("Failed to read HTML file '{}': {}", path.display(), e))
    })?;

    let document = scraper::Html::parse_document(&content);

    // Extract <title>
    let title_selector = scraper::Selector::parse("title")
        .map_err(|_| AppError::DocumentParse("Failed to parse HTML selector".to_string()))?;
    let title = document.select(&title_selector)
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .filter(|t| !t.is_empty())
        .or_else(|| title_from_filename(path));

    // Extract meta description as abstract
    let meta_selector = scraper::Selector::parse("meta[name='description']")
        .map_err(|_| AppError::DocumentParse("Failed to parse HTML selector".to_string()))?;
    let abstract_text = document.select(&meta_selector)
        .next()
        .and_then(|el| el.value().attr("content"))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    // Extract meta author
    let author_selector = scraper::Selector::parse("meta[name='author']")
        .map_err(|_| AppError::DocumentParse("Failed to parse HTML selector".to_string()))?;
    let authors = document.select(&author_selector)
        .next()
        .and_then(|el| el.value().attr("content"))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    // Extract body text (strip tags)
    let body_selector = scraper::Selector::parse("body")
        .map_err(|_| AppError::DocumentParse("Failed to parse HTML selector".to_string()))?;
    let body_text = document.select(&body_selector)
        .next()
        .map(|el| {
            el.text()
                .collect::<Vec<_>>()
                .join(" ")
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
        })
        .filter(|t| !t.trim().is_empty());

    Ok(DocumentContent {
        title,
        authors,
        abstract_text,
        journal: None,
        publication_date: None,
        pages: None,
        doi: None,
        text_content: body_text,
        page_count: 0,
    })
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Try to decode bytes as CJK encoding (GBK, Big5, Shift_JIS, EUC-KR).
/// Reuses the same heuristic as pdf_service::try_decode_cjk.
fn try_decode_cjk_bytes(bytes: &[u8]) -> Option<String> {
    if bytes.len() < 2 {
        return None;
    }

    let encodings = [
        encoding_rs::GBK,
        encoding_rs::BIG5,
        encoding_rs::SHIFT_JIS,
        encoding_rs::EUC_KR,
    ];

    for encoding in encodings {
        let (decoded, _encoding_used, had_errors) = encoding.decode(bytes);
        if had_errors {
            continue;
        }
        let text = decoded.into_owned();
        let cjk_count = text.chars().filter(|c| {
            ('\u{4E00}'..='\u{9FFF}').contains(c)
                || ('\u{F900}'..='\u{FAFF}').contains(c)
                || ('\u{3400}'..='\u{4DBF}').contains(c)
        }).count();
        let replacement_count = text.chars().filter(|c| *c == '\u{FFFD}').count();
        if cjk_count > 0 && replacement_count == 0 {
            return Some(text);
        }
    }

    None
}

/// Try to extract a title from the first non-empty line of text
fn extract_title_from_text(text: &str) -> Option<String> {
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .next()
        .map(|l| {
            let trimmed = l.trim();
            // Remove common heading markers like "# "
            trimmed.trim_start_matches('#').trim().to_string()
        })
        .filter(|t| t.len() >= 3 && t.len() <= 300)
}

/// Try to extract an abstract from text (first ~500 chars after title)
fn extract_abstract_from_text(text: &str) -> Option<String> {
    let lines: Vec<&str> = text.lines()
        .filter(|l| !l.trim().is_empty())
        .collect();

    if lines.len() < 2 {
        return None;
    }

    // Take the first few paragraphs after the title, up to ~500 chars
    let abstract_parts: Vec<&str> = lines[1..]
        .iter()
        .take(5)
        .take_while(|l| l.len() < 1000) // Skip very long lines (probably not abstract)
        .cloned()
        .collect();

    let abstract_text = abstract_parts.join(" ");
    if abstract_text.len() > 30 && abstract_text.len() < 5000 {
        Some(abstract_text)
    } else {
        None
    }
}
