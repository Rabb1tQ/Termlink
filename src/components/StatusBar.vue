<template>
  <div class="status-bar">
    <div class="status-left">
      <span class="status-connection" v-if="activeConnection">
        {{ activeConnection }}
      </span>
      <span v-else class="status-disconnected">
        未连接
      </span>
    </div>
    
    <div class="status-right">
      <span class="status-info">
        标签页: {{ tabCount }}
      </span>
      <span class="status-info">
        主题: {{ theme === 'dark' ? '深色' : '浅色' }}
      </span>
    </div>
  </div>
</template>

<script setup>
defineProps({
  activeConnection: {
    type: String,
    default: ''
  },
  tabCount: {
    type: Number,
    default: 0
  },
  theme: {
    type: String,
    default: 'dark'
  }
})
</script>

<style scoped>
.status-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 4px 12px;
  background: var(--status-bg);
  border-top: 1px solid var(--border-color);
  font-size: 12px;
  color: var(--muted-color);
}

.status-left, .status-right {
  display: flex;
  gap: 8px;
}

.status-connection {
  color: var(--success-color);
}

.status-disconnected {
  color: var(--muted-color);
}

.status-info {
  color: var(--muted-color);
}

/* 响应式设计 */
@media (max-width: 768px) {
  .status-bar {
    font-size: 10px !important;
    padding: 2px 8px !important;
  }
}
</style>
