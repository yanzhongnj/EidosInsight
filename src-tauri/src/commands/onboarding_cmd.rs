//! Onboarding command handlers

use crate::db::{DbState, PaperRepo, SettingsRepo};
use crate::errors::Result;
use crate::models::CreatePaperInput;
use tauri::State;

#[tauri::command]
pub fn is_first_launch(db: State<'_, DbState>) -> Result<bool> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    let value = SettingsRepo::get_value(db.conn(), "first_launch")?;
    Ok(value.unwrap_or_default() == "true")
}

#[tauri::command]
pub fn complete_onboarding(db: State<'_, DbState>) -> Result<()> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    SettingsRepo::set(db.conn(), "first_launch", "false")?;
    SettingsRepo::set(db.conn(), "onboarding_completed", "true")?;
    tracing::info!("Onboarding completed");
    Ok(())
}

#[tauri::command]
pub fn load_sample_data(db: State<'_, DbState>) -> Result<()> {
    let db = db.0.lock().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    let conn = db.conn();

    // Check if sample data already exists
    let count: i32 = conn.query_row("SELECT COUNT(*) FROM papers", [], |row| row.get(0)).unwrap_or(0);
    if count > 0 {
        tracing::info!("Sample data already exists, skipping");
        return Ok(());
    }

    let sample_papers = vec![
        CreatePaperInput {
            title: "Attention Is All You Need".to_string(),
            authors: Some("Vaswani, A., Shazeer, N., Parmar, N., et al.".to_string()),
            abstract_text: Some("The dominant sequence transduction models are based on complex recurrent or convolutional neural networks. We propose a new simple network architecture, the Transformer, based solely on attention mechanisms.".to_string()),
            publication_date: Some("2017".to_string()),
            journal: Some("Advances in Neural Information Processing Systems".to_string()),
            doi: None,
            arxiv_id: Some("1706.03762".to_string()),
            ..Default::default()
        },
        CreatePaperInput {
            title: "BERT: Pre-training of Deep Bidirectional Transformers for Language Understanding".to_string(),
            authors: Some("Devlin, J., Chang, M.-W., Lee, K., Toutanova, K.".to_string()),
            abstract_text: Some("We introduce a new language representation model called BERT, which stands for Bidirectional Encoder Representations from Transformers.".to_string()),
            publication_date: Some("2019".to_string()),
            journal: Some("Proceedings of NAACL-HLT".to_string()),
            doi: None,
            arxiv_id: Some("1810.04805".to_string()),
            ..Default::default()
        },
        CreatePaperInput {
            title: "GPT-4 Technical Report".to_string(),
            authors: Some("OpenAI".to_string()),
            abstract_text: Some("We report the development of GPT-4, a large-scale multimodal model capable of processing image and text inputs and producing text outputs.".to_string()),
            publication_date: Some("2023".to_string()),
            journal: Some("arXiv preprint".to_string()),
            doi: None,
            arxiv_id: Some("2303.08774".to_string()),
            ..Default::default()
        },
    ];

    for input in sample_papers {
        PaperRepo::insert(conn, &input)?;
    }

    tracing::info!("Sample data loaded successfully");
    Ok(())
}

impl Default for CreatePaperInput {
    fn default() -> Self {
        Self {
            title: String::new(),
            authors: None,
            abstract_text: None,
            publication_date: None,
            journal: None,
            volume: None,
            issue: None,
            pages: None,
            doi: None,
            arxiv_id: None,
            pdf_url: None,
            pdf_path: None,
            file_type: None,
            folder_id: None,
        }
    }
}
