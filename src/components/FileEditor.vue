<template>
  <div class="file-editor">
    <div class="editor-header">
      <div class="file-info">
        <span class="file-name">{{ fileInfo.name }}</span>
        <span class="file-path">{{ fileInfo.path }}</span>
        <span class="file-size">{{ formatFileSize(fileInfo.size) }}</span>
      </div>
      <div class="editor-actions">
        <a-button 
          v-if="!isEditing" 
          type="primary" 
          size="small" 
          @click="startEditing"
          :loading="loading"
        >
          编辑
        </a-button>
        <a-button-group v-else size="small">
          <a-button @click="saveFile" :loading="saving" type="primary">
            保存
          </a-button>
          <a-button @click="cancelEditing">
            取消
          </a-button>
        </a-button-group>
        <a-button size="small" @click="downloadFile" :loading="downloading">
          下载
        </a-button>
      </div>
    </div>
    
    <div class="editor-content">
      <a-spin :spinning="loading" tip="加载文件内容...">
        <div v-if="!isEditing" class="preview-mode">
          <pre class="file-content">{{ fileContent }}</pre>
        </div>
        <div v-else class="edit-mode">
          <a-textarea
            v-model:value="editContent"
            :rows="25"
            :auto-size="{ minRows: 25, maxRows: 40 }"
            placeholder="文件内容..."
            class="editor-textarea"
          />
        </div>
      </a-spin>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, watch } from 'vue'
import { message } from 'ant-design-vue'
import { invoke } from '@tauri-apps/api/core'

const props = defineProps({
  fileInfo: {
    type: Object,
    required: true
  },
  active: {
    type: Boolean,
    default: false
  },
  connectionId: {
    type: String,
    default: null
  }
})

// 状态管理
const fileContent = ref('')
const editContent = ref('')
const isEditing = ref(false)
const loading = ref(false)
const saving = ref(false)
const downloading = ref(false)

// 加载文件内容
async function loadFileContent() {
  if (!props.active || !props.connectionId) return
  
  loading.value = true
  try {
    const content = await invoke('read_sftp_file', { 
      connectionId: props.connectionId,
      remotePath: props.fileInfo.path 
    })
    fileContent.value = content
    editContent.value = content
  } catch (error) {
    console.error('加载文件失败:', error)
    if (typeof error === 'string' && error.includes('二进制文件')) {
      message.warning('无法编辑二进制文件，请下载后查看')
    } else {
      message.error('加载文件失败: ' + error)
    }
  } finally {
    loading.value = false
  }
}

// 开始编辑
function startEditing() {
  isEditing.value = true
  editContent.value = fileContent.value
}

// 取消编辑
function cancelEditing() {
  isEditing.value = false
  editContent.value = fileContent.value
}

// 保存文件
async function saveFile() {
  if (!props.connectionId) return
  
  saving.value = true
  try {
    await invoke('write_sftp_file', {
      connectionId: props.connectionId,
      remotePath: props.fileInfo.path,
      content: editContent.value
    })
    
    fileContent.value = editContent.value
    isEditing.value = false
    message.success('文件保存成功')
  } catch (error) {
    console.error('保存文件失败:', error)
    if (typeof error === 'string') {
      if (error.includes('权限')) {
        message.error('保存文件失败: 权限不足，请检查文件权限')
      } else {
        message.error('保存文件失败: ' + error)
      }
    } else {
      message.error('保存文件失败: 未知错误')
    }
  } finally {
    saving.value = false
  }
}

// 下载文件
async function downloadFile() {
  downloading.value = true
  try {
    const content = fileContent.value || editContent.value
    const blob = new Blob([content], { type: 'text/plain;charset=utf-8' })
    const url = URL.createObjectURL(blob)
    const link = document.createElement('a')
    link.href = url
    link.download = props.fileInfo.name
    document.body.appendChild(link)
    link.click()
    document.body.removeChild(link)
    URL.revokeObjectURL(url)
    
    message.success('文件下载成功')
  } catch (error) {
    console.error('下载文件失败:', error)
    message.error('下载文件失败: ' + error)
  } finally {
    downloading.value = false
  }
}

// 格式化文件大小
function formatFileSize(bytes) {
  if (!bytes || bytes === 0) return '-'
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(1024))
  return Math.round(bytes / Math.pow(1024, i) * 100) / 100 + ' ' + sizes[i]
}

// 监听active状态变化
watch(() => props.active, (newActive) => {
  if (newActive && !fileContent.value) {
    loadFileContent()
  }
})

// 组件挂载时加载文件
onMounted(() => {
  if (props.active) {
    loadFileContent()
  }
})
</script>

<style scoped>
.file-editor {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--terminal-bg);
}

.editor-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: var(--panel-header-bg);
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
}

.file-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.file-name {
  font-weight: 600;
  font-size: 14px;
  color: var(--primary-color);
}

.file-path {
  font-size: 12px;
  color: var(--muted-color);
  font-family: monospace;
}

.file-size {
  font-size: 11px;
  color: var(--muted-color);
}

.editor-actions {
  display: flex;
  gap: 8px;
  align-items: center;
}

.editor-content {
  flex: 1;
  padding: 16px;
  overflow: hidden;
}

.preview-mode {
  height: 100%;
  overflow: auto;
}

.file-content {
  margin: 0;
  padding: 16px;
  background: var(--panel-bg);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  font-family: 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  line-height: 1.5;
  white-space: pre-wrap;
  word-wrap: break-word;
  color: var(--text-color);
  max-height: 100%;
  overflow: auto;
}

.edit-mode {
  height: 100%;
}

.editor-textarea {
  font-family: 'Fira Code', 'Consolas', monospace !important;
  font-size: 13px !important;
  line-height: 1.5 !important;
  background: var(--panel-bg) !important;
  border-color: var(--border-color) !important;
  color: var(--text-color) !important;
}

.editor-textarea:focus {
  border-color: var(--primary-color) !important;
  box-shadow: 0 0 0 2px rgba(24, 144, 255, 0.2) !important;
}

/* 滚动条样式 */
.file-content::-webkit-scrollbar,
.editor-textarea::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

.file-content::-webkit-scrollbar-track,
.editor-textarea::-webkit-scrollbar-track {
  background: var(--panel-bg);
}

.file-content::-webkit-scrollbar-thumb,
.editor-textarea::-webkit-scrollbar-thumb {
  background: var(--border-color);
  border-radius: 4px;
}

.file-content::-webkit-scrollbar-thumb:hover,
.editor-textarea::-webkit-scrollbar-thumb:hover {
  background: var(--muted-color);
}
</style>
