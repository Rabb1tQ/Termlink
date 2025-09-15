<template>
  <div class="terminal-area">
    <div ref="container" class="terminal-container" />
    <!-- 滚动指示器 -->
    <div v-if="hasScrollContent" class="scroll-indicator">
      <span>↑ 向上滚动查看更多内容</span>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onBeforeUnmount, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import '@xterm/xterm/css/xterm.css'
import SshService from '../services/SshService'

const props = defineProps({
  id: {
    type: String,
    required: true
  },
  active: {
    type: Boolean,
    default: false
  },
  theme: {
    type: String,
    default: 'dark'
  },
  config: {
    type: Object,
    default: () => ({
      fontSize: 14,
      fontFamily: 'Consolas, monospace',
      cursorBlink: true,
      cursorStyle: 'block'
    })
  },
  autoPassword: {
    type: String,
    default: ''
  },
  type: {
    type: String,
    default: 'local' // 'local' or 'ssh'
  }
})

const emit = defineEmits(['close', 'reconnect'])

const container = ref(null)
const terminal = ref(null)
const fitAddon = ref(null)
const hasScrollContent = computed(() => {
  if (!terminal.value) return false
  return terminal.value.buffer.active.viewportY > 0
})

// 主题配置
const themes = {
  dark: {
    background: '#0b0e14',
    foreground: '#ffffff',
    cursor: '#ffffff',
    selection: '#264f78',
    selectionForeground: '#ffffff',
    black: '#000000',
    red: '#e06c75',
    green: '#98c379',
    yellow: '#d19a66',
    blue: '#61afef',
    magenta: '#c678dd',
    cyan: '#56b6c2',
    white: '#ffffff',
    brightBlack: '#5c6370',
    brightRed: '#e06c75',
    brightGreen: '#98c379',
    brightYellow: '#d19a66',
    brightBlue: '#61afef',
    brightMagenta: '#c678dd',
    brightCyan: '#56b6c2',
    brightWhite: '#ffffff'
  },
  light: {
    background: '#ffffff',
    foreground: '#000000',
    cursor: '#000000',
    selection: '#0078d4',
    selectionForeground: '#ffffff',
    black: '#000000',
    red: '#cd3131',
    green: '#00bc00',
    yellow: '#949800',
    blue: '#0451a5',
    magenta: '#bc05bc',
    cyan: '#0598bc',
    white: '#000000',
    brightBlack: '#000000',
    brightRed: '#cd3131',
    brightGreen: '#14ce14',
    brightYellow: '#b5ba00',
    brightBlue: '#0451a5',
    brightMagenta: '#bc05bc',
    brightCyan: '#0598bc',
    brightWhite: '#000000'
  }
}

// 创建终端实例
async function createTerminal() {
  const { Terminal } = await import('@xterm/xterm')
  const { FitAddon } = await import('@xterm/addon-fit')
  
  const term = new Terminal({
    convertEol: true,
    fontFamily: props.config.fontFamily,
    fontSize: props.config.fontSize,
    lineHeight: 1.2,
    windowsMode: false, // 禁用Windows模式以正确处理Clink输出
    cursorBlink: props.config.cursorBlink,
    cursorStyle: props.config.cursorStyle,
    theme: themes[props.theme],
    scrollback: 50000,
    rows: 24, // 适中的行数
    cols: 80, // 标准的列数，过大会导致输入位置问题
    allowTransparency: true,
    fastScrollModifier: 'alt',
    fastScrollSensitivity: 5,
    scrollOnUserInput: false,
    scrollOnOutput: false,
    rightClickSelectsWord: false, // 禁用右键选择单词，允许右键菜单
    wordSeparator: ' ()[]{}\',"`',
    disableStdin: false,  // 允许用户输入
    macOptionIsMeta: true, // Mac上的Option键作为Meta键
    macOptionClickForcesSelection: false, // 禁用Option点击强制选择
    altClickMovesCursor: false, // 禁用Alt点击移动光标
  })
  
  const fit = new FitAddon()
  term.loadAddon(fit)
  
  // 设置数据处理器
  term.onData(data => {
    // 只有当这个终端实例是激活状态时才发送数据
    if (props.active) {
      if (props.id.startsWith('ssh-')) {
        // SSH终端
        SshService.writeTerminal(props.id, data).catch(() => {})
      } else {
        // 本地终端
        invoke('write_pty', { id: props.id, data }).catch(() => {})
      }
    }
  })
  
  // 设置键盘事件处理器，用于处理重连功能
  term.onKey(({ key, domEvent }) => {
    // 如果按回车键且终端中有连接失败信息，尝试重连
    if (domEvent.code === 'Enter' && props.active) {
      const content = term.buffer.active.getLine(term.buffer.active.cursorY)?.translateToString()
      if (content && (content.includes('Connection refused') || 
                      content.includes('ssh:') ||
                      content.includes('Connection timed out') ||
                      content.includes('No route to host') ||
                      content.includes('Connection closed'))) {
        // 触发重连事件
        terminal.value.writeln('\r\n正在尝试重连...')
        emit('reconnect')
      }
    }
  })
  
  // 监听终端大小变化，自动同步PTY大小
  term.onResize(({ cols, rows }) => {
    if (props.active) {
      if (props.id.startsWith('ssh-')) {
        // SSH终端
        SshService.resizeTerminal(props.id, cols, rows).catch(() => {})
      } else {
        // 本地终端
        invoke('resize_pty', { id: props.id, cols, rows }).catch(() => {})
      }
    }
  })
  
  terminal.value = term
  fitAddon.value = fit
  
  // 打开终端
  term.open(container.value)
  fit.fit()
  
  // 等待终端完全初始化后再添加事件监听器
  setTimeout(() => {
    if (term.element) {
      term.element.addEventListener('contextmenu', (event) => {
        event.preventDefault()
        showContextMenu(event, term)
      })
    }
  }, 100)
}

// 绑定会话
async function bindSession() {
  // 根据终端类型绑定不同的事件
  if (props.id.startsWith('ssh-')) {
    // SSH终端
    const offDataP = listen(`ssh_data://${props.id}`, e => {
      // 只有当这个终端实例是激活状态时才写入数据
      if (terminal.value) {
        const output = String(e.payload || '')
        terminal.value.write(output)
      }
    })
    
    const offExitP = listen(`ssh_exit://${props.id}`, () => {
      emit('close')
    })
    
    const offErrorP = listen(`ssh_error`, e => {
      if (e.payload.startsWith(`${props.id}: `)) {
        if (terminal.value) {
          terminal.value.writeln(`\r\n\x1b[31m${e.payload.substring(props.id.length + 2)}\x1b[0m`)
        }
      }
    })
    
    return async () => { 
      (await offDataP)(); 
      (await offExitP)();
      (await offErrorP)();
    }
  } else {
    // 本地终端
    const offOutP = listen(`pty://${props.id}`, e => {
      // 只有当这个终端实例是激活状态时才写入数据
      if (terminal.value) {
        const output = String(e.payload || '')
        terminal.value.write(output)
        
        // 检测密码提示并自动输入密码
        if (props.autoPassword && (output.includes('password:') || output.includes('Password:'))) {
          setTimeout(() => {
            if (terminal.value && props.active) {
              invoke('write_pty', { id: props.id, data: props.autoPassword + '\r\n' }).catch(() => {})
            }
          }, 100) // 等待100ms让提示显示完成
        }
      }
    })
    
    const offExitP = listen(`pty_exit://${props.id}`, () => {
      emit('close')
    })
    
    return async () => { 
      (await offOutP)(); 
      (await offExitP)() 
    }
  }
}

// 应用主题
function applyTheme() {
  if (terminal.value && typeof terminal.value.setOption === 'function') {
    const theme = themes[props.theme]
    console.log('Applying theme:', props.theme, theme)
    terminal.value.setOption('theme', theme)
    
    // 强制刷新终端显示
    setTimeout(() => {
      if (terminal.value) {
        terminal.value.refresh(0, terminal.value.rows - 1)
      }
    }, 100)
  }
}

// 应用配置
function applyConfig() {
  if (terminal.value && typeof terminal.value.setOption === 'function') {
    terminal.value.setOption('fontSize', props.config.fontSize)
    terminal.value.setOption('fontFamily', props.config.fontFamily)
    terminal.value.setOption('cursorBlink', props.config.cursorBlink)
    terminal.value.setOption('cursorStyle', props.config.cursorStyle)
  }
}

// 应用大小
function applySize() {
  if (fitAddon.value) {
    fitAddon.value.fit()
  }
}

// 显示右键菜单
function showContextMenu(event, term) {
  const selection = term.getSelection()
  const hasSelection = selection && selection.length > 0
  
  // 创建菜单项
  const menuItems = []
  
  if (hasSelection) {
    menuItems.push({
      label: '复制',
      action: () => {
        navigator.clipboard.writeText(selection).then(() => {
          console.log('已复制到剪贴板')
        }).catch(err => {
          console.error('复制失败:', err)
        })
      }
    })
  }
  
  menuItems.push({
    label: '粘贴',
    action: () => {
      navigator.clipboard.readText().then(text => {
        if (props.active) {
          if (props.id.startsWith('ssh-')) {
            SshService.writeTerminal(props.id, text).catch(() => {})
          } else {
            invoke('write_pty', { id: props.id, data: text }).catch(() => {})
          }
        }
      }).catch(err => {
        console.error('粘贴失败:', err)
      })
    }
  })
  
  if (hasSelection) {
    menuItems.push({
      label: '全选',
      action: () => {
        term.selectAll()
      }
    })
  }
  
  menuItems.push({
    label: '清屏',
    action: () => {
      term.clear()
    }
  })
  
  // 显示菜单
  showMenu(event, menuItems)
}

// 显示菜单
function showMenu(event, items) {
  // 移除已存在的菜单
  const existingMenu = document.querySelector('.terminal-context-menu')
  if (existingMenu) {
    existingMenu.remove()
  }
  
  // 创建菜单
  const menu = document.createElement('div')
  menu.className = 'terminal-context-menu'
  menu.style.cssText = `
    position: fixed;
    left: ${event.clientX}px;
    top: ${event.clientY}px;
    background: var(--panel-bg);
    border: 1px solid var(--border-color);
    border-radius: 4px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    z-index: 10000;
    min-width: 120px;
    padding: 4px 0;
  `
  
  // 添加菜单项
  items.forEach(item => {
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
    menuItem.addEventListener('click', () => {
      item.action()
      menu.remove()
    })
    menu.appendChild(menuItem)
  })
  
  // 添加到页面
  document.body.appendChild(menu)
  
  // 点击其他地方关闭菜单
  const closeMenu = (e) => {
    if (!menu.contains(e.target)) {
      menu.remove()
      document.removeEventListener('click', closeMenu)
      document.removeEventListener('contextmenu', closeMenu)
    }
  }
  
  // 右键点击也关闭菜单
  const closeMenuOnRightClick = (e) => {
    menu.remove()
    document.removeEventListener('click', closeMenu)
    document.removeEventListener('contextmenu', closeMenu)
  }
  
  setTimeout(() => {
    document.addEventListener('click', closeMenu)
    document.addEventListener('contextmenu', closeMenuOnRightClick)
  }, 0)
}

// 监听主题变化
watch(() => props.theme, () => {
  applyTheme()
})

// 监听配置变化
watch(() => props.config, () => {
  applyConfig()
  applySize()
}, { deep: true })

// 监听活动状态变化
watch(() => props.active, (isActive) => {
  if (isActive) {
    // 激活时调整大小
    setTimeout(() => {
      applySize()
    }, 0)
  }
})

// 生命周期钩子
let unbindSession = null

onMounted(async () => {
  // 创建终端
  await createTerminal()
  
  // 绑定会话
  unbindSession = await bindSession()
  
  // 应用主题和配置
  applyTheme()
  applyConfig()
  
  // 如果是活动状态，调整大小
  if (props.active) {
    setTimeout(() => {
      applySize()
    }, 0)
  }
  
  // 监听窗口大小变化
  window.addEventListener('resize', applySize)
})

onBeforeUnmount(async () => {
  // 解绑会话
  if (unbindSession) {
    await unbindSession()
  }
  
  // 销毁终端
  if (terminal.value) {
    terminal.value.dispose()
  }
  
  // 移除事件监听
  window.removeEventListener('resize', applySize)
})
</script>

<style scoped>
.terminal-area {
  position: relative;
  width: 100%;
  height: 100%;
  overflow: hidden;
  background-color: var(--terminal-bg);
}

.terminal-container {
  width: 100%;
  height: 100%;
  padding: 2px;
}

.scroll-indicator {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  background-color: rgba(0, 0, 0, 0.7);
  color: white;
  text-align: center;
  padding: 4px;
  font-size: 12px;
  z-index: 10;
  opacity: 0.7;
  transition: opacity 0.3s;
}

.scroll-indicator:hover {
  opacity: 1;
}
</style>