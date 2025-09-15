import { invoke } from '@tauri-apps/api/core';

/**
 * SSH服务 - 处理SSH连接相关功能
 */
class SshService {
  /**
   * 启动已保存的SSH连接配置
   * @param {Object} profile SSH配置
   * @returns {Object} 连接信息
   */
  async launchProfile(profile) {
    const id = `ssh-${Date.now()}`;
    const title = profile.username ? `${profile.username}@${profile.host}` : profile.host;
    
    // 获取保存的密码
    let password = null;
    if (profile.save_password) {
      try {
        password = await invoke('get_ssh_password', { id: profile.id });
      } catch (e) {
        console.error('获取SSH密码失败:', e);
      }
    }
    
    // 启动SSH连接
    try {
      await invoke('start_ssh_terminal', { 
        id, 
        host: profile.host,
        port: profile.port || 22,
        username: profile.username,
        password: password,
        cols: 80, 
        rows: 24
      });
    } catch (error) {
      console.error('SSH连接失败:', error);
      // 抛出用户友好的错误消息
      throw this.formatSshError(error, profile);
    }
    
    // 延迟建立SFTP连接
    setTimeout(async () => {
      const sftpId = await this.establishSftpConnection(id, profile, password);
      if (sftpId) {
        // 通知前端更新SFTP连接ID
        window.dispatchEvent(new CustomEvent('sftp-connected', { 
          detail: { sshId: id, sftpId } 
        }));
      }
    }, 2000); // 等待2秒让SSH连接稳定
    
    return {
      id,
      title,
      type: 'ssh',
      profile,
      autoPassword: password,
      sftpConnectionId: null // 初始为null，连接成功后更新
    };
  }
  
  /**
   * 创建新的SSH连接
   * @param {Object} sshData SSH连接数据
   * @returns {Object} 连接信息
   */
  async createSshConnection(sshData) {
    const id = `ssh-${Date.now()}`;
    const title = sshData.name || sshData.host;
    const profile = { 
      id, 
      host: sshData.host, 
      port: Number(sshData.port) || 22, 
      username: sshData.username, 
      save_password: sshData.savePassword,
      name: sshData.name,
      group: sshData.group,
      tags: sshData.tags || [],
      private_key: sshData.usePrivateKey ? sshData.privateKey : null
    };
    
    const autoPassword = sshData.password && !sshData.usePrivateKey ? sshData.password : '';
    
    // 保存配置
    if (sshData.savePassword) {
      try {
        await invoke('save_ssh_profile', { 
          profile, 
          password: sshData.password 
        });
      } catch (e) {
        console.error('保存SSH配置失败:', e);
      }
    }
    
    // 启动SSH连接
    try {
      await invoke('start_ssh_terminal', { 
        id, 
        host: sshData.host,
        port: Number(sshData.port) || 22,
        username: sshData.username,
        password: sshData.password,
        cols: 80, 
        rows: 24
      });
    } catch (error) {
      console.error('SSH连接失败:', error);
      // 抛出用户友好的错误消息
      throw this.formatSshError(error, { 
        host: sshData.host, 
        port: Number(sshData.port) || 22, 
        username: sshData.username 
      });
    }
    
    // 延迟建立SFTP连接
    setTimeout(async () => {
      const sftpId = await this.establishSftpConnection(id, profile, sshData.password);
      if (sftpId) {
        // 通知前端更新SFTP连接ID
        window.dispatchEvent(new CustomEvent('sftp-connected', { 
          detail: { sshId: id, sftpId } 
        }));
      }
    }, 2000); // 等待2秒让SSH连接稳定
    
    return {
      id,
      title,
      type: 'ssh',
      profile,
      autoPassword,
      sftpConnectionId: null // 初始为null，连接成功后更新
    };
  }
  
  /**
   * 建立SFTP连接
   * @param {string} id SSH连接ID
   * @param {Object} profile SSH配置信息
   * @param {string} password SSH密码（可选）
   */
  async establishSftpConnection(id, profile, password = null) {
    try {
      if (!profile) {
        console.warn('无法获取SSH连接信息');
        return null;
      }
      
      // 直接创建独立的SFTP连接
      const sftpId = `sftp-${id}`;
      await invoke('connect_sftp', {
        connectionId: sftpId,
        host: profile.host,
        port: profile.port || 22,
        username: profile.username,
        password: password || profile.password || null
      });
      
      console.log(`独立SFTP连接已建立: ${sftpId}`);
      return sftpId;
    } catch (error) {
      console.warn(`SFTP连接初始化失败:`, error);
      return null;
    }
  }
  
  /**
   * 获取所有保存的SSH配置
   * @returns {Array} SSH配置列表
   */
  async getProfiles() {
    try {
      return await invoke('list_ssh_profiles');
    } catch (e) {
      console.error('获取SSH配置列表失败:', e);
      return [];
    }
  }
  
  /**
   * 更新SSH配置
   * @param {Object} profileData 更新的配置数据
   */
  async updateProfile(profileData) {
    try {
      const profile = {
        id: profileData.id,
        name: profileData.name,
        host: profileData.host,
        port: Number(profileData.port) || 22,
        username: profileData.username,
        save_password: profileData.savePassword,
        group: profileData.group,
        tags: profileData.tags || [],
        private_key: profileData.usePrivateKey ? profileData.privateKey : null
      };
      
      await invoke('save_ssh_profile', {
        profile,
        password: profileData.password || null
      });
    } catch (e) {
      console.error('更新SSH配置失败:', e);
      throw e;
    }
  }
  
  /**
   * 关闭SSH连接
   * @param {string} id 连接ID
   */
  async closeConnection(id) {
    try {
      await invoke('close_ssh_terminal', { id });
      // 尝试关闭SFTP连接，如果不存在则忽略错误
      try {
        await invoke('disconnect_sftp', { connectionId: id });
      } catch (sftpError) {
        // SFTP连接可能不存在，这是正常的，不需要报错
        console.log('SFTP连接已关闭或不存在:', id);
      }
    } catch (e) {
      // 如果SSH终端未找到，说明连接从未建立，这是正常情况，不需要报错
      if (e.toString().includes('SSH终端未找到') || e.toString().includes('未找到')) {
        console.log('SSH连接不存在或已关闭:', id);
      } else {
        console.error('关闭SSH连接失败:', e);
      }
    }
  }
  
  /**
   * 重新连接SSH
   * @param {string} id 连接ID
   * @param {Object} profile SSH配置
   */
  async reconnect(id, profile) {
    try {
      // 关闭旧连接
      try {
        await invoke('close_ssh_terminal', { id });
      } catch (e) {
        // 如果SSH终端未找到，说明连接从未建立，这是正常情况
        if (!e.toString().includes('SSH终端未找到') && !e.toString().includes('未找到')) {
          console.error('关闭旧SSH连接失败:', e);
        }
      }
      
      // 尝试关闭SFTP连接，如果不存在则忽略错误
      try {
        await invoke('disconnect_sftp', { connectionId: id });
      } catch (sftpError) {
        console.log('SFTP连接已关闭或不存在:', id);
      }
      
      // 获取保存的密码
      let password = null;
      if (profile.save_password) {
        try {
          password = await invoke('get_ssh_password', { id: profile.id });
        } catch {}
      }
      
      // 重新启动SSH连接
      await invoke('start_ssh_terminal', { 
        id, 
        host: profile.host,
        port: profile.port || 22,
        username: profile.username,
        password: password,
        cols: 80, 
        rows: 24
      });
      
      // 延迟建立SFTP连接
      setTimeout(async () => {
        const sftpId = await this.establishSftpConnection(id, profile, password);
        if (sftpId) {
          // 通知前端更新SFTP连接ID
          window.dispatchEvent(new CustomEvent('sftp-connected', { 
            detail: { sshId: id, sftpId } 
          }));
        }
      }, 2000);
    } catch (e) {
      console.error('重新连接SSH失败:', e);
    }
  }
  
  /**
   * 向SSH终端写入数据
   * @param {string} id 终端ID
   * @param {string} data 数据
   */
  async writeTerminal(id, data) {
    try {
      await invoke('write_ssh_terminal', { id, data });
    } catch (e) {
      console.error('向SSH终端写入数据失败:', e);
    }
  }
  
  /**
   * 调整SSH终端大小
   * @param {string} id 终端ID
   * @param {number} cols 列数
   * @param {number} rows 行数
   */
  async resizeTerminal(id, cols, rows) {
    try {
      await invoke('resize_ssh_terminal', { id, cols, rows });
    } catch (e) {
      console.error('调整SSH终端大小失败:', e);
    }
  }
  
  /**
   * 格式化SSH错误消息，返回用户友好的错误提示
   * @param {string|Error} error 原始错误
   * @param {Object} profile 连接配置
   * @returns {string} 格式化后的错误消息
   */
  formatSshError(error, profile) {
    const errorStr = error.toString().toLowerCase();
    const hostInfo = `${profile.username}@${profile.host}:${profile.port}`;
    
    if (errorStr.includes('connection refused') || errorStr.includes('拒绝连接')) {
      return `连接被拒绝 (${hostInfo})\n请检查：\n1. 主机地址和端口是否正确\n2. SSH服务是否运行\n3. 防火墙设置`;
    }
    
    if (errorStr.includes('timeout') || errorStr.includes('超时')) {
      return `连接超时 (${hostInfo})\n请检查：\n1. 网络连接是否正常\n2. 主机地址是否可达\n3. 端口是否正确`;
    }
    
    if (errorStr.includes('authentication') || errorStr.includes('认证') || errorStr.includes('密码') || errorStr.includes('password')) {
      return `认证失败 (${hostInfo})\n请检查：\n1. 用户名是否正确\n2. 密码是否正确\n3. 私钥文件是否有效`;
    }
    
    if (errorStr.includes('host key') || errorStr.includes('主机密钥')) {
      return `主机密钥验证失败 (${hostInfo})\n请检查：\n1. 主机密钥是否已更改\n2. 是否为第一次连接此主机`;
    }
    
    if (errorStr.includes('network') || errorStr.includes('网络')) {
      return `网络错误 (${hostInfo})\n请检查网络连接是否正常`;
    }
    
    if (errorStr.includes('permission') || errorStr.includes('权限')) {
      return `权限被拒绝 (${hostInfo})\n请检查用户是否有SSH登录权限`;
    }
    
    // 如果是未知错误，返回通用错误消息
    return `SSH连接失败 (${hostInfo})\n错误详情: ${error.toString()}`;
  }
}

// 导出单例
export default new SshService();