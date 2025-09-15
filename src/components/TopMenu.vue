<template>
  <div class="top-menu">
    <a-menu 
      mode="horizontal" 
      :theme="theme" 
      :triggerSubMenuAction="'click'"
      class="main-menu"
    >
      <a-sub-menu key="session" title="会话">
        <a-menu-item key="newLocal" @click="$emit('newLocal')">
          <MonitorOutlined />
          新建本地会话
        </a-menu-item>
        <a-menu-item key="newSsh" @click="$emit('newSsh')">
          <KeyOutlined />
          新建 SSH 会话
        </a-menu-item>
      </a-sub-menu>
      
      <a-sub-menu key="view" title="视图">
        <a-menu-item key="theme-dark" @click="$emit('toggleTheme', 'dark')">
          深色主题
        </a-menu-item>
        <a-menu-item key="theme-light" @click="$emit('toggleTheme', 'light')">
          浅色主题
        </a-menu-item>
      </a-sub-menu>
      
      <a-sub-menu key="settings" title="设置">
        <a-menu-item key="prefs" @click="$emit('showSettings')">
          <SettingOutlined />
          程序配置
        </a-menu-item>
      </a-sub-menu>
    </a-menu>
    
    <!-- 工具栏 -->
    <div class="toolbar">
      <a-space>
        <a-button size="small" type="primary" ghost @click="$emit('newLocal')">
          <PlusOutlined />
          本地
        </a-button>
        <a-button size="small" type="primary" @click="$emit('newSsh')">
          <KeyOutlined />
          SSH
        </a-button>
      </a-space>
    </div>
  </div>
</template>

<script setup>
import { 
  PlusOutlined, 
  KeyOutlined, 
  SettingOutlined, 
  MonitorOutlined,
  DashboardOutlined
} from '@ant-design/icons-vue'

defineProps({
  theme: {
    type: String,
    default: 'dark'
  }
})

defineEmits(['newLocal', 'newSsh', 'toggleTheme', 'showSettings'])
</script>

<style scoped>
.top-menu {
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: var(--menu-bg);
  border-bottom: 1px solid var(--border-color);
  padding: 0 16px;
  height: 48px;
}

.main-menu {
  background: transparent;
  border: none;
  flex: 1;
}

.main-menu :deep(.ant-menu-item),
.main-menu :deep(.ant-menu-submenu-title) {
  color: var(--text-color) !important;
}

.main-menu :deep(.ant-menu-item:hover),
.main-menu :deep(.ant-menu-submenu-title:hover) {
  background: var(--hover-bg) !important;
}

.toolbar {
  display: flex;
  align-items: center;
  gap: 8px;
}
</style>
