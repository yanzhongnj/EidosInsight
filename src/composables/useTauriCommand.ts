//! Tauri Command composable - unified invoke wrapper

import { invoke } from '@tauri-apps/api/core'

/**
 * Call a Tauri backend command with type safety.
 * All frontend-to-backend communication goes through this composable.
 */
export function useTauriCommand() {
  async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
    try {
      return await invoke<T>(command, args)
    } catch (error) {
      console.error(`[Tauri Command Error] ${command}:`, error)
      throw error
    }
  }

  // Paper commands
  const importPaper = (input: Record<string, unknown>) =>
    call<import('../types').Paper>('import_paper', { input })

  const importPdf = (folderId?: string | null) =>
    call<import('../types').Paper>('import_pdf', { folderId: folderId ?? null })

  const importPdfFromPath = (sourcePath: string, folderId?: string | null) =>
    call<import('../types').Paper>('import_pdf_from_path', { sourcePath, folderId: folderId ?? null })

  const batchImportPdfs = (folderId?: string | null) =>
    call<import('../types').Paper[]>('batch_import_pdfs', { folderId: folderId ?? null })

  // Multi-format document import commands
  const importDocument = (folderId?: string | null) =>
    call<import('../types').Paper>('import_document', { folderId: folderId ?? null })

  const importDocumentFromPath = (sourcePath: string, folderId?: string | null) =>
    call<import('../types').Paper>('import_document_from_path', { sourcePath, folderId: folderId ?? null })

  const batchImportDocuments = (folderId?: string | null) =>
    call<import('../types').Paper[]>('batch_import_documents', { folderId: folderId ?? null })

  const getPaper = (id: string) =>
    call<import('../types').Paper>('get_paper', { id })

  const listPapers = (filter?: Record<string, unknown>) =>
    call<import('../types').PaperListResult>('list_papers', { filter })

  const updatePaper = (input: Record<string, unknown>) =>
    call<import('../types').Paper>('update_paper', { input })

  const deletePaper = (id: string) =>
    call<void>('delete_paper', { id })

  const movePaperToFolder = (paperId: string, folderId: string | null) =>
    call<import('../types').Paper>('move_paper_to_folder', { paperId, folderId })

  const getTagsForPaper = (paperId: string) =>
    call<import('../types').Tag[]>('get_tags_for_paper', { paperId })

  const batchGetPaperTags = (paperIds: string[]) =>
    call<Record<string, import('../types').Tag[]>>('batch_get_paper_tags', { paperIds })

  const addPaperTag = (paperId: string, tagName: string) =>
    call<void>('add_paper_tag', { paperId, tagName })

  const removePaperTag = (paperId: string, tagId: string) =>
    call<void>('remove_paper_tag', { paperId, tagId })

  const readPdfFile = (paperId: string) =>
    call<string>('read_pdf_file', { paperId })

  const readDocumentFile = (paperId: string) =>
    call<string>('read_document_file', { paperId })

  // Search commands
  const searchPapers = (query: string, limit?: number) =>
    call<import('../types').SearchResult>('search_papers', { query, limit })

  const searchAdvanced = (filters: import('../types').SearchFilters) =>
    call<import('../types').SearchResult>('search_advanced', { filters })

  const searchSuggest = (query: string) =>
    call<import('../types').SuggestResult>('search_suggest', { query })

  // Folder commands
  const createFolder = (input: Record<string, unknown>) =>
    call<import('../types').Folder>('create_folder', { input })

  const listFolders = () =>
    call<import('../types').Folder[]>('list_folders')

  const updateFolder = (input: Record<string, unknown>) =>
    call<import('../types').Folder>('update_folder', { input })

  const deleteFolder = (id: string) =>
    call<void>('delete_folder', { id })

  // Tag commands
  const createTag = (name: string, color?: string | null) =>
    call<import('../types').Tag>('create_tag', { name, color })

  const listTags = () =>
    call<import('../types').Tag[]>('list_tags')

  const renameTag = (id: string, newName: string) =>
    call<void>('rename_tag', { id, newName })

  const deleteTag = (id: string) =>
    call<void>('delete_tag', { id })

  const updateTagColor = (id: string, color: string) =>
    call<void>('update_tag_color', { id, color })

  const suggestTags = (paperId: string) =>
    call<string[]>('suggest_tags', { paperId })

  // Note commands
  const createNote = (input: Record<string, unknown>) =>
    call<import('../types').Note>('create_note', { input })

  const getNote = (id: string) =>
    call<import('../types').Note>('get_note', { id })

  const listNotes = (paperId?: string, limit?: number, offset?: number) =>
    call<import('../types').Note[]>('list_notes', { paperId, limit, offset })

  const updateNote = (input: Record<string, unknown>) =>
    call<import('../types').Note>('update_note', { input })

  const deleteNote = (id: string) =>
    call<void>('delete_note', { id })

  // Note link commands
  const createNoteLink = (input: import('../types').CreateNoteLinkInput) =>
    call<import('../types').NoteLink>('create_note_link', { input })

  const deleteNoteLink = (sourceId: string, targetId: string) =>
    call<void>('delete_note_link', { sourceId, targetId })

  const getNoteWithLinks = (id: string) =>
    call<import('../types').NoteWithLinks>('get_note_with_links', { id })

  const listOutgoingLinks = (noteId: string) =>
    call<import('../types').NoteLinkTarget[]>('list_outgoing_links', { noteId })

  const listBacklinks = (noteId: string) =>
    call<import('../types').NoteLinkTarget[]>('list_backlinks', { noteId })

  // Note search commands
  const searchNotes = (query: string, paperId?: string, noteType?: string, limit?: number, offset?: number) =>
    call<import('../types').NoteSearchResult>('search_notes', { query, paperId, noteType, limit, offset })

  const noteLinkSuggest = (prefix: string, limit?: number) =>
    call<import('../types').NoteLinkTarget[]>('note_link_suggest', { prefix, limit })

  // Reading commands
  const getReadingState = (paperId: string) =>
    call<import('../types').ReadingState>('get_reading_state', { paperId })

  const updateReadingState = (input: Record<string, unknown>) =>
    call<import('../types').ReadingState>('update_reading_state', { input })

  // Export commands
  const exportPapers = (format: string, filter?: Record<string, unknown>) =>
    call<string>('export_papers', { format, filter })

  // Settings commands
  const getSettings = () =>
    call<import('../types').AppSettings>('get_settings')

  const updateSettings = (settings: Record<string, unknown>) =>
    call<void>('update_settings', { settings })

  const getLlmConfig = () =>
    call<import('../types').LlmConfig>('get_llm_config')

  const updateLlmConfig = (config: Record<string, unknown>) =>
    call<void>('update_llm_config', { config })

  const testLlmConnection = (config: Record<string, unknown>) =>
    call<boolean>('test_llm_connection', { config })

  // Onboarding commands
  const isFirstLaunch = () =>
    call<boolean>('is_first_launch')

  const completeOnboarding = () =>
    call<void>('complete_onboarding')

  const loadSampleData = () =>
    call<void>('load_sample_data')

  // LLM commands
  const getLlmProviderPresets = () =>
    call<import('../types').ProviderPreset[]>('get_llm_provider_presets')

  const listConfiguredProviders = () =>
    call<import('../types').ConfiguredProvider[]>('list_configured_providers')

  const switchActiveProvider = (provider: string) =>
    call<void>('switch_active_provider', { provider })

  const detectOllama = () =>
    call<import('../types').OllamaDetectionResult>('detect_ollama')

  const getLlmStatus = () =>
    call<import('../types').LlmStatusResult>('get_llm_status')

  const listLlmModels = () =>
    call<import('../types').LlmModelInfo[]>('list_llm_models')

  const llmComplete = (messages: import('../types').LlmMessage[], options?: import('../types').LlmCompletionOptions) =>
    call<string>('llm_complete', { messages, options })

  const llmChatStream = (messages: import('../types').LlmMessage[], options?: import('../types').LlmCompletionOptions) =>
    call<void>('llm_chat_stream', { messages, options })

  const saveApiKey = (provider: string, apiKey: string) =>
    call<void>('save_api_key', { provider, apiKey })

  const removeApiKey = (provider: string) =>
    call<void>('remove_api_key', { provider })

  const configureLlmProvider = (config: import('../types').LlmConfig) =>
    call<void>('configure_llm_provider', { config })

  const aiWizardDemo = () =>
    call<import('../types').WizardDemoResult>('ai_wizard_demo')

  // Browser extension commands
  const getExtensionStatus = () =>
    call<import('../types').ExtensionStatus>('get_extension_status')

  const registerNativeHost = () =>
    call<void>('register_native_host')

  const importFromBrowser = (url: string, metadata: import('../types').BrowserMetadata) =>
    call<import('../types').Paper>('import_from_browser', { url, metadata })

  // AI assistant commands
  const aiExplainTerm = (paperId: string, term: string, context?: string) =>
    call<void>('ai_explain_term', { paperId, term, context })

  const aiSummarize = (paperId: string, style?: string) =>
    call<void>('ai_summarize', { paperId, style })

  const aiExtractInsights = (paperId: string, focus?: string) =>
    call<void>('ai_extract_insights', { paperId, focus })

  const aiCheckAvailability = () =>
    call<import('../types').AiAvailability>('ai_check_availability')

  // Automation commands
  const listPresetTasks = () =>
    call<import('../types').PresetTemplate[]>('list_preset_tasks')

  const listAutoTasks = () =>
    call<import('../types').AutoTask[]>('list_auto_tasks')

  const enableTask = (args: { presetId?: string; name: string; definition: string; schedule: string; scheduleCron?: string; config?: string }) =>
    call<import('../types').AutoTask>('enable_task', {
      presetId: args.presetId,
      name: args.name,
      definition: args.definition,
      schedule: args.schedule,
      scheduleCron: args.scheduleCron,
      config: args.config,
    })

  const disableTask = (id: string) =>
    call<void>('disable_task', { id })

  const updateTaskSchedule = (id: string, schedule: string, scheduleCron?: string) =>
    call<void>('update_task_schedule', { id, schedule, scheduleCron })

  const updateTaskConfig = (id: string, config: string) =>
    call<void>('update_task_config', { id, config })

  const deleteTask = (id: string) =>
    call<void>('delete_task', { id })

  const getTaskHistory = (taskId: string, limit?: number) =>
    call<import('../types').TaskExecution[]>('get_task_history', { taskId, limit })

  const runTaskNow = (id: string) =>
    call<import('../types').TaskExecution>('run_task_now', { id })

  // Skill commands
  const listSkills = () =>
    call<import('../types').Skill[]>('list_skills')

  const createSkill = (name: string, definition: string, source: string, trigger?: string) =>
    call<import('../types').Skill>('create_skill', { name, definition, source, trigger })

  const importSkill = (markdown: string) =>
    call<import('../types').Skill>('import_skill', { markdown })

  const updateSkill = (id: string, args: { name?: string; definition?: string; trigger?: string; isActive?: boolean }) =>
    call<void>('update_skill', {
      id,
      name: args.name,
      definition: args.definition,
      trigger: args.trigger,
      isActive: args.isActive,
    })

  const deleteSkill = (id: string) =>
    call<void>('delete_skill', { id })

  const invokeSkill = (skillId: string, params?: Record<string, unknown>) =>
    call<import('../types').SkillResult>('invoke_skill', { skillId, params })

  const scheduleToCron = (schedule: string) =>
    call<string>('schedule_to_cron', { schedule })

  const scheduleDescription = (schedule: string) =>
    call<string>('schedule_description', { schedule })

  // Crawl / watch directory commands
  const addWatchDir = (input: import('../types').CreateWatchDirInput) =>
    call<import('../types').WatchDir>('add_watch_dir', { input })

  const listWatchDirs = () =>
    call<import('../types').WatchDir[]>('list_watch_dirs')

  const updateWatchDir = (input: import('../types').UpdateWatchDirInput) =>
    call<import('../types').WatchDir>('update_watch_dir', { input })

  const deleteWatchDir = (id: string) =>
    call<void>('delete_watch_dir', { id })

  const scanWatchDir = (id: string) =>
    call<import('../types').CrawlScanResult>('scan_watch_dir', { id })

  const scanAllWatchDirs = () =>
    call<import('../types').CrawlScanResult[]>('scan_all_watch_dirs')

  const getCrawlStatus = () =>
    call<import('../types').WatchDir[]>('get_crawl_status')

  // Collection commands
  const createCollection = (input: import('../types').CreateCollectionInput) =>
    call<import('../types').Collection>('create_collection', { input })

  const getCollection = (id: string) =>
    call<import('../types').Collection>('get_collection', { id })

  const listCollections = () =>
    call<import('../types').Collection[]>('list_collections')

  const updateCollection = (input: import('../types').UpdateCollectionInput) =>
    call<import('../types').Collection>('update_collection', { input })

  const deleteCollection = (id: string) =>
    call<void>('delete_collection', { id })

  const addPaperToCollection = (collectionId: string, paperId: string) =>
    call<void>('add_paper_to_collection', { collectionId, paperId })

  const removePaperFromCollection = (collectionId: string, paperId: string) =>
    call<void>('remove_paper_from_collection', { collectionId, paperId })

  const listCollectionPapers = (collectionId: string) =>
    call<import('../types').Paper[]>('list_collection_papers', { collectionId })

  const addPapersToCollection = (collectionId: string, paperIds: string[]) =>
    call<number>('add_papers_to_collection', { collectionId, paperIds })

  // AI result commands
  const listAiResults = (resultType?: string) =>
    call<import('../types').AiResult[]>('list_ai_results', { resultType })

  const listCollectionAiResults = (collectionId: string) =>
    call<import('../types').AiResult[]>('list_collection_ai_results', { collectionId })

  const getAiResult = (id: string) =>
    call<import('../types').AiResult>('get_ai_result', { id })

  const deleteAiResult = (id: string) =>
    call<void>('delete_ai_result', { id })

  const generateReview = (input: import('../types').GenerateReviewInput) =>
    call<import('../types').AiResult>('generate_review', { input })

  const generateComparison = (input: import('../types').GenerateComparisonInput) =>
    call<import('../types').AiResult>('generate_comparison', { input })

  const generateBaseline = (input: import('../types').GenerateBaselineInput) =>
    call<import('../types').AiResult>('generate_baseline', { input })

  // Update commands
  const checkForUpdates = () =>
    call<import('../composables/useUpdater').UpdateCheckResult>('check_for_updates')

  const getAppVersion = () =>
    call<string>('get_app_version')

  // Stats commands
  const getLibraryStats = () =>
    call<import('../types').LibraryStats>('get_library_stats')

  return {
    call,
    importPaper, importPdf, importPdfFromPath, batchImportPdfs, importDocument, importDocumentFromPath, batchImportDocuments,
    getPaper, listPapers, updatePaper, deletePaper,
    movePaperToFolder, getTagsForPaper, batchGetPaperTags, addPaperTag, removePaperTag, readPdfFile, readDocumentFile,
    searchPapers, searchAdvanced, searchSuggest,
    createFolder, listFolders, updateFolder, deleteFolder,
    listTags, renameTag, deleteTag, createTag, updateTagColor, suggestTags,
    createNote, getNote, listNotes, updateNote, deleteNote,
    createNoteLink, deleteNoteLink, getNoteWithLinks, listOutgoingLinks, listBacklinks,
    searchNotes, noteLinkSuggest,
    getReadingState, updateReadingState,
    exportPapers,
    getSettings, updateSettings, getLlmConfig, updateLlmConfig, testLlmConnection,
    isFirstLaunch, completeOnboarding, loadSampleData,
    detectOllama, getLlmStatus, listLlmModels, llmComplete, llmChatStream, saveApiKey, removeApiKey, configureLlmProvider, aiWizardDemo,
    getLlmProviderPresets, listConfiguredProviders, switchActiveProvider,
    getExtensionStatus, registerNativeHost, importFromBrowser,
    aiExplainTerm, aiSummarize, aiExtractInsights, aiCheckAvailability,
    listPresetTasks, listAutoTasks, enableTask, disableTask, updateTaskSchedule, updateTaskConfig, deleteTask, getTaskHistory, runTaskNow,
    listSkills, createSkill, importSkill, updateSkill, deleteSkill, invokeSkill, scheduleToCron, scheduleDescription,
    addWatchDir, listWatchDirs, updateWatchDir, deleteWatchDir, scanWatchDir, scanAllWatchDirs, getCrawlStatus,
    createCollection, getCollection, listCollections, updateCollection, deleteCollection,
    addPaperToCollection, removePaperFromCollection, listCollectionPapers, addPapersToCollection,
    listAiResults, listCollectionAiResults, getAiResult, deleteAiResult, generateReview, generateComparison, generateBaseline,
    checkForUpdates, getAppVersion, getLibraryStats,
  }
}
