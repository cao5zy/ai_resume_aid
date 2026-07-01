<script setup lang="ts">
import type { JobSearchResult } from '@/types'

defineProps<{
  originalText: string
  optimizedText: string
  encouragement: string
  jobResults: JobSearchResult[]
  exportingPdf: boolean
}>()

const emit = defineEmits<{
  'export-pdf': []
  'download-resume': []
}>()
</script>

<template>
  <section class="results-section" aria-label="优化结果">
    <div class="results-header">
      <h2 class="section-heading">✅ 优化完成</h2>
      <p class="results-sub">
        以下是优化前后的对比，您可以看看有什么变化
      </p>
    </div>

    <!-- Two-column comparison -->
    <div class="comparison-grid">
      <article class="comparison-card original-card" aria-label="原始简历">
        <h3 class="card-label">原始简历</h3>
        <div class="card-body">
          <p class="resume-text">{{ originalText }}</p>
        </div>
      </article>
      <article class="comparison-card optimized-card" aria-label="优化后简历">
        <h3 class="card-label">优化简历</h3>
        <div class="card-body">
          <p class="resume-text">{{ optimizedText }}</p>
        </div>
      </article>
    </div>

    <!-- Export & Download -->
    <div class="export-row">
      <button
        class="export-btn"
        :disabled="exportingPdf"
        :aria-disabled="exportingPdf"
        aria-label="导出优化后的简历为PDF文件"
        @click="emit('export-pdf')"
      >
        {{ exportingPdf ? '正在导出...' : '导出PDF' }}
      </button>
      <button
        class="export-btn"
        aria-label="下载优化后的简历"
        @click="emit('download-resume')"
      >
        下载简历
      </button>
    </div>

    <!-- Encouragement -->
    <transition name="fade">
      <section v-if="encouragement" class="encouragement-card" aria-label="给您的鼓励">
        <h3 class="encouragement-label">💬 给您的鼓励</h3>
        <p class="encouragement-text">{{ encouragement }}</p>
      </section>
    </transition>

    <!-- Job search results -->
    <transition name="fade">
      <section v-if="jobResults.length > 0" class="jobs-card" aria-label="包容性岗位推荐">
        <h3 class="jobs-label">🔍 包容性岗位推荐</h3>
        <ul class="jobs-list" role="list">
          <li v-for="job in jobResults" :key="job.url" class="job-item" role="listitem">
            <a
              :href="job.url"
              class="job-link"
              target="_blank"
              rel="noopener noreferrer"
              :aria-label="'查看岗位：' + job.title"
            >
              <span class="job-title">{{ job.title }}</span>
            </a>
            <p v-if="job.snippet" class="job-snippet">{{ job.snippet }}</p>
          </li>
        </ul>
      </section>
    </transition>
  </section>
</template>

<style scoped>
.results-section {
  width: 100%;
  max-width: var(--max-width);
  padding: 0 24px 48px;
}

.results-header {
  text-align: center;
  margin-bottom: 32px;
}

.section-heading {
  font-size: var(--font-size-xl);
  font-weight: 700;
  color: var(--text-color);
  margin-bottom: 24px;
  text-align: center;
}

.results-sub {
  font-size: 0.95rem;
  color: var(--text-muted);
}

/* ===== Comparison Grid ===== */
.comparison-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 20px;
  margin-bottom: 28px;
}

.comparison-card {
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius-lg);
  overflow: hidden;
  box-shadow: var(--shadow-sm);
}

.card-label {
  font-size: 1rem;
  font-weight: 600;
  padding: 14px 20px;
  margin: 0;
  color: #fff;
}

.original-card .card-label {
  background: var(--text-muted);
}

.optimized-card .card-label {
  background: var(--primary-color);
}

.card-body {
  padding: 20px;
}

.resume-text {
  font-size: 0.95rem;
  line-height: 1.85;
  color: var(--text-color);
  white-space: pre-wrap;
  word-break: break-word;
}

/* ===== Export ===== */
.export-row {
  display: flex;
  justify-content: center;
  gap: 12px;
  margin-bottom: 32px;
}

.export-btn {
  padding: 12px 36px;
  border: 2px solid var(--primary-color);
  border-radius: 24px;
  background: transparent;
  color: var(--primary-color);
  font-size: 1rem;
  font-weight: 600;
  font-family: inherit;
  cursor: pointer;
  transition: all var(--transition-base);
}
.export-btn:hover:not(:disabled) {
  background: var(--primary-color);
  color: #fff;
}
.export-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

/* ===== Encouragement ===== */
.encouragement-card {
  background: var(--bg-warm);
  border: 1px solid var(--accent-color);
  border-radius: var(--border-radius-lg);
  padding: 28px 32px;
  margin-bottom: 24px;
  box-shadow: var(--shadow-sm);
}

.encouragement-label {
  font-size: var(--font-size-lg);
  font-weight: 700;
  color: var(--accent-color);
  margin-bottom: 12px;
}

.encouragement-text {
  font-size: 1rem;
  line-height: 1.9;
  color: var(--text-color);
}

/* ===== Job Results ===== */
.jobs-card {
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius-lg);
  padding: 28px 32px;
  box-shadow: var(--shadow-sm);
}

.jobs-label {
  font-size: var(--font-size-lg);
  font-weight: 700;
  color: var(--text-color);
  margin-bottom: 16px;
}

.jobs-list {
  list-style: none;
  padding: 0;
  margin: 0;
}

.job-item {
  padding: 14px 0;
  border-bottom: 1px solid var(--border-color);
}
.job-item:last-child {
  border-bottom: none;
}

.job-link {
  display: inline-block;
  color: var(--primary-color);
  text-decoration: none;
  font-size: 1rem;
  font-weight: 500;
  transition: color var(--transition-base);
}
.job-link:hover {
  color: var(--primary-hover);
  text-decoration: underline;
}

.job-title {
  margin-right: 4px;
}

.job-snippet {
  font-size: 0.85rem;
  color: var(--text-muted);
  margin-top: 4px;
  line-height: 1.6;
}

/* ===== Responsive ===== */
@media (max-width: 768px) {
  .comparison-grid {
    grid-template-columns: 1fr;
  }

  .results-section {
    padding: 0 20px 40px;
  }

  .encouragement-card,
  .jobs-card {
    padding: 20px 20px;
  }
}

@media (min-width: 1200px) {
  .comparison-grid {
    gap: 28px;
  }
}
</style>
