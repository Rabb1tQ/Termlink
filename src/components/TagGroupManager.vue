<template>
  <div class="tag-group-manager">
    <a-tabs v-model:activeKey="activeTab" size="small">
      <!-- 分组管理 -->
      <a-tab-pane key="groups" tab="分组">
        <div class="manager-section">
          <div class="item-list" v-if="groups.length > 0">
            <div
              v-for="group in groups"
              :key="group.name"
              class="manager-item"
              @contextmenu.prevent="showGroupContextMenu($event, group)"
            >
              <div class="item-info">
                <div class="item-name">
                  <FolderOutlined style="margin-right: 6px; color: #1890ff;" />
                  <span v-if="editingGroup !== group.name">{{ group.name }}</span>
                  <div v-else class="inline-edit-wrapper">
                    <a-input
                      v-model:value="editingGroupNewName"
                      size="small"
                      class="inline-edit-input"
                      @click.stop
                      @pressEnter.stop
                      ref="groupInputRef"
                      autofocus
                    />
                    <a-button type="link" size="small" class="edit-action-btn" @click.stop="confirmRenameGroup(group.name)">
                      <CheckOutlined />
                    </a-button>
                    <a-button type="link" size="small" class="edit-action-btn" @click.stop="cancelRenameGroup">
                      <CloseOutlined />
                    </a-button>
                  </div>
                </div>
                <div class="item-count">{{ group.count }} 个连接</div>
              </div>
              <a-popconfirm
                :title="`确定要删除分组「${group.name}」吗？`"
                :description="`该分组下的 ${group.count} 个连接将变为未分组状态`"
                ok-text="删除"
                cancel-text="取消"
                ok-type="danger"
                @confirm="deleteGroup(group.name)"
              >
                <a-button type="text" size="small" danger class="item-delete-btn" @click.stop>
                  <DeleteOutlined />
                </a-button>
              </a-popconfirm>
            </div>
          </div>
          <a-empty v-else description="暂无分组" :image="simpleImage" />
        </div>
      </a-tab-pane>

      <!-- 标签管理 -->
      <a-tab-pane key="tags" tab="标签">
        <div class="manager-section">
          <div class="item-list" v-if="tags.length > 0">
            <div
              v-for="tag in tags"
              :key="tag.name"
              class="manager-item"
              @contextmenu.prevent="showTagContextMenu($event, tag)"
            >
              <div class="item-info">
                <div class="item-name">
                  <TagOutlined style="margin-right: 6px; color: #faad14;" />
                  <a-tag v-if="editingTag !== tag.name" color="blue" style="margin-right: 0;">{{ tag.name }}</a-tag>
                  <div v-else class="inline-edit-wrapper">
                    <a-input
                      v-model:value="editingTagNewName"
                      size="small"
                      class="inline-edit-input"
                      @click.stop
                      @pressEnter.stop
                      ref="tagInputRef"
                      autofocus
                    />
                    <a-button type="link" size="small" class="edit-action-btn" @click.stop="confirmRenameTag(tag.name)">
                      <CheckOutlined />
                    </a-button>
                    <a-button type="link" size="small" class="edit-action-btn" @click.stop="cancelRenameTag">
                      <CloseOutlined />
                    </a-button>
                  </div>
                </div>
                <div class="item-count">{{ tag.count }} 个连接</div>
              </div>
              <a-popconfirm
                :title="`确定要删除标签「${tag.name}」吗？`"
                :description="`使用该标签的 ${tag.count} 个连接将移除此标签`"
                ok-text="删除"
                cancel-text="取消"
                ok-type="danger"
                @confirm="deleteTag(tag.name)"
              >
                <a-button type="text" size="small" danger class="item-delete-btn" @click.stop>
                  <DeleteOutlined />
                </a-button>
              </a-popconfirm>
            </div>
          </div>
          <a-empty v-else description="暂无标签" :image="simpleImage" />
        </div>
      </a-tab-pane>
    </a-tabs>

    <!-- 右键菜单 - 分组 -->
    <div
      v-if="groupContextMenu.visible"
      class="context-menu"
      :style="{ left: groupContextMenu.x + 'px', top: groupContextMenu.y + 'px' }"
    >
      <div class="context-menu-item" @click="startRenameGroup">
        <EditOutlined style="margin-right: 8px;" />
        重命名
      </div>
      <div class="context-menu-divider"></div>
      <div class="context-menu-item danger" @click="deleteGroupFromContext">
        <DeleteOutlined style="margin-right: 8px;" />
        删除分组
      </div>
    </div>

    <!-- 右键菜单 - 标签 -->
    <div
      v-if="tagContextMenu.visible"
      class="context-menu"
      :style="{ left: tagContextMenu.x + 'px', top: tagContextMenu.y + 'px' }"
    >
      <div class="context-menu-item" @click="startRenameTag">
        <EditOutlined style="margin-right: 8px;" />
        重命名
      </div>
      <div class="context-menu-divider"></div>
      <div class="context-menu-item danger" @click="deleteTagFromContext">
        <DeleteOutlined style="margin-right: 8px;" />
        删除标签
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { message, Empty } from 'ant-design-vue'
import { DeleteOutlined, FolderOutlined, TagOutlined, EditOutlined, CheckOutlined, CloseOutlined } from '@ant-design/icons-vue'

const emit = defineEmits(['refreshProfiles'])

const activeTab = ref('groups')
const groups = ref([])
const tags = ref([])
const simpleImage = Empty.PRESENTED_IMAGE_SIMPLE

// 编辑状态
const editingGroup = ref(null)
const editingGroupNewName = ref('')
const editingTag = ref(null)
const editingTagNewName = ref('')
const groupInputRef = ref(null)
const tagInputRef = ref(null)

// 右键菜单状态 - 分组
const groupContextMenu = ref({
  visible: false,
  x: 0,
  y: 0,
  group: null
})

// 右键菜单状态 - 标签
const tagContextMenu = ref({
  visible: false,
  x: 0,
  y: 0,
  tag: null
})

// 点击其他区域关闭右键菜单
function handleGlobalClick() {
  groupContextMenu.value.visible = false
  tagContextMenu.value.visible = false
}

onMounted(() => {
  loadData()
  document.addEventListener('click', handleGlobalClick)
})

onUnmounted(() => {
  document.removeEventListener('click', handleGlobalClick)
})

// 加载所有分组和标签
async function loadData() {
  try {
    const profiles = await invoke('list_ssh_profiles')
    
    // 统计分组
    const groupMap = {}
    // 统计标签
    const tagMap = {}
    
    profiles.forEach(profile => {
      // 统计分组
      if (profile.group && profile.group.trim() !== '') {
        if (!groupMap[profile.group]) {
          groupMap[profile.group] = 0
        }
        groupMap[profile.group]++
      }
      
      // 统计标签
      if (profile.tags && Array.isArray(profile.tags)) {
        profile.tags.forEach(tag => {
          if (tag && tag.trim() !== '') {
            if (!tagMap[tag]) {
              tagMap[tag] = 0
            }
            tagMap[tag]++
          }
        })
      }
    })
    
    // 转换为数组并排序
    groups.value = Object.entries(groupMap)
      .map(([name, count]) => ({ name, count }))
      .sort((a, b) => a.name.localeCompare(b.name))
    
    tags.value = Object.entries(tagMap)
      .map(([name, count]) => ({ name, count }))
      .sort((a, b) => b.count - a.count || a.name.localeCompare(b.name))
  } catch (error) {
    console.error('加载标签和分组数据失败:', error)
    message.error('加载数据失败')
  }
}

// ========== 分组右键菜单 ==========
function showGroupContextMenu(event, group) {
  groupContextMenu.value = {
    visible: true,
    x: event.clientX,
    y: event.clientY,
    group: group
  }
  tagContextMenu.value.visible = false
}

function startRenameGroup() {
  const group = groupContextMenu.value.group
  if (!group) return
  groupContextMenu.value.visible = false
  editingGroup.value = group.name
  editingGroupNewName.value = group.name
  nextTick(() => {
    // 聚焦输入框并选中文本
    const inputs = document.querySelectorAll('.inline-edit-input')
    inputs.forEach(input => {
      if (input.closest('.manager-item')?.textContent?.includes(group.name)) {
        input.focus()
        input.select()
      }
    })
  })
}

async function confirmRenameGroup(oldName) {
  const newName = editingGroupNewName.value.trim()
  editingGroup.value = null
  
  if (!newName) {
    message.warning('分组名称不能为空')
    return
  }
  
  if (newName === oldName) {
    return
  }
  
  // 检查是否已存在同名分组
  if (groups.value.some(g => g.name === newName)) {
    message.warning(`分组「${newName}」已存在`)
    return
  }
  
  try {
    const profiles = await invoke('list_ssh_profiles')
    let updatedCount = 0
    
    for (const profile of profiles) {
      if (profile.group === oldName) {
        const updatedProfile = {
          ...profile,
          group: newName
        }
        await invoke('save_ssh_profile', {
          profile: updatedProfile,
          password: null
        })
        updatedCount++
      }
    }
    
    message.success(`已将分组「${oldName}」重命名为「${newName}」，共更新 ${updatedCount} 个连接`)
    await loadData()
    emit('refreshProfiles')
  } catch (error) {
    console.error('重命名分组失败:', error)
    message.error('重命名分组失败: ' + error)
  }
}

function cancelRenameGroup() {
  editingGroup.value = null
  editingGroupNewName.value = ''
}

function deleteGroupFromContext() {
  const group = groupContextMenu.value.group
  groupContextMenu.value.visible = false
  if (group) {
    deleteGroup(group.name)
  }
}

// ========== 标签右键菜单 ==========
function showTagContextMenu(event, tag) {
  tagContextMenu.value = {
    visible: true,
    x: event.clientX,
    y: event.clientY,
    tag: tag
  }
  groupContextMenu.value.visible = false
}

function startRenameTag() {
  const tag = tagContextMenu.value.tag
  if (!tag) return
  tagContextMenu.value.visible = false
  editingTag.value = tag.name
  editingTagNewName.value = tag.name
  nextTick(() => {
    // 聚焦输入框并选中文本
    const inputs = document.querySelectorAll('.inline-edit-input')
    inputs.forEach(input => {
      if (input.closest('.manager-item')?.textContent?.includes(tag.name)) {
        input.focus()
        input.select()
      }
    })
  })
}

async function confirmRenameTag(oldName) {
  const newName = editingTagNewName.value.trim()
  editingTag.value = null
  
  if (!newName) {
    message.warning('标签名称不能为空')
    return
  }
  
  if (newName === oldName) {
    return
  }
  
  // 检查是否已存在同名标签
  if (tags.value.some(t => t.name === newName)) {
    message.warning(`标签「${newName}」已存在`)
    return
  }
  
  try {
    const profiles = await invoke('list_ssh_profiles')
    let updatedCount = 0
    
    for (const profile of profiles) {
      if (profile.tags && Array.isArray(profile.tags) && profile.tags.includes(oldName)) {
        const updatedProfile = {
          ...profile,
          tags: profile.tags.map(t => t === oldName ? newName : t)
        }
        await invoke('save_ssh_profile', {
          profile: updatedProfile,
          password: null
        })
        updatedCount++
      }
    }
    
    message.success(`已将标签「${oldName}」重命名为「${newName}」，共更新 ${updatedCount} 个连接`)
    await loadData()
    emit('refreshProfiles')
  } catch (error) {
    console.error('重命名标签失败:', error)
    message.error('重命名标签失败: ' + error)
  }
}

function cancelRenameTag() {
  editingTag.value = null
  editingTagNewName.value = ''
}

function deleteTagFromContext() {
  const tag = tagContextMenu.value.tag
  tagContextMenu.value.visible = false
  if (tag) {
    deleteTag(tag.name)
  }
}

// 删除分组 - 从所有使用该分组的连接中移除
async function deleteGroup(groupName) {
  try {
    const profiles = await invoke('list_ssh_profiles')
    let updatedCount = 0
    
    for (const profile of profiles) {
      if (profile.group === groupName) {
        const updatedProfile = {
          ...profile,
          group: ''
        }
        await invoke('save_ssh_profile', {
          profile: updatedProfile,
          password: null
        })
        updatedCount++
      }
    }
    
    message.success(`已从 ${updatedCount} 个连接中移除分组「${groupName}」`)
    await loadData()
    emit('refreshProfiles')
  } catch (error) {
    console.error('删除分组失败:', error)
    message.error('删除分组失败: ' + error)
  }
}

// 删除标签 - 从所有使用该标签的连接中移除
async function deleteTag(tagName) {
  try {
    const profiles = await invoke('list_ssh_profiles')
    let updatedCount = 0
    
    for (const profile of profiles) {
      if (profile.tags && Array.isArray(profile.tags) && profile.tags.includes(tagName)) {
        const updatedProfile = {
          ...profile,
          tags: profile.tags.filter(t => t !== tagName)
        }
        await invoke('save_ssh_profile', {
          profile: updatedProfile,
          password: null
        })
        updatedCount++
      }
    }
    
    message.success(`已从 ${updatedCount} 个连接中移除标签「${tagName}」`)
    await loadData()
    emit('refreshProfiles')
  } catch (error) {
    console.error('删除标签失败:', error)
    message.error('删除标签失败: ' + error)
  }
}

// 暴露刷新方法，供父组件在需要时调用
defineExpose({
  refresh: loadData
})
</script>

<style scoped>
.tag-group-manager {
  height: 100%;
}

.manager-section {
  min-height: 200px;
}

.item-list {
  max-height: calc(100vh - 280px);
  overflow-y: auto;
}

.manager-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  border-radius: 6px;
  transition: background-color 0.2s;
  border-bottom: 1px solid var(--border-color);
  cursor: default;
}

.manager-item:last-child {
  border-bottom: none;
}

.manager-item:hover {
  background: var(--hover-bg);
}

.manager-item:hover .item-delete-btn {
  opacity: 1;
}

.item-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  flex: 1;
  min-width: 0;
}

.item-name {
  display: flex;
  align-items: center;
  font-size: 14px;
  font-weight: 500;
}

.item-count {
  font-size: 12px;
  color: var(--muted-color);
  margin-left: 22px;
}

.item-delete-btn {
  opacity: 0;
  transition: opacity 0.2s;
  flex-shrink: 0;
}

.inline-edit-wrapper {
  display: flex;
  align-items: center;
  gap: 2px;
}

.inline-edit-input {
  width: 140px;
  font-size: 13px;
}

.edit-action-btn {
  padding: 0 4px;
  font-size: 14px;
  line-height: 1;
}

/* 右键菜单样式 */
.context-menu {
  position: fixed;
  z-index: 1000;
  background: var(--bg-secondary, #fff);
  border: 1px solid var(--border-color, #e8e8e8);
  border-radius: 6px;
  padding: 4px 0;
  min-width: 140px;
  box-shadow: 0 3px 12px rgba(0, 0, 0, 0.15);
  animation: contextMenuFadeIn 0.15s ease;
}

@keyframes contextMenuFadeIn {
  from {
    opacity: 0;
    transform: scale(0.95);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

.context-menu-item {
  display: flex;
  align-items: center;
  padding: 6px 16px;
  font-size: 13px;
  cursor: pointer;
  transition: background-color 0.15s;
  color: var(--text-primary, #333);
  white-space: nowrap;
}

.context-menu-item:hover {
  background: var(--hover-bg, #f0f0f0);
}

.context-menu-item.danger {
  color: #ff4d4f;
}

.context-menu-item.danger:hover {
  background: #fff1f0;
}

.context-menu-divider {
  height: 1px;
  margin: 4px 0;
  background: var(--border-color, #e8e8e8);
}

:deep(.ant-tabs-nav) {
  margin-bottom: 12px;
}

:deep(.ant-tabs-tab) {
  font-size: 13px;
  padding: 8px 0;
}

:deep(.ant-empty-description) {
  color: var(--muted-color);
}
</style>
