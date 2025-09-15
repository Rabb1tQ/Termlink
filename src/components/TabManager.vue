<template>
  <div class="tabs-container">
    <a-tabs
      type="editable-card"
      :activeKey="activeId"
      @change="$emit('change', $event)"
      @edit="onEditTab"
      :hideAdd="true"
      :animated="false"
    >
      <a-tab-pane v-for="t in tabs" :key="t.id">
        <template #tab>
          <span class="tab-content">
            <span class="tab-icon">
              {{ t.type === 'ssh' ? '🔑' : '💻' }}
            </span>
            <span class="tab-title">{{ t.title }}</span>
          </span>
        </template>
      </a-tab-pane>
    </a-tabs>
  </div>
</template>

<script setup>
defineProps({
  tabs: {
    type: Array,
    default: () => []
  },
  activeId: {
    type: String,
    default: ''
  }
})

const emit = defineEmits(['change', 'close'])

function onEditTab(targetKey, action) {
  if (action === 'remove') {
    emit('close', targetKey)
  }
}
</script>

<style scoped>
.tabs-container {
  background: var(--tabs-bg);
  border-bottom: 1px solid var(--border-color);
}

.tab-content {
  display: flex;
  align-items: center;
  gap: 6px;
}

.tab-icon {
  font-size: 14px;
}

.tab-title {
  font-size: 12px;
}

:deep(.ant-tabs-tab) {
  background: var(--tabs-bg) !important;
  border-color: var(--border-color) !important;
  color: var(--text-color) !important;
}

:deep(.ant-tabs-tab-active) {
  background: var(--terminal-bg) !important;
  border-bottom-color: var(--terminal-bg) !important;
}

:deep(.ant-tabs-tab-btn) {
  color: var(--text-color) !important;
}

:deep(.ant-tabs-content-holder) {
  display: none;
}
</style>
