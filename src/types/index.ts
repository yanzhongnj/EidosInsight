//! TypeScript type definitions matching Rust models

// Paper
export interface Paper {
  id: string
  title: string
  authors: string | null
  abstract_text: string | null
  publication_date: string | null
  journal: string | null
  volume: string | null
  issue: string | null
  pages: string | null
  doi: string | null
  arxiv_id: string | null
  pdf_url: string | null
  pdf_path: string | null
  folder_id: string | null
  tldr: string | null
  reading_status: 'unread' | 'reading' | 'read'
  is_favorite: boolean
  date_added: string
  date_modified: string
  cover_image: string | null
}

export interface CreatePaperInput {
  title: string
  authors?: string | null
  abstract_text?: string | null
  publication_date?: string | null
  journal?: string | null
  volume?: string | null
  issue?: string | null
  pages?: string | null
  doi?: string | null
  arxiv_id?: string | null
  pdf_url?: string | null
  pdf_path?: string | null
  folder_id?: string | null
}

export interface UpdatePaperInput {
  id: string
  title?: string | null
  authors?: string | null
  abstract_text?: string | null
  publication_date?: string | null
  journal?: string | null
  volume?: string | null
  issue?: string | null
  pages?: string | null
  doi?: string | null
  arxiv_id?: string | null
  pdf_url?: string | null
  folder_id?: string | null
  tldr?: string | null
  reading_status?: 'unread' | 'reading' | 'read' | null
  is_favorite?: boolean | null
  cover_image?: string | null
}

export interface PaperFilter {
  folder_id?: string | null
  tag?: string | null
  reading_status?: string | null
  is_favorite?: boolean | null
  query?: string | null
  sort_by?: string | null
  sort_order?: string | null
  limit?: number | null
  offset?: number | null
}

export interface PaperListResult {
  papers: Paper[]
  total: number
}

// Folder
export interface Folder {
  id: string
  name: string
  parent_id: string | null
  color: string | null
  icon: string | null
  sort_order: number
  date_created: string
  date_modified: string
}

export interface CreateFolderInput {
  name: string
  parent_id?: string | null
  color?: string | null
  icon?: string | null
}

export interface UpdateFolderInput {
  id: string
  name?: string | null
  parent_id?: string | null
  color?: string | null
  icon?: string | null
  sort_order?: number | null
}

// Tag
export interface Tag {
  id: string
  name: string
  color: string | null
  usage_count: number
  date_created: string
}

// Note
export interface Note {
  id: string
  paper_id: string | null
  title: string
  content: string
  note_type: 'general' | 'annotation' | 'excerpt' | 'summary'
  page_number: number | null
  position: string | null
  date_created: string
  date_modified: string
}

export interface CreateNoteInput {
  paper_id?: string | null
  title: string
  content: string
  note_type?: string | null
  page_number?: number | null
  position?: string | null
}

export interface UpdateNoteInput {
  id: string
  title?: string | null
  content?: string | null
  note_type?: string | null
  page_number?: number | null
  position?: string | null
}

// ── Bidirectional note links ────────────────────────────────────────────────

export interface NoteLink {
  source_id: string
  target_id: string
  label: string | null
  date_created: string
}

export interface CreateNoteLinkInput {
  source_id: string
  target_id: string
  label?: string | null
}

export interface NoteLinkTarget {
  note_id: string
  title: string
  label: string | null
}

export interface NoteWithLinks {
  note: Note
  outgoing_links: NoteLinkTarget[]
  backlinks: NoteLinkTarget[]
}

// ── Note full-text search ───────────────────────────────────────────────────

export interface NoteSearchResultItem {
  note: Note
  score: number
  /** HTML snippet with <mark> tags wrapping matched terms */
  highlight: string | null
}

export interface NoteSearchResult {
  items: NoteSearchResultItem[]
  total: number
  query: string
}

// Reading state
export interface ReadingState {
  id: string
  paper_id: string
  current_page: number
  total_pages: number
  scroll_position: number
  zoom_level: number
  last_read_at: string
  reading_progress: number
}

export interface UpdateReadingStateInput {
  paper_id: string
  current_page?: number | null
  total_pages?: number | null
  scroll_position?: number | null
  zoom_level?: number | null
}

// Search
export interface SearchResult {
  papers: SearchResultItem[]
  total: number
  query: string
  search_type: string
}

export interface SearchResultItem {
  paper: Paper
  score: number
  /** HTML snippet with <mark> tags wrapping matched terms */
  highlight: string | null
}

export interface SearchFilters {
  query: string
  reading_status?: string | null
  folder_id?: string | null
  tag?: string | null
  is_favorite?: boolean | null
  year_from?: string | null
  year_to?: string | null
  /** 'all' | 'title' | 'abstract' | 'authors' */
  search_field?: string | null
  limit?: number | null
  offset?: number | null
}

export interface SuggestionItem {
  id: string
  title: string
  authors: string | null
  publication_date: string | null
}

export interface SuggestResult {
  items: SuggestionItem[]
  query: string
}

// Settings
export interface AppSettings {
  theme: 'light' | 'dark' | 'system'
  language: string
  auto_process_papers: boolean
  default_llm_provider: string
  check_updates_on_start: boolean
  paper_storage_path: string | null
  max_backup_count: number
  export_format: string
}

export interface LlmConfig {
  provider: string
  model_name: string
  api_base_url: string | null
  api_key_set: boolean
  max_tokens: number
  temperature: number
  is_local: boolean
}

// LLM adapter types
export interface LlmMessage {
  role: 'system' | 'user' | 'assistant' | 'tool'
  content: string
}

export interface LlmCompletionOptions {
  model?: string
  temperature?: number
  max_tokens?: number
  top_p?: number
}

export interface LlmStreamChunk {
  content: string
  done: boolean
}

export interface LlmModelInfo {
  id: string
  name: string
  size_bytes: number | null
  quantization: string | null
}

export interface OllamaDetectionResult {
  available: boolean
  models: LlmModelInfo[]
}

export interface LlmStatusResult {
  status: 'local_available' | 'cloud_available' | 'unavailable'
  has_cloud: boolean
}

// Browser extension types
export interface ExtensionStatus {
  registered: boolean
}

export interface BrowserMetadata {
  title?: string
  authors?: string
  abstract?: string
  url?: string
  doi?: string
  pdf_url?: string
  arxiv_id?: string
  published_date?: string
  journal?: string
  subjects?: string
  source?: string
}

// ── AI Assistant types ────────────────────────────────────────────────────────

/** Streamed AI event chunk (task-scoped, replaces bare llm-stream) */
export interface AiStreamChunk {
  task_id: string
  content: string
}

/** Fired when a specific AI task finishes */
export interface AiDonePayload {
  task_id: string
}

/** Fired when a specific AI task errors */
export interface AiErrorPayload {
  task_id: string
  message: string
}

/** Availability status of the LLM backend */
export interface AiAvailability {
  available: boolean
  /** "ollama" | "cloud" | "none" */
  provider: string
}

export type AiTaskType = 'explain' | 'summarize' | 'insights'

export type AiSummaryStyle = 'brief' | 'structured' | 'eli5'

export type AiInsightFocus = 'contributions' | 'methodology' | 'gaps' | 'all'

// ── AI Config Wizard types ──────────────────────────────────────────────────

export type WizardStep = 'detect' | 'select' | 'test' | 'demo' | 'done'

export interface WizardDemoResult {
  success: boolean
  /** Short demo text generated by the LLM */
  demo_text: string
  /** Provider that answered */
  provider: string
  /** Model that answered */
  model: string
  /** Time in ms the request took */
  latency_ms: number
  /** Error message if success == false */
  error: string | null
}

/** Per-task streaming state tracked by useAiAssist */
export interface AiTask {
  task_id: string
  type: AiTaskType
  streaming: boolean
  content: string
  error: string | null
  /** ISO timestamp when task started */
  started_at: string
  /** ISO timestamp when task completed (null while in progress) */
  completed_at: string | null
}

// ── Automation & Skills types ───────────────────────────────────────────────

export interface AutoTask {
  id: string
  name: string
  task_type: 'preset' | 'custom'
  preset_id: string | null
  definition: string
  schedule: string
  schedule_cron: string | null
  is_enabled: boolean
  last_run: string | null
  next_run: string | null
  config: string | null
  created_at: string
  updated_at: string
}

export interface TaskExecution {
  id: string
  task_id: string
  status: 'running' | 'success' | 'failed'
  started_at: string
  finished_at: string | null
  result: string | null
  error: string | null
  duration_ms: number | null
}

export interface Skill {
  id: string
  name: string
  definition: string
  source: 'builtin' | 'user_created' | 'ai_generated' | 'community'
  version: string
  trigger: string | null
  is_active: boolean
  created_at: string
  updated_at: string
}

export interface SkillResult {
  success: boolean
  output: string
  duration_ms: number
  error: string | null
}

export interface ScheduleConfig {
  frequency: 'daily' | 'weekly' | 'custom'
  time: 'morning_8' | 'noon_12' | 'evening_8' | 'custom'
  weekdays?: number[]
  custom_interval_hours?: number
  custom_hour?: number
  custom_minute?: number
}

export interface PresetTemplate {
  preset_id: string
  name: string
  description: string
  definition: string
  default_cron: string
  config_fields: PresetConfigField[]
}

export interface PresetConfigField {
  key: string
  label: string
  field_type: 'text' | 'select' | 'multiselect'
  required: boolean
  default_value: string | null
  options: string[] | null
}
