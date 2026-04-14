<template>
  <div class="app-container">
    <!-- 顶部菜单 -->
    <TopMenu 
      @new-local="newLocal" 
      @new-ssh="newSsh" 
      @toggle-theme="toggleTheme"
      @show-settings="showSettings = true"
      @show-file-manager="showFileManager"
      :theme="theme"
    />
    
    <!-- 主体区域 -->
    <div class="main-container">
      <!-- 左侧面板 -->
      <Sidebar
        ref="sidebarRef"
        :collapsed="leftPanelCollapsed"
        @toggle="leftPanelCollapsed = !leftPanelCollapsed"
        :profiles="profiles"
        @launch-profile="launchSavedProfile"
        :active-tab="getActiveTab()"
        @open-file-preview="openFilePreview"
        @refresh-profiles="refreshProfiles"
        @start-download="handleStartDownload"
        @edit-profile="editProfile"
      />
      
      <!-- 右侧内容区 -->
      <div class="content-container">
        <!-- 标签栏 -->
        <TabManager 
          :tabs="tabs" 
          :active-id="activeId" 
          @change="activeId = $event"
          @close="closeTab"
        />
        
        <!-- 终端/编辑器区域 -->
        <div class="terminals-container">
          <template v-for="tab in tabs" :key="tab.id">
            <!-- SSH终端 -->
            <Terminal 
              v-if="tab.type === 'ssh'" 
              :id="tab.id" 
              :active="activeId === tab.id" 
              :theme="theme"
              :config="terminalConfig"
              :auto-password="tab.autoPassword"
              :type="'ssh'"
              @close="closeTab(tab.id)"
              @reconnect="reconnectSsh(tab)"
              v-show="activeId === tab.id"
            />
            
            <!-- 本地终端 -->
            <Terminal 
              v-else-if="tab.type === 'local'" 
              :id="tab.id" 
              :active="activeId === tab.id" 
              :theme="theme"
              :config="terminalConfig"
              :type="'local'"
              @close="closeTab(tab.id)"
              v-show="activeId === tab.id"
            />
            
            <!-- 文件编辑器 -->
            <FileEditor
              v-else-if="tab.type === 'file'"
              :id="tab.id"
              :active="activeId === tab.id"
              :file-info="tab.fileInfo"
              :connection-id="tab.connectionId"
              :theme="theme"
              @close="closeTab(tab.id)"
              v-show="activeId === tab.id"
            />
          </template>
        </div>
      </div>
      
      <!-- 右侧面板 -->
      <RightPanel 
        ref="rightPanelRef"
        :collapsed="rightPanelCollapsed" 
        @toggle="rightPanelCollapsed = !rightPanelCollapsed"
        :connection-id="getActiveTab()?.type === 'ssh' ? getActiveTab()?.id : ''"
        :ssh-profile="getActiveTab()?.type === 'ssh' ? getActiveTab()?.profile : null"
      />
    </div>
    
    <!-- 状态栏 -->
    <StatusBar :active-connection="getActiveConnection()" :theme="theme" />
    
    <!-- SSH连接模态框 -->
    <SshModal 
      v-model:visible="showSshModal" 
      :edit-mode="sshEditMode"
      :edit-profile="editingProfile"
      @submit="submitSsh" 
    />
    
    <!-- 设置模态框 -->
    <SettingsModal 
      v-model:visible="showSettings" 
      :terminal-config="terminalConfig"
      @update-config="updateTerminalConfig"
    />
  </div>
</template>

<script setup>
import { onMounted, onBeforeUnmount, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { message } from 'ant-design-vue'
import 'ant-design-vue/dist/reset.css'

// 导入组件
import TopMenu from './components/TopMenu.vue'
import TabManager from './components/TabManager.vue'
import Sidebar from './components/Sidebar.vue'
import Terminal from './components/Terminal.vue'
import SshModal from './components/SshModal.vue'
import SettingsModal from './components/SettingsModal.vue'
import RightPanel from './components/RightPanel.vue'
import StatusBar from './components/StatusBar.vue'
import FileEditor from './components/FileEditor.vue'

// 导入服务
import SshService from './services/SshService'
import ThemeService from './services/ThemeService'

// 响应式数据
const tabs = ref([]) // { id, title, type, off?: () => void }
const activeId = ref('')
const leftPanelCollapsed = ref(false)
const showSshModal = ref(false)
const showSettings = ref(false)
const rightPanelRef = ref(null)
const rightPanelCollapsed = ref(true)
const sidebarRef = ref(null)
const sshEditMode = ref(false)
const editingProfile = ref(null)

// 主题和设置
const theme = ref(ThemeService.getTheme())
const terminalConfig = ref(ThemeService.getTerminalConfig())

// 已保存的连接配置
const profiles = ref([])

// 切换主题
function toggleTheme(next) { 
  theme.value = ThemeService.toggleTheme(next);
}

// 更新终端配置
function updateTerminalConfig(config) {
  terminalConfig.value = ThemeService.updateTerminalConfig(config);
}

// 刷新连接配置
async function refreshProfiles() {
  profiles.value = await SshService.getProfiles();
}

// 获取活动连接信息
function getActiveConnection() {
  if (!activeId.value) return ''
  const activeTab = tabs.value.find(t => t.id === activeId.value)
  return activeTab ? activeTab.title : ''
}

// 获取活动标签页
function getActiveTab() {
  if (!activeId.value) return null
  return tabs.value.find(t => t.id === activeId.value) || null
}

// 启动已保存的连接
async function launchSavedProfile(p) {
  try {
    const tabInfo = await SshService.launchProfile(p);
    tabs.value.push(tabInfo);
    activeId.value = tabInfo.id;
  } catch (error) {
    console.error('启动SSH连接失败:', error);
    message.error({
      content: error.toString(),
      duration: 8, // 显示8秒，给用户足够时间阅读
      style: {
        marginTop: '50px',
        maxWidth: '400px'
      }
    });
  }
}

// 处理开始下载
function handleStartDownload(downloadInfo) {
  if (rightPanelRef.value) {
    rightPanelRef.value.addDownload(
      downloadInfo.fileName,
      downloadInfo.remotePath,
      downloadInfo.savePath,
      downloadInfo.connectionId
    );
  }
}

// 提交 SSH 连接
async function submitSsh(sshData) {
  try {
    if (sshData.isEdit) {
      // 编辑模式：更新现有配置
      await SshService.updateProfile(sshData);
      await refreshProfiles();
      showSshModal.value = false;
    } else {
      // 新建模式：创建新连接
      const tabInfo = await SshService.createSshConnection(sshData);
      tabs.value.push(tabInfo);
      activeId.value = tabInfo.id;
      
      // 刷新配置列表
      if (sshData.savePassword) {
        await refreshProfiles();
      }
      
      showSshModal.value = false;
    }
  } catch (error) {
    console.error('SSH连接操作失败:', error);
    message.error({
      content: error.toString(),
      duration: 8, // 显示8秒，给用户足够时间阅读
      style: {
        marginTop: '50px',
        maxWidth: '400px'
      }
    });
  }
}

// 新建本地会话
async function newLocal() {
  const id = `local-${Date.now()}`
  tabs.value.push({ id, title: '本地终端', type: 'local' })
  activeId.value = id
  
  await invoke('start_pty', { id, cols: 120, rows: 30 })
}

// 新建 SSH 会话
async function newSsh() {
  sshEditMode.value = false
  editingProfile.value = null
  showSshModal.value = true
}

// 编辑 SSH 配置文件
function editProfile(profile) {
  sshEditMode.value = true
  editingProfile.value = profile
  showSshModal.value = true
}

// 通知侧边栏显示文件管理器
function showFileManager() {
  // 展开左侧面板（如果折叠）
  if (leftPanelCollapsed.value) {
    leftPanelCollapsed.value = false
  }
  
  // 切换到文件管理 Tab
  if (sidebarRef.value && sidebarRef.value.switchTab) {
    sidebarRef.value.switchTab('files')
  }
}

// 打开文件预览
async function openFilePreview(fileInfo) {
  const id = `file-${Date.now()}`
  const title = `📄 ${fileInfo.name}`
  
  // 获取当前活动SSH标签页的连接ID
  const activeTab = tabs.value.find(t => t.id === activeId.value)
  const connectionId = activeTab?.sftpConnectionId || activeTab?.id
  
  tabs.value.push({ 
    id, 
    title, 
    type: 'file',
    fileInfo,
    connectionId
  })
  activeId.value = id
}

// 关闭标签页
async function closeTab(id) {
  const index = tabs.value.findIndex(t => t.id === id)
  if (index === -1) return
  
  const tab = tabs.value[index]
  
  // 清理资源
  if (tab.type === 'ssh') {
    await SshService.closeConnection(id);
  } else if (tab.type === 'local') {
    await invoke('close_pty', { id });
  }
  
  // 移除标签页
  tabs.value.splice(index, 1)
  
  // 如果关闭的是当前活动标签页，切换到前一个标签页
  if (activeId.value === id) {
    activeId.value = tabs.value[index - 1]?.id || tabs.value[0]?.id || ''
  }
}

// 重新连接SSH
async function reconnectSsh(tab) {
  if (tab && tab.profile) {
    await SshService.reconnect(tab.id, tab.profile);
  }
}

// 生命周期钩子
onMounted(async () => {
  // 加载已保存的SSH配置
  await refreshProfiles()
  
  // 监听SFTP连接事件
  window.addEventListener('sftp-connected', (event) => {
    const { sshId, sftpId } = event.detail
    const tab = tabs.value.find(t => t.id === sshId)
    if (tab) {
      tab.sftpConnectionId = sftpId
      console.log(`SSH标签页 ${sshId} 的SFTP连接已建立: ${sftpId}`)
    }
  })
})

onBeforeUnmount(() => {
  // 关闭所有连接
  tabs.value.forEach(async tab => {
    if (tab.type === 'ssh') {
      await SshService.closeConnection(tab.id);
    } else if (tab.type === 'local') {
      await invoke('close_pty', { id: tab.id });
    }
  })
})
</script>

<style>
:root {
  --primary-color: #1890ff;
  --bg-color: #f0f2f5;
  --text-color: #000000;
  --border-color: #d9d9d9;
  --header-bg: #ffffff;
  --sidebar-bg: #ffffff;
  --terminal-bg: #000000;
  --terminal-text: #ffffff;
}

[data-theme="dark"] {
  --primary-color: #177ddc;
  --bg-color: #141414;
  --text-color: #ffffff;
  --border-color: #303030;
  --header-bg: #1f1f1f;
  --sidebar-bg: #1f1f1f;
  --terminal-bg: #0b0e14;
  --terminal-text: #ffffff;
}

body {
  margin: 0;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
  background-color: var(--bg-color);
  color: var(--text-color);
}

.app-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
  overflow: hidden;
}

.main-container {
  display: flex;
  flex: 1;
  overflow: hidden;
}

.content-container {
  display: flex;
  flex-direction: column;
  flex: 1;
  overflow: hidden;
}

.terminals-container {
  flex: 1;
  position: relative;
  overflow: hidden;
}

.terminals-container > * {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
}
</style>
