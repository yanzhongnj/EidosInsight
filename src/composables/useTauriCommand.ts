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

  const importPdf = () =>
    call<import('../types').Paper>('import_pdf')

  const importPdfFromPath = (sourcePath: string) =>
    call<import('../types').Paper>('import_pdf_from_path', { sourcePath })

  const batchImportPdfs = () =>
    call<import('../types').Paper[]>('batch_import_pdfs')

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

  const addPaperTag = (paperId: string, tagName: string) =>
    call<void>('add_paper_tag', { paperId, tagName })

  const removePaperTag = (paperId: string, tagId: string) =>
    call<void>('remove_paper_tag', { paperId, tagId })

  const readPdfFile = (paperId: string) =>
    call<string>('read_pdf_file', { paperId })

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
  const listTags = () =>
    call<import('../types').Tag[]>('list_tags')

  const renameTag = (id: string, newName: string) =>
    call<void>('rename_tag', { id, newName })

  const deleteTag = (id: string) =>
    call<void>('delete_tag', { id })

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

  return {
    call,
    importPaper, importPdf, importPdfFromPath, batchImportPdfs, getPaper, listPapers, updatePaper, deletePaper,
    movePaperToFolder, addPaperTag, removePaperTag, readPdfFile,
    searchPapers, searchAdvanced, searchSuggest,
    createFolder, listFolders, updateFolder, deleteFolder,
    listTags, renameTag, deleteTag,
    createNote, getNote, listNotes, updateNote, deleteNote,
    createNoteLink, deleteNoteLink, getNoteWithLinks, listOutgoingLinks, listBacklinks,
    searchNotes, noteLinkSuggest,
    getReadingState, updateReadingState,
    exportPapers,
    getSettings, updateSettings, getLlmConfig, updateLlmConfig, testLlmConnection,
    isFirstLaunch, completeOnboarding, loadSampleData,
    detectOllama, getLlmStatus, listLlmModels, llmComplete, llmChatStream, saveApiKey, removeApiKey, configureLlmProvider, aiWizardDemo,
    getExtensionStatus, registerNativeHost, importFromBrowser,
    aiExplainTerm, aiSummarize, aiExtractInsights, aiCheckAvailability,
    listPresetTasks, listAutoTasks, enableTask, disableTask, updateTaskSchedule, updateTaskConfig, deleteTask, getTaskHistory, runTaskNow,
    listSkills, createSkill, importSkill, updateSkill, deleteSkill, invokeSkill, scheduleToCron, scheduleDescription,
  }
}
