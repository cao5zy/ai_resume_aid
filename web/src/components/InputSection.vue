<script setup lang="ts">
import { ref, computed, onUnmounted, watch } from 'vue'
import { ElMessage } from 'element-plus'
import type { UploadFile, UploadRawFile } from 'element-plus'
import type { AppPhase, TargetGroup } from '@/types'

const props = defineProps<{
  appPhase: AppPhase
  inputMethod: 'text' | 'file' | null
  resumeText: string
  fileName: string
  canSubmit: boolean
  targetGroup: TargetGroup | null
  errorMessage: string
  uploadMaxSizeMb: number
}>()

const emit = defineEmits<{
  'update:resumeText': [value: string]
  'update:inputMethod': [value: 'text' | 'file']
  'file-change': [payload: { file: File; name: string }]
  'file-remove': []
  optimize: []
  retry: []
}>()

function handleFileChange(uploadFile: UploadFile) {
  const raw = uploadFile.raw as UploadRawFile | undefined
  if (raw) {
    emit('file-change', { file: raw, name: raw.name })
  }
}

function beforeFileUpload(rawFile: UploadRawFile): boolean {
  const isPdf = rawFile.type === 'application/pdf' || rawFile.name.endsWith('.pdf')
  if (!isPdf) {
    ElMessage.warning('目前只支持 PDF 文件，请您重新选择')
    return false
  }
  const maxMb = props.uploadMaxSizeMb
  const isUnderLimit = rawFile.size / 1024 / 1024 <= maxMb
  if (!isUnderLimit) {
    ElMessage.warning(`文件太大了，请控制在 ${maxMb}MB 以内`)
    return false
  }
  return true
}

// ── Optimizing carousel ────────────────────────────────────────────
const CAROUSEL_ITEMS = ['正在帮您优化简历，请稍等…', '我们正在仔细阅读您的内容，找出可以提升的地方', '据经验估算，你要等30秒到5分钟不等'] as const
const ITEM_HEIGHT = 56
/** Number of display items = 2x for seamless looping */
const DISPLAY_ITEMS = [...CAROUSEL_ITEMS, ...CAROUSEL_ITEMS]

const carouselStep = ref(0) // 0→1→2→3→0→...
const noTransition = ref(false)
let carouselTimer: ReturnType<typeof setInterval> | null = null

/** Which message index (0-2) is at the top position */
const topMessageIndex = computed(() => carouselStep.value % CAROUSEL_ITEMS.length)

const trackStyle = computed(() => ({
  transform: `translateY(${-carouselStep.value * ITEM_HEIGHT}px)`,
  transition: noTransition.value ? 'none' : 'transform 0.6s cubic-bezier(0.4, 0, 0.2, 1)',
}))

function advanceCarousel() {
  carouselStep.value++
  if (carouselStep.value >= CAROUSEL_ITEMS.length) {
    // Seamless reset: position 3 shows same content as position 0
    // Reset instantly (no transition) so user sees no jump
    noTransition.value = true
    carouselStep.value = 0
    void document.documentElement.offsetHeight
    noTransition.value = false
  }
}

function startCarousel() {
  stopCarousel()
  carouselStep.value = 0
  noTransition.value = false
  carouselTimer = setInterval(advanceCarousel, 4000)
}

function stopCarousel() {
  if (carouselTimer !== null) {
    clearInterval(carouselTimer)
    carouselTimer = null
  }
}

watch(
  () => props.appPhase,
  (phase) => {
    if (phase === 'optimizing') {
      startCarousel()
    } else {
      stopCarousel()
    }
  },
  { immediate: true },
)

onUnmounted(() => {
  stopCarousel()
})
</script>

<template>
  <section class="input-section" aria-label="简历输入">
    <!-- Phase: input -->
    <div v-if="appPhase === 'input'" class="input-area">
      <h2 class="section-heading">📝 输入简历内容</h2>

      <!-- Input method toggle -->
      <div class="input-method-tabs" role="tablist" aria-label="选择输入方式">
        <button
          class="method-tab"
          :class="{ active: inputMethod === 'text' }"
          role="tab"
          :aria-selected="inputMethod === 'text'"
          @click="emit('update:inputMethod', 'text')"
        >
          方式一：粘贴文本
        </button>
        <button
          class="method-tab"
          :class="{ active: inputMethod === 'file' }"
          role="tab"
          :aria-selected="inputMethod === 'file'"
          @click="emit('update:inputMethod', 'file')"
        >
          方式二：上传PDF
        </button>
      </div>

      <!-- Text input -->
      <div v-if="inputMethod === 'text'" class="input-method-panel">
        <label for="resume-textarea" class="sr-only">简历文本内容</label>
        <textarea
          id="resume-textarea"
          :value="resumeText"
          class="resume-textarea"
          placeholder="把您的简历内容粘贴到这里，比如您做过什么工作、有什么技能。不用怕写得不完美，我们来帮您打磨。"
          rows="10"
          aria-describedby="textarea-hint"
          @input="emit('update:resumeText', ($event.target as HTMLTextAreaElement).value)"
        ></textarea>
        <p id="textarea-hint" class="input-hint">
          没有固定格式要求，想到什么就写什么。我们会帮您整理好。
        </p>
      </div>

      <!-- File upload -->
      <div v-if="inputMethod === 'file'" class="input-method-panel">
        <el-upload
          class="file-uploader"
          drag
          :auto-upload="false"
          :limit="1"
          :on-change="handleFileChange"
          :on-remove="() => emit('file-remove')"
          :before-upload="beforeFileUpload"
          accept=".pdf,application/pdf"
          :file-list="[]"
        >
          <div class="upload-placeholder">
            <span class="upload-icon" aria-hidden="true">📄</span>
            <p class="upload-text">拖拽PDF文件到此处，或点击选择</p>
            <p class="upload-hint">仅支持 PDF 格式，文件不超过 {{ uploadMaxSizeMb }}MB</p>
          </div>
        </el-upload>
        <p v-if="fileName" class="file-name">
          已选择：<strong>{{ fileName }}</strong>
        </p>
      </div>

      <!-- Submit -->
      <div class="submit-row">
        <button
          class="submit-btn"
          :disabled="!canSubmit"
          :aria-disabled="!canSubmit"
          aria-label="开始优化简历"
          @click="emit('optimize')"
        >
          ✨ 开始优化
        </button>
        <p v-if="!canSubmit" class="submit-hint" role="status">
          {{ !targetGroup ? '请先在上方选择您的情况' : !inputMethod ? '请先选择输入方式（粘贴文本或上传PDF）' : '请填写或上传简历内容' }}
        </p>
      </div>
    </div>

    <!-- Phase: optimizing -->
    <div v-else-if="appPhase === 'optimizing'" class="optimizing-area" role="status" aria-live="polite">
      <div class="spinner" aria-hidden="true">
        <div class="spinner-ring"></div>
      </div>
      <div class="carousel-container" :aria-label="CAROUSEL_ITEMS[topMessageIndex]">
        <div class="carousel-track" :style="trackStyle">
          <p
            v-for="(msg, i) in DISPLAY_ITEMS"
            :key="i"
            class="carousel-item"
            :class="{ active: i % CAROUSEL_ITEMS.length === topMessageIndex }"
          >
            {{ msg }}
          </p>
        </div>
      </div>
    </div>

    <!-- Phase: error -->
    <div v-else-if="appPhase === 'error'" class="error-area" role="alert" aria-live="assertive">
      <div class="error-card">
        <p class="error-icon" aria-hidden="true">😥</p>
        <h3 class="error-title">出了点小问题</h3>
        <p class="error-message">{{ errorMessage }}</p>
        <button class="retry-btn" @click="emit('retry')" aria-label="重新尝试优化">
          重新尝试
        </button>
        <p class="error-help">
          如果多次失败，可以稍后再试试，我估计现在知乎的API忙不过来了，😂😂😂
        </p>
      </div>
    </div>
  </section>
</template>

<style scoped>
.input-section {
  width: 100%;
  max-width: var(--max-width);
  padding: 0 24px 48px;
}

.section-heading {
  font-size: var(--font-size-xl);
  font-weight: 700;
  color: var(--text-color);
  margin-bottom: 24px;
  text-align: center;
}

/* ===== Input Method Tabs ===== */
.input-method-tabs {
  display: flex;
  justify-content: center;
  gap: 4px;
  margin-bottom: 20px;
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  padding: 4px;
  max-width: 420px;
  margin-left: auto;
  margin-right: auto;
}

.method-tab {
  flex: 1;
  padding: 10px 16px;
  border: none;
  border-radius: var(--border-radius-sm);
  background: transparent;
  color: var(--text-secondary);
  font-size: 0.95rem;
  font-family: inherit;
  cursor: pointer;
  transition: all var(--transition-base);
  white-space: nowrap;
}
.method-tab:hover {
  color: var(--primary-color);
}
.method-tab.active {
  background: var(--primary-color);
  color: #fff;
}

.input-method-panel {
  margin-bottom: 24px;
}

/* ===== Textarea ===== */
.resume-textarea {
  width: 100%;
  min-height: 260px;
  padding: 20px;
  border: 2px solid var(--border-color);
  border-radius: var(--border-radius);
  background: var(--surface-color);
  color: var(--text-color);
  font-size: 1rem;
  font-family: inherit;
  line-height: 1.8;
  resize: vertical;
  transition: border-color var(--transition-base);
}
.resume-textarea:focus {
  border-color: var(--border-focus);
  outline: none;
}
.resume-textarea::placeholder {
  color: var(--text-muted);
}

.input-hint {
  font-size: 0.85rem;
  color: var(--text-muted);
  margin-top: 8px;
}

/* ===== File Upload ===== */
.file-uploader {
  width: 100%;
}

.upload-placeholder {
  text-align: center;
  padding: 36px 20px;
}

.upload-icon {
  font-size: 2.5rem;
  display: block;
  margin-bottom: 12px;
}

.upload-text {
  font-size: 1rem;
  color: var(--text-color);
  margin-bottom: 6px;
}

.upload-hint {
  font-size: 0.85rem;
  color: var(--text-muted);
}

.file-name {
  text-align: center;
  margin-top: 12px;
  font-size: 0.95rem;
  color: var(--success-color);
}

/* ===== Submit ===== */
.submit-row {
  text-align: center;
  margin-top: 8px;
}

.submit-btn {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 16px 48px;
  border: none;
  border-radius: 50px;
  background: var(--primary-color);
  color: #fff;
  font-size: var(--font-size-lg);
  font-weight: 600;
  font-family: inherit;
  cursor: pointer;
  transition: all var(--transition-base);
  box-shadow: var(--shadow-md);
  letter-spacing: 0.02em;
}
.submit-btn:hover:not(:disabled) {
  background: var(--primary-hover);
  box-shadow: var(--shadow-lg);
  transform: translateY(-1px);
}
.submit-btn:active:not(:disabled) {
  transform: translateY(0);
}
.submit-btn:disabled {
  background: var(--border-color);
  color: var(--text-muted);
  cursor: not-allowed;
  box-shadow: none;
}

.submit-hint {
  margin-top: 12px;
  font-size: 0.9rem;
  color: var(--text-muted);
}

/* ===== Optimizing ===== */
.optimizing-area {
  text-align: center;
  padding: 64px 24px;
}

.spinner {
  display: flex;
  justify-content: center;
  margin-bottom: 24px;
}

.spinner-ring {
  width: 48px;
  height: 48px;
  border: 4px solid var(--primary-light);
  border-top: 4px solid var(--primary-color);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.optimizing-text {
  font-size: var(--font-size-lg);
  font-weight: 600;
  color: var(--text-color);
  margin-bottom: 8px;
}

.optimizing-sub {
  font-size: 0.95rem;
  color: var(--text-muted);
}

/* ===== Optimizing Carousel ===== */
.carousel-container {
  height: 168px; /* 3 rows × 56px */
  overflow: hidden;
  position: relative;
}

.carousel-track {
  transition: transform 0.6s cubic-bezier(0.4, 0, 0.2, 1);
}

.carousel-item {
  height: 56px;
  display: flex;
  align-items: center;
  justify-content: center;
  margin: 0;
  font-size: 0.95rem;
  color: var(--text-muted);
  font-weight: 400;
  transition: color 0.4s ease, font-weight 0.4s ease, opacity 0.4s ease;
}

.carousel-item.active {
  color: var(--text-color);
  font-weight: 700;
  font-size: var(--font-size-lg);
  opacity: 1;
}

.carousel-item:not(.active) {
  opacity: 0.5;
}

/* ===== Error ===== */
.error-area {
  text-align: center;
  padding: 48px 24px;
}

.error-card {
  display: inline-block;
  max-width: 440px;
  padding: 36px 32px;
  background: var(--error-light);
  border: 1px solid var(--error-color);
  border-radius: var(--border-radius-lg);
}

.error-icon {
  font-size: 2.5rem;
  margin-bottom: 12px;
}

.error-title {
  font-size: var(--font-size-lg);
  font-weight: 600;
  color: var(--error-color);
  margin-bottom: 8px;
}

.error-message {
  font-size: 0.95rem;
  color: var(--text-secondary);
  margin-bottom: 20px;
  line-height: 1.7;
}

.retry-btn {
  padding: 12px 32px;
  border: none;
  border-radius: 24px;
  background: var(--error-color);
  color: #fff;
  font-size: 1rem;
  font-family: inherit;
  cursor: pointer;
  transition: background var(--transition-base);
}
.retry-btn:hover {
  background: #c43d39;
}

.error-help {
  margin-top: 16px;
  font-size: 0.85rem;
  color: var(--text-muted);
}

/* ===== Screen Reader Only ===== */
.sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border: 0;
}

/* ===== Responsive ===== */
@media (max-width: 768px) {
  .input-method-tabs {
    flex-direction: column;
    max-width: 100%;
  }
}
</style>
