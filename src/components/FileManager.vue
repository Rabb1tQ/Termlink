<template>
  <div class="file-manager">
    <div class="file-toolbar">
      <a-breadcrumb>
        <a-breadcrumb-item>
          <HomeOutlined />
        </a-breadcrumb-item>
        <a-breadcrumb-item v-for="(part, index) in pathParts" :key="index">
          <span @click="navigateTo(index)" class="breadcrumb-link">{{ part }}</span>
        </a-breadcrumb-item>
      </a-breadcrumb>
      
      <div class="toolbar-actions">
        <a-button size="small" @click="goBack" :disabled="!canGoBack" title="后退">
          <ArrowLeftOutlined />
        </a-button>
        <a-button size="small" @click="goForward" :disabled="!canGoForward" title="前进">
          <ArrowRightOutlined />
        </a-button>
        <a-button size="small" @click="goUp" :disabled="isAtRoot" title="上级目录">
          <ArrowUpOutlined />
        </a-button>
        <a-button size="small" @click="refresh" title="刷新">
          <ReloadOutlined />
        </a-button>
        <a-button size="small" @click="newFolder" title="新建文件夹">
          <FolderAddOutlined />
        </a-button>
        <a-switch 
          v-model:checked="showHidden" 
          @change="refresh"
          size="small"
          class="hidden-switch"
        />
        <span class="switch-label">显示隐藏文件</span>
      </div>
    </div>
    
    <div class="file-content">
      <a-spin :spinning="loading">
        <a-table 
          :columns="columns"
          :data-source="files"
          :pagination="false"
          size="small"
          :scroll="{ y: 400 }"
          @row-click="onRowClick"
          :row-selection="{ 
            selectedRowKeys: selectedFiles, 
            onChange: onSelectionChange,
            type: 'checkbox' 
          }"
        >
          <template #bodyCell="{ column, record }">
            <template v-if="column.key === 'name'">
              <div class="file-item">
                <component 
                  :is="getFileIcon(record)" 
                  :style="{ color: getFileColor(record) }"
                />
                <span>{{ record.name }}</span>
              </div>
            </template>
            <template v-else-if="column.key === 'size'">
              {{ formatSize(record.size) }}
            </template>
            <template v-else-if="column.key === 'modified'">
              {{ formatDate(record.modified) }}
            </template>
          </template>
        </a-table>
      </a-spin>
    </div>
    
    <div class="file-status">
      <span>{{ files.length }} 个项目</span>
      <span v-if="selectedFiles.length > 0">（已选择 {{ selectedFiles.length }} 个）</span>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { 
  HomeOutlined, 
  ArrowLeftOutlined, 
  ArrowRightOutlined, 
  ArrowUpOutlined,
  ReloadOutlined,
  FolderAddOutlined,
  FolderOutlined,
  FileOutlined,
  FileTextOutlined,
  FileImageOutlined,
  FileZipOutlined,
  VideoCameraOutlined,
  SoundOutlined
} from '@ant-design/icons-vue'
import { message } from 'ant-design-vue'
import { invoke } from '@tauri-apps/api/core'

const currentPath = ref('')
const files = ref([])
const loading = ref(false)
const selectedFiles = ref([])
const history = ref([])
const historyIndex = ref(-1)
const showHidden = ref(false)
const keepAliveTimer = ref(null)
const lastActivity = ref(Date.now())

// 表格列定义
const columns = [
  {
    title: '名称',
    dataIndex: 'name',
    key: 'name',
    sorter: (a, b) => a.name.localeCompare(b.name),
  },
  {
    title: '大小',
    dataIndex: 'size',
    key: 'size',
    width: 100,
    sorter: (a, b) => (a.size || 0) - (b.size || 0),
  },
  {
    title: '修改时间',
    dataIndex: 'modified',
    key: 'modified',
    width: 150,
    sorter: (a, b) => new Date(a.modified) - new Date(b.modified),
  },
]

// 路径部分
const pathParts = computed(() => {
  if (!currentPath.value) return []
  // 处理Windows和Unix路径
  let parts = currentPath.value.split(/[/\\]/).filter(part => part)
  
  // 如果是Windows路径且第一部分包含冒号（如C:），保持原样
  if (parts.length > 0 && parts[0].includes(':')) {
    return parts
  }
  
  return parts
})

// 导航状态
const canGoBack = computed(() => historyIndex.value > 0)
const canGoForward = computed(() => historyIndex.value < history.value.length - 1)
const isAtRoot = computed(() => !currentPath.value || currentPath.value === '/')

// 获取文件图标
function getFileIcon(file) {
  if (file.isDirectory) return FolderOutlined
  
  const ext = file.name.split('.').pop()?.toLowerCase()
  if (!ext) return FileOutlined
  
  const imageExts = ['jpg', 'jpeg', 'png', 'gif', 'bmp', 'svg', 'webp']
  const videoExts = ['mp4', 'avi', 'mkv', 'mov', 'wmv', 'flv']
  const audioExts = ['mp3', 'wav', 'flac', 'aac', 'ogg']
  const textExts = ['txt', 'md', 'json', 'xml', 'html', 'css', 'js', 'ts', 'vue']
  const archiveExts = ['zip', 'rar', '7z', 'tar', 'gz']
  
  if (imageExts.includes(ext)) return FileImageOutlined
  if (videoExts.includes(ext)) return VideoCameraOutlined
  if (audioExts.includes(ext)) return SoundOutlined
  if (textExts.includes(ext)) return FileTextOutlined
  if (archiveExts.includes(ext)) return FileZipOutlined
  
  return FileOutlined
}

// 获取文件颜色
function getFileColor(file) {
  if (file.isDirectory) return '#1890ff'
  
  const ext = file.name.split('.').pop()?.toLowerCase()
  if (!ext) return '#666'
  
  const colorMap = {
    'jpg': '#f56a00', 'jpeg': '#f56a00', 'png': '#f56a00', 'gif': '#f56a00',
    'mp4': '#722ed1', 'avi': '#722ed1', 'mkv': '#722ed1',
    'mp3': '#13c2c2', 'wav': '#13c2c2', 'flac': '#13c2c2',
    'txt': '#52c41a', 'md': '#52c41a', 'json': '#52c41a',
    'zip': '#fa8c16', 'rar': '#fa8c16', '7z': '#fa8c16'
  }
  
  return colorMap[ext] || '#666'
}

// 格式化文件大小
function formatSize(bytes) {
  if (!bytes || bytes === 0) return '-'
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(1024))
  return Math.round(bytes / Math.pow(1024, i) * 100) / 100 + ' ' + sizes[i]
}

// 格式化日期
function formatDate(date) {
  if (!date) return '-'
  return new Date(date).toLocaleString()
}

// 加载文件列表
async function loadFiles(path = '') {
  loading.value = true
  try {
    // 如果路径为空，获取用户主目录
    const targetPath = path || await invoke('get_home_dir')
    
    // 调用后端API获取文件列表
    const fileList = await invoke('list_files', { 
      path: targetPath, 
      showHidden: showHidden.value 
    })
    
    // 转换数据格式并添加key
    files.value = fileList.map((file, index) => ({
      key: index,
      name: file.name,
      path: file.path,
      isDirectory: file.is_directory,
      size: file.size,
      modified: file.modified,
      isHidden: file.is_hidden
    }))
    
    // 添加到历史记录
    if (historyIndex.value === -1 || history.value[historyIndex.value] !== targetPath) {
      history.value = history.value.slice(0, historyIndex.value + 1)
      history.value.push(targetPath)
      historyIndex.value = history.value.length - 1
    }
    
    currentPath.value = targetPath
  } catch (error) {
    console.error('加载文件列表失败:', error)
    message.error('加载文件列表失败: ' + error)
  } finally {
    loading.value = false
  }
}

// 行点击事件
function onRowClick(record) {
  updateActivity()
  if (record.isDirectory) {
    loadFiles(record.path)
  } else {
    // 尝试用系统默认程序打开文件
    openFileWithSystem(record.path)
  }
}

// 用系统默认程序打开文件
async function openFileWithSystem(filePath) {
  try {
    await invoke('open_file_explorer', { path: filePath })
    message.success('文件已打开')
  } catch (error) {
    message.error('无法打开文件: ' + error)
  }
}

// 选择变化
function onSelectionChange(selectedRowKeys) {
  updateActivity()
  selectedFiles.value = selectedRowKeys
}

// 导航操作
async function navigateTo(index) {
  try {
    // 构建路径
    const parts = pathParts.value.slice(0, index + 1)
    let newPath
    
    if (currentPath.value.includes('\\')) {
      // Windows路径
      newPath = parts.join('\\')
      // 如果第一部分是驱动器（如C:），需要添加反斜杠
      if (parts.length === 1 && parts[0].includes(':')) {
        newPath += '\\'
      }
    } else {
      // Unix路径
      newPath = '/' + parts.join('/')
    }
    
    loadFiles(newPath)
  } catch (error) {
    message.error('导航失败: ' + error)
  }
}

function goBack() {
  if (canGoBack.value) {
    historyIndex.value--
    loadFiles(history.value[historyIndex.value])
  }
}

function goForward() {
  if (canGoForward.value) {
    historyIndex.value++
    loadFiles(history.value[historyIndex.value])
  }
}

async function goUp() {
  if (!isAtRoot.value) {
    try {
      const parentPath = await invoke('get_parent_dir', { path: currentPath.value })
      if (parentPath) {
        loadFiles(parentPath)
      }
    } catch (error) {
      message.error('无法访问上级目录: ' + error)
    }
  }
}

function refresh() {
  updateActivity()
  loadFiles(currentPath.value)
}

function newFolder() {
  message.info('新建文件夹功能开发中...')
}

// 保活机制
function startKeepAlive() {
  // 每30秒发送一次保活请求
  keepAliveTimer.value = setInterval(async () => {
    try {
      // 发送一个轻量级的请求来保持连接活跃
      await invoke('keep_alive')
      lastActivity.value = Date.now()
    } catch (error) {
      console.warn('保活请求失败:', error)
      // 如果保活失败，尝试重新加载文件列表
      if (currentPath.value) {
        try {
          await loadFiles(currentPath.value)
        } catch (reloadError) {
          console.error('重新加载文件列表失败:', reloadError)
        }
      }
    }
  }, 30000) // 30秒间隔
}

function stopKeepAlive() {
  if (keepAliveTimer.value) {
    clearInterval(keepAliveTimer.value)
    keepAliveTimer.value = null
  }
}

// 更新活动时间
function updateActivity() {
  lastActivity.value = Date.now()
}

onMounted(() => {
  loadFiles()
  startKeepAlive()
})

onUnmounted(() => {
  stopKeepAlive()
})
</script>

<style scoped>
.file-manager {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--bg-color);
}

.file-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 16px;
  border-bottom: 1px solid var(--border-color);
  background: var(--panel-bg);
}

.breadcrumb-link {
  cursor: pointer;
  color: var(--primary-color);
}

.breadcrumb-link:hover {
  text-decoration: underline;
}

.toolbar-actions {
  display: flex;
  gap: 8px;
  align-items: center;
}

.hidden-switch {
  margin-left: 8px;
}

.switch-label {
  font-size: 12px;
  color: var(--text-color);
  margin-left: 4px;
}

.file-content {
  flex: 1;
  padding: 8px;
  overflow: hidden;
}

.file-item {
  display: flex;
  align-items: center;
  gap: 8px;
}

.file-status {
  padding: 8px 16px;
  border-top: 1px solid var(--border-color);
  background: var(--panel-bg);
  font-size: 12px;
  color: var(--muted-color);
}

:deep(.ant-table-tbody > tr) {
  cursor: pointer;
}

:deep(.ant-table-tbody > tr:hover) {
  background: var(--hover-bg) !important;
}

:deep(.ant-breadcrumb) {
  color: var(--text-color);
}

:deep(.ant-table) {
  background: var(--panel-bg);
  color: var(--text-color);
}

:deep(.ant-table-thead > tr > th) {
  background: var(--panel-header-bg);
  color: var(--text-color);
  border-color: var(--border-color);
}

:deep(.ant-table-tbody > tr > td) {
  border-color: var(--border-color);
}
</style>
