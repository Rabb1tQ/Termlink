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
          v-if="hasUnsavedChanges" 
          type="primary" 
          size="small" 
          @click="saveFile"
          :loading="saving"
        >
          保存
        </a-button>
        <a-button 
          v-if="hasUnsavedChanges" 
          size="small" 
          @click="discardChanges"
        >
          撤销更改
        </a-button>
        <a-button 
          size="small" 
          @click="downloadFile" 
          :loading="downloading"
        >
          下载
        </a-button>
        <a-switch
          v-model:checked="readOnly"
          size="small"
          checked-children="只读"
          un-checked-children="编辑"
          style="margin-left: 8px;"
        />
      </div>
    </div>
    
    <div class="editor-content" ref="editorContainer">
      <a-spin :spinning="loading" tip="加载文件内容..." size="large">
        <div v-show="!loading" ref="monacoEditor" class="monaco-container"></div>
      </a-spin>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onBeforeUnmount, watch, nextTick } from 'vue'
import { message } from 'ant-design-vue'
import { invoke } from '@tauri-apps/api/core'
import * as monaco from 'monaco-editor'

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
  },
  theme: {
    type: String,
    default: 'light'
  }
})

const emit = defineEmits(['startDownload'])

// 状态管理
const monacoEditor = ref(null)
const editorContainer = ref(null)
let editor = null
const fileContent = ref('')
const originalContent = ref('')
const readOnly = ref(false)
const hasUnsavedChanges = ref(false)
const loading = ref(false)
const saving = ref(false)
const downloading = ref(false)

// 根据文件扩展名获取语言
function getLanguageFromFilename(filename) {
  const ext = filename.split('.').pop()?.toLowerCase()
  const languageMap = {
    'js': 'javascript',
    'jsx': 'javascript',
    'ts': 'typescript',
    'tsx': 'typescript',
    'json': 'json',
    'html': 'html',
    'htm': 'html',
    'css': 'css',
    'scss': 'scss',
    'less': 'less',
    'md': 'markdown',
    'py': 'python',
    'java': 'java',
    'c': 'c',
    'cpp': 'cpp',
    'h': 'c',
    'hpp': 'cpp',
    'cs': 'csharp',
    'go': 'go',
    'rs': 'rust',
    'php': 'php',
    'rb': 'ruby',
    'sh': 'shell',
    'bash': 'shell',
    'bat': 'bat',
    'ps1': 'powershell',
    'sql': 'sql',
    'xml': 'xml',
    'yaml': 'yaml',
    'yml': 'yaml',
    'toml': 'toml',
    'ini': 'ini',
    'conf': 'plaintext',
    'log': 'plaintext',
    'txt': 'plaintext',
    'vue': 'html'
  }
  return languageMap[ext] || 'plaintext'
}

// 初始化编辑器
function initMonacoEditor() {
  if (!monacoEditor.value || editor) return
  
  const language = getLanguageFromFilename(props.fileInfo.name)
  
  editor = monaco.editor.create(monacoEditor.value, {
    value: fileContent.value,
    language: language,
    theme: props.theme === 'dark' ? 'vs-dark' : 'vs',
    readOnly: readOnly.value,
    automaticLayout: true,
    fontSize: 14,
    lineNumbers: 'on',
    minimap: {
      enabled: true
    },
    scrollBeyondLastLine: false,
    wordWrap: 'on',
    tabSize: 2,
    insertSpaces: true,
    renderWhitespace: 'selection',
    scrollbar: {
      vertical: 'visible',
      horizontal: 'visible',
      verticalScrollbarSize: 12,
      horizontalScrollbarSize: 12
    }
  })
  
  // 监听内容变化
  editor.onDidChangeModelContent(() => {
    const currentValue = editor.getValue()
    hasUnsavedChanges.value = currentValue !== originalContent.value
  })
}

// 加载文件内容
async function loadFileContent() {
  if (!props.active || !props.connectionId) return
  
  loading.value = true
  try {
    const content = await invoke('read_sftp_file', { 
      connectionId: props.connectionId,
      path: props.fileInfo.path 
    })
    fileContent.value = content
    originalContent.value = content
    
    // 等待DOM更新后初始化编辑器
    await nextTick()
    if (editor) {
      editor.setValue(content)
      hasUnsavedChanges.value = false
    } else {
      initMonacoEditor()
    }
  } catch (error) {
    console.error('加载文件失败:', error)
    if (typeof error === 'string' && error.includes('UTF-8')) {
      message.warning('无法加载非文本文件，请下载后查看')
    } else {
      message.error('加载文件失败: ' + error)
    }
  } finally {
    loading.value = false
  }
}

// 保存文件
async function saveFile() {
  if (!props.connectionId || !editor) return
  
  saving.value = true
  try {
    const content = editor.getValue()
    await invoke('write_sftp_file', {
      connectionId: props.connectionId,
      path: props.fileInfo.path,
      content: content
    })
    
    originalContent.value = content
    hasUnsavedChanges.value = false
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

// 撤销更改
function discardChanges() {
  if (editor && originalContent.value) {
    editor.setValue(originalContent.value)
    hasUnsavedChanges.value = false
    message.info('已撤销所有更改')
  }
}

// 下载文件
async function downloadFile() {
  if (!props.connectionId) {
    message.error('无法获取连接信息')
    return
  }
  
  downloading.value = true
  try {
    // 选择下载位置
    const savePath = await invoke('select_download_location', {
      fileName: props.fileInfo.name
    })
    
    if (!savePath) {
      downloading.value = false
      return // 用户取消了选择
    }
    
    // 通过事件通知父组件开始下载
    emit('startDownload', {
      fileName: props.fileInfo.name,
      remotePath: props.fileInfo.path,
      savePath: savePath,
      connectionId: props.connectionId
    })
    
    message.info(`正在下载到: ${savePath}`)
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

// 监听只读模式切换
watch(() => readOnly.value, (newValue) => {
  if (editor) {
    editor.updateOptions({ readOnly: newValue })
  }
})

// 监听主题变化
watch(() => props.theme, (newTheme) => {
  if (editor) {
    monaco.editor.setTheme(newTheme === 'dark' ? 'vs-dark' : 'vs')
  }
})

// 监听active状态变化
watch(() => props.active, async (newActive) => {
  if (newActive && !fileContent.value) {
    await loadFileContent()
  }
  // 当标签页激活时，重新调整编辑器大小
  if (newActive && editor) {
    await nextTick()
    editor.layout()
  }
})

// 组件挂载时加载文件
onMounted(async () => {
  if (props.active) {
    await loadFileContent()
  }
})

// 组件卸载时销毁编辑器
onBeforeUnmount(() => {
  if (editor) {
    editor.dispose()
    editor = null
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
  overflow: hidden;
  position: relative;
  min-height: 0;
}

.monaco-container {
  width: 100%;
  height: 100%;
}

/* 加载状态时的容器 */
:deep(.ant-spin-container) {
  height: 100%;
}

:deep(.ant-spin-nested-loading) {
  height: 100%;
}
</style>
