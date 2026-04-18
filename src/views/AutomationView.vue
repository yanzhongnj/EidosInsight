<script setup lang="ts">
import { onMounted, computed, ref } from 'vue'
import { useAutomationStore } from '../stores/automation'
import { useSkillStore } from '../stores/skill'
import type { ScheduleConfig, AutoTask, PresetTemplate } from '../types'

const automationStore = useAutomationStore()
const skillStore = useSkillStore()

const activeTab = ref<'tasks' | 'skills' | 'history'>('tasks')
const showEnableDialog = ref(false)
const showSkillCreator = ref(false)
const selectedPreset = ref<PresetTemplate | null>(null)

// Schedule form state
const scheduleForm = ref<ScheduleConfig>({
  frequency: 'daily',
  time: 'morning_8',
})
const configForm = ref<Record<string, string>>({})

// Task detail panel
const showTaskDetail = ref(false)
const detailTask = ref<AutoTask | null>(null)

onMounted(async () => {
  await Promise.all([
    automationStore.fetchPresets(),
    automationStore.fetchTasks(),
    skillStore.fetchSkills(),
  ])
})

const enabledTasks = computed(() => automationStore.tasks.filter(t => t.is_enabled))
const disabledPresets = computed(() => {
  const enabledPresetIds = new Set(enabledTasks.value.map(t => t.preset_id).filter(Boolean))
  return automationStore.presets.filter(p => !enabledPresetIds.has(p.preset_id))
})

function openEnableDialog(preset: PresetTemplate) {
  selectedPreset.value = preset
  configForm.value = {}
  preset.config_fields.forEach(f => {
    configForm.value[f.key] = f.default_value || ''
  })
  scheduleForm.value = { frequency: 'daily', time: 'morning_8' }
  showEnableDialog.value = true
}

async function confirmEnable() {
  if (!selectedPreset.value) return
  try {
    await automationStore.enablePreset(
      selectedPreset.value.preset_id,
      selectedPreset.value.name,
      selectedPreset.value.definition,
      scheduleForm.value,
      configForm.value,
    )
    showEnableDialog.value = false
  } catch (e) {
    console.error('Enable failed:', e)
  }
}

function openTaskDetail(task: AutoTask) {
  detailTask.value = task
  showTaskDetail.value = true
  automationStore.selectTask(task.id)
}

async function toggleTaskEnabled(task: AutoTask) {
  if (task.is_enabled) {
    await automationStore.disableTask(task.id)
  } else {
    // Re-enable: update schedule
    const schedule: ScheduleConfig = JSON.parse(task.schedule)
    await automationStore.updateSchedule(task.id, schedule)
  }
}

async function handleRunNow(task: AutoTask) {
  try {
    await automationStore.runNow(task.id)
  } catch { /* handled in store */ }
}

function formatTime(dateStr: string | null) {
  if (!dateStr) return '-'
  try {
    return new Date(dateStr).toLocaleString('zh-CN')
  } catch {
    return dateStr
  }
}

function getScheduleLabel(scheduleStr: string) {
  try {
    const cfg: ScheduleConfig = JSON.parse(scheduleStr)
    const timeMap: Record<string, string> = {
      morning_8: '08:00',
      noon_12: '12:00',
      evening_8: '20:00',
      custom: `${String(cfg.custom_hour ?? 8).padStart(2, '0')}:${String(cfg.custom_minute ?? 0).padStart(2, '0')}`,
    }
    const time = timeMap[cfg.time] || '08:00'
    if (cfg.frequency === 'daily') return `每天 ${time}`
    if (cfg.frequency === 'weekly') {
      const dayNames = ['日', '一', '二', '三', '四', '五', '六']
      const days = (cfg.weekdays || [1]).map(d => dayNames[d] || d).join('、')
      return `每周${days} ${time}`
    }
    if (cfg.custom_interval_hours) return `每隔${cfg.custom_interval_hours}小时`
    return time
  } catch {
    return scheduleStr
  }
}

const sourceLabels: Record<string, string> = {
  builtin: '内置',
  user_created: '自建',
  ai_generated: 'AI 生成',
  community: '社区导入',
}

function toggleWeekday(dayIndex: number) {
  const arr = scheduleForm.value.weekdays || []
  const idx = arr.indexOf(dayIndex)
  if (idx >= 0) {
    arr.splice(idx, 1)
  } else {
    arr.push(dayIndex)
  }
  scheduleForm.value.weekdays = [...arr]
}
</script>

<template>
  <div class="automation-view">
    <div class="page-header">
      <h1>自动化</h1>
      <p class="page-desc">预设任务定时执行，技能一键复用科研流程</p>
    </div>

    <!-- Tab navigation -->
    <div class="tab-bar">
      <button
        v-for="tab in (['tasks', 'skills', 'history'] as const)"
        :key="tab"
        class="tab-btn"
        :class="{ active: activeTab === tab }"
        @click="activeTab = tab"
      >
        {{ tab === 'tasks' ? '任务' : tab === 'skills' ? '技能' : '执行历史' }}
        <span v-if="tab === 'tasks' && enabledTasks.length" class="tab-badge">{{ enabledTasks.length }}</span>
        <span v-if="tab === 'skills' && skillStore.skills.length" class="tab-badge">{{ skillStore.skills.filter(s => s.is_active).length }}</span>
      </button>
    </div>

    <!-- ── Tasks Tab ──────────────────────────────────────────────────────── -->
    <div v-if="activeTab === 'tasks'" class="tab-content">
      <!-- Enabled tasks -->
      <section v-if="enabledTasks.length" class="section">
        <h2 class="section-title">已启用的任务</h2>
        <div class="task-grid">
          <div
            v-for="task in enabledTasks"
            :key="task.id"
            class="task-card enabled"
            @click="openTaskDetail(task)"
          >
            <div class="task-card-header">
              <span class="task-icon">{{ task.task_type === 'preset' ? '📋' : '⚙️' }}</span>
              <h3>{{ task.name }}</h3>
              <span class="status-dot active" title="已启用"></span>
            </div>
            <div class="task-meta">
              <span class="meta-item">{{ getScheduleLabel(task.schedule) }}</span>
              <span class="meta-item" v-if="task.last_run">上次：{{ formatTime(task.last_run) }}</span>
              <span class="meta-item" v-if="task.next_run">下次：{{ formatTime(task.next_run) }}</span>
            </div>
            <div class="task-actions">
              <button class="btn-sm btn-primary" @click.stop="handleRunNow(task)">立即执行</button>
              <button class="btn-sm btn-ghost" @click.stop="toggleTaskEnabled(task)">停用</button>
            </div>
          </div>
        </div>
      </section>

      <!-- Available presets -->
      <section class="section">
        <h2 class="section-title">可用任务模板</h2>
        <div class="preset-grid">
          <div
            v-for="preset in disabledPresets"
            :key="preset.preset_id"
            class="preset-card"
          >
            <div class="preset-header">
              <h3>{{ preset.name }}</h3>
            </div>
            <p class="preset-desc">{{ preset.description }}</p>
            <button class="btn-sm btn-primary" @click="openEnableDialog(preset)">启用</button>
          </div>
        </div>
      </section>
    </div>

    <!-- ── Skills Tab ─────────────────────────────────────────────────────── -->
    <div v-if="activeTab === 'skills'" class="tab-content">
      <div class="section-header-row">
        <h2 class="section-title">技能列表</h2>
        <button class="btn-sm btn-primary" @click="showSkillCreator = true">+ 创建技能</button>
      </div>

      <div v-if="skillStore.skills.length" class="skill-grid">
        <div
          v-for="skill in skillStore.skills"
          :key="skill.id"
          class="skill-card"
          :class="{ inactive: !skill.is_active }"
        >
          <div class="skill-header">
            <h3>{{ skill.name }}</h3>
            <span class="skill-source">{{ sourceLabels[skill.source] || skill.source }}</span>
          </div>
          <div class="skill-meta">
            <span v-if="skill.trigger" class="meta-item trigger">触发：{{ skill.trigger }}</span>
            <span class="meta-item">v{{ skill.version }}</span>
          </div>
          <div class="skill-actions">
            <button
              class="btn-sm btn-primary"
              :disabled="!skill.is_active || skillStore.executing"
              @click="skillStore.invokeSkill(skill.id)"
            >
              执行
            </button>
            <button class="btn-sm btn-ghost" @click="skillStore.toggleSkill(skill.id, !skill.is_active)">
              {{ skill.is_active ? '停用' : '启用' }}
            </button>
            <button
              v-if="skill.source !== 'builtin'"
              class="btn-sm btn-danger"
              @click="skillStore.removeSkill(skill.id)"
            >
              删除
            </button>
          </div>
        </div>
      </div>
      <div v-else class="empty-state">
        <p>还没有技能，点击右上角创建或导入</p>
      </div>
    </div>

    <!-- ── History Tab ────────────────────────────────────────────────────── -->
    <div v-if="activeTab === 'history'" class="tab-content">
      <div class="section">
        <h2 class="section-title">最近执行记录</h2>
        <div v-if="automationStore.executionHistory.length" class="history-list">
          <div
            v-for="exec in automationStore.executionHistory"
            :key="exec.id"
            class="history-item"
            :class="exec.status"
          >
            <div class="history-header">
              <span class="history-status" :class="exec.status">
                {{ exec.status === 'success' ? '成功' : exec.status === 'failed' ? '失败' : '运行中' }}
              </span>
              <span class="history-time">{{ formatTime(exec.started_at) }}</span>
              <span v-if="exec.duration_ms" class="history-duration">{{ exec.duration_ms }}ms</span>
            </div>
            <p v-if="exec.result" class="history-result">{{ exec.result }}</p>
            <p v-if="exec.error" class="history-error">{{ exec.error }}</p>
          </div>
        </div>
        <div v-else class="empty-state">
          <p>选择一个任务查看执行记录</p>
        </div>
      </div>
    </div>

    <!-- ── Enable Task Dialog ─────────────────────────────────────────────── -->
    <div v-if="showEnableDialog" class="dialog-overlay" @click.self="showEnableDialog = false">
      <div class="dialog">
        <h2>启用「{{ selectedPreset?.name }}」</h2>
        <p class="dialog-desc">{{ selectedPreset?.description }}</p>

        <!-- Config fields -->
        <div v-if="selectedPreset?.config_fields?.length" class="form-section">
          <h3>配置</h3>
          <div v-for="field in selectedPreset.config_fields" :key="field.key" class="form-group">
            <label>{{ field.label }}<span v-if="field.required" class="required">*</span></label>
            <input
              v-if="field.field_type === 'text'"
              v-model="configForm[field.key]"
              class="form-input"
              :placeholder="field.label"
            />
            <select
              v-else-if="field.field_type === 'select'"
              v-model="configForm[field.key]"
              class="form-input"
            >
              <option v-for="opt in field.options" :key="opt" :value="opt">{{ opt }}</option>
            </select>
          </div>
        </div>

        <!-- Schedule config -->
        <div class="form-section">
          <h3>推送时间</h3>
          <div class="form-group">
            <label>频率</label>
            <select v-model="scheduleForm.frequency" class="form-input">
              <option value="daily">每天</option>
              <option value="weekly">每周</option>
              <option value="custom">自定义</option>
            </select>
          </div>
          <div v-if="scheduleForm.frequency !== 'custom'" class="form-group">
            <label>时间</label>
            <select v-model="scheduleForm.time" class="form-input">
              <option value="morning_8">早上 8 点</option>
              <option value="noon_12">中午 12 点</option>
              <option value="evening_8">晚上 8 点</option>
            </select>
          </div>
          <div v-if="scheduleForm.frequency === 'weekly'" class="form-group">
            <label>推送日</label>
            <div class="weekday-picker">
              <button
                v-for="(d, i) in ['日','一','二','三','四','五','六']"
                :key="i"
                class="weekday-btn"
                :class="{ active: scheduleForm.weekdays?.includes(i) }"
                @click="toggleWeekday(i)"
              >{{ d }}</button>
            </div>
          </div>
          <div v-if="scheduleForm.frequency === 'custom'" class="form-group">
            <label>间隔（小时）</label>
            <input v-model.number="scheduleForm.custom_interval_hours" type="number" min="1" class="form-input" />
          </div>
        </div>

        <div class="dialog-actions">
          <button class="btn btn-ghost" @click="showEnableDialog = false">取消</button>
          <button class="btn btn-primary" @click="confirmEnable">启用</button>
        </div>
      </div>
    </div>

    <!-- ── Skill Creator Dialog ───────────────────────────────────────────── -->
    <div v-if="showSkillCreator" class="dialog-overlay" @click.self="showSkillCreator = false">
      <div class="dialog dialog-wide">
        <SkillCreator
          @save="(name: string, def: string, trigger: string) => {
            skillStore.createSkill(name, def, 'user_created', trigger || undefined)
            showSkillCreator = false
          }"
          @cancel="showSkillCreator = false"
        />
      </div>
    </div>

    <!-- ── Task Detail Panel ──────────────────────────────────────────────── -->
    <div v-if="showTaskDetail && detailTask" class="dialog-overlay" @click.self="showTaskDetail = false">
      <div class="dialog">
        <h2>{{ detailTask.name }}</h2>
        <div class="detail-section">
          <div class="detail-row"><span class="detail-label">类型</span><span>{{ detailTask.task_type === 'preset' ? '预设' : '自定义' }}</span></div>
          <div class="detail-row"><span class="detail-label">计划</span><span>{{ getScheduleLabel(detailTask.schedule) }}</span></div>
          <div class="detail-row"><span class="detail-label">状态</span><span :class="detailTask.is_enabled ? 'text-success' : 'text-muted'">{{ detailTask.is_enabled ? '已启用' : '已停用' }}</span></div>
          <div class="detail-row"><span class="detail-label">上次执行</span><span>{{ formatTime(detailTask.last_run) }}</span></div>
          <div class="detail-row"><span class="detail-label">下次执行</span><span>{{ formatTime(detailTask.next_run) }}</span></div>
        </div>
        <div v-if="automationStore.executionHistory.length" class="detail-section">
          <h3>最近执行</h3>
          <div v-for="exec in automationStore.executionHistory.slice(0, 5)" :key="exec.id" class="history-item" :class="exec.status">
            <span class="history-status" :class="exec.status">{{ exec.status === 'success' ? '成功' : exec.status === 'failed' ? '失败' : '运行中' }}</span>
            <span class="history-time">{{ formatTime(exec.started_at) }}</span>
            <span v-if="exec.duration_ms">{{ exec.duration_ms }}ms</span>
          </div>
        </div>
        <div class="dialog-actions">
          <button class="btn btn-ghost" @click="showTaskDetail = false">关闭</button>
          <button class="btn btn-primary" @click="handleRunNow(detailTask); showTaskDetail = false">立即执行</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import SkillCreator from '../components/automation/SkillCreator.vue'
export default { components: { SkillCreator } }
</script>

<style scoped>
.automation-view { max-width: 960px; margin: 0 auto; }
.page-header { margin-bottom: 24px; }
.page-header h1 { font-size: 24px; font-weight: 700; margin: 0 0 4px; color: var(--text-primary); }
.page-desc { color: var(--text-secondary); font-size: 14px; margin: 0; }

/* Tabs */
.tab-bar { display: flex; gap: 4px; margin-bottom: 24px; border-bottom: 1px solid var(--border-primary); padding-bottom: 0; }
.tab-btn { padding: 8px 16px; border: none; background: transparent; cursor: pointer; font-size: 14px; color: var(--text-secondary); border-bottom: 2px solid transparent; transition: all .15s; position: relative; }
.tab-btn.active { color: var(--color-primary); border-bottom-color: var(--color-primary); font-weight: 500; }
.tab-btn:hover { color: var(--text-primary); }
.tab-badge { font-size: 11px; background: var(--color-primary-light); color: var(--color-primary); padding: 1px 6px; border-radius: 9999px; margin-left: 4px; }

/* Sections */
.section { margin-bottom: 24px; }
.section-title { font-size: 16px; font-weight: 600; color: var(--text-primary); margin: 0 0 12px; }
.section-header-row { display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px; }
.section-header-row .section-title { margin: 0; }

/* Task grid */
.task-grid, .preset-grid, .skill-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 12px; }

/* Task card */
.task-card, .preset-card, .skill-card {
  background: var(--bg-primary);
  border: 1px solid var(--border-primary);
  border-radius: var(--radius-md);
  padding: 16px;
  cursor: pointer;
  transition: all .15s;
}
.task-card:hover, .preset-card:hover, .skill-card:hover { border-color: var(--color-primary); box-shadow: 0 2px 8px rgba(0,0,0,.06); }
.task-card.enabled { border-left: 3px solid var(--color-primary); }
.task-card-header, .preset-header, .skill-header { display: flex; align-items: center; gap: 8px; margin-bottom: 8px; }
.task-card-header h3, .preset-header h3, .skill-header h3 { font-size: 14px; font-weight: 600; margin: 0; flex: 1; }
.task-icon { font-size: 16px; }
.status-dot { width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0; }
.status-dot.active { background: var(--color-success, #10b981); }
.task-meta, .skill-meta { font-size: 12px; color: var(--text-tertiary); margin-bottom: 8px; }
.meta-item { margin-right: 12px; }
.meta-item.trigger { font-family: monospace; background: var(--bg-tertiary); padding: 1px 4px; border-radius: 3px; }
.preset-desc { font-size: 13px; color: var(--text-secondary); margin: 0 0 12px; }
.skill-source { font-size: 11px; padding: 1px 6px; border-radius: 4px; background: var(--bg-tertiary); color: var(--text-tertiary); }

/* Buttons */
.task-actions, .skill-actions { display: flex; gap: 6px; }
.btn-sm { font-size: 12px; padding: 4px 10px; border-radius: var(--radius-sm); border: 1px solid var(--border-primary); background: transparent; cursor: pointer; transition: all .15s; }
.btn-sm.btn-primary { background: var(--color-primary); color: white; border-color: var(--color-primary); }
.btn-sm.btn-primary:hover { opacity: .85; }
.btn-sm.btn-ghost { color: var(--text-secondary); }
.btn-sm.btn-ghost:hover { background: var(--bg-hover); }
.btn-sm.btn-danger { color: var(--color-danger, #ef4444); border-color: var(--color-danger, #ef4444); }
.btn-sm:disabled { opacity: .4; cursor: not-allowed; }
.btn { padding: 8px 16px; border-radius: var(--radius-md); border: 1px solid var(--border-primary); background: transparent; cursor: pointer; font-size: 14px; transition: all .15s; }
.btn.btn-primary { background: var(--color-primary); color: white; border-color: var(--color-primary); }
.btn.btn-ghost { color: var(--text-secondary); }
.btn.btn-ghost:hover { background: var(--bg-hover); }

/* Dialog */
.dialog-overlay { position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,.4); display: flex; align-items: center; justify-content: center; z-index: 100; }
.dialog { background: var(--bg-primary); border-radius: var(--radius-lg); padding: 24px; min-width: 400px; max-width: 560px; max-height: 80vh; overflow-y: auto; }
.dialog-wide { max-width: 720px; }
.dialog h2 { font-size: 18px; font-weight: 600; margin: 0 0 8px; }
.dialog-desc { color: var(--text-secondary); font-size: 13px; margin: 0 0 16px; }
.dialog-actions { display: flex; justify-content: flex-end; gap: 8px; margin-top: 16px; }

/* Forms */
.form-section { margin-bottom: 16px; }
.form-section h3 { font-size: 14px; font-weight: 600; margin: 0 0 8px; }
.form-group { margin-bottom: 10px; }
.form-group label { display: block; font-size: 13px; color: var(--text-secondary); margin-bottom: 4px; }
.required { color: var(--color-danger, #ef4444); }
.form-input { width: 100%; padding: 6px 10px; border: 1px solid var(--border-primary); border-radius: var(--radius-sm); background: var(--bg-primary); color: var(--text-primary); font-size: 13px; }
.form-input:focus { outline: none; border-color: var(--color-primary); }

/* Weekday picker */
.weekday-picker { display: flex; gap: 4px; }
.weekday-btn { width: 32px; height: 32px; border: 1px solid var(--border-primary); border-radius: var(--radius-sm); background: transparent; cursor: pointer; font-size: 12px; color: var(--text-secondary); }
.weekday-btn.active { background: var(--color-primary); color: white; border-color: var(--color-primary); }

/* History */
.history-list { display: flex; flex-direction: column; gap: 8px; }
.history-item { padding: 10px 12px; border-radius: var(--radius-sm); border: 1px solid var(--border-primary); background: var(--bg-primary); }
.history-item.success { border-left: 3px solid var(--color-success, #10b981); }
.history-item.failed { border-left: 3px solid var(--color-danger, #ef4444); }
.history-item.running { border-left: 3px solid var(--color-primary); }
.history-header { display: flex; gap: 8px; align-items: center; font-size: 13px; }
.history-status { font-weight: 600; font-size: 12px; }
.history-status.success { color: var(--color-success, #10b981); }
.history-status.failed { color: var(--color-danger, #ef4444); }
.history-status.running { color: var(--color-primary); }
.history-time { color: var(--text-tertiary); }
.history-duration { color: var(--text-tertiary); font-size: 11px; }
.history-result { font-size: 12px; color: var(--text-secondary); margin: 4px 0 0; }
.history-error { font-size: 12px; color: var(--color-danger, #ef4444); margin: 4px 0 0; }

/* Detail */
.detail-section { margin-bottom: 16px; }
.detail-row { display: flex; justify-content: space-between; padding: 4px 0; font-size: 13px; }
.detail-label { color: var(--text-tertiary); }
.text-success { color: var(--color-success, #10b981); }
.text-muted { color: var(--text-tertiary); }

/* Empty */
.empty-state { text-align: center; padding: 40px; color: var(--text-tertiary); }

/* Inactive skill */
.skill-card.inactive { opacity: .6; }
</style>
