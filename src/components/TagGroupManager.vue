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
            >
              <div class="item-info">
                <div class="item-name">
                  <FolderOutlined style="margin-right: 6px; color: #1890ff;" />
                  {{ group.name }}
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
                <a-button type="text" size="small" danger class="item-delete-btn">
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
            >
              <div class="item-info">
                <div class="item-name">
                  <TagOutlined style="margin-right: 6px; color: #faad14;" />
                  <a-tag color="blue" style="margin-right: 0;">{{ tag.name }}</a-tag>
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
                <a-button type="text" size="small" danger class="item-delete-btn">
                  <DeleteOutlined />
                </a-button>
              </a-popconfirm>
            </div>
          </div>
          <a-empty v-else description="暂无标签" :image="simpleImage" />
        </div>
      </a-tab-pane>
    </a-tabs>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { message, Empty } from 'ant-design-vue'
import { DeleteOutlined, FolderOutlined, TagOutlined } from '@ant-design/icons-vue'

const emit = defineEmits(['refreshProfiles'])

const activeTab = ref('groups')
const groups = ref([])
const tags = ref([])
const simpleImage = Empty.PRESENTED_IMAGE_SIMPLE

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

// 组件挂载时加载数据
onMounted(() => {
  loadData()
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
