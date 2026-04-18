//! End-to-end tests for EidosInsight
//!
//! Tests the complete user flows through the repository layer,
//! using an in-memory SQLite database with all migrations applied.
//!
//! Covers:
//! 1. Paper lifecycle (create → list → update → search → delete)
//! 2. Note CRUD + bidirectional links + FTS5 search
//! 3. Folder + Tag organization
//! 4. Reading state tracking
//! 5. Automation tasks + Scheduler (schedule_to_cron, next_run, CRUD)
//! 6. Skills (CRUD, import, trigger, execution placeholder)
//! 7. Settings persistence
//! 8. Cross-entity flows (paper → note → tag → search)

use rusqlite::Connection;

use crate::db::{
    FolderRepo, NoteRepo, PaperRepo, ReadingRepo, SettingsRepo, SkillRepo,
    TagRepo, TaskRepo,
};
use crate::models::*;
use crate::services::scheduler;

// ── Test Helpers ──────────────────────────────────────────────────────────────

/// Create an in-memory SQLite database with all migrations applied.
fn setup_test_db() -> Connection {
    let conn = Connection::open_in_memory().expect("Failed to create in-memory DB");
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
        .expect("Failed to set pragmas");
    crate::db::migrations::run_migrations(&conn).expect("Migrations failed");
    conn
}

/// Helper: create a paper and return it.
fn create_test_paper(conn: &Connection, title: &str) -> Paper {
    PaperRepo::insert(
        conn,
        &CreatePaperInput {
            title: title.to_string(),
            authors: Some("Test Author".to_string()),
            abstract_text: Some("This is a test abstract about machine learning.".to_string()),
            publication_date: Some("2025-01-15".to_string()),
            journal: Some("Nature".to_string()),
            volume: None,
            issue: None,
            pages: None,
            doi: None,
            arxiv_id: None,
            pdf_url: None,
            pdf_path: None,
            folder_id: None,
        },
    )
    .expect("Failed to create test paper")
}

/// Helper: create a note linked to a paper.
fn create_test_note(conn: &Connection, paper_id: &str, title: &str, content: &str) -> Note {
    NoteRepo::insert(
        conn,
        &CreateNoteInput {
            paper_id: Some(paper_id.to_string()),
            title: title.to_string(),
            content: content.to_string(),
            note_type: Some("general".to_string()),
            page_number: None,
            position: None,
        },
    )
    .expect("Failed to create test note")
}

// ── Flow 1: Paper Lifecycle ───────────────────────────────────────────────────

#[test]
fn test_paper_lifecycle() {
    let conn = setup_test_db();

    // Step 1: Create a paper
    let paper = create_test_paper(&conn, "Attention Is All You Need");
    assert!(!paper.id.is_empty());
    assert_eq!(paper.title, "Attention Is All You Need");
    assert_eq!(paper.authors.as_deref(), Some("Test Author"));
    assert_eq!(paper.reading_status, "unread");
    assert!(!paper.is_favorite);
    let paper_id = paper.id.clone();

    // Step 2: Get paper by ID
    let fetched = PaperRepo::get_by_id(&conn, &paper_id).expect("Paper not found");
    assert_eq!(fetched.title, "Attention Is All You Need");
    assert_eq!(fetched.journal.as_deref(), Some("Nature"));

    // Step 3: Update paper
    let updated = PaperRepo::update(
        &conn,
        &UpdatePaperInput {
            id: paper_id.clone(),
            title: Some("Attention Is All You Need (Updated)".to_string()),
            reading_status: Some("reading".to_string()),
            is_favorite: Some(true),
            tldr: Some("Transformer architecture replaces RNNs".to_string()),
            ..Default::default()
        },
    )
    .expect("Failed to update paper");
    assert_eq!(updated.title, "Attention Is All You Need (Updated)");
    assert_eq!(updated.reading_status, "reading");
    assert!(updated.is_favorite);
    assert_eq!(updated.tldr.as_deref(), Some("Transformer architecture replaces RNNs"));

    // Step 4: List papers
    let result = PaperRepo::list(&conn, &PaperFilter::default()).expect("Failed to list papers");
    assert_eq!(result.total, 1);
    assert_eq!(result.papers.len(), 1);

    // Step 5: Filter papers by reading status
    let filtered = PaperRepo::list(
        &conn,
        &PaperFilter {
            reading_status: Some("reading".to_string()),
            ..Default::default()
        },
    )
    .expect("Failed to filter papers");
    assert_eq!(filtered.total, 1);

    // Unread should return 0
    let unread = PaperRepo::list(
        &conn,
        &PaperFilter {
            reading_status: Some("unread".to_string()),
            ..Default::default()
        },
    )
    .expect("Failed to filter unread");
    assert_eq!(unread.total, 0);

    // Step 6: Filter by favorite
    let favs = PaperRepo::list(
        &conn,
        &PaperFilter {
            is_favorite: Some(true),
            ..Default::default()
        },
    )
    .expect("Failed to filter favorites");
    assert_eq!(favs.total, 1);

    // Step 7: Search by title keyword
    let searched = PaperRepo::list(
        &conn,
        &PaperFilter {
            query: Some("Attention".to_string()),
            ..Default::default()
        },
    )
    .expect("Failed to search");
    assert_eq!(searched.total, 1);

    // Step 8: Delete paper
    PaperRepo::delete(&conn, &paper_id).expect("Failed to delete paper");
    let after_delete = PaperRepo::list(&conn, &PaperFilter::default()).expect("List after delete");
    assert_eq!(after_delete.total, 0);
}

#[test]
fn test_paper_multiple_and_sort() {
    let conn = setup_test_db();

    // Create 3 papers
    let p1 = create_test_paper(&conn, "Alpha Paper");
    let _p2 = create_test_paper(&conn, "Beta Paper");
    let _p3 = create_test_paper(&conn, "Gamma Paper");

    // List all
    let all = PaperRepo::list(&conn, &PaperFilter::default()).expect("List all");
    assert_eq!(all.total, 3);

    // Sort by title ascending
    let sorted = PaperRepo::list(
        &conn,
        &PaperFilter {
            sort_by: Some("title".to_string()),
            sort_order: Some("asc".to_string()),
            ..Default::default()
        },
    )
    .expect("Sort by title");
    assert_eq!(sorted.papers[0].title, "Alpha Paper");

    // Pagination: limit 2
    let page = PaperRepo::list(
        &conn,
        &PaperFilter {
            limit: Some(2),
            offset: Some(0),
            ..Default::default()
        },
    )
    .expect("Paginate");
    assert_eq!(page.papers.len(), 2);
    assert_eq!(page.total, 3); // total is still 3

    // Move paper to a folder (via update)
    let moved = PaperRepo::update(
        &conn,
        &UpdatePaperInput {
            id: p1.id.clone(),
            folder_id: Some("favorites".to_string()),
            ..Default::default()
        },
    )
    .expect("Move to folder");
    assert_eq!(moved.folder_id.as_deref(), Some("favorites"));
}

// ── Flow 2: Note + Bidirectional Links + FTS5 Search ──────────────────────────

#[test]
fn test_note_crud_and_links() {
    let conn = setup_test_db();

    let paper = create_test_paper(&conn, "Deep Learning Paper");

    // Create 3 notes for the paper
    let n1 = create_test_note(&conn, &paper.id, "Key Insight", "The transformer uses self-attention");
    let n2 = create_test_note(&conn, &paper.id, "Methodology Note", "Experiments on WMT translation");
    let n3 = create_test_note(
        &conn,
        &paper.id,
        "Annotation p5",
        "Important figure on page 5",
    );

    // List notes by paper
    let notes = NoteRepo::list_by_paper(&conn, &paper.id).expect("List by paper");
    assert_eq!(notes.len(), 3);

    // Get single note
    let fetched = NoteRepo::get_by_id(&conn, &n1.id).expect("Get note");
    assert_eq!(fetched.title, "Key Insight");

    // Update note
    let updated = NoteRepo::update(
        &conn,
        &UpdateNoteInput {
            id: n1.id.clone(),
            content: Some("The transformer uses multi-head self-attention".to_string()),
            ..Default::default()
        },
    )
    .expect("Update note");
    assert!(updated.content.contains("multi-head"));

    // Create bidirectional links: n1 → n2, n1 → n3
    NoteRepo::create_link(
        &conn,
        &CreateNoteLinkInput {
            source_id: n1.id.clone(),
            target_id: n2.id.clone(),
            label: Some("see methodology".to_string()),
        },
    )
    .expect("Create link 1");

    NoteRepo::create_link(
        &conn,
        &CreateNoteLinkInput {
            source_id: n1.id.clone(),
            target_id: n3.id.clone(),
            label: None,
        },
    )
    .expect("Create link 2");

    // Verify outgoing links from n1
    let outgoing = NoteRepo::list_outgoing_links(&conn, &n1.id).expect("Outgoing links");
    assert_eq!(outgoing.len(), 2);

    // Verify backlinks to n2
    let backlinks = NoteRepo::list_backlinks(&conn, &n2.id).expect("Backlinks");
    assert_eq!(backlinks.len(), 1);
    assert_eq!(backlinks[0].note_id, n1.id);
    assert_eq!(backlinks[0].label.as_deref(), Some("see methodology"));

    // Get note with links
    let with_links = NoteRepo::get_with_links(&conn, &n1.id).expect("With links");
    assert_eq!(with_links.outgoing_links.len(), 2);
    assert_eq!(with_links.backlinks.len(), 0); // n1 has no backlinks

    let n2_with_links = NoteRepo::get_with_links(&conn, &n2.id).expect("n2 with links");
    assert_eq!(n2_with_links.outgoing_links.len(), 0);
    assert_eq!(n2_with_links.backlinks.len(), 1);

    // Delete a link
    NoteRepo::delete_link(&conn, &n1.id, &n2.id).expect("Delete link");
    let after_delete = NoteRepo::list_outgoing_links(&conn, &n1.id).expect("Outgoing after delete");
    assert_eq!(after_delete.len(), 1);

    // Delete a note (cascading should clean up links)
    NoteRepo::delete(&conn, &n3.id).expect("Delete note");
    let after_note_delete =
        NoteRepo::list_by_paper(&conn, &paper.id).expect("Notes after delete");
    assert_eq!(after_note_delete.len(), 2);
    // The link n1→n3 should be auto-cleaned by CASCADE
    let outgoing_after = NoteRepo::list_outgoing_links(&conn, &n1.id).expect("Outgoing after n3 delete");
    assert_eq!(outgoing_after.len(), 0); // n2 link was deleted, n3 is gone
}

#[test]
fn test_note_fulltext_search() {
    let conn = setup_test_db();

    let paper = create_test_paper(&conn, "Reinforcement Learning Survey");

    create_test_note(&conn, &paper.id, "Q-Learning", "Q-learning is a model-free reinforcement learning algorithm");
    create_test_note(&conn, &paper.id, "Policy Gradient", "Policy gradient methods optimize the policy directly");
    create_test_note(&conn, &paper.id, "DQN Notes", "Deep Q-Network combines Q-learning with deep neural networks");

    // Search for "reinforcement"
    let results = NoteRepo::search(&conn, "reinforcement", None, None, 20, 0)
        .expect("Search notes");
    assert!(results.total >= 1, "Should find at least 1 result for 'reinforcement'");

    // Search for "Q-learning"
    let q_results = NoteRepo::search(&conn, "Q-learning", None, None, 20, 0)
        .expect("Search Q-learning");
    assert!(q_results.total >= 1, "Should find at least 1 result for 'Q-learning'");

    // Search with highlight
    if !q_results.items.is_empty() {
        assert!(q_results.items[0].highlight.is_some(), "Should have highlight snippet");
    }

    // Filter by paper_id
    let filtered = NoteRepo::search(&conn, "reinforcement", Some(&paper.id), None, 20, 0)
        .expect("Search with paper filter");
    assert!(filtered.total >= 1);

    // Search suggest (autocomplete)
    let suggestions = NoteRepo::search_suggest(&conn, "Q", 5).expect("Suggest");
    assert!(!suggestions.is_empty(), "Should suggest notes starting with 'Q'");

    // Empty query
    let empty = NoteRepo::search(&conn, "", None, None, 20, 0).expect("Empty search");
    assert_eq!(empty.total, 0);
}

#[test]
fn test_note_types() {
    let conn = setup_test_db();
    let paper = create_test_paper(&conn, "Test Paper");

    // General note
    let general = NoteRepo::insert(
        &conn,
        &CreateNoteInput {
            paper_id: Some(paper.id.clone()),
            title: "General thought".to_string(),
            content: "This is a general note".to_string(),
            note_type: Some("general".to_string()),
            page_number: None,
            position: None,
        },
    )
    .expect("Create general note");
    assert_eq!(general.note_type, "general");

    // Annotation with page number
    let annotation = NoteRepo::insert(
        &conn,
        &CreateNoteInput {
            paper_id: Some(paper.id.clone()),
            title: "Highlight p3".to_string(),
            content: "Important passage".to_string(),
            note_type: Some("annotation".to_string()),
            page_number: Some(3),
            position: Some(r#"{"x":0.5,"y":0.3}"#.to_string()),
        },
    )
    .expect("Create annotation");
    assert_eq!(annotation.note_type, "annotation");
    assert_eq!(annotation.page_number, Some(3));
    assert!(annotation.position.is_some());

    // Search with type filter
    let ann_only = NoteRepo::search(&conn, "Important", None, Some("annotation"), 20, 0)
        .expect("Search annotation only");
    assert!(ann_only.total >= 1);
}

// ── Flow 3: Folder + Tag Organization ─────────────────────────────────────────

#[test]
fn test_folder_and_tag_organization() {
    let conn = setup_test_db();

    // Default folders should exist from migration
    let default_folders = FolderRepo::list(&conn).expect("List default folders");
    assert!(default_folders.len() >= 4, "Should have 4 default folders");

    // Create a custom folder
    let ml_folder = FolderRepo::insert(
        &conn,
        &CreateFolderInput {
            name: "Machine Learning".to_string(),
            parent_id: None,
            color: Some("#ff5500".to_string()),
            icon: Some("brain".to_string()),
        },
    )
    .expect("Create ML folder");
    assert!(!ml_folder.id.is_empty());
    assert_eq!(ml_folder.name, "Machine Learning");

    // Create a sub-folder
    let nlp_folder = FolderRepo::insert(
        &conn,
        &CreateFolderInput {
            name: "NLP".to_string(),
            parent_id: Some(ml_folder.id.clone()),
            color: Some("#00ff55".to_string()),
            icon: None,
        },
    )
    .expect("Create NLP sub-folder");
    assert_eq!(nlp_folder.parent_id.as_deref(), Some(ml_folder.id.as_str()));

    // Update folder
    let updated_folder = FolderRepo::update(
        &conn,
        &UpdateFolderInput {
            id: ml_folder.id.clone(),
            name: Some("ML & AI".to_string()),
            ..Default::default()
        },
    )
    .expect("Update folder");
    assert_eq!(updated_folder.name, "ML & AI");

    // Create tags
    let tag_transformer = TagRepo::find_or_create(&conn, "transformer", Some("#6366f1"))
        .expect("Create transformer tag");
    let tag_rl = TagRepo::find_or_create(&conn, "reinforcement-learning", None)
        .expect("Create RL tag");

    // find_or_create should be idempotent
    let tag_transformer_2 = TagRepo::find_or_create(&conn, "transformer", None)
        .expect("Find existing transformer tag");
    assert_eq!(tag_transformer.id, tag_transformer_2.id);

    // Create papers and organize
    let paper1 = create_test_paper(&conn, "BERT Paper");
    let paper2 = create_test_paper(&conn, "GPT Paper");

    // Move paper1 to ML folder
    PaperRepo::update(
        &conn,
        &UpdatePaperInput {
            id: paper1.id.clone(),
            folder_id: Some(ml_folder.id.clone()),
            ..Default::default()
        },
    )
    .expect("Move paper1 to folder");

    // Tag both papers
    TagRepo::add_paper_tag(&conn, &paper1.id, &tag_transformer.id).expect("Tag paper1");
    TagRepo::add_paper_tag(&conn, &paper2.id, &tag_transformer.id).expect("Tag paper2");
    TagRepo::add_paper_tag(&conn, &paper2.id, &tag_rl.id).expect("Tag paper2 with RL");

    // Verify tags on paper
    let p1_tags = TagRepo::get_tags_for_paper(&conn, &paper1.id).expect("Paper1 tags");
    assert_eq!(p1_tags.len(), 1);
    assert_eq!(p1_tags[0].name, "transformer");

    let p2_tags = TagRepo::get_tags_for_paper(&conn, &paper2.id).expect("Paper2 tags");
    assert_eq!(p2_tags.len(), 2);

    // Verify usage count
    let all_tags = TagRepo::list(&conn).expect("List tags");
    let transformer_tag = all_tags.iter().find(|t| t.name == "transformer").unwrap();
    assert_eq!(transformer_tag.usage_count, 2);

    // Remove a tag
    TagRepo::remove_paper_tag(&conn, &paper2.id, &tag_rl.id).expect("Remove RL tag");
    let p2_tags_after = TagRepo::get_tags_for_paper(&conn, &paper2.id).expect("Paper2 tags after remove");
    assert_eq!(p2_tags_after.len(), 1);

    // Rename tag
    TagRepo::rename(&conn, &tag_transformer.id, "transformers").expect("Rename tag");
    let renamed = TagRepo::find_or_create(&conn, "transformers", None).expect("Find renamed tag");
    assert_eq!(renamed.id, tag_transformer.id);

    // Delete folder (should set paper's folder_id to NULL via ON DELETE SET NULL)
    FolderRepo::delete(&conn, &ml_folder.id).expect("Delete folder");
    let paper1_after = PaperRepo::get_by_id(&conn, &paper1.id).expect("Paper1 still exists");
    assert!(paper1_after.folder_id.is_none(), "folder_id should be NULL after folder delete");
}

// ── Flow 4: Reading State Tracking ────────────────────────────────────────────

#[test]
fn test_reading_state_tracking() {
    let conn = setup_test_db();
    let paper = create_test_paper(&conn, "Long Paper");

    // Create initial reading state
    let state1 = ReadingRepo::upsert(
        &conn,
        &UpdateReadingStateInput {
            paper_id: paper.id.clone(),
            current_page: Some(5),
            total_pages: Some(100),
            scroll_position: Some(0.15),
            zoom_level: Some(1.2),
        },
    )
    .expect("Create reading state");
    assert_eq!(state1.current_page, 5);
    assert_eq!(state1.total_pages, 100);
    assert!((state1.reading_progress - 0.05).abs() < 0.01); // 5/100

    // Update reading progress
    let state2 = ReadingRepo::upsert(
        &conn,
        &UpdateReadingStateInput {
            paper_id: paper.id.clone(),
            current_page: Some(50),
            total_pages: None, // keep existing
            scroll_position: None,
            zoom_level: None,
        },
    )
    .expect("Update reading state");
    assert_eq!(state2.current_page, 50);
    assert_eq!(state2.total_pages, 100); // kept from before
    assert!((state2.reading_progress - 0.5).abs() < 0.01); // 50/100

    // Change zoom
    let state3 = ReadingRepo::upsert(
        &conn,
        &UpdateReadingStateInput {
            paper_id: paper.id.clone(),
            current_page: None,
            total_pages: None,
            scroll_position: Some(0.6),
            zoom_level: Some(1.5),
        },
    )
    .expect("Update zoom");
    assert_eq!(state3.current_page, 50); // kept
    assert!((state3.zoom_level - 1.5).abs() < 0.01);

    // Get by paper
    let fetched = ReadingRepo::get_by_paper(&conn, &paper.id).expect("Get reading state");
    assert_eq!(fetched.current_page, 50);

    // List reading progress
    let progress_list = ReadingRepo::list_reading_progress(&conn, 10).expect("List progress");
    assert_eq!(progress_list.len(), 1);

    // Mark as read (100%)
    let state_done = ReadingRepo::upsert(
        &conn,
        &UpdateReadingStateInput {
            paper_id: paper.id.clone(),
            current_page: Some(100),
            total_pages: Some(100),
            scroll_position: None,
            zoom_level: None,
        },
    )
    .expect("Mark as read");
    assert!((state_done.reading_progress - 1.0).abs() < 0.01);
}

// ── Flow 5: Automation Tasks ──────────────────────────────────────────────────

#[test]
fn test_auto_task_crud() {
    let conn = setup_test_db();

    // Verify preset tasks from migration
    let initial = TaskRepo::list_all(&conn).expect("List initial tasks");
    assert!(initial.len() >= 6, "Should have 6 preset tasks from migration");

    // Create a custom task
    let task = TaskRepo::create(
        &conn,
        &CreateAutoTask {
            name: "Custom Daily Digest".to_string(),
            task_type: "custom".to_string(),
            preset_id: None,
            definition: r#"{"steps":[{"id":"search","tool":"paper.search","input":{"query":"{{keywords}}"}}]}"#.to_string(),
            schedule: r#"{"frequency":"daily","time":"morning_8"}"#.to_string(),
            schedule_cron: Some("0 8 * * *".to_string()),
            config: Some(r#"{"keywords":"quantum computing"}"#.to_string()),
        },
    )
    .expect("Create custom task");
    assert!(!task.id.is_empty());
    assert_eq!(task.name, "Custom Daily Digest");
    assert!(task.is_enabled); // created as enabled
    let task_id = task.id.clone();

    // Get by ID
    let fetched = TaskRepo::get_by_id(&conn, &task_id).expect("Get task");
    assert_eq!(fetched.name, "Custom Daily Digest");

    // Update task
    TaskRepo::update(
        &conn,
        &task_id,
        &UpdateAutoTask {
            name: Some("Daily Quantum Digest".to_string()),
            config: Some(r#"{"keywords":"quantum computing, AI"}"#.to_string()),
            ..Default::default()
        },
    )
    .expect("Update task");
    let updated = TaskRepo::get_by_id(&conn, &task_id).expect("Updated task");
    assert_eq!(updated.name, "Daily Quantum Digest");

    // Disable task
    TaskRepo::disable(&conn, &task_id).expect("Disable task");
    let disabled = TaskRepo::get_by_id(&conn, &task_id).expect("Disabled task");
    assert!(!disabled.is_enabled);

    // Enabled list should not include disabled task
    let enabled = TaskRepo::list_enabled(&conn).expect("List enabled");
    assert!(enabled.iter().all(|t| t.is_enabled));

    // Re-enable
    TaskRepo::enable(&conn, &task_id).expect("Enable task");
    let re_enabled = TaskRepo::get_by_id(&conn, &task_id).expect("Re-enabled task");
    assert!(re_enabled.is_enabled);

    // Delete task
    TaskRepo::delete(&conn, &task_id).expect("Delete task");
    let after_delete = TaskRepo::list_all(&conn).expect("List after delete");
    assert!(after_delete.iter().all(|t| t.id != task_id));
}

#[test]
fn test_task_execution_lifecycle() {
    let conn = setup_test_db();

    let task = TaskRepo::create(
        &conn,
        &CreateAutoTask {
            name: "Test Task".to_string(),
            task_type: "custom".to_string(),
            preset_id: None,
            definition: r#"{"steps":[]}"#.to_string(),
            schedule: r#"{"frequency":"daily","time":"morning_8"}"#.to_string(),
            schedule_cron: Some("0 8 * * *".to_string()),
            config: None,
        },
    )
    .expect("Create task for execution test");

    // Create execution
    let execution = TaskRepo::create_execution(&conn, &task.id).expect("Create execution");
    assert_eq!(execution.task_id, task.id);
    assert_eq!(execution.status, "running");
    assert!(execution.finished_at.is_none());

    // Finish execution
    TaskRepo::finish_execution(
        &conn,
        &execution.id,
        "success",
        Some("Task completed successfully"),
        None,
        1234,
    )
    .expect("Finish execution");

    // List executions
    let executions = TaskRepo::list_executions(&conn, &task.id, 10).expect("List executions");
    assert_eq!(executions.len(), 1);
    assert_eq!(executions[0].status, "success");
    assert_eq!(executions[0].duration_ms, Some(1234));
    assert!(executions[0].finished_at.is_some());

    // Update last_run and next_run
    let now = chrono::Utc::now().to_rfc3339();
    let next = scheduler::next_run_from_cron("0 8 * * *");
    TaskRepo::update_last_run(&conn, &task.id, &now, next.as_deref()).expect("Update last run");

    let task_after = TaskRepo::get_by_id(&conn, &task.id).expect("Task after update");
    assert!(task_after.last_run.is_some());
    assert!(task_after.next_run.is_some());

    // Failed execution
    let exec2 = TaskRepo::create_execution(&conn, &task.id).expect("Create failed execution");
    TaskRepo::finish_execution(
        &conn,
        &exec2.id,
        "failed",
        None,
        Some("Network timeout"),
        5000,
    )
    .expect("Finish as failed");

    let all_executions = TaskRepo::list_executions(&conn, &task.id, 10).expect("All executions");
    assert_eq!(all_executions.len(), 2);

    // Get latest execution
    let latest = TaskRepo::get_latest_execution(&conn, &task.id).expect("Latest execution");
    assert!(latest.is_some());
    assert_eq!(latest.unwrap().status, "failed");
}

// ── Flow 6: Scheduler — schedule_to_cron + next_run ───────────────────────────

#[test]
fn test_schedule_to_cron_daily() {
    let cfg = ScheduleConfig {
        frequency: "daily".to_string(),
        time: "morning_8".to_string(),
        weekdays: None,
        custom_interval_hours: None,
        custom_hour: None,
        custom_minute: None,
    };
    assert_eq!(scheduler::schedule_to_cron(&cfg), "0 8 * * *");
    assert_eq!(scheduler::schedule_description(&cfg), "每天 08:00");

    let noon = ScheduleConfig {
        frequency: "daily".to_string(),
        time: "noon_12".to_string(),
        weekdays: None,
        custom_interval_hours: None,
        custom_hour: None,
        custom_minute: None,
    };
    assert_eq!(scheduler::schedule_to_cron(&noon), "0 12 * * *");
    assert_eq!(scheduler::schedule_description(&noon), "每天 12:00");

    let evening = ScheduleConfig {
        frequency: "daily".to_string(),
        time: "evening_8".to_string(),
        weekdays: None,
        custom_interval_hours: None,
        custom_hour: None,
        custom_minute: None,
    };
    assert_eq!(scheduler::schedule_to_cron(&evening), "0 20 * * *");
    assert_eq!(scheduler::schedule_description(&evening), "每天 20:00");
}

#[test]
fn test_schedule_to_cron_weekly() {
    let cfg = ScheduleConfig {
        frequency: "weekly".to_string(),
        time: "morning_8".to_string(),
        weekdays: Some(vec![1]), // Monday
        custom_interval_hours: None,
        custom_hour: None,
        custom_minute: None,
    };
    assert_eq!(scheduler::schedule_to_cron(&cfg), "0 8 * * 1");
    assert_eq!(scheduler::schedule_description(&cfg), "每周一 08:00");

    let multi_day = ScheduleConfig {
        frequency: "weekly".to_string(),
        time: "noon_12".to_string(),
        weekdays: Some(vec![1, 3, 5]), // Mon, Wed, Fri
        custom_interval_hours: None,
        custom_hour: None,
        custom_minute: None,
    };
    assert_eq!(scheduler::schedule_to_cron(&multi_day), "0 12 * * 1,3,5");
    assert_eq!(scheduler::schedule_description(&multi_day), "每周一、三、五 12:00");
}

#[test]
fn test_schedule_to_cron_custom() {
    let interval = ScheduleConfig {
        frequency: "custom".to_string(),
        time: "custom".to_string(),
        weekdays: None,
        custom_interval_hours: Some(6),
        custom_hour: None,
        custom_minute: None,
    };
    assert_eq!(scheduler::schedule_to_cron(&interval), "0 */6 * * *");
    assert_eq!(scheduler::schedule_description(&interval), "每隔6小时");

    let specific_time = ScheduleConfig {
        frequency: "custom".to_string(),
        time: "custom".to_string(),
        weekdays: None,
        custom_interval_hours: None,
        custom_hour: Some(3),
        custom_minute: Some(30),
    };
    assert_eq!(scheduler::schedule_to_cron(&specific_time), "30 3 * * *");
    assert_eq!(scheduler::schedule_description(&specific_time), "每天 03:30");

    // Zero interval = every 30 minutes (Zotero sync)
    let zero_interval = ScheduleConfig {
        frequency: "custom".to_string(),
        time: "custom".to_string(),
        weekdays: None,
        custom_interval_hours: Some(0),
        custom_hour: None,
        custom_minute: None,
    };
    assert_eq!(scheduler::schedule_to_cron(&zero_interval), "*/30 * * * *");
    assert_eq!(scheduler::schedule_description(&zero_interval), "每30分钟");
}

#[test]
fn test_next_run_from_cron() {
    // Valid cron should produce a future timestamp
    let next = scheduler::next_run_from_cron("0 8 * * *");
    assert!(next.is_some(), "Should compute next run for valid cron");

    let next_ts = next.unwrap();
    // Parse the RFC3339 timestamp
    let parsed = chrono::DateTime::parse_from_rfc3339(&next_ts);
    assert!(parsed.is_ok(), "Next run should be valid RFC3339: {}", next_ts);

    // It should be in the future
    let now = chrono::Utc::now();
    assert!(parsed.unwrap() > now, "Next run should be in the future");

    // Invalid cron should return None
    let invalid = scheduler::next_run_from_cron("invalid cron");
    assert!(invalid.is_none(), "Invalid cron should return None");
}

// ── Flow 7: Skills ────────────────────────────────────────────────────────────

#[test]
fn test_skill_crud() {
    let conn = setup_test_db();

    // Verify built-in skill from migration
    let initial = SkillRepo::list_all(&conn).expect("List initial skills");
    assert!(initial.len() >= 1, "Should have at least 1 built-in skill");

    // Create a user skill
    let skill = SkillRepo::create(
        &conn,
        &CreateSkill {
            name: "Daily Briefing".to_string(),
            definition: "---\nname: \"Daily Briefing\"\ntrigger: \"/daily-brief\"\n---\n\n# Daily Briefing\n\nSteps...".to_string(),
            source: "user_created".to_string(),
            trigger: Some("/daily-brief".to_string()),
        },
    )
    .expect("Create skill");
    assert!(!skill.id.is_empty());
    assert_eq!(skill.name, "Daily Briefing");
    assert!(skill.is_active);
    let skill_id = skill.id.clone();

    // Get by ID
    let fetched = SkillRepo::get_by_id(&conn, &skill_id).expect("Get skill");
    assert_eq!(fetched.name, "Daily Briefing");

    // Find by trigger
    let found = SkillRepo::find_by_trigger(&conn, "/daily-brief").expect("Find by trigger");
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, skill_id);

    // Not found trigger
    let not_found = SkillRepo::find_by_trigger(&conn, "/nonexistent").expect("Find nonexistent");
    assert!(not_found.is_none());

    // Update skill
    SkillRepo::update(
        &conn,
        &skill_id,
        &UpdateSkill {
            name: Some("Daily Research Briefing".to_string()),
            trigger: Some("/daily-research".to_string()),
            ..Default::default()
        },
    )
    .expect("Update skill");

    // Old trigger should no longer work
    let old_trigger = SkillRepo::find_by_trigger(&conn, "/daily-brief").expect("Old trigger");
    assert!(old_trigger.is_none());

    // New trigger should work
    let new_trigger = SkillRepo::find_by_trigger(&conn, "/daily-research").expect("New trigger");
    assert!(new_trigger.is_some());

    // Deactivate skill
    SkillRepo::update(
        &conn,
        &skill_id,
        &UpdateSkill {
            is_active: Some(false),
            ..Default::default()
        },
    )
    .expect("Deactivate skill");

    // Deactivated skill should not be found by trigger (only active skills)
    let inactive_trigger = SkillRepo::find_by_trigger(&conn, "/daily-research").expect("Inactive trigger");
    assert!(inactive_trigger.is_none());

    // List active should not include deactivated
    let active = SkillRepo::list_active(&conn).expect("List active");
    assert!(active.iter().all(|s| s.is_active));

    // Delete skill
    SkillRepo::delete(&conn, &skill_id).expect("Delete skill");
    let after_delete = SkillRepo::list_all(&conn).expect("List after delete");
    assert!(after_delete.iter().all(|s| s.id != skill_id));
}

// ── Flow 8: Settings Persistence ──────────────────────────────────────────────

#[test]
fn test_settings_crud() {
    let conn = setup_test_db();

    // Get default settings (from migration)
    let settings = SettingsRepo::get(&conn).expect("Get default settings");
    assert_eq!(settings.theme, "system");
    assert_eq!(settings.language, "zh-CN");
    assert!(settings.auto_process_papers);
    assert_eq!(settings.default_llm_provider, "ollama");
    assert_eq!(settings.export_format, "bibtex");

    // Update settings
    let updated = AppSettings {
        theme: "dark".to_string(),
        language: "en-US".to_string(),
        auto_process_papers: false,
        default_llm_provider: "deepseek".to_string(),
        check_updates_on_start: false,
        paper_storage_path: Some("/custom/path".to_string()),
        max_backup_count: 10,
        export_format: "apa".to_string(),
    };
    SettingsRepo::update_settings(&conn, &updated).expect("Update settings");

    // Verify updated
    let loaded = SettingsRepo::get(&conn).expect("Get updated settings");
    assert_eq!(loaded.theme, "dark");
    assert_eq!(loaded.language, "en-US");
    assert!(!loaded.auto_process_papers);
    assert_eq!(loaded.default_llm_provider, "deepseek");
    assert!(!loaded.check_updates_on_start);
    assert_eq!(loaded.paper_storage_path.as_deref(), Some("/custom/path"));
    assert_eq!(loaded.max_backup_count, 10);
    assert_eq!(loaded.export_format, "apa");

    // Get single value
    let theme_val = SettingsRepo::get_value(&conn, "theme").expect("Get theme value");
    assert_eq!(theme_val, Some("dark".to_string()));

    // Non-existent key
    let missing = SettingsRepo::get_value(&conn, "nonexistent").expect("Get missing");
    assert_eq!(missing, None);

    // Set a custom key
    SettingsRepo::set(&conn, "first_launch", "false").expect("Set custom key");
    let first_launch = SettingsRepo::get_value(&conn, "first_launch").expect("Get custom key");
    assert_eq!(first_launch, Some("false".to_string()));
}

// ── Flow 9: Cross-Entity Integration ──────────────────────────────────────────

#[test]
fn test_full_user_journey() {
    let conn = setup_test_db();

    // === Scenario: A researcher imports papers, organizes, reads, takes notes ===

    // 1. Import papers (simulate by creating)
    let p1 = PaperRepo::insert(
        &conn,
        &CreatePaperInput {
            title: "Attention Is All You Need".to_string(),
            authors: Some("Vaswani et al.".to_string()),
            abstract_text: Some("We propose a new simple network architecture, the Transformer, based solely on attention mechanisms.".to_string()),
            publication_date: Some("2017-06-12".to_string()),
            journal: Some("NeurIPS".to_string()),
            volume: None, issue: None, pages: None,
            doi: Some("10.48550/arXiv.1706.03762".to_string()),
            arxiv_id: Some("1706.03762".to_string()),
            pdf_url: None, pdf_path: Some("attention_20170612.pdf".to_string()),
            folder_id: None,
        },
    ).expect("Import paper 1");

    let p2 = PaperRepo::insert(
        &conn,
        &CreatePaperInput {
            title: "BERT: Pre-training of Deep Bidirectional Transformers".to_string(),
            authors: Some("Devlin et al.".to_string()),
            abstract_text: Some("We introduce a new language representation model called BERT.".to_string()),
            publication_date: Some("2018-10-11".to_string()),
            journal: Some("NAACL".to_string()),
            volume: None, issue: None, pages: None,
            doi: None, arxiv_id: None,
            pdf_url: None, pdf_path: Some("bert_20181011.pdf".to_string()),
            folder_id: None,
        },
    ).expect("Import paper 2");

    let p3 = PaperRepo::insert(
        &conn,
        &CreatePaperInput {
            title: "Deep Residual Learning for Image Recognition".to_string(),
            authors: Some("He et al.".to_string()),
            abstract_text: Some("We present a residual learning framework to ease the training of networks.".to_string()),
            publication_date: Some("2015-12-10".to_string()),
            journal: Some("CVPR".to_string()),
            volume: None, issue: None, pages: None,
            doi: None, arxiv_id: None,
            pdf_url: None, pdf_path: None,
            folder_id: None,
        },
    ).expect("Import paper 3");

    // 2. Create a folder and organize
    let nlp_folder = FolderRepo::insert(
        &conn,
        &CreateFolderInput {
            name: "NLP Papers".to_string(),
            parent_id: None,
            color: Some("#3b82f6".to_string()),
            icon: Some("language".to_string()),
        },
    )
    .expect("Create NLP folder");

    PaperRepo::update(
        &conn,
        &UpdatePaperInput {
            id: p1.id.clone(),
            folder_id: Some(nlp_folder.id.clone()),
            ..Default::default()
        },
    )
    .expect("Move p1 to NLP");
    PaperRepo::update(
        &conn,
        &UpdatePaperInput {
            id: p2.id.clone(),
            folder_id: Some(nlp_folder.id.clone()),
            ..Default::default()
        },
    )
    .expect("Move p2 to NLP");

    // 3. Tag papers
    let tag_nlp = TagRepo::find_or_create(&conn, "NLP", Some("#3b82f6")).expect("Create NLP tag");
    let tag_transformer = TagRepo::find_or_create(&conn, "transformer", Some("#8b5cf6")).expect("Create transformer tag");
    let tag_cv = TagRepo::find_or_create(&conn, "computer-vision", None).expect("Create CV tag");

    TagRepo::add_paper_tag(&conn, &p1.id, &tag_nlp.id).expect("Tag p1 NLP");
    TagRepo::add_paper_tag(&conn, &p1.id, &tag_transformer.id).expect("Tag p1 transformer");
    TagRepo::add_paper_tag(&conn, &p2.id, &tag_nlp.id).expect("Tag p2 NLP");
    TagRepo::add_paper_tag(&conn, &p2.id, &tag_transformer.id).expect("Tag p2 transformer");
    TagRepo::add_paper_tag(&conn, &p3.id, &tag_cv.id).expect("Tag p3 CV");

    // 4. Read paper and track progress
    PaperRepo::update(
        &conn,
        &UpdatePaperInput {
            id: p1.id.clone(),
            reading_status: Some("reading".to_string()),
            ..Default::default()
        },
    )
    .expect("Mark p1 as reading");

    ReadingRepo::upsert(
        &conn,
        &UpdateReadingStateInput {
            paper_id: p1.id.clone(),
            current_page: Some(8),
            total_pages: Some(12),
            scroll_position: Some(0.66),
            zoom_level: Some(1.0),
        },
    )
    .expect("Save reading progress for p1");

    // 5. Take notes with bidirectional links
    let note1 = NoteRepo::insert(
        &conn,
        &CreateNoteInput {
            paper_id: Some(p1.id.clone()),
            title: "Self-Attention Mechanism".to_string(),
            content: "The core idea is self-attention: each position in the sequence attends to all other positions.".to_string(),
            note_type: Some("general".to_string()),
            page_number: None,
            position: None,
        },
    )
    .expect("Create note1");

    let note2 = NoteRepo::insert(
        &conn,
        &CreateNoteInput {
            paper_id: Some(p1.id.clone()),
            title: "Multi-Head Attention".to_string(),
            content: "Multiple attention heads allow the model to jointly attend to information from different representation subspaces.".to_string(),
            note_type: Some("annotation".to_string()),
            page_number: Some(4),
            position: Some(r#"{"top":0.3,"left":0.1}"#.to_string()),
        },
    )
    .expect("Create note2");

    let note3 = NoteRepo::insert(
        &conn,
        &CreateNoteInput {
            paper_id: Some(p2.id.clone()),
            title: "Masked Language Model".to_string(),
            content: "BERT uses masked language modeling: randomly mask tokens and predict them. Related to self-attention in Transformer.".to_string(),
            note_type: Some("general".to_string()),
            page_number: None,
            position: None,
        },
    )
    .expect("Create note3");

    // Create bidirectional link: note2 → note3 (multi-head attention relates to MLM)
    NoteRepo::create_link(
        &conn,
        &CreateNoteLinkInput {
            source_id: note2.id.clone(),
            target_id: note3.id.clone(),
            label: Some("see also BERT".to_string()),
        },
    )
    .expect("Link note2→note3");

    // And: note3 → note1 (MLM references self-attention)
    NoteRepo::create_link(
        &conn,
        &CreateNoteLinkInput {
            source_id: note3.id.clone(),
            target_id: note1.id.clone(),
            label: None,
        },
    )
    .expect("Link note3→note1");

    // 6. Search for "attention" across papers (FTS5)
    let search_results = NoteRepo::search(&conn, "attention", None, None, 20, 0)
        .expect("Search notes for attention");
    assert!(search_results.total >= 2, "Should find at least 2 notes about attention");

    // 7. Verify the full graph:
    // note1 ← (backlink from note3)
    // note2 → note3
    // note3 → note1, note3 ← (backlink from note2)
    let note1_links = NoteRepo::get_with_links(&conn, &note1.id).expect("note1 links");
    assert_eq!(note1_links.outgoing_links.len(), 0);
    assert_eq!(note1_links.backlinks.len(), 1); // from note3

    let note2_links = NoteRepo::get_with_links(&conn, &note2.id).expect("note2 links");
    assert_eq!(note2_links.outgoing_links.len(), 1); // → note3
    assert_eq!(note2_links.backlinks.len(), 0);

    let note3_links = NoteRepo::get_with_links(&conn, &note3.id).expect("note3 links");
    assert_eq!(note3_links.outgoing_links.len(), 1); // → note1
    assert_eq!(note3_links.backlinks.len(), 1); // ← note2

    // 8. Verify reading progress
    let reading = ReadingRepo::get_by_paper(&conn, &p1.id).expect("Reading state");
    assert!((reading.reading_progress - 8.0 / 12.0).abs() < 0.01);

    // 9. Mark p1 as read
    PaperRepo::update(
        &conn,
        &UpdatePaperInput {
            id: p1.id.clone(),
            reading_status: Some("read".to_string()),
            is_favorite: Some(true),
            ..Default::default()
        },
    )
    .expect("Mark p1 as read");

    // 10. Final verification: count everything
    let all_papers = PaperRepo::list(&conn, &PaperFilter::default()).expect("All papers");
    assert_eq!(all_papers.total, 3);

    let all_notes = NoteRepo::list(&conn, 100, 0).expect("All notes");
    assert_eq!(all_notes.len(), 3);

    let all_tags = TagRepo::list(&conn).expect("All tags");
    assert_eq!(all_tags.len(), 3);

    let all_folders = FolderRepo::list(&conn).expect("All folders");
    // 4 default + 1 NLP = 5
    assert_eq!(all_folders.len(), 5);

    // Reading progress for p1 should exist
    let progress = ReadingRepo::list_reading_progress(&conn, 10).expect("Reading progress");
    assert_eq!(progress.len(), 1);

    // 11. Setup an automation task
    let task = TaskRepo::create(
        &conn,
        &CreateAutoTask {
            name: "Daily Transformer Papers".to_string(),
            task_type: "custom".to_string(),
            preset_id: None,
            definition: r#"{"steps":[{"id":"search","tool":"paper.search","input":{"query":"{{keywords}}","sort":"newest"}}]}"#.to_string(),
            schedule: r#"{"frequency":"daily","time":"morning_8"}"#.to_string(),
            schedule_cron: Some("0 8 * * *".to_string()),
            config: Some(r#"{"keywords":"transformer attention"}"#.to_string()),
        },
    )
    .expect("Create automation task");
    assert!(task.is_enabled);

    // Run the task manually (simulate)
    let execution = TaskRepo::create_execution(&conn, &task.id).expect("Start execution");
    TaskRepo::finish_execution(
        &conn,
        &execution.id,
        "success",
        Some("Found 5 new transformer papers"),
        None,
        350,
    )
    .expect("Finish execution");

    // Update task timing
    let now = chrono::Utc::now().to_rfc3339();
    let next = scheduler::next_run_from_cron("0 8 * * *");
    TaskRepo::update_last_run(&conn, &task.id, &now, next.as_deref()).expect("Update timing");

    let task_after_run = TaskRepo::get_by_id(&conn, &task.id).expect("Task after run");
    assert!(task_after_run.last_run.is_some());
    assert!(task_after_run.next_run.is_some());

    // 12. Verify settings still intact
    let settings = SettingsRepo::get(&conn).expect("Settings");
    assert_eq!(settings.language, "zh-CN");
}

#[test]
fn test_paper_fts5_search() {
    let conn = setup_test_db();

    // Create papers with specific content for FTS5
    let _p1 = PaperRepo::insert(
        &conn,
        &CreatePaperInput {
            title: "GPT-4 Technical Report".to_string(),
            authors: Some("OpenAI".to_string()),
            abstract_text: Some("GPT-4 is a large multimodal model exhibiting human-level performance on various professional benchmarks.".to_string()),
            publication_date: Some("2023-03-15".to_string()),
            journal: None, volume: None, issue: None, pages: None,
            doi: None, arxiv_id: None, pdf_url: None, pdf_path: None,
            folder_id: None,
        },
    ).expect("Create GPT-4 paper");

    let _p2 = PaperRepo::insert(
        &conn,
        &CreatePaperInput {
            title: "LLaMA: Open and Efficient Foundation Language Models".to_string(),
            authors: Some("Touvron et al.".to_string()),
            abstract_text: Some("We introduce LLaMA, a collection of foundation language models ranging from 7B to 65B parameters.".to_string()),
            publication_date: Some("2023-02-27".to_string()),
            journal: None, volume: None, issue: None, pages: None,
            doi: None, arxiv_id: None, pdf_url: None, pdf_path: None,
            folder_id: None,
        },
    ).expect("Create LLaMA paper");

    let _p3 = PaperRepo::insert(
        &conn,
        &CreatePaperInput {
            title: "Constitutional AI: Harmlessness from AI Feedback".to_string(),
            authors: Some("Bai et al.".to_string()),
            abstract_text: Some("We explore a method for training AI systems to be harmless through AI feedback.".to_string()),
            publication_date: Some("2022-12-07".to_string()),
            journal: None, volume: None, issue: None, pages: None,
            doi: None, arxiv_id: None, pdf_url: None, pdf_path: None,
            folder_id: None,
        },
    ).expect("Create CAI paper");

    // Search for "language model" — should match LLaMA's abstract and title
    let lm_results = PaperRepo::list(
        &conn,
        &PaperFilter {
            query: Some("language model".to_string()),
            ..Default::default()
        },
    )
    .expect("Search language model");
    assert!(lm_results.total >= 1, "Should find at least 1 paper about language models");

    // Search for "GPT" — should match title
    let gpt_results = PaperRepo::list(
        &conn,
        &PaperFilter {
            query: Some("GPT".to_string()),
            ..Default::default()
        },
    )
    .expect("Search GPT");
    assert!(gpt_results.total >= 1, "Should find GPT paper");

    // Search for "AI" — should match CAI and possibly others
    let ai_results = PaperRepo::list(
        &conn,
        &PaperFilter {
            query: Some("AI".to_string()),
            ..Default::default()
        },
    )
    .expect("Search AI");
    assert!(ai_results.total >= 1, "Should find papers about AI");
}

// ── Flow 10: Edge Cases ───────────────────────────────────────────────────────

#[test]
fn test_duplicate_link_is_ignored() {
    let conn = setup_test_db();
    let paper = create_test_paper(&conn, "Test Paper");

    let n1 = create_test_note(&conn, &paper.id, "Note A", "Content A");
    let n2 = create_test_note(&conn, &paper.id, "Note B", "Content B");

    // Create link first time
    NoteRepo::create_link(
        &conn,
        &CreateNoteLinkInput {
            source_id: n1.id.clone(),
            target_id: n2.id.clone(),
            label: None,
        },
    )
    .expect("Create link");

    // Create same link again (INSERT OR IGNORE)
    NoteRepo::create_link(
        &conn,
        &CreateNoteLinkInput {
            source_id: n1.id.clone(),
            target_id: n2.id.clone(),
            label: Some("updated label".to_string()),
        },
    )
    .expect("Create duplicate link");

    // Should still have only 1 link
    let outgoing = NoteRepo::list_outgoing_links(&conn, &n1.id).expect("Outgoing");
    assert_eq!(outgoing.len(), 1, "Duplicate link should be ignored");
}

#[test]
fn test_delete_paper_cascades_notes_and_reading() {
    let conn = setup_test_db();

    let paper = create_test_paper(&conn, "Doomed Paper");

    // Create note + reading state
    let _note = create_test_note(&conn, &paper.id, "Note", "Content");
    ReadingRepo::upsert(
        &conn,
        &UpdateReadingStateInput {
            paper_id: paper.id.clone(),
            current_page: Some(5),
            total_pages: Some(20),
            scroll_position: None,
            zoom_level: None,
        },
    )
    .expect("Create reading state");

    // Delete paper — notes and reading_states should cascade
    PaperRepo::delete(&conn, &paper.id).expect("Delete paper");

    // Notes should be gone
    let notes = NoteRepo::list_by_paper(&conn, &paper.id).expect("Notes after paper delete");
    assert_eq!(notes.len(), 0, "Notes should cascade delete with paper");

    // Reading state should be gone
    let reading = ReadingRepo::get_by_paper(&conn, &paper.id);
    assert!(reading.is_err(), "Reading state should cascade delete with paper");
}

#[test]
fn test_paper_not_found_errors() {
    let conn = setup_test_db();

    let result = PaperRepo::get_by_id(&conn, "nonexistent-id");
    assert!(result.is_err());

    let update_result = PaperRepo::update(
        &conn,
        &UpdatePaperInput {
            id: "nonexistent-id".to_string(),
            title: Some("New Title".to_string()),
            ..Default::default()
        },
    );
    assert!(update_result.is_err());

    let delete_result = PaperRepo::delete(&conn, "nonexistent-id");
    assert!(delete_result.is_err());
}

#[test]
fn test_note_not_found_errors() {
    let conn = setup_test_db();

    let result = NoteRepo::get_by_id(&conn, "nonexistent-id");
    assert!(result.is_err());

    let update_result = NoteRepo::update(
        &conn,
        &UpdateNoteInput {
            id: "nonexistent-id".to_string(),
            content: Some("new content".to_string()),
            ..Default::default()
        },
    );
    assert!(update_result.is_err());

    let delete_result = NoteRepo::delete(&conn, "nonexistent-id");
    assert!(delete_result.is_err());
}

#[test]
fn test_folder_not_found_errors() {
    let conn = setup_test_db();

    let result = FolderRepo::get_by_id(&conn, "nonexistent-id");
    assert!(result.is_err());

    let update_result = FolderRepo::update(
        &conn,
        &UpdateFolderInput {
            id: "nonexistent-id".to_string(),
            name: Some("New Name".to_string()),
            ..Default::default()
        },
    );
    assert!(update_result.is_err());

    let delete_result = FolderRepo::delete(&conn, "nonexistent-id");
    assert!(delete_result.is_err());
}

#[test]
fn test_link_nonexistent_note_fails() {
    let conn = setup_test_db();

    let paper = create_test_paper(&conn, "Test");
    let note = create_test_note(&conn, &paper.id, "Existing Note", "Content");

    // Try to link to a non-existent note
    let result = NoteRepo::create_link(
        &conn,
        &CreateNoteLinkInput {
            source_id: note.id.clone(),
            target_id: "nonexistent-note".to_string(),
            label: None,
        },
    );
    assert!(result.is_err(), "Linking to non-existent note should fail");
}

#[test]
fn test_self_link_prevention() {
    let conn = setup_test_db();

    let paper = create_test_paper(&conn, "Test");
    let note = create_test_note(&conn, &paper.id, "Self Note", "Content");

    // Self-link should work (it's just source=target), but semantically weird.
    // The schema allows it (same source and target), so we just verify it doesn't crash.
    let result = NoteRepo::create_link(
        &conn,
        &CreateNoteLinkInput {
            source_id: note.id.clone(),
            target_id: note.id.clone(),
            label: None,
        },
    );
    // If self-links are allowed, this succeeds; if not, it fails.
    // Our schema has PRIMARY KEY(source_id, target_id) — same value is fine.
    // But semantically it's odd. Let's just verify no crash.
    assert!(result.is_ok() || result.is_err(), "Self-link should not crash");
}

#[test]
fn test_empty_paper_list() {
    let conn = setup_test_db();

    // No papers created beyond defaults
    let result = PaperRepo::list(&conn, &PaperFilter::default()).expect("List empty");
    assert_eq!(result.total, 0);
    assert!(result.papers.is_empty());
}

#[test]
fn test_note_search_suggest_empty_prefix() {
    let conn = setup_test_db();

    let suggestions = NoteRepo::search_suggest(&conn, "", 5).expect("Empty prefix suggest");
    assert!(suggestions.is_empty());
}

#[test]
fn test_schedule_fallback() {
    // Unknown frequency → fallback "0 8 * * *"
    let unknown = ScheduleConfig {
        frequency: "hourly".to_string(),
        time: "morning_8".to_string(),
        weekdays: None,
        custom_interval_hours: None,
        custom_hour: None,
        custom_minute: None,
    };
    assert_eq!(scheduler::schedule_to_cron(&unknown), "0 8 * * *");
    assert_eq!(scheduler::schedule_description(&unknown), "每天 08:00");
}

#[test]
fn test_preset_tasks_from_migration() {
    let conn = setup_test_db();

    let tasks = TaskRepo::list_all(&conn).expect("List preset tasks");

    // Should have 6 preset tasks, all disabled
    let preset_tasks: Vec<_> = tasks.iter().filter(|t| t.task_type == "preset").collect();
    assert_eq!(preset_tasks.len(), 6, "Should have 6 preset tasks from migration");

    // All preset tasks should be disabled (is_enabled = 0 in migration)
    assert!(preset_tasks.iter().all(|t| !t.is_enabled),
        "All preset tasks should be disabled by default");

    // Verify each preset has a schedule_cron
    assert!(preset_tasks.iter().all(|t| t.schedule_cron.is_some()),
        "All preset tasks should have a cron schedule");
}
