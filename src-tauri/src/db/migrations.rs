//! Database migrations

use rusqlite::Connection;
use crate::errors::Result;

/// Run all migrations in order. Each migration is idempotent (uses IF NOT EXISTS).
pub fn run_migrations(conn: &Connection) -> Result<()> {
    // Create migrations tracking table
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _migrations (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        );"
    )?;

    let applied: Vec<String> = {
        let mut stmt = conn.prepare("SELECT name FROM _migrations")?;
        let rows: Vec<String> = stmt
            .query_map([], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();
        rows
    };

    let migrations = get_migrations();
    for (name, sql) in migrations {
        if !applied.contains(&name.to_string()) {
            tracing::info!("Running migration: {}", name);
            conn.execute_batch(sql)?;
            conn.execute(
                "INSERT INTO _migrations (name) VALUES (?1)",
                [&name],
            )?;
            tracing::info!("Migration {} applied successfully", name);
        }
    }

    Ok(())
}

fn get_migrations() -> Vec<(&'static str, &'static str)> {
    vec![
        ("001_create_papers_table", M001_CREATE_PAPERS),
        ("002_create_folders_table", M002_CREATE_FOLDERS),
        ("003_create_tags_table", M003_CREATE_TAGS),
        ("004_create_notes_table", M004_CREATE_NOTES),
        ("005_create_reading_states_table", M005_CREATE_READING_STATES),
        ("006_create_settings_table", M006_CREATE_SETTINGS),
        ("007_create_operation_log_table", M007_CREATE_OPERATION_LOG),
        ("008_create_fts5_index", M008_CREATE_FTS5),
        ("009_create_fts_triggers", M009_CREATE_FTS_TRIGGERS),
        ("010_create_indexes", M010_CREATE_INDEXES),
        ("011_insert_default_settings", M011_DEFAULT_SETTINGS),
        ("012_create_note_links_and_notes_fts", M012_NOTE_LINKS_AND_NOTES_FTS),
        ("013_create_auto_tasks_and_skills", M013_AUTO_TASKS_AND_SKILLS),
        ("014_add_file_type_and_crawl_tables", M014_FILE_TYPE_AND_CRAWL),
        ("015_create_collections_and_ai_results", M015_COLLECTIONS_AND_AI_RESULTS),
        ("016_remove_default_folders", M016_REMOVE_DEFAULT_FOLDERS),
    ]
}

static M001_CREATE_PAPERS: &str = r#"
CREATE TABLE IF NOT EXISTS papers (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    authors TEXT,
    abstract_text TEXT,
    publication_date TEXT,
    journal TEXT,
    volume TEXT,
    issue TEXT,
    pages TEXT,
    doi TEXT UNIQUE,
    arxiv_id TEXT,
    pdf_url TEXT,
    pdf_path TEXT,
    folder_id TEXT,
    tldr TEXT,
    reading_status TEXT NOT NULL DEFAULT 'unread' CHECK (reading_status IN ('unread', 'reading', 'read')),
    is_favorite INTEGER NOT NULL DEFAULT 0,
    date_added TEXT NOT NULL DEFAULT (datetime('now')),
    date_modified TEXT NOT NULL DEFAULT (datetime('now')),
    cover_image TEXT,
    FOREIGN KEY (folder_id) REFERENCES folders(id) ON DELETE SET NULL
);
"#;

static M002_CREATE_FOLDERS: &str = r#"
CREATE TABLE IF NOT EXISTS folders (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    parent_id TEXT,
    color TEXT,
    icon TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    date_created TEXT NOT NULL DEFAULT (datetime('now')),
    date_modified TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (parent_id) REFERENCES folders(id) ON DELETE CASCADE
);
"#;

static M003_CREATE_TAGS: &str = r#"
CREATE TABLE IF NOT EXISTS tags (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    color TEXT,
    usage_count INTEGER NOT NULL DEFAULT 0,
    date_created TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS paper_tags (
    paper_id TEXT NOT NULL,
    tag_id TEXT NOT NULL,
    date_added TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (paper_id, tag_id),
    FOREIGN KEY (paper_id) REFERENCES papers(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);
"#;

static M004_CREATE_NOTES: &str = r#"
CREATE TABLE IF NOT EXISTS notes (
    id TEXT PRIMARY KEY,
    paper_id TEXT,
    title TEXT NOT NULL,
    content TEXT NOT NULL DEFAULT '',
    note_type TEXT NOT NULL DEFAULT 'general' CHECK (note_type IN ('general', 'annotation', 'excerpt', 'summary')),
    page_number INTEGER,
    position TEXT,
    date_created TEXT NOT NULL DEFAULT (datetime('now')),
    date_modified TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (paper_id) REFERENCES papers(id) ON DELETE CASCADE
);
"#;

static M005_CREATE_READING_STATES: &str = r#"
CREATE TABLE IF NOT EXISTS reading_states (
    id TEXT PRIMARY KEY,
    paper_id TEXT NOT NULL UNIQUE,
    current_page INTEGER NOT NULL DEFAULT 0,
    total_pages INTEGER NOT NULL DEFAULT 0,
    scroll_position REAL NOT NULL DEFAULT 0.0,
    zoom_level REAL NOT NULL DEFAULT 1.0,
    last_read_at TEXT NOT NULL DEFAULT (datetime('now')),
    reading_progress REAL NOT NULL DEFAULT 0.0,
    FOREIGN KEY (paper_id) REFERENCES papers(id) ON DELETE CASCADE
);
"#;

static M006_CREATE_SETTINGS: &str = r#"
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
"#;

static M007_CREATE_OPERATION_LOG: &str = r#"
CREATE TABLE IF NOT EXISTS operation_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    operation TEXT NOT NULL,
    target_type TEXT NOT NULL,
    target_id TEXT,
    detail TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
"#;

static M008_CREATE_FTS5: &str = r#"
CREATE VIRTUAL TABLE IF NOT EXISTS papers_fts USING fts5(
    title,
    authors,
    abstract_text,
    journal,
    tldr,
    content='papers',
    content_rowid='rowid',
    tokenize='unicode61'
);
"#;

static M009_CREATE_FTS_TRIGGERS: &str = r#"
-- Keep FTS index in sync with papers table
CREATE TRIGGER IF NOT EXISTS papers_fts_insert AFTER INSERT ON papers BEGIN
    INSERT INTO papers_fts(rowid, title, authors, abstract_text, journal, tldr)
    VALUES (new.rowid, new.title, new.authors, new.abstract_text, new.journal, new.tldr);
END;

CREATE TRIGGER IF NOT EXISTS papers_fts_delete AFTER DELETE ON papers BEGIN
    INSERT INTO papers_fts(papers_fts, rowid, title, authors, abstract_text, journal, tldr)
    VALUES ('delete', old.rowid, old.title, old.authors, old.abstract_text, old.journal, old.tldr);
END;

CREATE TRIGGER IF NOT EXISTS papers_fts_update AFTER UPDATE ON papers BEGIN
    INSERT INTO papers_fts(papers_fts, rowid, title, authors, abstract_text, journal, tldr)
    VALUES ('delete', old.rowid, old.title, old.authors, old.abstract_text, old.journal, old.tldr);
    INSERT INTO papers_fts(rowid, title, authors, abstract_text, journal, tldr)
    VALUES (new.rowid, new.title, new.authors, new.abstract_text, new.journal, new.tldr);
END;
"#;

static M010_CREATE_INDEXES: &str = r#"
CREATE INDEX IF NOT EXISTS idx_papers_folder_id ON papers(folder_id);
CREATE INDEX IF NOT EXISTS idx_papers_reading_status ON papers(reading_status);
CREATE INDEX IF NOT EXISTS idx_papers_date_added ON papers(date_added);
CREATE INDEX IF NOT EXISTS idx_papers_is_favorite ON papers(is_favorite);
CREATE INDEX IF NOT EXISTS idx_papers_doi ON papers(doi);
CREATE INDEX IF NOT EXISTS idx_folders_parent_id ON folders(parent_id);
CREATE INDEX IF NOT EXISTS idx_notes_paper_id ON notes(paper_id);
CREATE INDEX IF NOT EXISTS idx_reading_states_paper_id ON reading_states(paper_id);
CREATE INDEX IF NOT EXISTS idx_paper_tags_tag_id ON paper_tags(tag_id);
CREATE INDEX IF NOT EXISTS idx_operation_log_created_at ON operation_log(created_at);
"#;

static M011_DEFAULT_SETTINGS: &str = r#"
INSERT OR IGNORE INTO settings (key, value) VALUES
    ('theme', 'system'),
    ('language', 'zh-CN'),
    ('auto_process_papers', 'true'),
    ('default_llm_provider', 'ollama'),
    ('check_updates_on_start', 'true'),
    ('max_backup_count', '5'),
    ('export_format', 'bibtex'),
    ('first_launch', 'true'),
    ('onboarding_completed', 'false');
"#;

static M012_NOTE_LINKS_AND_NOTES_FTS: &str = r#"
-- ── Bidirectional note links ────────────────────────────────────────────────
-- source_id links TO target_id.  Reverse link = swap source/target.
CREATE TABLE IF NOT EXISTS note_links (
    source_id TEXT NOT NULL,
    target_id TEXT NOT NULL,
    label     TEXT,          -- optional display label for the link
    date_created TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (source_id, target_id),
    FOREIGN KEY (source_id) REFERENCES notes(id) ON DELETE CASCADE,
    FOREIGN KEY (target_id) REFERENCES notes(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_note_links_source ON note_links(source_id);
CREATE INDEX IF NOT EXISTS idx_note_links_target ON note_links(target_id);

-- ── Notes full-text search (FTS5) ───────────────────────────────────────────
CREATE VIRTUAL TABLE IF NOT EXISTS notes_fts USING fts5(
    title,
    content,
    content='notes',
    content_rowid='rowid',
    tokenize='unicode61'
);

-- Keep notes_fts in sync with notes table
CREATE TRIGGER IF NOT EXISTS notes_fts_insert AFTER INSERT ON notes BEGIN
    INSERT INTO notes_fts(rowid, title, content)
    VALUES (new.rowid, new.title, new.content);
END;

CREATE TRIGGER IF NOT EXISTS notes_fts_delete AFTER DELETE ON notes BEGIN
    INSERT INTO notes_fts(notes_fts, rowid, title, content)
    VALUES ('delete', old.rowid, old.title, old.content);
END;

CREATE TRIGGER IF NOT EXISTS notes_fts_update AFTER UPDATE ON notes BEGIN
    INSERT INTO notes_fts(notes_fts, rowid, title, content)
    VALUES ('delete', old.rowid, old.title, old.content);
    INSERT INTO notes_fts(rowid, title, content)
    VALUES (new.rowid, new.title, new.content);
END;
"#;

// ── M013: Automation tasks + Task executions + Skills ───────────────────────

static M013_AUTO_TASKS_AND_SKILLS: &str = r#"
-- ── Auto tasks ──────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS auto_tasks (
    id            TEXT PRIMARY KEY,
    name          TEXT NOT NULL,
    task_type     TEXT NOT NULL DEFAULT 'preset',  -- preset/custom
    preset_id     TEXT,
    definition    TEXT NOT NULL,                    -- JSON: task step definitions
    schedule      TEXT NOT NULL,                    -- JSON: human-readable schedule config
    schedule_cron TEXT,                             -- internal cron (user never sees)
    is_enabled    INTEGER NOT NULL DEFAULT 1,
    last_run      TEXT,
    next_run      TEXT,
    config        TEXT,                             -- JSON: user config (keywords etc.)
    created_at    TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at    TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_tasks_enabled  ON auto_tasks(is_enabled);
CREATE INDEX IF NOT EXISTS idx_tasks_next_run ON auto_tasks(next_run);

-- ── Task executions ─────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS task_executions (
    id          TEXT PRIMARY KEY,
    task_id     TEXT NOT NULL REFERENCES auto_tasks(id) ON DELETE CASCADE,
    status      TEXT NOT NULL DEFAULT 'running',  -- running/success/failed
    started_at  TEXT NOT NULL,
    finished_at TEXT,
    result      TEXT,                              -- JSON: execution result
    error       TEXT,
    duration_ms INTEGER
);

CREATE INDEX IF NOT EXISTS idx_exec_task ON task_executions(task_id);
CREATE INDEX IF NOT EXISTS idx_exec_time ON task_executions(started_at DESC);

-- ── Skills ──────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS skills (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    definition  TEXT NOT NULL,                      -- Markdown + YAML frontmatter
    source      TEXT NOT NULL DEFAULT 'user',       -- builtin/user_created/ai_generated/community
    version     TEXT NOT NULL DEFAULT '1.0.0',
    trigger     TEXT,                               -- e.g. /trending-weekly
    is_active   INTEGER NOT NULL DEFAULT 1,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_skills_trigger ON skills(trigger);
CREATE INDEX IF NOT EXISTS idx_skills_source ON skills(source);

-- ── Preset task templates (inserted as default data) ────────────────────────
INSERT OR IGNORE INTO auto_tasks (id, name, task_type, preset_id, definition, schedule, schedule_cron, is_enabled, config)
VALUES
  ('preset-daily-trending', '每日技术热点', 'preset', 'daily_trending',
   '{"steps":[{"id":"search","tool":"paper.search","input":{"query":"{{keywords}}","sort":"trending","time_range":"day","limit":10}},{"id":"brief","tool":"paper.brief","input":{"papers":"{{search.results}}"}}],"output":{"type":"notification","template":"今日{{keywords}}热点：\n{{brief.results}}"}}',
   '{"frequency":"daily","time":"morning_8"}',
   '0 8 * * *',
   0,
   '{"keywords":""}'),

  ('preset-weekly-trending', '每周热点追踪', 'preset', 'weekly_trending',
   '{"steps":[{"id":"search","tool":"paper.search","input":{"query":"{{keywords}}","sort":"trending","time_range":"week","limit":10}},{"id":"brief","tool":"paper.brief","input":{"papers":"{{search.results}}"}}],"output":{"type":"notification","template":"本周{{keywords}}热点：\n{{brief.results}}"}}',
   '{"frequency":"weekly","time":"morning_8","weekdays":[1]}',
   '0 9 * * 1',
   0,
   '{"keywords":""}'),

  ('preset-new-paper-sub', '新论文订阅', 'preset', 'new_paper_sub',
   '{"steps":[{"id":"search","tool":"paper.search","input":{"query":"{{keywords}}","sort":"newest","time_range":"day","limit":5}}],"output":{"type":"notification","template":"新论文通知：\n{{search.results}}"}}',
   '{"frequency":"custom","custom_interval_hours":6}',
   '0 10,16 * * *',
   0,
   '{"keywords":""}'),

  ('preset-daily-reading', '每日阅读提醒', 'preset', 'daily_reading',
   '{"steps":[{"id":"check","tool":"reading.unread","input":{"limit":5}}],"output":{"type":"notification","template":"你有{{check.count}}篇未读论文"}}',
   '{"frequency":"daily","time":"evening_8"}',
   '0 20 * * *',
   0,
   NULL),

  ('preset-citation-refresh', '引用数据刷新', 'preset', 'citation_refresh',
   '{"steps":[{"id":"refresh","tool":"citation.refresh","input":{}}],"output":{"type":"silent"}}',
   '{"frequency":"daily","time":"custom","custom_hour":3,"custom_minute":0}',
   '0 3 * * *',
   0,
   NULL),

  ('preset-zotero-sync', 'Zotero增量同步', 'preset', 'zotero_sync',
   '{"steps":[{"id":"sync","tool":"zotero.sync","input":{}}],"output":{"type":"silent"}}',
   '{"frequency":"custom","custom_interval_hours":0,"custom_hour":0,"custom_minute":30}',
   '*/30 * * * *',
   0,
   NULL);

-- ── Built-in skills ─────────────────────────────────────────────────────────
INSERT OR IGNORE INTO skills (id, name, definition, source, version, trigger, is_active)
VALUES
  ('skill-trending-weekly', '每周热点追踪',
   '---\nname: "每周热点追踪"\nversion: "1.0.0"\ndescription: "自动追踪指定领域每周热点论文并生成摘要"\nauthor: "eidosinsight-built-in"\ntags: ["trending", "weekly", "auto"]\ntrigger: "/trending-weekly"\ntools_required:\n  - paper.search\n  - paper.brief\nparameters:\n  - name: keyword\n    type: string\n    required: true\n    description: "追踪的关键词或领域"\n  - name: time_range\n    type: enum\n    options: ["week", "month"]\n    default: "week"\n---\n\n# 每周热点追踪\n\n## 步骤\n\n1. **搜索热点论文**\n   tool: paper.search\n   input:\n     query: "{{keyword}}"\n     sort: trending\n     time_range: "{{time_range}}"\n     limit: 10\n\n2. **批量生成TL;DR**\n   tool: paper.brief\n   input:\n     papers: "{{step1.results}}"',
   'builtin', '1.0.0', '/trending-weekly', 1);
"#;

// ── M014: Add file_type column to papers + crawl/watch tables ───────────────

static M014_FILE_TYPE_AND_CRAWL: &str = r#"
-- Add file_type column to papers table for multi-format support
-- Legacy records (pdf_path only) default to 'pdf'
ALTER TABLE papers ADD COLUMN file_type TEXT;

-- Backfill existing records: anything with a pdf_path is a PDF
UPDATE papers SET file_type = 'pdf' WHERE pdf_path IS NOT NULL AND file_type IS NULL;

-- ── Watch directories ──────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS watch_dirs (
    id                TEXT PRIMARY KEY,
    path              TEXT NOT NULL,
    is_enabled        INTEGER NOT NULL DEFAULT 1,
    file_filter       TEXT,          -- comma-separated extensions, e.g. "pdf,txt,docx"
    folder_id         TEXT,
    recursive         INTEGER NOT NULL DEFAULT 1,
    scan_interval_min INTEGER NOT NULL DEFAULT 60,
    last_scan         TEXT,
    created_at        TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at        TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (folder_id) REFERENCES folders(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_watch_dirs_enabled ON watch_dirs(is_enabled);

-- ── File fingerprints (for incremental crawl change detection) ─────────────
CREATE TABLE IF NOT EXISTS file_fingerprints (
    id              TEXT PRIMARY KEY,
    file_path       TEXT NOT NULL,
    watch_dir_id    TEXT NOT NULL REFERENCES watch_dirs(id) ON DELETE CASCADE,
    file_size       INTEGER NOT NULL DEFAULT 0,
    modified_time   INTEGER NOT NULL DEFAULT 0,
    content_hash    TEXT,
    is_imported     INTEGER NOT NULL DEFAULT 0,
    paper_id        TEXT,
    discovered_at   TEXT NOT NULL DEFAULT (datetime('now')),
    last_checked    TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (paper_id) REFERENCES papers(id) ON DELETE SET NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_fingerprints_path ON file_fingerprints(file_path);
CREATE INDEX IF NOT EXISTS idx_fingerprints_watch_dir ON file_fingerprints(watch_dir_id);
CREATE INDEX IF NOT EXISTS idx_fingerprints_imported ON file_fingerprints(is_imported);
"#;

// ── M015: Collections, collection_papers, ai_results ───────────────────────

static M015_COLLECTIONS_AND_AI_RESULTS: &str = r#"
-- ── Collections (user-curated paper groups) ──────────────────────────────────
CREATE TABLE IF NOT EXISTS collections (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    description TEXT,
    color       TEXT,
    icon        TEXT,
    sort_order  INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ── Collection ↔ Paper (many-to-many) ───────────────────────────────────────
CREATE TABLE IF NOT EXISTS collection_papers (
    collection_id TEXT NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
    paper_id      TEXT NOT NULL REFERENCES papers(id) ON DELETE CASCADE,
    date_added    TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (collection_id, paper_id)
);

CREATE INDEX IF NOT EXISTS idx_coll_papers_collection ON collection_papers(collection_id);
CREATE INDEX IF NOT EXISTS idx_coll_papers_paper ON collection_papers(paper_id);

-- ── AI analysis results ─────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS ai_results (
    id              TEXT PRIMARY KEY,
    result_type     TEXT NOT NULL,          -- 'review' | 'comparison' | 'baseline'
    collection_id   TEXT REFERENCES collections(id) ON DELETE SET NULL,
    title           TEXT NOT NULL,
    content         TEXT NOT NULL,           -- Markdown output
    structured_data TEXT,                    -- JSON for frontend table rendering (baseline)
    model_used      TEXT,                    -- e.g. 'llama3', 'gpt-4o'
    paper_ids       TEXT NOT NULL,           -- JSON array of paper IDs
    dimensions      TEXT,                    -- JSON array of comparison dimensions
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_ai_results_type ON ai_results(result_type);
CREATE INDEX IF NOT EXISTS idx_ai_results_collection ON ai_results(collection_id);
"#;

// ── M016: Remove default system folders (all/favorites/unread/recently-added) ──
// These are now handled as UI-level filter views, not database records.

static M016_REMOVE_DEFAULT_FOLDERS: &str = r#"
-- Remove legacy system default folders that are now handled in the UI layer
DELETE FROM folders WHERE id IN ('all', 'favorites', 'unread', 'recently-added');
"#;
