<template>
  <a-modal
    :open="visible"
    :title="editMode ? '编辑 SSH 连接' : '新建 SSH 连接'"
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
        <a-col :span="12">
          <a-form-item label="端口" name="port">
            <a-input-number 
              v-model:value="form.port" 
              :min="1" 
              :max="65535" 
              style="width: 100%" 
              placeholder="22"
            />
          </a-form-item>
        </a-col>
        <a-col :span="12">
          <a-form-item label="用户名" name="username">
            <a-input v-model:value="form.username" placeholder="用户名" />
          </a-form-item>
        </a-col>
      </a-row>
      
      <a-form-item label="认证方式">
        <a-radio-group v-model:value="form.usePrivateKey">
          <a-radio :value="false">密码认证</a-radio>
          <a-radio :value="true">私钥认证</a-radio>
        </a-radio-group>
      </a-form-item>
      
      <a-form-item v-if="!form.usePrivateKey" label="密码" name="password">
        <a-input-password v-model:value="form.password" placeholder="密码" />
      </a-form-item>
      
      <a-form-item v-if="form.usePrivateKey" label="私钥文件" name="privateKey">
        <a-input-group compact>
          <a-input 
            v-model:value="form.privateKey" 
            placeholder="私钥文件路径" 
            style="width: calc(100% - 80px)"
          />
          <a-button @click="selectPrivateKey" style="width: 80px">浏览</a-button>
        </a-input-group>
      </a-form-item>
      
      <a-form-item>
        <a-checkbox v-model:checked="form.savePassword">
          保存连接配置
        </a-checkbox>
        <div v-if="form.savePassword" style="margin-top: 8px; color: var(--muted-color); font-size: 12px;">
          配置将保存在本地，密码会被安全加密存储
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
  port: 22,
  username: '',
  password: '',
  privateKey: '',
  usePrivateKey: false,
  savePassword: true,
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
    // 编辑模式，填充现有数据
    form.value = {
      id: props.editProfile.id,
      name: props.editProfile.name || '',
      host: props.editProfile.host || '',
      port: props.editProfile.port || 22,
      username: props.editProfile.username || '',
      password: '', // 不显示密码
      privateKey: props.editProfile.privateKey || '',
      usePrivateKey: props.editProfile.usePrivateKey || false,
      savePassword: true,
      group: props.editProfile.group || '',
      tags: props.editProfile.tags || []
    }
  } else {
    // 新建模式
    form.value = {
      name: '',
      host: '',
      port: 22,
      username: '',
      password: '',
      privateKey: '',
      usePrivateKey: false,
      savePassword: true,
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
    
    const submitData = { ...form.value }
    if (props.editMode) {
      submitData.isEdit = true
    }
    
    emit('submit', submitData)
    
    // 延迟关闭，让父组件处理
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

// 选择私钥文件
async function selectPrivateKey() {
  try {
    // 使用浏览器的文件选择
    const input = document.createElement('input')
    input.type = 'file'
    input.accept = '.pem,.key,.ppk,*'
    input.onchange = (e) => {
      const file = e.target.files[0]
      if (file) {
        // 获取文件路径（在桌面应用中这会是完整路径）
        form.value.privateKey = file.name
      }
    }
    input.click()
  } catch (error) {
    console.error('选择文件失败:', error)
  }
}

// 获取已有分组
async function loadGroups() {
  try {
    const profiles = await invoke('list_ssh_profiles')
    const groups = [...new Set(profiles
      .map(p => p.group)
      .filter(g => g && g.trim() !== '')
    )]
    groupOptions.value = groups.map(g => ({ value: g, label: g }))
  } catch (error) {
    console.error('获取分组失败:', error)
  }
}

// 监听 visible 变化，重置表单
watch(() => props.visible, (visible) => {
  if (visible) {
    loadGroups() // 打开时加载分组
    resetForm() // 重置表单（包括编辑模式的数据填充）
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
