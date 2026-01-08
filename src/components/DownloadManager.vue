<template>
  <div class="download-manager" v-if="downloads.length > 0">
    <div class="download-header">
      <span>下载管理器</span>
      <a-button type="text" size="small" @click="clearCompleted" title="清除已完成">
        <ClearOutlined />
      </a-button>
    </div>
    
    <div class="download-list">
      <div 
        v-for="download in downloads" 
        :key="download.id"
        class="download-item"
        :class="{ completed: download.status === 'completed', error: download.status === 'error' }"
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
              {{ formatSize(download.downloaded) }} / {{ formatSize(download.total) }}
              ({{ download.progress }}%) - {{ formatSpeed(download.speed) }}
            </span>
            <span v-else-if="download.status === 'completed'" class="success">
              下载完成 - {{ formatSize(download.total) }}
            </span>
            <span v-else-if="download.status === 'error'" class="error">
              下载失败: {{ download.error }}
            </span>
            <span v-else-if="download.status === 'cancelled'" class="cancelled">
              已取消
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
            title="取消下载"
          >
            <StopOutlined />
          </a-button>
          
          <a-button 
            v-if="download.status === 'completed'"
            type="text" 
            size="small"
            @click="openFileLocation(download.savePath)"
            title="打开文件位置"
          >
            <FolderOpenOutlined />
          </a-button>
          
          <a-button 
            v-if="download.status === 'error'"
            type="text" 
            size="small"
            @click="retryDownload(download.id)"
            title="重试下载"
          >
            <ReloadOutlined />
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
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import { 
  ClearOutlined, 
  StopOutlined, 
  FolderOpenOutlined, 
  ReloadOutlined,
  DeleteOutlined 
} from '@ant-design/icons-vue'
import { message } from 'ant-design-vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

const downloads = ref([])
let downloadIdCounter = 0
let progressUnlisten = null

// 添加下载任务
function addDownload(fileName, remotePath, savePath, connectionId) {
  const downloadId = ++downloadIdCounter
  const download = {
    id: downloadId,
    fileName,
    remotePath,
    savePath,
    connectionId,
    status: 'downloading', // downloading, completed, error, cancelled
    progress: 0,
    downloaded: 0,
    total: 0,
    speed: 0,
    startTime: Date.now(),
    error: null
  }
  
  downloads.value.push(download)
  startDownload(download)
  
  return downloadId
}

// 开始下载
async function startDownload(download) {
  try {
    console.log('=== DownloadManager 开始下载 ===', download)
    
    // 开始下载（使用正确的API）
    await invoke('download_sftp_file', {
      connectionId: download.connectionId,
      remotePath: download.remotePath,
      localPath: download.savePath,
      downloadId: download.id
    })
    
    console.log('✓ DownloadManager 下载API调用成功')
    
    if (download.status !== 'cancelled') {
      download.status = 'completed'
      download.progress = 100
      download.downloaded = download.total
      message.success(`文件下载完成: ${download.fileName}`)
    }
  } catch (error) {
    console.error('✗ DownloadManager 下载失败:', error)
    if (download.status !== 'cancelled') {
      download.status = 'error'
      download.error = error.toString()
      message.error(`下载失败: ${download.fileName}`)
    }
  }
}

// 计算下载速度
function calculateSpeed(download) {
  const elapsed = (Date.now() - download.startTime) / 1000
  return elapsed > 0 ? download.downloaded / elapsed : 0
}

// 取消下载
async function cancelDownload(downloadId) {
  const download = downloads.value.find(d => d.id === downloadId)
  if (download && download.status === 'downloading') {
    download.status = 'cancelled'
    
    try {
      await invoke('cancel_download', { downloadId })
      message.info(`已取消下载: ${download.fileName}`)
    } catch (error) {
      console.error('取消下载失败:', error)
    }
  }
}

// 重试下载
function retryDownload(downloadId) {
  const download = downloads.value.find(d => d.id === downloadId)
  if (download) {
    download.status = 'downloading'
    download.progress = 0
    download.downloaded = 0
    download.error = null
    download.startTime = Date.now()
    startDownload(download)
  }
}

// 打开文件位置
async function openFileLocation(filePath) {
  try {
    await invoke('open_file_location', { path: filePath })
  } catch (error) {
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
    d.status === 'downloading' || d.status === 'error'
  )
}

// 格式化文件大小
function formatSize(bytes) {
  if (!bytes || bytes === 0) return '0 B'
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(1024))
  return Math.round(bytes / Math.pow(1024, i) * 100) / 100 + ' ' + sizes[i]
}

// 格式化速度
function formatSpeed(bytesPerSecond) {
  return formatSize(bytesPerSecond) + '/s'
}

// 暴露方法给父组件
defineExpose({
  addDownload,
  cancelDownload
})

// 生命周期钩子
onMounted(async () => {
  // 监听下载进度事件
  progressUnlisten = await listen('download-progress', (event) => {
    const { downloadId, downloaded, total, progress } = event.payload
    const download = downloads.value.find(d => d.id === downloadId)
    if (download && download.status === 'downloading') {
      download.downloaded = downloaded
      download.total = total
      download.progress = progress
      console.log(`📥 DownloadManager 进度: ${download.fileName} - ${progress}% (${formatSize(downloaded)}/${formatSize(total)})`)
    }
  })
})

onUnmounted(() => {
  // 取消事件监听
  if (progressUnlisten) {
    progressUnlisten()
  }
})
</script>

<style scoped>
.download-manager {
  position: fixed;
  bottom: 20px;
  right: 20px;
  width: 400px;
  background: var(--panel-bg);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  z-index: 1000;
}

.download-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: var(--panel-header-bg);
  border-bottom: 1px solid var(--border-color);
  border-radius: 8px 8px 0 0;
  font-weight: 500;
}

.download-list {
  max-height: 300px;
  overflow-y: auto;
}

.download-item {
  display: flex;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border-color);
}

.download-item:last-child {
  border-bottom: none;
}

.download-item.completed {
  background: rgba(82, 196, 26, 0.05);
}

.download-item.error {
  background: rgba(255, 77, 79, 0.05);
}

.download-info {
  flex: 1;
  margin-right: 12px;
}

.file-name {
  font-weight: 500;
  margin-bottom: 4px;
  color: var(--text-color);
}

.file-path {
  font-size: 12px;
  color: var(--muted-color);
  margin-bottom: 8px;
  font-family: monospace;
}

.download-status {
  font-size: 12px;
  margin-top: 4px;
}

.download-status .success {
  color: var(--success-color);
}

.download-status .error {
  color: var(--error-color);
}

.download-status .cancelled {
  color: var(--muted-color);
}

.download-actions {
  display: flex;
  gap: 4px;
}

</style>
