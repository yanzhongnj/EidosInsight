//! AI result command handlers

use crate::db::{AiResultRepo, CollectionRepo, DbState};
use crate::errors::{AppError, Result};
use crate::models::*;
use crate::services::LlmState;
use tauri::State;

const REVIEW_SYSTEM_PROMPT: &str = r#"你是一位专业的学术文献综述撰写专家。请基于提供的论文集合，撰写一篇结构化的文献综述。

要求：
1. 按研究主题/方法分类组织
2. 每个分类下列出代表性论文及其核心贡献
3. 指出研究趋势和空白
4. 使用 Markdown 格式输出
5. 引用论文时使用 [作者, 年份] 格式
6. 包含：引言、主体（按主题分类）、研究空白与展望、结论"#;

const COMPARISON_SYSTEM_PROMPT: &str = r#"你是一位专业的学术论文对比分析专家。请基于提供的论文集合，从指定维度进行对比分析。

要求：
1. 对每个维度，逐一分析各论文的表现
2. 使用表格形式展示对比结果
3. 指出各论文的独特优势和不足
4. 使用 Markdown 格式输出
5. 在每个维度分析末尾给出简要总结"#;

const BASELINE_SYSTEM_PROMPT: &str = r#"你是一位专业的学术基线表格构建专家。请基于提供的论文集合，提取关键实验数据和指标，构建结构化的基线对比表。

要求：
1. 提取每篇论文的关键实验指标
2. 使用 Markdown 表格展示
3. 标注数据集、指标名称、数值
4. 高亮最优结果
5. 输出纯 JSON 格式的结构化数据（用于前端渲染）"#;

#[tauri::command]
pub fn list_ai_results(result_type: Option<String>, db: State<'_, DbState>) -> Result<Vec<AiResult>> {
    let db = db.0.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    match result_type {
        Some(rt) => AiResultRepo::list_by_type(db.conn(), &rt),
        None => AiResultRepo::list_all(db.conn()),
    }
}

#[tauri::command]
pub fn list_collection_ai_results(collection_id: String, db: State<'_, DbState>) -> Result<Vec<AiResult>> {
    let db = db.0.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    AiResultRepo::list_by_collection(db.conn(), &collection_id)
}

#[tauri::command]
pub fn get_ai_result(id: String, db: State<'_, DbState>) -> Result<AiResult> {
    let db = db.0.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    AiResultRepo::get_by_id(db.conn(), &id)
}

#[tauri::command]
pub fn delete_ai_result(id: String, db: State<'_, DbState>) -> Result<()> {
    let db = db.0.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    AiResultRepo::delete(db.conn(), &id)
}

#[tauri::command]
pub async fn generate_review(
    input: GenerateReviewInput,
    db: State<'_, DbState>,
    llm: State<'_, LlmState>,
) -> Result<AiResult> {
    // Get collection papers
    let papers = {
        let db_lock = db.0.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        CollectionRepo::list_papers(db_lock.conn(), &input.collection_id)?
    };

    if papers.is_empty() {
        return Err(AppError::InvalidInput("Collection has no papers to review".to_string()));
    }

    let collection = {
        let db_lock = db.0.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        CollectionRepo::get_by_id(db_lock.conn(), &input.collection_id)?
    };

    // Build prompt with paper info
    let papers_text = papers.iter().enumerate().map(|(i, p)| {
        format!(
            "[{}] {}\n作者: {}\n摘要: {}\n发表: {}",
            i + 1,
            p.title,
            p.authors.as_deref().unwrap_or("未知"),
            p.abstract_text.as_deref().unwrap_or("无摘要"),
            p.publication_date.as_deref().unwrap_or("未知")
        )
    }).collect::<Vec<_>>().join("\n\n---\n\n");

    let focus_text = input.focus.as_deref().unwrap_or("综合综述");
    let user_prompt = format!(
        "请基于以下{}篇论文，撰写一篇关于「{}」的文献综述：\n\n{}",
        papers.len(),
        focus_text,
        papers_text
    );

    // Call LLM
    let messages = vec![
        crate::services::llm::adapter::Message::system(REVIEW_SYSTEM_PROMPT),
        crate::services::llm::adapter::Message::user(user_prompt),
    ];

    let content = llm.0.complete(messages, crate::services::llm::adapter::CompletionOptions::default()).await
        .map_err(|e| AppError::Llm(e.to_string()))?;

    let paper_ids = papers.iter().map(|p| p.id.clone()).collect::<Vec<_>>();
    let now = chrono::Utc::now().naive_utc().format("%Y-%m-%d %H:%M:%S").to_string();

    let result = AiResult {
        id: uuid::Uuid::new_v4().to_string(),
        result_type: "review".to_string(),
        collection_id: Some(input.collection_id),
        title: format!("文献综述: {}", collection.name),
        content,
        structured_data: None,
        model_used: None,
        paper_ids: serde_json::to_string(&paper_ids).unwrap_or_default(),
        dimensions: None,
        created_at: now,
    };

    let db_lock = db.0.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    AiResultRepo::insert(db_lock.conn(), &result)
}

#[tauri::command]
pub async fn generate_comparison(
    input: GenerateComparisonInput,
    db: State<'_, DbState>,
    llm: State<'_, LlmState>,
) -> Result<AiResult> {
    let default_dimensions = vec![
        "方法".to_string(), "数据集".to_string(), "实验设计".to_string(),
        "评估指标".to_string(), "局限性".to_string(), "创新点".to_string(), "可复现性".to_string(),
    ];
    let dimensions = input.dimensions.as_ref().unwrap_or(&default_dimensions);

    let papers = {
        let db_lock = db.0.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        CollectionRepo::list_papers(db_lock.conn(), &input.collection_id)?
    };

    if papers.is_empty() {
        return Err(AppError::InvalidInput("Collection has no papers to compare".to_string()));
    }

    let collection = {
        let db_lock = db.0.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        CollectionRepo::get_by_id(db_lock.conn(), &input.collection_id)?
    };

    let papers_text = papers.iter().enumerate().map(|(i, p)| {
        format!(
            "[{}] {}\n作者: {}\n摘要: {}\n方法: {}\nTL;DR: {}",
            i + 1,
            p.title,
            p.authors.as_deref().unwrap_or("未知"),
            p.abstract_text.as_deref().unwrap_or("无摘要"),
            "见摘要",
            p.tldr.as_deref().unwrap_or("无")
        )
    }).collect::<Vec<_>>().join("\n\n---\n\n");

    let dims_str = dimensions.join("、");
    let user_prompt = format!(
        "请从以下维度对{}篇论文进行对比分析：{}\n\n{}",
        papers.len(),
        dims_str,
        papers_text
    );

    let messages = vec![
        crate::services::llm::adapter::Message::system(COMPARISON_SYSTEM_PROMPT),
        crate::services::llm::adapter::Message::user(user_prompt),
    ];

    let content = llm.0.complete(messages, crate::services::llm::adapter::CompletionOptions::default()).await
        .map_err(|e| AppError::Llm(e.to_string()))?;

    let paper_ids = papers.iter().map(|p| p.id.clone()).collect::<Vec<_>>();
    let now = chrono::Utc::now().naive_utc().format("%Y-%m-%d %H:%M:%S").to_string();

    let result = AiResult {
        id: uuid::Uuid::new_v4().to_string(),
        result_type: "comparison".to_string(),
        collection_id: Some(input.collection_id),
        title: format!("对比研究: {}", collection.name),
        content,
        structured_data: None,
        model_used: None,
        paper_ids: serde_json::to_string(&paper_ids).unwrap_or_default(),
        dimensions: Some(serde_json::to_string(dimensions).unwrap_or_default()),
        created_at: now,
    };

    let db_lock = db.0.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    AiResultRepo::insert(db_lock.conn(), &result)
}

#[tauri::command]
pub async fn generate_baseline(
    input: GenerateBaselineInput,
    db: State<'_, DbState>,
    llm: State<'_, LlmState>,
) -> Result<AiResult> {
    let papers = {
        let db_lock = db.0.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        CollectionRepo::list_papers(db_lock.conn(), &input.collection_id)?
    };

    if papers.is_empty() {
        return Err(AppError::InvalidInput("Collection has no papers for baseline".to_string()));
    }

    let collection = {
        let db_lock = db.0.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        CollectionRepo::get_by_id(db_lock.conn(), &input.collection_id)?
    };

    let papers_text = papers.iter().enumerate().map(|(i, p)| {
        format!(
            "[{}] {}\n作者: {}\n摘要: {}",
            i + 1,
            p.title,
            p.authors.as_deref().unwrap_or("未知"),
            p.abstract_text.as_deref().unwrap_or("无摘要")
        )
    }).collect::<Vec<_>>().join("\n\n---\n\n");

    let metric_text = input.metric.as_deref().unwrap_or("所有关键指标");
    let user_prompt = format!(
        "请从以下{}篇论文中提取基线数据，对比维度: {}\n\n{}\n\n请同时输出Markdown表格和JSON结构化数据。",
        papers.len(),
        metric_text,
        papers_text
    );

    let messages = vec![
        crate::services::llm::adapter::Message::system(BASELINE_SYSTEM_PROMPT),
        crate::services::llm::adapter::Message::user(user_prompt),
    ];

    let content = llm.0.complete(messages, crate::services::llm::adapter::CompletionOptions::default()).await
        .map_err(|e| AppError::Llm(e.to_string()))?;

    // Try to extract JSON from content
    let structured_data = extract_json_from_content(&content);

    let paper_ids = papers.iter().map(|p| p.id.clone()).collect::<Vec<_>>();
    let now = chrono::Utc::now().naive_utc().format("%Y-%m-%d %H:%M:%S").to_string();

    let result = AiResult {
        id: uuid::Uuid::new_v4().to_string(),
        result_type: "baseline".to_string(),
        collection_id: Some(input.collection_id),
        title: format!("基线对比: {}", collection.name),
        content,
        structured_data,
        model_used: None,
        paper_ids: serde_json::to_string(&paper_ids).unwrap_or_default(),
        dimensions: input.metric.clone().map(|m| serde_json::to_string(&[m]).unwrap_or_default()),
        created_at: now,
    };

    let db_lock = db.0.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    AiResultRepo::insert(db_lock.conn(), &result)
}

/// Extract JSON block from LLM content (between ```json ... ```)
fn extract_json_from_content(content: &str) -> Option<String> {
    if let Some(start) = content.find("```json") {
        let json_start = start + 7;
        if let Some(end) = content[json_start..].find("```") {
            return Some(content[json_start..json_start + end].trim().to_string());
        }
    }
    // Try to find raw JSON array/object
    if let Some(start) = content.find('[') {
        if let Some(end) = content.rfind(']') {
            if end > start {
                return Some(content[start..=end].to_string());
            }
        }
    }
    if let Some(start) = content.find('{') {
        if let Some(end) = content.rfind('}') {
            if end > start {
                return Some(content[start..=end].to_string());
            }
        }
    }
    None
}
