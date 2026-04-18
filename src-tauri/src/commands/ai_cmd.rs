//! AI Assistant command handlers
//!
//! Three core AI features, all streaming via Tauri events:
//!   - `ai_explain_term`   — explain a selected term in the context of a paper
//!   - `ai_summarize`      — generate a structured summary of a paper
//!   - `ai_extract_insights` — extract key insights / contributions / methodology
//!
//! Each command fires a unique task_id in the event payload so multiple
//! concurrent AI operations can coexist on the frontend:
//!
//!   llm-stream  → { task_id, content, done: false }
//!   llm-done    → { task_id }
//!   llm-error   → { task_id, message }

use crate::db::{DbState, PaperRepo};
use crate::errors::{AppError, Result};
use crate::services::LlmState;
use crate::services::llm::{Message, CompletionOptions};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

// ── Shared event payloads ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct AiStreamChunk {
    pub task_id: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AiDonePayload {
    pub task_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AiErrorPayload {
    pub task_id: String,
    pub message: String,
}

// ── Helper: build a unique task_id and start streaming ───────────────────────

/// Streams the LLM response, emitting `ai-stream`, `ai-done`, `ai-error`.
/// Returns the task_id so the caller can pass it to the frontend.
async fn stream_to_frontend(
    task_id: &str,
    messages: Vec<Message>,
    options: CompletionOptions,
    app: &AppHandle,
    router: &std::sync::Arc<crate::services::llm::LlmRouter>,
) -> Result<()> {
    let mut stream = router.complete_stream(messages, options).await?;

    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                if chunk.done {
                    app.emit("ai-done", AiDonePayload { task_id: task_id.to_string() }).ok();
                    break;
                }
                app.emit(
                    "ai-stream",
                    AiStreamChunk {
                        task_id: task_id.to_string(),
                        content: chunk.content,
                    },
                )
                .ok();
            }
            Err(e) => {
                app.emit(
                    "ai-error",
                    AiErrorPayload {
                        task_id: task_id.to_string(),
                        message: e.to_string(),
                    },
                )
                .ok();
                return Err(e);
            }
        }
    }

    Ok(())
}

// ── Command: ai_explain_term ──────────────────────────────────────────────────

/// Input for term explanation
#[derive(Debug, Deserialize)]
pub struct ExplainTermInput {
    /// The term the user selected / typed
    pub term: String,
    /// Optional paper ID — if provided, title + abstract are injected as context
    pub paper_id: Option<String>,
    /// Optional extra context text (e.g. the sentence containing the term)
    pub context_text: Option<String>,
    /// Response language: "zh" (default) or "en"
    pub lang: Option<String>,
}

/// Explain a scientific term, optionally grounded in a specific paper context.
/// Streams the response via `ai-stream` / `ai-done` / `ai-error` events.
/// Returns the task_id immediately so the frontend can track this request.
#[tauri::command]
pub async fn ai_explain_term(
    input: ExplainTermInput,
    app: AppHandle,
    db: State<'_, DbState>,
    llm: State<'_, LlmState>,
) -> Result<String> {
    let task_id = Uuid::new_v4().to_string();

    // Optionally fetch paper context
    let paper_context = if let Some(ref pid) = input.paper_id {
        let conn = db.0.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        match PaperRepo::get_by_id(conn.conn(), pid) {
            Ok(p) => {
                let mut ctx = format!("Paper title: {}", p.title);
                if let Some(abs) = p.abstract_text {
                    ctx.push_str(&format!("\n\nAbstract: {}", &abs[..abs.len().min(1200)]));
                }
                Some(ctx)
            }
            Err(_) => None,
        }
    } else {
        None
    };

    let lang_note = match input.lang.as_deref().unwrap_or("zh") {
        "en" => "Respond in English.",
        _ => "请用中文回答。",
    };

    let mut system_prompt = format!(
        "You are a research assistant specializing in academic papers. \
         Explain scientific terms clearly and concisely for a researcher. \
         {lang_note}"
    );

    if let Some(ref ctx) = paper_context {
        system_prompt.push_str(&format!(
            "\n\nYou are helping the researcher understand a term from this paper:\n{ctx}"
        ));
    }

    let user_content = {
        let mut parts = Vec::new();
        parts.push(format!("Please explain the term: **{}**", input.term));
        if let Some(ref ctx_text) = input.context_text {
            parts.push(format!(
                "\n\nThe term appears in this context:\n> {}",
                ctx_text
            ));
        }
        if paper_context.is_some() {
            parts.push(
                "\n\nInclude: (1) general definition, (2) how it's used in this paper's context, \
                 (3) related concepts."
                    .to_string(),
            );
        } else {
            parts.push(
                "\n\nInclude: (1) general definition, (2) usage in academic/scientific context, \
                 (3) related concepts."
                    .to_string(),
            );
        }
        parts.join("")
    };

    let messages = vec![
        Message::system(system_prompt),
        Message::user(user_content),
    ];

    let options = CompletionOptions {
        temperature: Some(0.3),
        max_tokens: Some(800),
        ..Default::default()
    };

    let task_id_clone = task_id.clone();
    let router = llm.0.clone();
    tauri::async_runtime::spawn(async move {
        let _ = stream_to_frontend(&task_id_clone, messages, options, &app, &router).await;
    });

    Ok(task_id)
}

// ── Command: ai_summarize ─────────────────────────────────────────────────────

/// Input for paper summarization
#[derive(Debug, Deserialize)]
pub struct SummarizeInput {
    /// Paper ID to summarize
    pub paper_id: String,
    /// Summary style: "brief" (3-5 sentences), "structured" (sections), "eli5"
    pub style: Option<String>,
    /// Response language
    pub lang: Option<String>,
}

/// Generate a summary of a paper. Streams via `ai-stream` / `ai-done` / `ai-error`.
/// Returns task_id immediately.
#[tauri::command]
pub async fn ai_summarize(
    input: SummarizeInput,
    app: AppHandle,
    db: State<'_, DbState>,
    llm: State<'_, LlmState>,
) -> Result<String> {
    let task_id = Uuid::new_v4().to_string();

    // Fetch paper
    let paper = {
        let conn = db.0.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        PaperRepo::get_by_id(conn.conn(), &input.paper_id)
            .map_err(|_| AppError::PaperNotFound { id: input.paper_id.clone() })?
    };

    let lang_note = match input.lang.as_deref().unwrap_or("zh") {
        "en" => "Respond in English.",
        _ => "请用中文回答。",
    };

    let style = input.style.as_deref().unwrap_or("structured");

    let style_instruction = match style {
        "brief" => "Provide a brief summary in 3-5 sentences covering the main objective, method, and key finding.",
        "eli5" => "Explain this paper as if the reader has no domain expertise. Use simple language and analogies.",
        _ => "Provide a structured summary with the following sections:\n\
              **研究问题** What problem does this paper address?\n\
              **核心方法** What approach/methodology is used?\n\
              **主要发现** What are the key results and findings?\n\
              **研究贡献** What does this contribute to the field?\n\
              **局限性** Any noted limitations or future work?",
    };

    let system_prompt = format!(
        "You are a research assistant. Summarize academic papers clearly and accurately. {lang_note}"
    );

    let mut paper_text = format!("Title: {}", paper.title);
    if let Some(ref authors) = paper.authors {
        paper_text.push_str(&format!("\nAuthors: {authors}"));
    }
    if let Some(ref journal) = paper.journal {
        paper_text.push_str(&format!("\nJournal: {journal}"));
    }
    if let Some(ref date) = paper.publication_date {
        paper_text.push_str(&format!("\nDate: {date}"));
    }
    if let Some(ref abs) = paper.abstract_text {
        paper_text.push_str(&format!("\n\nAbstract:\n{}", &abs[..abs.len().min(3000)]));
    }
    if let Some(ref tldr) = paper.tldr {
        paper_text.push_str(&format!("\n\nTL;DR: {tldr}"));
    }

    let user_content = format!(
        "Please summarize the following paper.\n\n{style_instruction}\n\n---\n{paper_text}"
    );

    let messages = vec![
        Message::system(system_prompt),
        Message::user(user_content),
    ];

    let options = CompletionOptions {
        temperature: Some(0.4),
        max_tokens: Some(1200),
        ..Default::default()
    };

    let task_id_clone = task_id.clone();
    let router = llm.0.clone();
    tauri::async_runtime::spawn(async move {
        let _ = stream_to_frontend(&task_id_clone, messages, options, &app, &router).await;
    });

    Ok(task_id)
}

// ── Command: ai_extract_insights ─────────────────────────────────────────────

/// Input for insight extraction
#[derive(Debug, Deserialize)]
pub struct ExtractInsightsInput {
    /// Paper ID
    pub paper_id: String,
    /// Focus area: "contributions", "methodology", "gaps", "all" (default)
    pub focus: Option<String>,
    /// Response language
    pub lang: Option<String>,
}

/// Extract structured insights from a paper.
/// Returns task_id immediately; streams content via events.
#[tauri::command]
pub async fn ai_extract_insights(
    input: ExtractInsightsInput,
    app: AppHandle,
    db: State<'_, DbState>,
    llm: State<'_, LlmState>,
) -> Result<String> {
    let task_id = Uuid::new_v4().to_string();

    // Fetch paper
    let paper = {
        let conn = db.0.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        PaperRepo::get_by_id(conn.conn(), &input.paper_id)
            .map_err(|_| AppError::PaperNotFound { id: input.paper_id.clone() })?
    };

    let lang_note = match input.lang.as_deref().unwrap_or("zh") {
        "en" => "Respond in English.",
        _ => "请用中文回答。",
    };

    let focus = input.focus.as_deref().unwrap_or("all");

    let focus_instruction = match focus {
        "contributions" => {
            "Extract and list the key **contributions** of this paper:\n\
             - Novel concepts, methods, or frameworks introduced\n\
             - Theoretical or empirical advances\n\
             - Artifacts produced (datasets, tools, models)"
        }
        "methodology" => {
            "Analyze the **methodology** in depth:\n\
             - Research design and approach\n\
             - Data sources and collection methods\n\
             - Analysis techniques and evaluation metrics\n\
             - Reproducibility considerations"
        }
        "gaps" => {
            "Identify **research gaps and future directions**:\n\
             - Limitations explicitly stated by authors\n\
             - Open questions or unsolved problems\n\
             - Potential follow-up research directions\n\
             - Connections to other research areas"
        }
        _ => {
            "Extract the following structured insights:\n\n\
             **核心贡献** (Core Contributions)\n\
             List 3-5 specific contributions this paper makes.\n\n\
             **方法亮点** (Methodology Highlights)\n\
             Key methodological innovations or notable design decisions.\n\n\
             **重要发现** (Key Findings)\n\
             The most important empirical or theoretical results.\n\n\
             **研究局限** (Limitations & Gaps)\n\
             What the paper does not address or could be improved.\n\n\
             **延伸阅读** (Related Directions)\n\
             2-3 related research threads worth exploring."
        }
    };

    let system_prompt = format!(
        "You are an expert research analyst. Extract precise, actionable insights from academic papers. \
         Be specific — cite specific numbers, techniques, or claims from the paper when possible. {lang_note}"
    );

    let mut paper_text = format!("Title: {}", paper.title);
    if let Some(ref authors) = paper.authors {
        paper_text.push_str(&format!("\nAuthors: {authors}"));
    }
    if let Some(ref journal) = paper.journal {
        paper_text.push_str(&format!("\nPublished in: {journal}"));
    }
    if let Some(ref date) = paper.publication_date {
        paper_text.push_str(&format!("\nDate: {date}"));
    }
    if let Some(ref abs) = paper.abstract_text {
        paper_text.push_str(&format!("\n\nAbstract:\n{}", &abs[..abs.len().min(3000)]));
    }
    if let Some(ref tldr) = paper.tldr {
        paper_text.push_str(&format!("\n\nTL;DR: {tldr}"));
    }

    let user_content = format!(
        "{focus_instruction}\n\n---\n{paper_text}"
    );

    let messages = vec![
        Message::system(system_prompt),
        Message::user(user_content),
    ];

    let options = CompletionOptions {
        temperature: Some(0.35),
        max_tokens: Some(1500),
        ..Default::default()
    };

    let task_id_clone = task_id.clone();
    let router = llm.0.clone();
    tauri::async_runtime::spawn(async move {
        let _ = stream_to_frontend(&task_id_clone, messages, options, &app, &router).await;
    });

    Ok(task_id)
}

// ── Command: ai_check_availability ────────────────────────────────────────────

/// Check if any LLM backend is currently available (fast, non-streaming)
#[tauri::command]
pub async fn ai_check_availability(llm: State<'_, LlmState>) -> Result<AiAvailability> {
    let router = &llm.0;
    let available = router.is_ollama_available().await || router.has_cloud_adapter().await;
    let provider = if router.is_ollama_available().await {
        "ollama".to_string()
    } else if router.has_cloud_adapter().await {
        "cloud".to_string()
    } else {
        "none".to_string()
    };
    Ok(AiAvailability { available, provider })
}

#[derive(Debug, Clone, Serialize)]
pub struct AiAvailability {
    pub available: bool,
    pub provider: String,
}
