<template>
  <div class="left-panel" :class="{ collapsed: collapsed }">
    <div class="panel-header">
      <span>连接管理</span>
      <a-button 
        type="text" 
        size="small" 
        @click="$emit('toggle')"
        class="collapse-btn"
      >
        {{ collapsed ? '>' : '<' }}
      </a-button>
    </div>
    
    <div class="panel-content" v-if="!collapsed">
      <div class="section-title">已保存的连接</div>
      
      <!-- 搜索框 -->
      <div class="search-section">
        <a-input-search
          v-model:value="searchText"
          placeholder="搜索连接..."
          size="small"
          allow-clear
          @search="onSearch"
          @change="onSearch"
        />
      </div>
      
      <!-- 分组视图切换 -->
      <div class="view-controls">
        <a-segmented 
          v-model:value="viewMode" 
          :options="[
            { label: '列表', value: 'list' },
            { label: '分组', value: 'group' }
          ]"
          size="small"
        />
      </div>
      
      <!-- 列表视图 -->
      <div v-if="viewMode === 'list'" class="list-view">
      <a-list 
          :data-source="filteredProfiles" 
        :split="false"
        size="small"
        class="profile-list"
      >
        <template #renderItem="{ item }">
          <a-list-item 
            @click="$emit('launchProfile', item)" 
              @contextmenu.prevent="handleContextMenu($event, item)"
            class="profile-item"
          >
              <a-list-item-meta>
                <template #title>
                  <div class="profile-title">
                    <span>{{ item.name || (item.username ? `${item.username}@${item.host}` : item.host) }}</span>
                    <div class="profile-tags" v-if="item.tags && item.tags.length">
                      <a-tag v-for="tag in item.tags" :key="tag" size="small">{{ tag }}</a-tag>
                    </div>
                  </div>
                </template>
                <template #description>
                  <div class="profile-desc">
                    <span>{{ item.host }}:{{ item.port }}</span>
                    <a-tag v-if="item.group" size="small" color="blue">{{ item.group }}</a-tag>
                  </div>
                </template>
              </a-list-item-meta>
              <template #actions>
                <a-button 
                  type="text" 
                  size="small" 
                  danger
                  @click.stop="deleteProfile(item)"
                  class="delete-btn"
                  title="删除连接"
                >
                  <DeleteOutlined />
                </a-button>
              </template>
          </a-list-item>
        </template>
      </a-list>
      </div>
      
      <!-- 分组视图 -->
      <div v-else class="group-view">
        <div v-for="(groupProfiles, groupName) in groupedProfiles" :key="groupName" class="group-section">
          <div class="group-header" @click="toggleGroup(groupName)">
            <span class="group-icon">
              {{ expandedGroups.has(groupName) ? '▼' : '▶' }}
            </span>
            <span class="group-name">{{ groupName || '未分组' }}</span>
            <a-tag size="small">{{ groupProfiles.length }}</a-tag>
          </div>
          <div v-show="expandedGroups.has(groupName)" class="group-content">
            <div 
              v-for="item in groupProfiles" 
              :key="item.id"
              @click="$emit('launchProfile', item)"
              @contextmenu.prevent="handleContextMenu($event, item)"
              class="group-item"
            >
              <div class="item-content">
                <div class="item-title">{{ item.name || (item.username ? `${item.username}@${item.host}` : item.host) }}</div>
                <div class="item-desc">{{ item.host }}:{{ item.port }}</div>
                <div class="item-tags" v-if="item.tags && item.tags.length">
                  <a-tag v-for="tag in item.tags" :key="tag" size="small">{{ tag }}</a-tag>
                </div>
              </div>
              <a-button 
                type="text" 
                size="small" 
                danger
                @click.stop="deleteProfile(item)"
                class="delete-btn"
                title="删除连接"
              >
                <DeleteOutlined />
              </a-button>
            </div>
          </div>
        </div>
      </div>
      
      <div class="section-title">文件管理器</div>
      <div class="file-manager-section">
        <!-- SFTP连接状态显示 -->
        <div v-if="currentSftpConnection" class="current-connection">
          <div class="connection-info">
            <div class="status-indicator connected"></div>
            <span>{{ currentSftpConnection.title }}</span>
          </div>
        </div>
        
        <!-- SFTP文件浏览器 -->
        <div v-if="currentSftpConnection" class="sftp-browser">
          <div class="browser-toolbar">
            <a-button-group size="small">
              <a-button @click="sftpGoBack" :disabled="!canSftpGoBack" title="返回">
                <ArrowLeftOutlined />
              </a-button>
              <a-button @click="sftpGoUp" :disabled="sftpIsAtRoot" title="上级目录">
                <ArrowUpOutlined />
              </a-button>
              <a-button @click="refreshSftpFiles" title="刷新">
                <ReloadOutlined />
              </a-button>
              <a-button @click="createNewFolder" title="新建文件夹">
                <FolderAddOutlined />
              </a-button>
            </a-button-group>
            
            <div class="toolbar-right">
              <a-tooltip title="显示隐藏文件">
                <a-button 
                  type="text" 
                  size="small" 
                  @click="toggleShowHidden"
                  :class="{ active: showHiddenFiles }"
                >
                  <EyeOutlined v-if="showHiddenFiles" />
                  <EyeInvisibleOutlined v-else />
                </a-button>
              </a-tooltip>
            </div>
          </div>
          
          <div class="current-path">
            <a-input 
              v-model:value="currentSftpState.pathInput" 
              @pressEnter="navigateToPath"
              @blur="navigateToPath"
              size="small"
              placeholder="输入路径后按Enter"
              class="path-input"
            />
          </div>
          
          <div 
            class="file-list" 
            ref="fileListRef"
            @drop.prevent="handleDrop"
            @dragover.prevent="handleDragOver"
            @dragleave.prevent="handleDragLeave"
            @dragenter.prevent="handleDragEnter"
            :class="{ 'drag-over': isDraggingOver }"
          >
            <a-spin :spinning="currentSftpState?.loading || false" size="small">
              <div 
                v-for="file in currentSftpState?.files || []" 
                :key="file.name"
                @click="handleSftpFileClick(file)"
                @dblclick="handleSftpFileDoubleClick(file)"
                @contextmenu.prevent="showSftpContextMenu($event, file)"
                class="file-item"
                :class="{ directory: file.is_dir }"
              >
                <component :is="getSftpFileIcon(file)" class="file-icon" />
                <span class="file-name">{{ file.name }}</span>
                <div v-if="!file.is_dir" class="file-size">
                  {{ formatFileSize(file.size) }}
                </div>
              </div>
              
              <!-- 拖拽提示 -->
              <div v-if="isDraggingOver" class="drag-overlay">
                <div class="drag-hint">
                  <CloudUploadOutlined style="font-size: 48px;" />
                  <div>释放以上传文件到当前目录</div>
                </div>
              </div>
            </a-spin>
          </div>
        </div>
        
        <!-- 未连接状态 -->
        <div v-else class="no-connection">
          <a-empty description="请切换到SSH标签页来浏览远程文件" size="small" />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch } from 'vue'
import { 
  FolderOutlined, 
  DeleteOutlined,
  CloseOutlined,
  ArrowLeftOutlined,
  ArrowUpOutlined,
  ReloadOutlined,
  FileOutlined,
  FileTextOutlined,
  FileImageOutlined,
  FileZipOutlined,
  VideoCameraOutlined,
  SoundOutlined,
  EyeOutlined,
  EyeInvisibleOutlined,
  CloudUploadOutlined,
  FolderAddOutlined
} from '@ant-design/icons-vue'
import { Modal, message, Dropdown, Menu } from 'ant-design-vue'
import { invoke } from '@tauri-apps/api/core'

const props = defineProps({
  collapsed: {
    type: Boolean,
    default: false
  },
  profiles: {
    type: Array,
    default: () => []
  },
  activeTab: {
    type: Object,
    default: null
  }
})

const emit = defineEmits(['toggle', 'launchProfile', 'showFileManager', 'refreshProfiles', 'openFilePreview', 'startDownload', 'editProfile'])

// 搜索和视图状态
const searchText = ref('')
const viewMode = ref('list')
const expandedGroups = ref(new Set(['未分组'])) // 默认展开未分组

// 搜索功能
function onSearch() {
  // 搜索时的逻辑在computed中处理
}

// 过滤后的配置文件
const filteredProfiles = computed(() => {
  if (!searchText.value.trim()) {
    return props.profiles
  }
  
  const search = searchText.value.toLowerCase()
  return props.profiles.filter(profile => {
    const name = profile.name?.toLowerCase() || ''
    const host = profile.host?.toLowerCase() || ''
    const username = profile.username?.toLowerCase() || ''
    const group = profile.group?.toLowerCase() || ''
    const tags = profile.tags?.join(' ')?.toLowerCase() || ''
    
    return name.includes(search) || 
           host.includes(search) || 
           username.includes(search) ||
           group.includes(search) ||
           tags.includes(search)
  })
})

// 分组后的配置文件
const groupedProfiles = computed(() => {
  const filtered = filteredProfiles.value
  const groups = {}
  
  filtered.forEach(profile => {
    const groupName = profile.group || '未分组'
    if (!groups[groupName]) {
      groups[groupName] = []
    }
    groups[groupName].push(profile)
  })
  
  // 按分组名排序，未分组放在最前面
  const sortedGroups = {}
  if (groups['未分组']) {
    sortedGroups['未分组'] = groups['未分组']
  }
  
  Object.keys(groups)
    .filter(name => name !== '未分组')
    .sort()
    .forEach(name => {
      sortedGroups[name] = groups[name]
    })
  
  return sortedGroups
})

// 切换分组展开状态
function toggleGroup(groupName) {
  if (expandedGroups.value.has(groupName)) {
    expandedGroups.value.delete(groupName)
  } else {
    expandedGroups.value.add(groupName)
  }
}

// SFTP相关状态 - 为每个连接保存独立状态
const sftpStatesByConnection = ref(new Map())
const fileListRef = ref(null)
const showHiddenFiles = ref(false)
const isDraggingOver = ref(false)

// 创建初始SFTP状态
function createInitialSftpState() {
  return {
    currentPath: '/',
    pathInput: '/',
    files: [],
    loading: false,
    history: [],
    historyIndex: -1
  }
}

// 获取当前连接ID
const currentConnectionId = computed(() => {
  if (!props.activeTab) return null
  
  // SSH标签页：使用 sftpConnectionId
  if (props.activeTab.type === 'ssh' && props.activeTab.sftpConnectionId) {
    return props.activeTab.sftpConnectionId
  }
  
  // File标签页：使用 connectionId（从SSH继承）
  if (props.activeTab.type === 'file' && props.activeTab.connectionId) {
    return props.activeTab.connectionId
  }
  
  // 其他类型（如local）不显示SFTP
  return null
})

// 获取或创建当前连接的状态
const currentSftpState = computed(() => {
  const connId = currentConnectionId.value
  if (!connId) return null
  
  // 如果这个连接是第一次使用，创建初始状态
  if (!sftpStatesByConnection.value.has(connId)) {
    sftpStatesByConnection.value.set(connId, createInitialSftpState())
  }
  
  return sftpStatesByConnection.value.get(connId)
})

// 当前SFTP连接信息（用于显示）
const currentSftpConnection = computed(() => {
  if (!currentConnectionId.value) return null
  
  return {
    id: currentConnectionId.value,
    title: props.activeTab?.title || 'SFTP'
  }
})

// SFTP导航状态
const canSftpGoBack = computed(() => {
  const state = currentSftpState.value
  return state ? state.historyIndex > 0 : false
})

const sftpIsAtRoot = computed(() => {
  const state = currentSftpState.value
  return state ? (!state.currentPath || state.currentPath === '/') : true
})

// 监听连接ID变化，按需加载SFTP文件
watch(() => currentConnectionId.value, async (newConnId, oldConnId) => {
  if (newConnId && newConnId !== oldConnId) {
    // 切换到了不同的连接
    const state = currentSftpState.value
    
    // 如果是新连接且还没加载过文件，加载根目录
    if (state && state.files.length === 0 && state.currentPath === '/') {
      try {
        await loadSftpFiles('/')
      } catch (error) {
        console.warn('加载SFTP文件失败:', error)
      }
    }
  }
}, { immediate: false })

// 加载SFTP文件列表
async function loadSftpFiles(path) {
  const state = currentSftpState.value
  if (!state || !currentSftpConnection.value) return
  
  state.loading = true
  try {
    // 使用后端API加载文件
    const files = await invoke('list_sftp_files', { 
      connectionId: currentSftpConnection.value.id,
      path 
    })
    
    // 根据设置过滤隐藏文件
    state.files = showHiddenFiles.value 
      ? files 
      : files.filter(file => !file.name.startsWith('.'))
    
    // 更新历史记录
    if (state.historyIndex === -1 || state.history[state.historyIndex] !== path) {
      state.history = state.history.slice(0, state.historyIndex + 1)
      state.history.push(path)
      state.historyIndex = state.history.length - 1
    }
    
    state.currentPath = path
    state.pathInput = path // 同步更新路径输入框
  } catch (error) {
    console.error('加载文件列表失败:', error)
    message.error('加载文件列表失败: ' + error)
  } finally {
    state.loading = false
  }
}

// 导航到指定路径
function navigateToPath() {
  const state = currentSftpState.value
  if (!state) return
  
  const path = state.pathInput.trim()
  if (path && path !== state.currentPath) {
    loadSftpFiles(path)
  } else {
    // 如果输入为空或与当前路径相同，恢复原值
    state.pathInput = state.currentPath
  }
}

// SFTP文件点击处理（单击选中）
function handleSftpFileClick(file) {
  // 单击仅用于选中，不执行操作
  console.log('选中文件:', file.name)
}

// SFTP文件双击处理
function handleSftpFileDoubleClick(file) {
  const state = currentSftpState.value
  if (!state) return
  
  if (file.is_dir) {
    // 文件夹双击进入
    const newPath = state.currentPath === '/' 
      ? `/${file.name}` 
      : `${state.currentPath}/${file.name}`
    loadSftpFiles(newPath)
  } else {
    // 文件双击仅在文本文件时打开预览，其他文件不做任何操作
    if (isTextFile(file.name)) {
      openFilePreview(file)
    }
    // 移除了对非文本文件的showFileActions调用
  }
}

// 判断是否为文本文件
function isTextFile(filename) {
  const textExts = [
    'txt', 'md', 'json', 'xml', 'html', 'css', 'js', 'ts', 'vue', 
    'py', 'java', 'cpp', 'c', 'h', 'rs', 'go', 'php', 'rb', 'sh',
    'yml', 'yaml', 'ini', 'conf', 'log', 'sql', 'csv'
  ]
  const ext = filename.split('.').pop()?.toLowerCase()
  return textExts.includes(ext)
}

// 打开文件预览
async function openFilePreview(file) {
  const state = currentSftpState.value
  if (!state) return
  
  const filePath = state.currentPath === '/' 
    ? `/${file.name}` 
    : `${state.currentPath}/${file.name}`
    
  emit('openFilePreview', {
    name: file.name,
    path: filePath,
    size: file.size
  })
}

// 显示文件操作选项
function showFileActions(file) {
  Modal.confirm({
    title: `文件操作: ${file.name}`,
    content: '选择要执行的操作',
    okText: '下载',
    cancelText: '取消',
    onOk: () => downloadFile(file)
  })
}

// 下载文件
async function downloadFile(file) {
  const state = currentSftpState.value
  if (!state || !currentSftpConnection.value) return
  
  try {
    const remotePath = state.currentPath === '/' 
      ? `/${file.name}` 
      : `${state.currentPath}/${file.name}`
    
    // 选择下载位置
    const savePath = await invoke('select_download_location', {
      fileName: file.name
    })
    
    if (!savePath) {
      return // 用户取消了选择
    }
    
    // 通过事件通知父组件开始下载
    emit('startDownload', {
      fileName: file.name,
      remotePath: remotePath,
      savePath: savePath,
      connectionId: currentSftpConnection.value.id
    })
    
  } catch (error) {
    console.error('下载文件失败:', error)
    message.error('下载文件失败: ' + error)
  }
}

// SFTP导航
function sftpGoBack() {
  const state = currentSftpState.value
  if (!state || !canSftpGoBack.value) return
  
  state.historyIndex--
  loadSftpFiles(state.history[state.historyIndex])
}

async function sftpGoUp() {
  const state = currentSftpState.value
  if (!state || sftpIsAtRoot.value) return
  
  const parts = state.currentPath.split('/').filter(p => p)
  parts.pop()
  const newPath = parts.length > 0 ? '/' + parts.join('/') : '/'
  loadSftpFiles(newPath)
}

function refreshSftpFiles() {
  const state = currentSftpState.value
  if (!state) return
  
  loadSftpFiles(state.currentPath)
}

// 切换隐藏文件显示
function toggleShowHidden() {
  const state = currentSftpState.value
  if (!state) return
  
  showHiddenFiles.value = !showHiddenFiles.value
  // 重新加载当前目录
  loadSftpFiles(state.currentPath)
}

// 显示SFTP右键菜单
function showSftpContextMenu(event, file) {
  event.preventDefault()
  
  // 移除已存在的菜单
  const existingMenu = document.querySelector('.sftp-context-menu')
  if (existingMenu) {
    existingMenu.remove()
  }
  
  // 创建菜单
  const menu = document.createElement('div')
  menu.className = 'sftp-context-menu'
  menu.style.cssText = `
    position: fixed;
    left: ${event.clientX}px;
    top: ${event.clientY}px;
    background: var(--panel-bg);
    border: 1px solid var(--border-color);
    border-radius: 4px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    z-index: 10000;
    min-width: 150px;
    padding: 4px 0;
  `
  
  // 菜单项
  const menuItems = []
  
  // 下载
  menuItems.push({
    label: '📥 下载',
    action: () => {
      downloadFile(file)
      menu.remove()
    }
  })
  
  // 如果是文本文件，添加打开/预览选项
  if (!file.is_dir && isTextFile(file.name)) {
    menuItems.push({
      label: '📄 打开',
      action: () => {
        openFilePreview(file)
        menu.remove()
      }
    })
  }
  
  // 分隔线
  menuItems.push({ divider: true })
  
  // 重命名
  menuItems.push({
    label: '✏️ 重命名',
    action: () => {
      renameFile(file)
      menu.remove()
    }
  })
  
  // 删除
  menuItems.push({
    label: '🗑️ 删除',
    action: () => {
      deleteFile(file)
      menu.remove()
    },
    danger: true
  })
  
  // 分隔线
  menuItems.push({ divider: true })
  
  // 复制路径
  menuItems.push({
    label: '📋 复制路径',
    action: () => {
      copyFilePath(file)
      menu.remove()
    }
  })
  
  // 添加菜单项
  menuItems.forEach(item => {
    if (item.divider) {
      const divider = document.createElement('div')
      divider.style.cssText = `
        height: 1px;
        background: var(--border-color);
        margin: 4px 0;
      `
      menu.appendChild(divider)
    } else {
      const menuItem = document.createElement('div')
      menuItem.style.cssText = `
        padding: 8px 16px;
        cursor: pointer;
        color: ${item.danger ? 'var(--error-color)' : 'var(--text-color)'};
        font-size: 14px;
        transition: background-color 0.2s;
      `
      menuItem.textContent = item.label
      menuItem.addEventListener('mouseenter', () => {
        menuItem.style.backgroundColor = 'var(--hover-bg)'
      })
      menuItem.addEventListener('mouseleave', () => {
        menuItem.style.backgroundColor = 'transparent'
      })
      menuItem.addEventListener('click', item.action)
      menu.appendChild(menuItem)
    }
  })
  
  // 添加到页面
  document.body.appendChild(menu)
  
  // 点击其他地方关闭菜单
  const closeMenu = (e) => {
    if (!menu.contains(e.target)) {
      menu.remove()
      document.removeEventListener('click', closeMenu)
    }
  }
  
  setTimeout(() => {
    document.addEventListener('click', closeMenu)
  }, 0)
}

// 获取文件图标
function getSftpFileIcon(file) {
  if (file.is_dir) return FolderOutlined
  
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

// 格式化文件大小
function formatFileSize(bytes) {
  if (!bytes || bytes === 0) return '-'
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(1024))
  return Math.round(bytes / Math.pow(1024, i) * 100) / 100 + ' ' + sizes[i]
}

// 重命名文件/文件夹
async function renameFile(file) {
  Modal.confirm({
    title: '重命名',
    content: () => {
      const input = document.createElement('input')
      input.value = file.name
      input.style.cssText = 'width: 100%; padding: 8px; border: 1px solid var(--border-color); border-radius: 4px; background: var(--panel-bg); color: var(--text-color);'
      setTimeout(() => {
        input.focus()
        // 选中文件名（不包含扩展名）
        if (!file.is_dir) {
          const lastDotIndex = file.name.lastIndexOf('.')
          if (lastDotIndex > 0) {
            input.setSelectionRange(0, lastDotIndex)
          } else {
            input.select()
          }
        } else {
          input.select()
        }
      }, 100)
      
      return input
    },
    okText: '重命名',
    cancelText: '取消',
    onOk: async () => {
      const input = document.querySelector('.ant-modal-body input')
      const newName = input?.value?.trim()
      
      if (!newName || newName === file.name) {
        return
      }
      
      const state = currentSftpState.value
      if (!state || !currentSftpConnection.value) return
      
      try {
        const oldPath = state.currentPath === '/' 
          ? `/${file.name}` 
          : `${state.currentPath}/${file.name}`
        const newPath = state.currentPath === '/' 
          ? `/${newName}` 
          : `${state.currentPath}/${newName}`
        
        await invoke('rename_sftp_file', {
          connectionId: currentSftpConnection.value.id,
          oldPath,
          newPath
        })
        
        message.success('重命名成功')
        refreshSftpFiles()
      } catch (error) {
        console.error('重命名失败:', error)
        message.error('重命名失败: ' + error)
      }
    }
  })
}

// 删除文件/文件夹
async function deleteFile(file) {
  Modal.confirm({
    title: '确认删除',
    content: `确定要删除 "${file.name}" 吗？此操作无法撤销。`,
    okText: '删除',
    okType: 'danger',
    cancelText: '取消',
    onOk: async () => {
      const state = currentSftpState.value
      if (!state || !currentSftpConnection.value) return
      
      try {
        const filePath = state.currentPath === '/' 
          ? `/${file.name}` 
          : `${state.currentPath}/${file.name}`
        
        if (file.is_dir) {
          await invoke('delete_sftp_directory', {
            connectionId: currentSftpConnection.value.id,
            path: filePath
          })
        } else {
          await invoke('delete_sftp_file', {
            connectionId: currentSftpConnection.value.id,
            path: filePath
          })
        }
        
        message.success('删除成功')
        refreshSftpFiles()
      } catch (error) {
        console.error('删除失败:', error)
        message.error('删除失败: ' + error)
      }
    }
  })
}

// 复制文件路径
function copyFilePath(file) {
  const state = currentSftpState.value
  if (!state) return
  
  const filePath = state.currentPath === '/' 
    ? `/${file.name}` 
    : `${state.currentPath}/${file.name}`
  
  navigator.clipboard.writeText(filePath).then(() => {
    message.success('路径已复制到剪贴板')
  }).catch(err => {
    console.error('复制失败:', err)
    message.error('复制失败')
  })
}

// 拖拽事件处理
function handleDragEnter(event) {
  if (!currentSftpConnection.value) return
  isDraggingOver.value = true
}

function handleDragOver(event) {
  if (!currentSftpConnection.value) return
  event.dataTransfer.dropEffect = 'copy'
}

function handleDragLeave(event) {
  // 只在离开整个区域时设置为false
  if (event.target === fileListRef.value) {
    isDraggingOver.value = false
  }
}

async function handleDrop(event) {
  isDraggingOver.value = false
  
  if (!currentSftpConnection.value) {
    message.warning('没有活动的SFTP连接')
    return
  }
  
  const files = event.dataTransfer?.files
  if (!files || files.length === 0) return
  
  // 上传所有文件
  for (const file of files) {
    await uploadFileToServer(file)
  }
}

// 上传文件到服务器
async function uploadFileToServer(file) {
  const state = currentSftpState.value
  if (!state || !currentSftpConnection.value) return
  
  try {
    // 获取文件路径（Tauri会处理这个）
    const localPath = file.path
    
    if (!localPath) {
      message.error(`无法获取文件路径: ${file.name}`)
      return
    }
    
    const remotePath = state.currentPath === '/' 
      ? `/${file.name}` 
      : `${state.currentPath}/${file.name}`
    
    message.loading(`正在上传 ${file.name}...`, 0)
    
    await invoke('upload_sftp_file', {
      connectionId: currentSftpConnection.value.id,
      localPath,
      remotePath
    })
    
    message.destroy()
    message.success(`上传成功: ${file.name}`)
    
    // 刷新文件列表
    refreshSftpFiles()
    
  } catch (error) {
    message.destroy()
    console.error('上传文件失败:', error)
    message.error(`上传失败: ${file.name} - ${error}`)
  }
}

// 创建新文件夹
async function createNewFolder() {
  const state = currentSftpState.value
  if (!state || !currentSftpConnection.value) return
  
  Modal.confirm({
    title: '新建文件夹',
    content: () => {
      const input = document.createElement('input')
      input.placeholder = '请输入文件夹名称'
      input.style.cssText = 'width: 100%; padding: 8px; border: 1px solid var(--border-color); border-radius: 4px; background: var(--panel-bg); color: var(--text-color);'
      setTimeout(() => input.focus(), 100)
      return input
    },
    okText: '创建',
    cancelText: '取消',
    onOk: async () => {
      const input = document.querySelector('.ant-modal-body input')
      const folderName = input?.value?.trim()
      
      if (!folderName) {
        message.warning('请输入文件夹名称')
        return Promise.reject()
      }
      
      try {
        const folderPath = state.currentPath === '/' 
          ? `/${folderName}` 
          : `${state.currentPath}/${folderName}`
        
        await invoke('create_sftp_directory', {
          connectionId: currentSftpConnection.value.id,
          path: folderPath
        })
        
        message.success('文件夹创建成功')
        refreshSftpFiles()
      } catch (error) {
        console.error('创建文件夹失败:', error)
        message.error('创建文件夹失败: ' + error)
        return Promise.reject()
      }
    }
  })
}

// 删除配置文件
async function deleteProfile(profile) {
  Modal.confirm({
    title: '确认删除',
    content: `确定要删除连接 "${profile.username ? `${profile.username}@${profile.host}` : profile.host}" 吗？此操作无法撤销。`,
    okText: '删除',
    okType: 'danger',
    cancelText: '取消',
    async onOk() {
      try {
        await invoke('delete_ssh_profile', { profileId: profile.id })
        message.success('连接已删除')
        emit('refreshProfiles')
      } catch (error) {
        console.error('删除连接失败:', error)
        message.error('删除连接失败')
      }
    }
  })
}

// 右键菜单
function handleContextMenu(event, profile) {
  event.preventDefault()
  showProfileContextMenu(event, profile)
}

// 显示SSH配置文件的右键菜单
function showProfileContextMenu(event, profile) {
  // 移除已存在的菜单
  const existingMenu = document.querySelector('.profile-context-menu')
  if (existingMenu) {
    existingMenu.remove()
  }
  
  // 创建菜单
  const menu = document.createElement('div')
  menu.className = 'profile-context-menu'
  menu.style.cssText = `
    position: fixed;
    left: ${event.clientX}px;
    top: ${event.clientY}px;
    background: var(--panel-bg);
    border: 1px solid var(--border-color);
    border-radius: 4px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    z-index: 10000;
    min-width: 150px;
    padding: 4px 0;
  `
  
  // 菜单项
  const menuItems = [
    {
      label: '编辑',
      action: () => {
        editProfile(profile)
        menu.remove()
      }
    },
    {
      label: '复制配置',
      action: () => {
        copyProfileConfig(profile)
        menu.remove()
      }
    },
    {
      label: '删除',
      action: () => {
        deleteProfile(profile)
        menu.remove()
      }
    }
  ]
  
  // 添加菜单项
  menuItems.forEach(item => {
    const menuItem = document.createElement('div')
    menuItem.style.cssText = `
      padding: 8px 16px;
      cursor: pointer;
      color: var(--text-color);
      font-size: 14px;
      transition: background-color 0.2s;
    `
    menuItem.textContent = item.label
    menuItem.addEventListener('mouseenter', () => {
      menuItem.style.backgroundColor = 'var(--hover-bg)'
    })
    menuItem.addEventListener('mouseleave', () => {
      menuItem.style.backgroundColor = 'transparent'
    })
    menuItem.addEventListener('click', item.action)
    menu.appendChild(menuItem)
  })
  
  // 添加到页面
  document.body.appendChild(menu)
  
  // 点击其他地方关闭菜单
  const closeMenu = (e) => {
    if (!menu.contains(e.target)) {
      menu.remove()
      document.removeEventListener('click', closeMenu)
    }
  }
  
  setTimeout(() => {
    document.addEventListener('click', closeMenu)
  }, 0)
}

// 编辑配置文件
function editProfile(profile) {
  // 触发编辑事件，让父组件处理
  emit('editProfile', profile)
}

// 复制配置信息
function copyProfileConfig(profile) {
  const configText = `名称: ${profile.name || '未命名'}
主机: ${profile.host}
端口: ${profile.port}
用户名: ${profile.username}
分组: ${profile.group || '未分组'}
标签: ${profile.tags ? profile.tags.join(', ') : '无'}
认证方式: ${profile.usePrivateKey ? '私钥' : '密码'}`
  
  navigator.clipboard.writeText(configText).then(() => {
    message.success('配置信息已复制到剪贴板')
  }).catch(err => {
    console.error('复制失败:', err)
    message.error('复制失败')
  })
}
</script>

<style scoped>
.left-panel {
  width: 250px;
  background: var(--panel-bg);
  border-right: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  transition: width 0.3s ease;
}

.left-panel.collapsed {
  width: 50px;
}

.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: var(--panel-header-bg);
  border-bottom: 1px solid var(--border-color);
  font-weight: 500;
}

.collapse-btn {
  color: var(--text-color);
  padding: 0;
  width: 20px;
  height: 20px;
}

.panel-content {
  flex: 1;
  padding: 16px;
  overflow-y: auto;
}

.section-title {
  font-size: 12px;
  font-weight: 500;
  color: var(--muted-color);
  margin-bottom: 8px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.profile-list {
  margin-bottom: 24px;
}

.profile-item {
  padding: 8px 12px;
  border-radius: 6px;
  cursor: pointer;
  transition: background-color 0.2s;
}

.profile-item:hover {
  background: var(--hover-bg);
}

.profile-item .delete-btn {
  opacity: 0;
  transition: opacity 0.2s;
}

.profile-item:hover .delete-btn {
  opacity: 1;
}

.delete-btn {
  color: var(--error-color) !important;
}

/* 搜索和控制区域 */
.search-section {
  margin-bottom: 12px;
}

.view-controls {
  margin-bottom: 16px;
  display: flex;
  justify-content: center;
}

/* 列表视图样式 */
.list-view {
  max-height: 300px;
  overflow-y: auto;
}

.profile-title {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.profile-tags {
  display: flex;
  gap: 4px;
  flex-wrap: wrap;
}

.profile-desc {
  display: flex;
  align-items: center;
  gap: 8px;
}

/* 分组视图样式 */
.group-view {
  max-height: 300px;
  overflow-y: auto;
}

.group-section {
  margin-bottom: 8px;
}

.group-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: var(--panel-header-bg);
  border-radius: 4px;
  cursor: pointer;
  transition: background-color 0.2s;
  user-select: none;
}

.group-header:hover {
  background: var(--hover-bg);
}

.group-icon {
  font-size: 12px;
  color: var(--muted-color);
  min-width: 12px;
}

.group-name {
  flex: 1;
  font-weight: 500;
  font-size: 13px;
}

.group-content {
  margin-left: 12px;
  border-left: 2px solid var(--border-color);
  padding-left: 8px;
}

.group-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  margin: 4px 0;
  border-radius: 4px;
  cursor: pointer;
  transition: background-color 0.2s;
}

.group-item:hover {
  background: var(--hover-bg);
}

.group-item:hover .delete-btn {
  opacity: 1;
}

.item-content {
  flex: 1;
}

.item-title {
  font-weight: 500;
  font-size: 13px;
  margin-bottom: 2px;
}

.item-desc {
  font-size: 12px;
  color: var(--muted-color);
  margin-bottom: 4px;
}

.item-tags {
  display: flex;
  gap: 4px;
  flex-wrap: wrap;
}

/* SFTP文件管理样式 */
.file-manager-section {
  margin-bottom: 24px;
}

.current-connection {
  margin-bottom: 12px;
}

.connection-info {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: var(--panel-header-bg);
  border-radius: 4px;
  font-size: 12px;
}

.status-indicator {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--success-color);
}

.status-indicator.connected {
  background: #52c41a;
  box-shadow: 0 0 4px rgba(82, 196, 26, 0.5);
}

.sftp-browser {
  border: 1px solid var(--border-color);
  border-radius: 4px;
  overflow: hidden;
}

.browser-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px;
  background: var(--panel-header-bg);
  border-bottom: 1px solid var(--border-color);
}

.toolbar-right {
  display: flex;
  gap: 4px;
}

.toolbar-right .ant-btn.active {
  background: var(--primary-color);
  color: white;
}

.current-path {
  padding: 4px 8px;
  background: var(--panel-bg);
  border-bottom: 1px solid var(--border-color);
}

.path-input {
  font-family: monospace;
  font-size: 12px;
}

.file-list {
  min-height: 300px;
  max-height: 500px;
  overflow-y: auto;
  resize: vertical;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  position: relative;
  transition: border-color 0.3s, background-color 0.3s;
}

.file-list.drag-over {
  border-color: var(--primary-color);
  border-width: 2px;
  background-color: rgba(24, 144, 255, 0.05);
}

.drag-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(24, 144, 255, 0.1);
  backdrop-filter: blur(2px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  pointer-events: none;
}

.drag-hint {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  color: var(--primary-color);
  font-size: 16px;
  font-weight: 500;
  text-align: center;
  padding: 20px;
}

.file-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px;
  cursor: pointer;
  transition: background-color 0.2s;
  font-size: 12px;
  border-bottom: 1px solid transparent;
}

.file-item:hover {
  background: var(--hover-bg);
}

.file-item.directory {
  font-weight: 500;
}

.file-icon {
  flex-shrink: 0;
  font-size: 14px;
}

.file-item.directory .file-icon {
  color: #1890ff;
}

.file-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.file-size {
  font-size: 10px;
  color: var(--muted-color);
  flex-shrink: 0;
}

.no-connection {
  padding: 20px 10px;
  text-align: center;
}

/* 响应式调整 */
@media (max-width: 768px) {
  .file-manager-section {
    margin-bottom: 16px;
  }
  
  .file-list {
    max-height: 150px;
  }
  
  .file-item {
    padding: 4px 6px;
    font-size: 11px;
  }
}

.file-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 12px;
  cursor: pointer;
  border-radius: 6px;
  transition: background-color 0.2s;
}

.file-item:hover {
  background: var(--hover-bg);
}

/* 响应式设计 */
@media (max-width: 768px) {
  .left-panel {
    width: 200px !important;
  }
  
  .left-panel.collapsed {
    width: 50px !important;
  }
}
</style>
