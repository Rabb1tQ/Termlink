<template>
  <div class="right-panel">
    <!-- 内容区 - 可折叠，在左侧 -->
    <div class="panel-content-wrapper" :class="{ collapsed: collapsed }">
    
      <!-- 标题栏 -->
      <div class="panel-header">
        <span>{{ activeTab === 'monitor' ? '系统监控' : '下载管理' }}</span>
        <a-button 
          type="text" 
          size="small" 
          class="collapse-btn"
          @click="$emit('toggle')"
        >
          <RightOutlined />
        </a-button>
      </div>
      
      <!-- 系统监控内容 -->
      <div class="panel-content monitor-content" v-if="activeTab === 'monitor'">
      <a-spin :spinning="loading">
        <!-- 系统基本信息 -->
        <div class="info-section">
          <div class="section-header">
            <DesktopOutlined class="section-icon" />
            <h4>系统信息</h4>
          </div>
          <div class="info-card">
            <div class="info-item">
              <div class="info-label">
                <LaptopOutlined class="item-icon" />
                主机名
              </div>
              <div class="info-value">{{ systemInfo.hostname || '-' }}</div>
            </div>
            <div class="info-item">
              <div class="info-label">
                <WindowsOutlined class="item-icon" />
                操作系统
              </div>
              <div class="info-value">{{ systemInfo.os || '-' }}</div>
            </div>
            <div class="info-item">
              <div class="info-label">
                <ControlOutlined class="item-icon" />
                架构
              </div>
              <div class="info-value">{{ systemInfo.arch || '-' }}</div>
            </div>
            <div class="info-item">
              <div class="info-label">
                <CodeOutlined class="item-icon" />
                内核
              </div>
              <div class="info-value">{{ systemInfo.kernel || '-' }}</div>
            </div>
            <div class="info-item">
              <div class="info-label">
                <ClockCircleOutlined class="item-icon" />
                运行时间
              </div>
              <div class="info-value">{{ formatUptime(systemInfo.uptime) }}</div>
            </div>
          </div>
        </div>
        
        <!-- CPU信息 -->
        <div class="info-section">
          <div class="section-header">
            <ThunderboltOutlined class="section-icon" />
            <h4>CPU使用率</h4>
          </div>
          <div class="info-card">
            <div class="cpu-info">
              <div class="cpu-model">{{ cpuInfo.model || 'CPU' }}</div>
              <div class="cpu-usage">{{ cpuInfo.usage?.toFixed(1) || 0 }}%</div>
            </div>
            <div class="progress-container">
              <a-progress 
                :percent="cpuInfo.usage || 0" 
                :show-info="false"
                :stroke-color="getProgressColor(cpuInfo.usage || 0)"
                :stroke-width="8"
              />
            </div>
          </div>
        </div>
        
        <!-- 内存信息 -->
        <div class="info-section">
          <div class="section-header">
            <DatabaseOutlined class="section-icon" />
            <h4>内存使用</h4>
          </div>
          <div class="info-card">
            <div class="memory-info">
              <div class="memory-stats">
                <span class="memory-label">物理内存</span>
                <span class="memory-usage">{{ formatSize(memoryInfo.used) }} / {{ formatSize(memoryInfo.total) }}</span>
              </div>
              <div class="usage-percent">{{ memoryInfo.usage?.toFixed(1) || 0 }}%</div>
            </div>
            <div class="progress-container">
              <a-progress 
                :percent="memoryInfo.usage || 0" 
                :show-info="false"
                :stroke-color="getProgressColor(memoryInfo.usage || 0)"
                :stroke-width="8"
              />
            </div>
            <div class="memory-details">
              <div class="detail-item">
                <span class="detail-label">可用:</span>
                <span class="detail-value">{{ formatSize(memoryInfo.available) }}</span>
              </div>
              <div class="detail-item">
                <span class="detail-label">缓存:</span>
                <span class="detail-value">{{ formatSize(memoryInfo.cached) }}</span>
              </div>
            </div>
          </div>
        </div>
        
        <!-- 磁盘信息 -->
        <div class="info-section">
          <div class="section-header">
            <HddOutlined class="section-icon" />
            <h4>磁盘使用</h4>
          </div>
          <div class="info-card">
            <div 
              v-for="disk in diskInfo.slice(0, 2)" 
              :key="disk.device"
              class="disk-item"
            >
              <div class="disk-header">
                <div class="disk-info">
                  <span class="disk-device">{{ disk.device }}</span>
                  <span class="disk-mount">{{ disk.mountpoint }}</span>
                </div>
                <span class="disk-usage">{{ disk.usage?.toFixed(1) || 0 }}%</span>
              </div>
              <div class="progress-container">
                <a-progress 
                  :percent="disk.usage || 0" 
                  :show-info="false"
                  :stroke-color="getProgressColor(disk.usage || 0)"
                  :stroke-width="6"
                />
              </div>
              <div class="disk-stats">
                <span class="disk-stat">{{ formatSize(disk.used) }} / {{ formatSize(disk.total) }}</span>
                <span class="disk-type">{{ disk.filesystem }}</span>
              </div>
            </div>
          </div>
        </div>
        
        <!-- 网络信息 -->
        <div class="info-section">
          <div class="section-header">
            <WifiOutlined class="section-icon" />
            <h4>网络接口</h4>
          </div>
          <div class="info-card">
            <div 
              v-for="interface_ in networkInfo.slice(0, 2)" 
              :key="interface_.name"
              class="network-item"
            >
              <div class="network-header">
                <div class="interface-info">
                  <span class="interface-name">{{ interface_.name }}</span>
                  <span class="interface-ip" v-if="interface_.ip">{{ interface_.ip }}</span>
                </div>
                <span class="interface-status" :class="{ active: interface_.status === 'up' }">
                  {{ interface_.status }}
                </span>
              </div>
              <div class="network-stats">
                <div class="network-stat">
                  <div class="stat-item">
                    <span class="stat-icon">↓</span>
                    <span class="stat-label">接收</span>
                    <span class="stat-value">{{ formatSize(interface_.rx_bytes) }}</span>
                  </div>
                  <div class="stat-item">
                    <span class="stat-icon">↑</span>
                    <span class="stat-label">发送</span>
                    <span class="stat-value">{{ formatSize(interface_.tx_bytes) }}</span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </a-spin>
    </div>
    
    <!-- 下载管理内容 -->
    <div class="panel-content download-content" v-if="activeTab === 'download'">
      <div v-if="downloads.length === 0" class="empty-state">
        <a-empty description="暂无下载任务" />
      </div>
      
      <div v-else class="download-list">
        <div 
          v-for="download in downloads" 
          :key="download.id"
          class="download-item"
          :class="{ 
            completed: download.status === 'completed', 
            error: download.status === 'error' 
          }"
        >
          <div class="download-info">
            <div class="file-name">{{ download.fileName }}</div>
            <div class="file-path">{{ download.savePath }}</div>
            
            <!-- 进度条 -->
            <a-progress 
              v-if="download.status === 'downloading'"
              :percent="download.progress" 
              size="small"
              :show-info="false"
            />
            
            <!-- 状态信息 -->
            <div class="download-status">
              <span v-if="download.status === 'downloading'">
                <template v-if="download.total > 0">
                  {{ formatSize(download.downloaded) }} / {{ formatSize(download.total) }}
                  <span v-if="download.progress > 0">({{ download.progress }}%)</span>
                </template>
                <template v-else>
                  正在下载...
                </template>
              </span>
              <span v-else-if="download.status === 'completed'" class="success">
                完成 - {{ formatSize(download.total) }}
              </span>
              <span v-else-if="download.status === 'error'" class="error">
                失败: {{ download.error }}
              </span>
            </div>
          </div>
          
          <div class="download-actions">
            <a-button 
              v-if="download.status === 'downloading'"
              type="text" 
              size="small" 
              danger
              @click="cancelDownload(download.id)"
              title="取消"
            >
              <StopOutlined />
            </a-button>
            
            <a-button 
              v-if="download.status === 'completed'"
              type="text" 
              size="small"
              @click="openFileLocation(download.savePath)"
              title="打开"
            >
              <FolderOpenOutlined />
            </a-button>
            
            <a-button 
              type="text" 
              size="small" 
              danger
              @click="removeDownload(download.id)"
              title="移除"
            >
              <DeleteOutlined />
            </a-button>
          </div>
        </div>
      </div>
      
      <div class="download-footer" v-if="downloads.length > 0">
        <a-button size="small" @click="clearCompleted">
          清除已完成
        </a-button>
      </div>
    </div>
    
    <!-- 底部状态栏 - 仅监控页显示 -->
    <div class="panel-footer" v-if="activeTab === 'monitor'">
      <a-button size="small" @click="manualRefresh" :loading="loading">
        <ReloadOutlined />
        刷新
      </a-button>
      <span class="last-update" :class="{ updating: silentLoading }">
        <span v-if="silentLoading" class="update-indicator">●</span>
        {{ lastUpdate ? new Date(lastUpdate).toLocaleTimeString() : '-' }}
      </span>
    </div>
    </div>
    
    <!-- 按钮栏 - 始终显示，在最右侧 -->
    <div class="sidebar-buttons">
      <a-tooltip placement="left" title="系统监控">
        <a-button 
          :type="activeTab === 'monitor' ? 'primary' : 'default'"
          size="large"
          @click="handleTabClick('monitor')"
          class="sidebar-btn"
        >
          <DesktopOutlined />
        </a-button>
      </a-tooltip>
      
      <a-tooltip placement="left" title="下载管理">
        <a-button 
          :type="activeTab === 'download' ? 'primary' : 'default'"
          size="large"
          @click="handleTabClick('download')"
          class="sidebar-btn"
        >
          <DownloadOutlined />
        </a-button>
      </a-tooltip>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted, computed, watch, nextTick } from 'vue'
import { 
  ReloadOutlined,
  DesktopOutlined,
  LaptopOutlined,
  WindowsOutlined,
  ControlOutlined,
  CodeOutlined,
  ClockCircleOutlined,
  ThunderboltOutlined,
  DatabaseOutlined,
  HddOutlined,
  WifiOutlined,
  DownloadOutlined,
  StopOutlined,
  FolderOpenOutlined,
  DeleteOutlined,
  RightOutlined
} from '@ant-design/icons-vue'
import { message } from 'ant-design-vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

const props = defineProps({
  collapsed: {
    type: Boolean,
    default: false
  },
  connectionId: {
    type: String,
    default: ''
  }
})

const emit = defineEmits(['toggle'])

// 状态数据
const loading = ref(false)
const silentLoading = ref(false) // 静默加载状态
const lastUpdate = ref(null)
const systemInfo = ref({})
const cpuInfo = ref({})
const memoryInfo = ref({})
const diskInfo = ref([])
const networkInfo = ref([])

// 下载管理状态
const activeTab = ref('monitor') // 'monitor' or 'download'
const downloads = ref([])
let downloadIdCounter = 0
let progressUnlisten = null

// 活跃下载数量
const activeDownloads = computed(() => {
  return downloads.value.filter(d => d.status === 'downloading').length
})

// 处理标签点击
function handleTabClick(tab) {
  // 如果点击的是当前激活的标签，切换折叠状态
  if (activeTab.value === tab) {
    emit('toggle')
  } else {
    // 切换到新标签
    activeTab.value = tab
    // 如果当前是折叠状态，自动展开
    if (props.collapsed) {
      emit('toggle')
    }
  }
}

let refreshTimer = null
let refreshInterval = 3000 // 初始刷新间隔3秒
const minInterval = 2000 // 最小间隔2秒
const maxInterval = 30000 // 最大间隔30秒
let errorCount = 0

// 监控数据刷新
async function refreshData(silent = false) {
  if (!props.connectionId) return
  
  if (silent) {
    silentLoading.value = true
  } else {
    loading.value = true
  }
  
  try {
    // 并行获取所有系统信息
    const [system, cpu, memory, disk, network] = await Promise.all([
      invoke('get_system_info', { connectionId: props.connectionId }),
      invoke('get_cpu_info', { connectionId: props.connectionId }),
      invoke('get_memory_info', { connectionId: props.connectionId }),
      invoke('get_disk_info', { connectionId: props.connectionId }),
      invoke('get_network_info', { connectionId: props.connectionId })
    ])
    
    systemInfo.value = system
    cpuInfo.value = cpu
    memoryInfo.value = memory
    diskInfo.value = disk
    networkInfo.value = network
    
    lastUpdate.value = Date.now()
    errorCount = 0 // 重置错误计数
    
    // 成功后适当减少刷新间隔（更频繁）
    refreshInterval = Math.max(minInterval, refreshInterval * 0.9)
    
  } catch (error) {
    console.error('获取系统信息失败:', error)
    errorCount++
    
    // 只在非静默模式下显示错误消息
    if (!silent) {
      message.error('获取系统信息失败: ' + error)
    }
    
    // 错误时增加刷新间隔（降低频率）
    refreshInterval = Math.min(maxInterval, refreshInterval * 1.5)
    
  } finally {
    loading.value = false
    silentLoading.value = false
  }
}

// 自动刷新
function startAutoRefresh() {
  stopAutoRefresh()
  
  function scheduleNext() {
    refreshTimer = setTimeout(async () => {
      if (!props.collapsed && props.connectionId) {
        await refreshData(true) // 静默刷新
        scheduleNext() // 递归调度下一次刷新
      }
    }, refreshInterval)
  }
  
  scheduleNext()
}

// 手动刷新（带loading）
async function manualRefresh() {
  await refreshData(false)
  // 手动刷新后重置间隔
  refreshInterval = 3000
}

function stopAutoRefresh() {
  if (refreshTimer) {
    clearTimeout(refreshTimer)
    refreshTimer = null
  }
}

// 格式化文件大小
function formatSize(bytes) {
  if (!bytes || bytes === 0) return '0 B'
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(1024))
  return Math.round(bytes / Math.pow(1024, i) * 100) / 100 + ' ' + sizes[i]
}

// 格式化运行时间
function formatUptime(seconds) {
  if (!seconds) return '-'
  
  const days = Math.floor(seconds / 86400)
  const hours = Math.floor((seconds % 86400) / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  
  if (days > 0) {
    return `${days}天 ${hours}小时`
  } else if (hours > 0) {
    return `${hours}小时 ${minutes}分钟`
  } else {
    return `${minutes}分钟`
  }
}

// 获取进度条颜色
function getProgressColor(percentage) {
  if (percentage < 50) return '#52c41a'
  if (percentage < 80) return '#faad14'
  return '#ff4d4f'
}

// 生命周期
onMounted(async () => {
  // 不自动刷新，等待用户手动展开
  
  // 监听下载进度事件
  progressUnlisten = await listen('download-progress', (event) => {
    const { downloadId, downloaded, total, progress } = event.payload
    const download = downloads.value.find(d => d.id === downloadId)
    if (download && download.status === 'downloading') {
      download.downloaded = downloaded
      download.total = total
      download.progress = progress
      console.log(`📥 下载进度: ${download.fileName} - ${progress}% (${formatSize(downloaded)}/${formatSize(total)})`)
    }
  })
})

onUnmounted(() => {
  stopAutoRefresh()
  // 取消事件监听
  if (progressUnlisten) {
    progressUnlisten()
  }
})

// 监听属性变化
watch(() => props.collapsed, (newCollapsed) => {
  if (!newCollapsed && props.connectionId) {
    // 展开时才开始刷新数据
    manualRefresh() // 首次手动刷新，带loading
    startAutoRefresh()
  } else {
    // 折叠时停止刷新
    stopAutoRefresh()
  }
})

watch(() => props.connectionId, (newConnectionId) => {
  if (newConnectionId && !props.collapsed) {
    // 只有在面板展开且有连接时才刷新
    manualRefresh() // 首次手动刷新，带loading
    startAutoRefresh()
  } else {
    // 没有连接或面板折叠时停止刷新
    stopAutoRefresh()
  }
})

// ============ 下载管理函数 ============

// 添加下载任务
function addDownload(fileName, remotePath, savePath, connectionId) {
  console.log('=== addDownload 被调用 ===', {
    fileName,
    remotePath,
    savePath,
    connectionId
  })
  
  const downloadId = ++downloadIdCounter
  const download = {
    id: downloadId,
    fileName,
    remotePath,
    savePath,
    connectionId,
    status: 'downloading',
    progress: 0,
    downloaded: 0,
    total: 0,
    startTime: Date.now(),
    error: null
  }
  
  downloads.value.push(download)
  console.log('下载任务已添加到列表，开始下载...')
  startDownload(download)
  
  // 自动切换到下载标签页
  activeTab.value = 'download'
  
  return downloadId
}

// 开始下载
async function startDownload(download) {
  console.log('=== startDownload 开始（真实进度）===', download)
  
  try {
    console.log('开始调用 download_sftp_file API（带真实进度）...')
    
    // 调用后端下载API（带真实进度）
    await invoke('download_sftp_file', {
      connectionId: download.connectionId,
      remotePath: download.remotePath,
      localPath: download.savePath,
      downloadId: download.id
    })
    
    console.log('✓ download_sftp_file API 调用成功')
    
    if (download.status !== 'cancelled') {
      download.status = 'completed'
      download.progress = 100
      console.log('✓ 下载完成！')
      message.success(`文件下载完成: ${download.fileName}`)
    }
  } catch (error) {
    console.error('✗ 下载过程中出错:', error)
    if (download.status !== 'cancelled') {
      download.status = 'error'
      download.error = error.toString()
      message.error(`下载失败: ${download.fileName}`)
    }
  }
}

// 取消下载
function cancelDownload(downloadId) {
  const download = downloads.value.find(d => d.id === downloadId)
  if (download) {
    download.status = 'cancelled'
    message.info(`已取消下载: ${download.fileName}`)
  }
}

// 打开文件位置
async function openFileLocation(filePath) {
  console.log('打开文件位置:', filePath)
  try {
    await invoke('open_file_location', { path: filePath })
    message.success('已打开文件所在位置')
  } catch (error) {
    console.error('打开文件位置失败:', error)
    message.error('无法打开文件位置: ' + error)
  }
}

// 移除下载记录
function removeDownload(downloadId) {
  const index = downloads.value.findIndex(d => d.id === downloadId)
  if (index !== -1) {
    downloads.value.splice(index, 1)
  }
}

// 清除已完成的下载
function clearCompleted() {
  downloads.value = downloads.value.filter(d => 
    d.status === 'downloading'
  )
}

// 暴露方法给父组件
defineExpose({
  addDownload
})
</script>

<style scoped>
.right-panel {
  display: flex;
  background: var(--panel-bg);
  height: 100%;
  position: relative;
}

/* 内容区 - 在左侧，可折叠 */
.panel-content-wrapper {
  display: flex;
  flex-direction: column;
  width: 280px;
  border-left: 1px solid var(--border-color);
  transition: width 0.3s ease, opacity 0.3s ease;
  overflow: hidden;
  flex-shrink: 0;
}

.panel-content-wrapper.collapsed {
  width: 0;
  opacity: 0;
  pointer-events: none;
  border-left: none;
}

/* 按钮栏 - 在最右侧，始终可见 */
.sidebar-buttons {
  width: 60px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 12px 8px;
  background: var(--panel-header-bg);
  border-left: 1px solid var(--border-color);
  margin-left: auto;
}

.sidebar-btn {
  width: 44px !important;
  height: 44px !important;
  padding: 0 !important;
  display: flex !important;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
}

.sidebar-btn :deep(.anticon) {
  font-size: 20px;
}

.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: var(--panel-header-bg);
  border-bottom: 1px solid var(--border-color);
  font-weight: 500;
  cursor: pointer;
  transition: background-color 0.2s ease;
}

.panel-header:hover {
  background: var(--hover-bg);
}

.collapse-btn {
  color: var(--text-color);
  padding: 0;
  width: 20px;
  height: 20px;
}

.panel-content {
  padding: 16px;
  overflow-y: auto;
}

.monitor-content {
  flex: 1;
  min-height: 0;
}

.panel-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 16px;
  background: var(--panel-header-bg);
  border-top: 1px solid var(--border-color);
  font-size: 12px;
}

.last-update {
  color: var(--muted-color);
  transition: color 0.3s ease;
}

.last-update.updating {
  color: var(--primary-color);
}

.update-indicator {
  display: inline-block;
  color: var(--primary-color);
  margin-right: 4px;
  animation: pulse 1.5s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% {
    opacity: 0.4;
  }
  50% {
    opacity: 1;
  }
}

.info-section {
  margin-bottom: 24px;
}

.section-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
  padding-bottom: 8px;
  border-bottom: 2px solid var(--border-color);
}

.section-icon {
  color: var(--primary-color);
  font-size: 16px;
}

.section-header h4 {
  margin: 0;
  color: var(--text-color);
  font-size: 14px;
  font-weight: 600;
}

.info-card {
  background: var(--hover-bg);
  border-radius: 8px;
  padding: 16px;
  border: 1px solid var(--border-color);
}

.info-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 0;
  border-bottom: 1px solid var(--border-color);
}

.info-item:last-child {
  border-bottom: none;
}

.info-label {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--muted-color);
  font-size: 12px;
  font-weight: 500;
}

.item-icon {
  color: var(--primary-color);
  font-size: 14px;
}

.info-value {
  color: var(--text-color);
  font-family: 'Courier New', monospace;
  font-size: 12px;
  font-weight: 500;
}

/* CPU 样式 */
.cpu-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.cpu-model {
  color: var(--text-color);
  font-size: 12px;
  font-weight: 500;
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.cpu-usage {
  color: var(--primary-color);
  font-size: 16px;
  font-weight: 700;
  font-family: 'Courier New', monospace;
}

/* 内存样式 */
.memory-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.memory-stats {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.memory-label {
  color: var(--text-color);
  font-size: 12px;
  font-weight: 500;
}

.memory-usage {
  color: var(--muted-color);
  font-size: 11px;
  font-family: 'Courier New', monospace;
}

.usage-percent {
  color: var(--primary-color);
  font-size: 16px;
  font-weight: 700;
  font-family: 'Courier New', monospace;
}

.memory-details {
  display: flex;
  justify-content: space-between;
  margin-top: 8px;
  padding-top: 8px;
  border-top: 1px solid var(--border-color);
}

.detail-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
}

.detail-label {
  color: var(--muted-color);
  font-size: 10px;
}

.detail-value {
  color: var(--text-color);
  font-size: 11px;
  font-weight: 500;
  font-family: 'Courier New', monospace;
}

/* 进度条容器 */
.progress-container {
  margin: 8px 0;
}

/* 磁盘样式 */
.disk-item {
  margin-bottom: 16px;
  padding: 12px;
  background: var(--panel-bg);
  border-radius: 6px;
  border: 1px solid var(--border-color);
}

.disk-item:last-child {
  margin-bottom: 0;
}

.disk-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.disk-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.disk-device {
  color: var(--text-color);
  font-size: 12px;
  font-weight: 600;
  font-family: 'Courier New', monospace;
}

.disk-mount {
  color: var(--muted-color);
  font-size: 10px;
}

.disk-usage {
  color: var(--primary-color);
  font-size: 14px;
  font-weight: 700;
  font-family: 'Courier New', monospace;
}

.disk-stats {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 6px;
  font-size: 10px;
}

.disk-stat {
  color: var(--text-color);
  font-family: 'Courier New', monospace;
}

.disk-type {
  color: var(--muted-color);
  background: var(--hover-bg);
  padding: 2px 6px;
  border-radius: 3px;
  font-size: 9px;
  text-transform: uppercase;
}

/* 网络样式 */
.network-item {
  margin-bottom: 16px;
  padding: 12px;
  background: var(--panel-bg);
  border-radius: 6px;
  border: 1px solid var(--border-color);
}

.network-item:last-child {
  margin-bottom: 0;
}

.network-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.interface-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.interface-name {
  color: var(--text-color);
  font-size: 12px;
  font-weight: 600;
  font-family: 'Courier New', monospace;
}

.interface-ip {
  color: var(--muted-color);
  font-size: 10px;
  font-family: 'Courier New', monospace;
}

.interface-status {
  padding: 2px 8px;
  border-radius: 12px;
  font-size: 10px;
  font-weight: 500;
  text-transform: uppercase;
  background: var(--error-color);
  color: white;
}

.interface-status.active {
  background: var(--success-color);
}

.network-stats {
  margin-top: 8px;
}

.network-stat {
  display: flex;
  justify-content: space-between;
  gap: 8px;
}

.stat-item {
  display: flex;
  align-items: center;
  gap: 6px;
  flex: 1;
  padding: 6px 8px;
  background: var(--hover-bg);
  border-radius: 4px;
}

.stat-icon {
  color: var(--primary-color);
  font-size: 14px;
  font-weight: bold;
}

.stat-label {
  color: var(--muted-color);
  font-size: 10px;
  flex: 1;
}

.stat-value {
  color: var(--text-color);
  font-size: 10px;
  font-weight: 500;
  font-family: 'Courier New', monospace;
}

.progress-header {
  display: flex;
  justify-content: space-between;
  margin-bottom: 4px;
  font-size: 11px;
  color: var(--text-color);
}

.memory-details, .disk-details {
  margin-top: 4px;
  font-size: 10px;
  color: var(--muted-color);
  display: flex;
  justify-content: space-between;
}

.network-item {
  margin-bottom: 12px;
  padding: 8px;
  background: var(--hover-bg);
  border-radius: 4px;
}

.network-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 6px;
}

.interface-name {
  font-weight: 500;
  color: var(--text-color);
  font-size: 12px;
}

.interface-status {
  padding: 1px 4px;
  border-radius: 2px;
  font-size: 10px;
  background: var(--error-color);
  color: white;
}

.interface-status.active {
  background: var(--success-color);
}

.network-details {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.network-stat {
  display: flex;
  justify-content: space-between;
  font-size: 10px;
}

.stat-label {
  color: var(--muted-color);
}

.stat-value {
  color: var(--text-color);
  font-family: monospace;
}

/* 滚动条样式 */
.panel-content::-webkit-scrollbar {
  width: 4px;
}

.panel-content::-webkit-scrollbar-track {
  background: var(--panel-bg);
}

.panel-content::-webkit-scrollbar-thumb {
  background: var(--border-color);
  border-radius: 2px;
}

.panel-content::-webkit-scrollbar-thumb:hover {
  background: var(--muted-color);
}


/* 下载管理内容 */
.download-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.empty-state {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.download-list {
  flex: 1;
  overflow-y: auto;
}

.download-item {
  display: flex;
  align-items: center;
  padding: 12px;
  border-bottom: 1px solid var(--border-color);
  transition: background-color 0.2s;
}

.download-item:hover {
  background: var(--hover-bg);
}

.download-item.completed {
  background: rgba(82, 196, 26, 0.05);
}

.download-item.error {
  background: rgba(255, 77, 79, 0.05);
}

.download-info {
  flex: 1;
  margin-right: 8px;
  min-width: 0;
}

.download-info .file-name {
  font-weight: 500;
  font-size: 13px;
  color: var(--text-color);
  margin-bottom: 4px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.download-info .file-path {
  font-size: 11px;
  color: var(--muted-color);
  font-family: monospace;
  margin-bottom: 6px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.download-status {
  font-size: 11px;
  margin-top: 4px;
}

.download-status .success {
  color: var(--success-color);
}

.download-status .error {
  color: var(--error-color);
}

.download-actions {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.download-footer {
  padding: 8px 16px;
  border-top: 1px solid var(--border-color);
  background: var(--panel-header-bg);
}

/* 响应式设计 */
@media (max-width: 768px) {
  .right-panel {
    width: 220px !important;
  }
  
  .right-panel.collapsed {
    width: 50px !important;
  }
}
</style>
