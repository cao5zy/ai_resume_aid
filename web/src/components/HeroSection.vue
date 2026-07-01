<script setup lang="ts">
import type { TargetGroup } from '@/types'

defineProps<{
  targetGroup: TargetGroup | null
  showMission: boolean
}>()

const emit = defineEmits<{
  'select-group': [group: TargetGroup]
  'toggle-mission': []
}>()
</script>

<template>
  <header class="hero">
    <div class="hero-icon" aria-hidden="true">🧭</div>
    <h1 class="hero-title">AI微光求职</h1>
    <p class="hero-subtitle">
      免费帮您优化简历，让您的经验和能力更好地被看到
    </p>

    <div class="group-select" role="radiogroup" aria-label="选择您的情况">
      <p class="group-select-label">请告诉我们您的情况：</p>
      <div class="group-buttons">
        <button
          class="group-btn"
          :class="{ active: targetGroup === 'disabled' }"
          role="radio"
          :aria-checked="targetGroup === 'disabled'"
          aria-label="障碍者求职"
          @click="emit('select-group', 'disabled')"
        >
          <span class="group-btn-icon" aria-hidden="true">♿</span>
          <span>障碍者求职</span>
        </button>
        <button
          class="group-btn"
          :class="{ active: targetGroup === 'elderly' }"
          role="radio"
          :aria-checked="targetGroup === 'elderly'"
          aria-label="大龄求职者，四十五岁以上"
          @click="emit('select-group', 'elderly')"
        >
          <span class="group-btn-icon" aria-hidden="true">🧓</span>
          <span>大龄求职者（45岁以上）</span>
        </button>
      </div>
    </div>

    <button
      class="mission-toggle"
      :aria-expanded="showMission"
      aria-controls="mission-panel"
      @click="emit('toggle-mission')"
    >
      <span>{{ showMission ? '收起' : '了解更多' }}项目背景</span>
      <span class="mission-arrow" aria-hidden="true">{{ showMission ? '▲' : '▼' }}</span>
    </button>

    <transition name="slide-up">
      <section
        v-if="showMission"
        id="mission-panel"
        class="mission-panel"
        aria-label="项目背景介绍"
      >
        <p>
          AI微光求职是一个纯粹的公益项目。我们相信，每个人都值得被公平对待
          ——无论身体是否健全，无论年龄大小。
        </p>
        <p>
          许多障碍者和45岁以上的求职者在找工作时会面临额外的困难。
          简历是他们展示自己的第一扇窗，但往往因为不知道怎么写、写不好，
          而被一些机会拒之门外。
        </p>
        <p>
          我们利用AI技术，免费帮您把简历打磨得更专业、更有温度。
          这不是冰冷的机器评估，而是希望能帮您把那些被忽视的闪光点亮出来。
        </p>
        <p class="mission-footer">用AI重新看见人 · 2026人文季</p>
      </section>
    </transition>
  </header>
</template>

<style scoped>
.hero {
  text-align: center;
  padding: 80px 24px 48px;
  max-width: var(--max-width);
  width: 100%;
}

.hero-icon {
  font-size: 3rem;
  margin-bottom: 8px;
}

.hero-title {
  font-size: var(--font-size-3xl);
  font-weight: 700;
  color: var(--primary-color);
  margin-bottom: 12px;
  letter-spacing: 0.02em;
}

.hero-subtitle {
  font-size: var(--font-size-lg);
  color: var(--text-secondary);
  margin-bottom: 36px;
  max-width: 480px;
  margin-left: auto;
  margin-right: auto;
  line-height: 1.8;
}

.group-select {
  margin-bottom: 32px;
}

.group-select-label {
  font-size: var(--font-size-lg);
  color: var(--text-color);
  margin-bottom: 16px;
  font-weight: 500;
}

.group-buttons {
  display: flex;
  justify-content: center;
  gap: 16px;
  flex-wrap: wrap;
}

.group-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  padding: 24px 32px;
  border: 2px solid var(--border-color);
  border-radius: var(--border-radius-lg);
  background: var(--surface-color);
  color: var(--text-color);
  font-size: var(--font-size-lg);
  font-family: inherit;
  cursor: pointer;
  transition: all var(--transition-base);
  min-width: 200px;
  box-shadow: var(--shadow-sm);
}
.group-btn:hover {
  border-color: var(--primary-color);
  box-shadow: var(--shadow-md);
}
.group-btn.active {
  border-color: var(--primary-color);
  background: var(--primary-light);
  box-shadow: 0 0 0 4px rgba(26, 122, 122, 0.12);
}
.group-btn-icon {
  font-size: 2rem;
}

.mission-toggle {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 10px 20px;
  border: none;
  background: transparent;
  color: var(--primary-color);
  font-size: 0.95rem;
  font-family: inherit;
  cursor: pointer;
  transition: color var(--transition-base);
}
.mission-toggle:hover {
  color: var(--primary-hover);
}
.mission-arrow {
  font-size: 0.75rem;
}

.mission-panel {
  margin-top: 20px;
  padding: 28px 32px;
  background: var(--bg-warm);
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius-lg);
  text-align: left;
  max-width: 640px;
  margin-left: auto;
  margin-right: auto;
  box-shadow: var(--shadow-md);
}
.mission-panel p {
  font-size: 0.95rem;
  color: var(--text-secondary);
  margin-bottom: 12px;
  line-height: 1.8;
}
.mission-panel p:last-child {
  margin-bottom: 0;
}
.mission-footer {
  font-weight: 600;
  color: var(--accent-color) !important;
  text-align: center;
  margin-top: 16px !important;
  font-size: 1rem !important;
}

@media (max-width: 768px) {
  .hero {
    padding: 48px 20px 32px;
  }

  .hero-title {
    font-size: var(--font-size-2xl);
  }

  .hero-subtitle {
    font-size: var(--font-size-base);
  }

  .group-buttons {
    flex-direction: column;
    align-items: center;
    gap: 12px;
  }

  .group-btn {
    width: 100%;
    max-width: 320px;
    flex-direction: row;
    justify-content: center;
    padding: 18px 24px;
  }
}
</style>
