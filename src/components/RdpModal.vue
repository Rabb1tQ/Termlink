<template>
  <a-modal
    :open="visible"
    :title="editMode ? '编辑远程桌面连接' : '新建远程桌面连接'"
    width="500px"
    @ok="handleSubmit"
    @cancel="handleCancel"
    :confirmLoading="loading"
  >
    <a-form layout="vertical" :model="form" ref="formRef">
      <a-form-item label="连接名称" name="name">
        <a-input v-model:value="form.name" placeholder="给连接起个名字" />
      </a-form-item>
      
      <a-row :gutter="16">
        <a-col :span="12">
          <a-form-item label="分组" name="group">
            <a-auto-complete 
              v-model:value="form.group" 
              :options="groupOptions"
              placeholder="选择或创建分组"
              :filter-option="filterOption"
              allow-clear
            />
          </a-form-item>
        </a-col>
        <a-col :span="12">
          <a-form-item label="标签" name="tags">
            <a-select
              v-model:value="form.tags"
              mode="tags"
              placeholder="添加标签"
              :token-separators="[',', ' ']"
              allow-clear
            />
          </a-form-item>
        </a-col>
      </a-row>
      
      <a-form-item label="主机地址" name="host" :rules="[{ required: true, message: '请输入主机地址' }]">
        <a-input v-model:value="form.host" placeholder="192.168.1.100 或 example.com" />
      </a-form-item>
      
      <a-row :gutter="16">
        <a-col :span="8">
          <a-form-item label="端口" name="port">
            <a-input-number 
              v-model:value="form.port" 
              :min="1" 
              :max="65535" 
              style="width: 100%" 
              placeholder="3389"
            />
          </a-form-item>
        </a-col>
        <a-col :span="8">
          <a-form-item label="用户名" name="username">
            <a-input v-model:value="form.username" placeholder="用户名" />
          </a-form-item>
        </a-col>
        <a-col :span="8">
          <a-form-item label="域名" name="domain">
            <a-input v-model:value="form.domain" placeholder="域名（可选）" />
          </a-form-item>
        </a-col>
      </a-row>
      
      <a-form-item label="显示设置">
        <a-radio-group v-model:value="form.displayMode">
          <a-radio value="fullscreen">全屏</a-radio>
          <a-radio value="window">自定义分辨率</a-radio>
        </a-radio-group>
      </a-form-item>
      
      <a-row v-if="form.displayMode === 'window'" :gutter="16">
        <a-col :span="12">
          <a-form-item label="宽度" name="width">
            <a-input-number 
              v-model:value="form.width" 
              :min="640" 
              :max="7680" 
              style="width: 100%" 
              placeholder="1920"
            />
          </a-form-item>
        </a-col>
        <a-col :span="12">
          <a-form-item label="高度" name="height">
            <a-input-number 
              v-model:value="form.height" 
              :min="480" 
              :max="4320" 
              style="width: 100%" 
              placeholder="1080"
            />
          </a-form-item>
        </a-col>
      </a-row>
      
      <a-form-item>
        <a-checkbox v-model:checked="form.adminMode">
          管理模式（/console）
        </a-checkbox>
        <div v-if="form.adminMode" style="margin-top: 4px; color: var(--muted-color); font-size: 12px;">
          使用管理模式连接到服务器的控制台会话
        </div>
      </a-form-item>
    </a-form>
  </a-modal>
</template>

<script setup>
import { ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const props = defineProps({
  visible: {
    type: Boolean,
    default: false
  },
  editMode: {
    type: Boolean,
    default: false
  },
  editProfile: {
    type: Object,
    default: null
  }
})

const emit = defineEmits(['update:visible', 'submit'])

const formRef = ref()
const loading = ref(false)

const form = ref({
  name: '',
  host: '',
  port: 3389,
  username: '',
  domain: '',
  displayMode: 'fullscreen',
  width: 1920,
  height: 1080,
  adminMode: false,
  group: '',
  tags: []
})

const groupOptions = ref([])

// 过滤分组选项
function filterOption(inputValue, option) {
  return option.value.toLowerCase().includes(inputValue.toLowerCase())
}

// 重置表单
function resetForm() {
  if (props.editMode && props.editProfile) {
    const p = props.editProfile
    form.value = {
      id: p.id,
      name: p.name || '',
      host: p.host || '',
      port: p.port || 3389,
      username: p.username || '',
      domain: p.domain || '',
      displayMode: p.fullscreen ? 'fullscreen' : 'window',
      width: p.width || 1920,
      height: p.height || 1080,
      adminMode: p.admin_mode || false,
      group: p.group || '',
      tags: p.tags || []
    }
  } else {
    form.value = {
      name: '',
      host: '',
      port: 3389,
      username: '',
      domain: '',
      displayMode: 'fullscreen',
      width: 1920,
      height: 1080,
      adminMode: false,
      group: '',
      tags: []
    }
  }
  formRef.value?.resetFields()
}

// 提交表单
async function handleSubmit() {
  try {
    await formRef.value.validate()
    loading.value = true
    
    const submitData = {
      ...form.value,
      fullscreen: form.value.displayMode === 'fullscreen'
    }
    delete submitData.displayMode
    
    if (props.editMode) {
      submitData.isEdit = true
    }
    
    emit('submit', submitData)
    
    setTimeout(() => {
      handleCancel()
    }, 100)
  } catch (error) {
    console.error('表单验证失败:', error)
  } finally {
    loading.value = false
  }
}

// 取消
function handleCancel() {
  emit('update:visible', false)
  resetForm()
}

// 获取已有分组
async function loadGroups() {
  try {
    const profiles = await invoke('list_rdp_profiles')
    const groups = [...new Set(profiles
      .map(p => p.group)
      .filter(g => g && g.trim() !== '')
    )]
    groupOptions.value = groups.map(g => ({ value: g, label: g }))
  } catch (error) {
    console.error('获取分组失败:', error)
  }
}

// 监听 visible 变化
watch(() => props.visible, (visible) => {
  if (visible) {
    loadGroups()
    resetForm()
  } else {
    resetForm()
  }
})

// 监听编辑配置文件变化
watch(() => props.editProfile, () => {
  if (props.visible && props.editMode) {
    resetForm()
  }
})
</script>

<style scoped>
:deep(.ant-form-item-label > label) {
  color: var(--text-color);
}

:deep(.ant-input),
:deep(.ant-input-number),
:deep(.ant-input-password) {
  background: var(--panel-bg);
  border-color: var(--border-color);
  color: var(--text-color);
}

:deep(.ant-input::placeholder),
:deep(.ant-input-number::placeholder),
:deep(.ant-input-password::placeholder) {
  color: var(--muted-color) !important;
}

:deep(.ant-input:focus),
:deep(.ant-input-number:focus),
:deep(.ant-input-password:focus) {
  border-color: var(--primary-color);
  box-shadow: 0 0 0 2px rgba(24, 144, 255, 0.2);
}

:deep(.ant-radio-wrapper) {
  color: var(--text-color);
}

:deep(.ant-checkbox-wrapper) {
  color: var(--text-color);
}
</style>
