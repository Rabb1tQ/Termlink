import { invoke } from '@tauri-apps/api/core';

/**
 * SFTP服务 - 处理SFTP文件操作相关功能
 */
class SftpService {
  /**
   * 列出SFTP目录文件
   * @param {string} connectionId 连接ID
   * @param {string} path 目录路径
   * @param {boolean} showHidden 是否显示隐藏文件
   * @returns {Array} 文件列表
   */
  async listFiles(connectionId, path, showHidden = false) {
    try {
      const files = await invoke('list_sftp_files', { 
        connectionId, 
        path 
      });
      
      // 如果不显示隐藏文件，过滤掉隐藏文件
      if (!showHidden) {
        return files.filter(file => !file.name.startsWith('.') || file.name === '..');
      }
      
      return files;
    } catch (e) {
      console.error(`列出SFTP目录失败 (${path}):`, e);
      throw e;
    }
  }
  
  /**
   * 读取SFTP文件内容
   * @param {string} connectionId 连接ID
   * @param {string} path 文件路径
   * @returns {string} 文件内容
   */
  async readFile(connectionId, path) {
    try {
      return await invoke('read_sftp_file', { 
        connectionId, 
        remotePath: path 
      });
    } catch (e) {
      console.error(`读取SFTP文件失败 (${path}):`, e);
      throw e;
    }
  }
  
  /**
   * 写入SFTP文件内容
   * @param {string} connectionId 连接ID
   * @param {string} path 文件路径
   * @param {string} content 文件内容
   */
  async writeFile(connectionId, path, content) {
    try {
      await invoke('write_sftp_file', { 
        connectionId, 
        remotePath: path, 
        content 
      });
    } catch (e) {
      console.error(`写入SFTP文件失败 (${path}):`, e);
      throw e;
    }
  }
  
  /**
   * 下载SFTP文件
   * @param {string} connectionId 连接ID
   * @param {string} remotePath 远程文件路径
   * @param {string} localPath 本地文件路径
   */
  async downloadFile(connectionId, remotePath, localPath) {
    try {
      await invoke('download_sftp_file', { 
        connectionId, 
        remotePath, 
        localPath 
      });
    } catch (e) {
      console.error(`下载SFTP文件失败 (${remotePath}):`, e);
      throw e;
    }
  }
  
  /**
   * 上传文件到SFTP
   * @param {string} connectionId 连接ID
   * @param {string} localPath 本地文件路径
   * @param {string} remotePath 远程文件路径
   */
  async uploadFile(connectionId, localPath, remotePath) {
    try {
      await invoke('upload_sftp_file', { 
        connectionId, 
        localPath, 
        remotePath 
      });
    } catch (e) {
      console.error(`上传文件到SFTP失败 (${remotePath}):`, e);
      throw e;
    }
  }
  
  /**
   * 删除SFTP文件
   * @param {string} connectionId 连接ID
   * @param {string} path 文件路径
   */
  async deleteFile(connectionId, path) {
    try {
      await invoke('delete_sftp_file', { 
        connectionId, 
        remotePath: path 
      });
    } catch (e) {
      console.error(`删除SFTP文件失败 (${path}):`, e);
      throw e;
    }
  }
  
  /**
   * 创建SFTP目录
   * @param {string} connectionId 连接ID
   * @param {string} path 目录路径
   */
  async createDirectory(connectionId, path) {
    try {
      await invoke('create_sftp_directory', { 
        connectionId, 
        remotePath: path 
      });
    } catch (e) {
      console.error(`创建SFTP目录失败 (${path}):`, e);
      throw e;
    }
  }
  
  /**
   * 获取文件预览信息
   * @param {string} connectionId 连接ID
   * @param {Object} fileInfo 文件信息
   * @returns {Object} 文件预览信息
   */
  async getFilePreview(connectionId, fileInfo) {
    // 如果是目录，返回null
    if (fileInfo.is_directory) {
      return null;
    }
    
    try {
      // 获取文件内容
      const content = await this.readFile(connectionId, fileInfo.path);
      
      // 根据文件扩展名确定语言
      const extension = fileInfo.name.split('.').pop().toLowerCase();
      const language = this.getLanguageByExtension(extension);
      
      return {
        content,
        language,
        path: fileInfo.path,
        name: fileInfo.name
      };
    } catch (e) {
      console.error(`获取文件预览失败 (${fileInfo.path}):`, e);
      throw e;
    }
  }
  
  /**
   * 根据文件扩展名获取语言
   * @param {string} extension 文件扩展名
   * @returns {string} 语言
   */
  getLanguageByExtension(extension) {
    const languageMap = {
      'js': 'javascript',
      'ts': 'typescript',
      'jsx': 'javascript',
      'tsx': 'typescript',
      'html': 'html',
      'css': 'css',
      'scss': 'scss',
      'less': 'less',
      'json': 'json',
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
      'bat': 'bat',
      'ps1': 'powershell',
      'sql': 'sql',
      'vue': 'vue',
      'xml': 'xml',
      'yaml': 'yaml',
      'yml': 'yaml',
      'toml': 'toml',
      'ini': 'ini',
      'conf': 'plaintext',
      'log': 'plaintext',
      'txt': 'plaintext'
    };
    
    return languageMap[extension] || 'plaintext';
  }
}

// 导出单例
export default new SftpService();
