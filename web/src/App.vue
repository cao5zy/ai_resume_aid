<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import {
  optimizeResumeText,
  optimizeResumeFile,
  searchJobs,
  exportPdf,
  fetchConfig,
} from '@/api/client'
import type {
  TargetGroup,
  AppPhase,
  FontMode,
  ContrastMode,
  JobSearchResult,
} from '@/types'
import A11yBar from '@/components/A11yBar.vue'
import HeroSection from '@/components/HeroSection.vue'
import InputSection from '@/components/InputSection.vue'
import ResultsSection from '@/components/ResultsSection.vue'
import AppFooter from '@/components/AppFooter.vue'

// ── State ──────────────────────────────────────────────
const targetGroup = ref<TargetGroup | null>(null)
const inputMethod = ref<'text' | 'file' | null>(null)
const resumeText = ref('')
const selectedFile = ref<File | null>(null)
const fileName = ref('')
const appPhase = ref<AppPhase>('input')
const errorMessage = ref('')
const optimizedText = ref('')
const originalText = ref('')
const encouragement = ref('')
const jobResults = ref<JobSearchResult[]>([])
const showMission = ref(false)
const exportingPdf = ref(false)
const fontMode = ref<FontMode>('normal')
const contrastMode = ref<ContrastMode>('normal')
const uploadMaxSizeMb = ref(10)

// ── Lifecycle ──────────────────────────────────────────
onMounted(async () => {
  try {
    const res = await fetchConfig()
    if (res.success) {
      uploadMaxSizeMb.value = res.data.upload_max_size_mb
    }
  } catch {
    // Use default 10MB if fetch fails
  }
})

// ── Computed ───────────────────────────────────────────
const canSubmit = computed(() => {
  return (
    targetGroup.value !== null &&
    inputMethod.value !== null &&
    (inputMethod.value === 'text'
      ? resumeText.value.trim().length > 0
      : selectedFile.value !== null)
  )
})

// ── Methods ────────────────────────────────────────────

function selectGroup(group: TargetGroup) {
  targetGroup.value = group
}

function handleFileChange(payload: { file: File; name: string }) {
  selectedFile.value = payload.file
  fileName.value = payload.name
  inputMethod.value = 'file'
}

function handleFileRemove() {
  selectedFile.value = null
  fileName.value = ''
  if (inputMethod.value === 'file') {
    inputMethod.value = null
  }
}

async function handleOptimize() {
  if (!canSubmit.value || !targetGroup.value) return

  appPhase.value = 'optimizing'
  errorMessage.value = ''

  try {
    let result
    if (inputMethod.value === 'text') {
      result = await optimizeResumeText(resumeText.value.trim(), targetGroup.value)
    } else if (inputMethod.value === 'file' && selectedFile.value) {
      result = await optimizeResumeFile(selectedFile.value, targetGroup.value)
    } else {
      throw new Error('请先输入或上传您的简历')
    }

    if (result.success) {
      optimizedText.value = result.data.optimized_text
      originalText.value = result.data.original_text
      encouragement.value = result.data.encouragement
      appPhase.value = 'results'
    } else {
      throw new Error('优化未成功，请稍后重试')
    }
  } catch (err: any) {
    const msg =
      err?.response?.data?.error?.message ??
      err?.message ??
      '出了点意外，请您稍后重试'
    errorMessage.value = msg
    appPhase.value = 'error'
  }

  // Fetch job recommendations (non-blocking)
  if (appPhase.value === 'results' && targetGroup.value) {
    try {
      const jobs = await searchJobs(targetGroup.value)
      if (jobs.success) {
        jobResults.value = jobs.data
      }
    } catch {
      // Job search failure shouldn't break the main flow
    }
  }
}

function handleRetry() {
  appPhase.value = 'input'
  errorMessage.value = ''
}

async function handleExportPdf() {
  if (!optimizedText.value) return

  exportingPdf.value = true
  try {
    const blob = await exportPdf(optimizedText.value, '优化简历')
    const url = window.URL.createObjectURL(blob)
    const link = document.createElement('a')
    link.href = url
    link.download = '优化简历.pdf'
    document.body.appendChild(link)
    link.click()
    document.body.removeChild(link)
    window.URL.revokeObjectURL(url)
    ElMessage.success('简历已导出，请查看下载')
  } catch {
    ElMessage.error('导出失败，请稍后重试')
  } finally {
    exportingPdf.value = false
  }
}

function handleDownloadResume() {
  if (!optimizedText.value) return

  try {
    const blob = new Blob([optimizedText.value], { type: 'text/markdown;charset=utf-8' })
    const url = window.URL.createObjectURL(blob)
    const link = document.createElement('a')
    link.href = url
    link.download = '优化简历.md'
    document.body.appendChild(link)
    link.click()
    document.body.removeChild(link)
    window.URL.revokeObjectURL(url)
    ElMessage.success('简历已下载，请查看文件')
  } catch {
    ElMessage.error('下载失败，请稍后重试')
  }
}

function toggleFontMode() {
  fontMode.value = fontMode.value === 'normal' ? 'large' : 'normal'
}

function toggleContrastMode() {
  contrastMode.value = contrastMode.value === 'normal' ? 'high' : 'normal'
}
</script>

<template>
  <div
    class="app-root"
    :class="{
      'large-font': fontMode === 'large',
      'high-contrast': contrastMode === 'high',
    }"
  >
    <A11yBar
      :font-mode="fontMode"
      :contrast-mode="contrastMode"
      @toggle-font-mode="toggleFontMode"
      @toggle-contrast-mode="toggleContrastMode"
    />

    <HeroSection
      :target-group="targetGroup"
      :show-mission="showMission"
      @select-group="selectGroup"
      @toggle-mission="showMission = !showMission"
    />

    <InputSection
      :app-phase="appPhase"
      :input-method="inputMethod"
      :resume-text="resumeText"
      :file-name="fileName"
      :can-submit="canSubmit"
      :target-group="targetGroup"
      :error-message="errorMessage"
      :upload-max-size-mb="uploadMaxSizeMb"
      @update:resume-text="resumeText = $event"
      @update:input-method="inputMethod = $event"
      @file-change="handleFileChange"
      @file-remove="handleFileRemove"
      @optimize="handleOptimize"
      @retry="handleRetry"
    />

    <transition name="slide-up">
      <ResultsSection
        v-if="appPhase === 'results'"
        :original-text="originalText"
        :optimized-text="optimizedText"
        :encouragement="encouragement"
        :job-results="jobResults"
        :exporting-pdf="exportingPdf"
        @export-pdf="handleExportPdf"
        @download-resume="handleDownloadResume"
      />
    </transition>

    <AppFooter />
  </div>
</template>

<style scoped>
.app-root {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  align-items: center;
}
</style>
