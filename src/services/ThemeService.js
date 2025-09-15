/**
 * 主题服务 - 处理主题相关功能
 */
class ThemeService {
  constructor() {
    // 初始化主题
    this.theme = localStorage.getItem('termlink_theme') || 'dark';
    this.terminalConfig = {
      fontSize: parseInt(localStorage.getItem('termlink_fontSize') || '14'),
      fontFamily: localStorage.getItem('termlink_fontFamily') || 'Consolas, monospace',
      cursorBlink: localStorage.getItem('termlink_cursorBlink') !== 'false',
      cursorStyle: localStorage.getItem('termlink_cursorStyle') || 'block'
    };
    
    // 主题配置
    this.themes = {
      dark: {
        background: '#0b0e14',
        foreground: '#ffffff',
        cursor: '#ffffff',
        selection: '#264f78',
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
    };
    
    // 应用初始主题
    this.applyTheme(this.theme);
  }
  
  /**
   * 获取当前主题
   * @returns {string} 主题名称
   */
  getTheme() {
    return this.theme;
  }
  
  /**
   * 获取终端配置
   * @returns {Object} 终端配置
   */
  getTerminalConfig() {
    return { ...this.terminalConfig };
  }
  
  /**
   * 获取主题配置
   * @param {string} themeName 主题名称
   * @returns {Object} 主题配置
   */
  getThemeConfig(themeName) {
    return this.themes[themeName || this.theme];
  }
  
  /**
   * 切换主题
   * @param {string} themeName 主题名称
   */
  toggleTheme(themeName) {
    this.theme = themeName;
    this.applyTheme(themeName);
    return themeName;
  }
  
  /**
   * 应用主题
   * @param {string} themeName 主题名称
   */
  applyTheme(themeName) {
    // 设置 body 主题属性
    document.body.setAttribute('data-theme', themeName);
    localStorage.setItem('termlink_theme', themeName);
    
    // 强制更新 Ant Design 主题
    const isDark = themeName === 'dark';
    document.body.classList.toggle('ant-dark', isDark);
    document.body.classList.toggle('ant-light', !isDark);
  }
  
  /**
   * 更新终端配置
   * @param {Object} config 终端配置
   */
  updateTerminalConfig(config) {
    this.terminalConfig = { ...this.terminalConfig, ...config };
    
    // 保存到本地存储
    localStorage.setItem('termlink_fontSize', this.terminalConfig.fontSize);
    localStorage.setItem('termlink_fontFamily', this.terminalConfig.fontFamily);
    localStorage.setItem('termlink_cursorBlink', this.terminalConfig.cursorBlink);
    localStorage.setItem('termlink_cursorStyle', this.terminalConfig.cursorStyle);
    
    return { ...this.terminalConfig };
  }
}

// 导出单例
export default new ThemeService();
