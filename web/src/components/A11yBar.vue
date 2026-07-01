<script setup lang="ts">
import type { FontMode, ContrastMode } from '@/types'

defineProps<{
  fontMode: FontMode
  contrastMode: ContrastMode
}>()

const emit = defineEmits<{
  'toggle-font-mode': []
  'toggle-contrast-mode': []
}>()
</script>

<template>
  <div class="a11y-bar" role="toolbar" aria-label="辅助功能">
    <button
      class="a11y-btn"
      :aria-pressed="fontMode === 'large'"
      :aria-label="fontMode === 'large' ? '切换为正常字体' : '切换为大字体模式'"
      @click="emit('toggle-font-mode')"
    >
      <span aria-hidden="true">Aa</span>
      <span class="a11y-label">{{ fontMode === 'large' ? '正常字体' : '大字体' }}</span>
    </button>
    <button
      class="a11y-btn"
      :aria-pressed="contrastMode === 'high'"
      :aria-label="contrastMode === 'high' ? '切换为普通对比度' : '切换为高对比度模式'"
      @click="emit('toggle-contrast-mode')"
    >
      <span aria-hidden="true">◐</span>
      <span class="a11y-label">{{ contrastMode === 'high' ? '普通对比' : '高对比' }}</span>
    </button>
  </div>
</template>

<style scoped>
.a11y-bar {
  position: fixed;
  top: 12px;
  right: 16px;
  z-index: 100;
  display: flex;
  gap: 8px;
}

.a11y-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 14px;
  border: 1px solid var(--border-color);
  border-radius: 20px;
  background: var(--surface-color);
  color: var(--text-secondary);
  font-size: 0.875rem;
  font-family: inherit;
  cursor: pointer;
  transition: all var(--transition-base);
  box-shadow: var(--shadow-sm);
}
.a11y-btn:hover {
  border-color: var(--primary-color);
  color: var(--primary-color);
}
.a11y-btn[aria-pressed='true'] {
  background: var(--primary-color);
  color: #fff;
  border-color: var(--primary-color);
}
.a11y-label {
  font-size: 0.8rem;
}

@media (max-width: 768px) {
  .a11y-bar {
    top: 8px;
    right: 8px;
    gap: 4px;
  }

  .a11y-btn {
    padding: 5px 10px;
  }

  .a11y-label {
    display: none;
  }
}
</style>
