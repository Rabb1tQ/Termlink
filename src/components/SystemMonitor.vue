<template>
  <div class="system-monitor" v-if="visible">
    <div class="monitor-header">
      <span>系统监控</span>
      <a-button type="text" size="small" @click="$emit('close')" title="关闭">
        <CloseOutlined />
      </a-button>
    </div>
    
    <div class="monitor-content">
      <a-spin :spinning="loading">
        <!-- 系统基本信息 -->
        <div class="info-section">
          <h4>系统信息</h4>
          <div class="info-grid">
            <div class="info-item">
              <span class="label">主机名:</span>
              <span class="value">{{ systemInfo.hostname || '-' }}</span>
            </div>
            <div class="info-item">
              <span class="label">操作系统:</span>
              <span class="value">{{ systemInfo.os || '-' }}</span>
            </div>
            <div class="info-item">
              <span class="label">架构:</span>
              <span class="value">{{ systemInfo.arch || '-' }}</span>
            </div>
            <div class="info-item">
              <span class="label">内核:</span>
              <span class="value">{{ systemInfo.kernel || '-' }}</span>
            </div>
            <div class="info-item">
              <span class="label">运行时间:</span>
              <span class="value">{{ formatUptime(systemInfo.uptime) }}</span>
            </div>
          </div>
        </div>
        
        <!-- CPU信息 -->
        <div class="info-section">
          <h4>CPU使用率</h4>
          <div class="progress-item">
            <div class="progress-header">
              <span>{{ cpuInfo.model || 'CPU' }}</span>
              <span>{{ cpuInfo.usage?.toFixed(1) || 0 }}%</span>
            </div>
            <a-progress 
              :percent="cpuInfo.usage || 0" 
              :show-info="false"
              :stroke-color="getProgressColor(cpuInfo.usage || 0)"
            />
            <div class="cpu-cores" v-if="cpuInfo.cores && cpuInfo.cores.length">
              <div 
                v-for="(core, index) in cpuInfo.cores" 
                :key="index"
                class="core-item"
              >
                <span class="core-label">Core {{ index }}</span>
                <a-progress 
                  :percent="core" 
                  size="small"
                  :show-info="false"
                  :stroke-color="getProgressColor(core)"
                />
                <span class="core-value">{{ core.toFixed(1) }}%</span>
              </div>
            </div>
          </div>
        </div>
        
        <!-- 内存信息 -->
        <div class="info-section">
          <h4>内存使用</h4>
          <div class="progress-item">
            <div class="progress-header">
              <span>物理内存</span>
              <span>{{ formatSize(memoryInfo.used) }} / {{ formatSize(memoryInfo.total) }}</span>
            </div>
            <a-progress 
              :percent="memoryInfo.usage || 0" 
              :show-info="false"
              :stroke-color="getProgressColor(memoryInfo.usage || 0)"
            />
            <div class="memory-details">
              <div class="memory-item">
                <span>可用: {{ formatSize(memoryInfo.available) }}</span>
              </div>
              <div class="memory-item">
                <span>缓存: {{ formatSize(memoryInfo.cached) }}</span>
              </div>
            </div>
          </div>
        </div>
        
        <!-- 磁盘信息 -->
        <div class="info-section">
          <h4>磁盘使用</h4>
          <div 
            v-for="disk in diskInfo" 
            :key="disk.device"
            class="progress-item"
          >
            <div class="progress-header">
              <span>{{ disk.device }} ({{ disk.filesystem }})</span>
              <span>{{ formatSize(disk.used) }} / {{ formatSize(disk.total) }}</span>
            </div>
            <a-progress 
              :percent="disk.usage || 0" 
              :show-info="false"
              :stroke-color="getProgressColor(disk.usage || 0)"
            />
            <div class="disk-details">
              <span>挂载点: {{ disk.mountpoint }}</span>
            </div>
          </div>
        </div>
        
        <!-- 网络监控 -->
        <div class="info-section">
          <h4>网络监控</h4>
          <div 
            v-for="interface_ in networkInfo" 
            :key="interface_.name"
            class="network-item"
          >
            <div class="network-header">
              <span class="interface-name">{{ interface_.name }}</span>
              <span class="interface-status" :class="{ active: interface_.status === 'up' }">
                {{ interface_.status }}
              </span>
            </div>
            <div class="network-details">
              <div class="network-stat">
                <span class="stat-label">↓ 接收速度:</span>
                <span class="stat-value">{{ formatNetworkSpeed(interface_.rx_speed || 0) }}</span>
              </div>
              <div class="network-stat">
                <span class="stat-label">↑ 发送速度:</span>
                <span class="stat-value">{{ formatNetworkSpeed(interface_.tx_speed || 0) }}</span>
              </div>
              <div class="network-stat" v-if="interface_.ip">
                <span class="stat-label">IP:</span>
                <span class="stat-value">{{ interface_.ip }}</span>
              </div>
            </div>
          </div>
        </div>
        
        <!-- 进程信息 -->
        <div class="info-section">
          <h4>进程信息</h4>
          <div class="process-summary">
            <div class="process-stat">
              <span class="stat-label">总进程数:</span>
              <span class="stat-value">{{ processInfo.total || 0 }}</span>
            </div>
            <div class="process-stat">
              <span class="stat-label">运行中:</span>
              <span class="stat-value">{{ processInfo.running || 0 }}</span>
            </div>
            <div class="process-stat">
              <span class="stat-label">休眠:</span>
              <span class="stat-value">{{ processInfo.sleeping || 0 }}</span>
            </div>
          </div>
        </div>
      </a-spin>
    </div>
    
    <div class="monitor-footer">
      <a-button size="small" @click="refreshData" :loading="loading">
        <ReloadOutlined />
        刷新
      </a-button>
      <span class="last-update">
        最后更新: {{ lastUpdate ? new Date(lastUpdate).toLocaleTimeString() : '-' }}
      </span>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted, computed, watch } from 'vue'
import { CloseOutlined, ReloadOutlined } from '@ant-design/icons-vue'
import { message } from 'ant-design-vue'
import { invoke } from '@tauri-apps/api/core'

const props = defineProps({
  visible: {
    type: Boolean,
    default: false
  },
  connectionId: {
    type: String,
    required: true
  }
})

const emit = defineEmits(['close'])

// 状态数据
const loading = ref(false)
const lastUpdate = ref(null)
const systemInfo = ref({}) // 静态系统信息（hostname, os, arch, kernel, boot_time）
const cpuInfo = ref({}) // 包含静态的 model 和动态的 usage、cores
const memoryInfo = ref({})
const diskInfo = ref([])
const networkInfo = ref([])
const processInfo = ref({})

// 标记是否已获取过首次数据
const isFirstFetch = ref(true)

let refreshTimer = null
let refreshDebounceTimer = null

// 本地计算运行时间
function calculateLocalUptime(bootTime) {
  if (!bootTime) return 0
  const now = Math.floor(Date.now() / 1000) // 当前时间戳（秒）
  return Math.max(0, now - bootTime)
}

// 监控数据刷新（优化：首次全量，后续增量）
async function refreshData() {
  if (!props.connectionId) return
  
  loading.value = true
  try {
    if (isFirstFetch.value) {
      // 首次获取：全量数据
      const data = await invoke('get_all_system_info_batch', { 
        connectionId: props.connectionId 
      })
      
      systemInfo.value = data.system
      cpuInfo.value = data.cpu
      memoryInfo.value = data.memory
      diskInfo.value = data.disk
      networkInfo.value = data.network
      processInfo.value = data.process
      
      isFirstFetch.value = false
    } else {
      // 后续获取：只有动态数据
      const data = await invoke('get_dynamic_system_info_batch', { 
        connectionId: props.connectionId 
      })
      
      // 保留 CPU model（静态数据）
      const cpuModel = cpuInfo.value.model || ''
      
      // 更新动态数据
      cpuInfo.value = {
        ...data.cpu,
        model: cpuModel // 保留静态的 CPU 型号
      }
      memoryInfo.value = data.memory
      diskInfo.value = data.disk
      networkInfo.value = data.network
      processInfo.value = data.process
    }
    
    lastUpdate.value = Date.now()
  } catch (error) {
    console.error('获取系统信息失败:', error)
    message.error('获取系统信息失败: ' + error)
  } finally {
    loading.value = false
  }
}

// 静默刷新（不显示loading）
async function refreshDataSilent() {
  if (!props.connectionId) return
  
  try {
    // 只获取动态数据
    const data = await invoke('get_dynamic_system_info_batch', { 
      connectionId: props.connectionId 
    })
    
    // 保留 CPU model（静态数据）
    const cpuModel = cpuInfo.value.model || ''
    
    // 更新动态数据
    cpuInfo.value = {
      ...data.cpu,
      model: cpuModel
    }
    memoryInfo.value = data.memory
    diskInfo.value = data.disk
    networkInfo.value = data.network
    processInfo.value = data.process
    
    lastUpdate.value = Date.now()
  } catch (error) {
    console.error('静默刷新失败:', error)
    // 不显示错误消息，避免干扰用户
  }
}

// 自动刷新（优化：首次全量，后续增量）
function startAutoRefresh() {
  if (refreshTimer) {
    clearInterval(refreshTimer)
  }
  
  refreshTimer = setInterval(() => {
    if (props.visible) {
      // 使用防抖机制
      if (refreshDebounceTimer) {
        clearTimeout(refreshDebounceTimer)
      }
      refreshDebounceTimer = setTimeout(() => {
        refreshDataSilent()
      }, 500) // 500ms防抖
    }
  }, 3000) // 改为每3秒刷新一次（因为已经优化了，可以更频繁）
}

function stopAutoRefresh() {
  if (refreshTimer) {
    clearInterval(refreshTimer)
    refreshTimer = null
  }
  if (refreshDebounceTimer) {
    clearTimeout(refreshDebounceTimer)
    refreshDebounceTimer = null
  }
}

// 格式化文件大小
function formatSize(bytes) {
  if (!bytes || bytes === 0) return '0 B'
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(1024))
  return Math.round(bytes / Math.pow(1024, i) * 100) / 100 + ' ' + sizes[i]
}

// 格式化网络速度
function formatNetworkSpeed(bytesPerSecond) {
  if (!bytesPerSecond || bytesPerSecond === 0) return '0 B/s'
  const sizes = ['B/s', 'KB/s', 'MB/s', 'GB/s']
  const i = Math.floor(Math.log(bytesPerSecond) / Math.log(1024))
  return Math.round(bytesPerSecond / Math.pow(1024, i) * 100) / 100 + ' ' + sizes[i]
}

// 格式化运行时间（本地计算）
function formatUptime(uptime) {
  // 如果有 boot_time，使用本地计算
  if (systemInfo.value.boot_time) {
    const localUptime = calculateLocalUptime(systemInfo.value.boot_time)
    return formatUptimeImpl(localUptime)
  }
  // 否则使用传入的 uptime
  return formatUptimeImpl(uptime)
}

// 实际格式化函数
function formatUptimeImpl(seconds) {
  if (!seconds || seconds === 0) return '-'
  
  const days = Math.floor(seconds / 86400)
  const hours = Math.floor((seconds % 86400) / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  
  if (days > 0) {
    return `${days}天 ${hours}小时 ${minutes}分钟`
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

// 监听可见性变化
const visible = computed(() => props.visible)
const connectionId = computed(() => props.connectionId)

// 生命周期
onMounted(() => {
  // 重置首次获取标记（每次打开面板都重新获取全量数据）
  isFirstFetch.value = true
  
  if (props.visible) {
    refreshData()
    startAutoRefresh()
  }
})

onUnmounted(() => {
  stopAutoRefresh()
})

// 监听属性变化
watch(() => props.visible, (newVisible) => {
  if (newVisible) {
    // 打开面板时重置首次获取标记
    isFirstFetch.value = true
    refreshData()
    startAutoRefresh()
  } else {
    stopAutoRefresh()
  }
})

watch(() => props.connectionId, (newConnectionId) => {
  if (newConnectionId && props.visible) {
    refreshData()
  }
})
</script>

<style scoped>
.system-monitor {
  position: fixed;
  top: 80px;
  left: 20px;
  width: 350px;
  max-height: calc(100vh - 120px);
  background: var(--panel-bg);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
  z-index: 1000;
  overflow: hidden;
}

.monitor-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: var(--panel-header-bg);
  border-bottom: 1px solid var(--border-color);
  font-weight: 500;
}

.monitor-content {
  max-height: calc(80vh - 120px);
  overflow-y: auto;
  padding: 16px;
}

.monitor-footer {
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
}

.info-section {
  margin-bottom: 20px;
}

.info-section h4 {
  margin: 0 0 12px 0;
  color: var(--text-color);
  font-size: 14px;
  font-weight: 500;
  border-bottom: 1px solid var(--border-color);
  padding-bottom: 4px;
}

.info-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
}

.info-item {
  display: flex;
  justify-content: space-between;
  padding: 4px 0;
  font-size: 12px;
}

.label {
  color: var(--muted-color);
  font-weight: 500;
}

.value {
  color: var(--text-color);
  font-family: monospace;
}

.progress-item {
  margin-bottom: 16px;
}

.progress-header {
  display: flex;
  justify-content: space-between;
  margin-bottom: 4px;
  font-size: 12px;
  color: var(--text-color);
}

.cpu-cores {
  margin-top: 8px;
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 8px;
}

.core-item {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 11px;
}

.core-label {
  min-width: 50px;
  color: var(--muted-color);
}

.core-value {
  min-width: 35px;
  text-align: right;
  color: var(--text-color);
}

.memory-details, .disk-details {
  margin-top: 4px;
  font-size: 11px;
  color: var(--muted-color);
}

.memory-details {
  display: flex;
  gap: 16px;
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
  margin-bottom: 8px;
}

.interface-name {
  font-weight: 500;
  color: var(--text-color);
}

.interface-status {
  padding: 2px 6px;
  border-radius: 3px;
  font-size: 11px;
  background: var(--error-color);
  color: white;
}

.interface-status.active {
  background: var(--success-color);
}

.network-details {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 8px;
}

.network-stat {
  display: flex;
  justify-content: space-between;
  font-size: 11px;
}

.stat-label {
  color: var(--muted-color);
}

.stat-value {
  color: var(--text-color);
  font-family: monospace;
}

.process-summary {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 12px;
}

.process-stat {
  text-align: center;
  padding: 8px;
  background: var(--hover-bg);
  border-radius: 4px;
}

.process-stat .stat-label {
  display: block;
  font-size: 11px;
  color: var(--muted-color);
  margin-bottom: 4px;
}

.process-stat .stat-value {
  display: block;
  font-size: 16px;
  font-weight: 500;
  color: var(--text-color);
}

/* 滚动条样式 */
.monitor-content::-webkit-scrollbar {
  width: 6px;
}

.monitor-content::-webkit-scrollbar-track {
  background: var(--panel-bg);
}

.monitor-content::-webkit-scrollbar-thumb {
  background: var(--border-color);
  border-radius: 3px;
}

.monitor-content::-webkit-scrollbar-thumb:hover {
  background: var(--muted-color);
}
</style>
