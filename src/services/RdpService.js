import { invoke } from '@tauri-apps/api/core';

/**
 * RDP服务 - 处理远程桌面连接相关功能
 */
class RdpService {
  /**
   * 获取所有 RDP 配置
   * @returns {Promise<Array>} RDP配置列表
   */
  async getProfiles() {
    try {
      return await invoke('list_rdp_profiles');
    } catch (e) {
      console.error('获取RDP配置列表失败:', e);
      return [];
    }
  }

  /**
   * 保存 RDP 配置
   * @param {Object} profile RDP配置
   * @returns {Promise<void>}
   */
  async saveProfile(profile) {
    return await invoke('save_rdp_profile', { profile });
  }

  /**
   * 删除 RDP 配置
   * @param {string} profileId 配置ID
   * @returns {Promise<void>}
   */
  async deleteProfile(profileId) {
    return await invoke('delete_rdp_profile', { profileId });
  }

  /**
   * 启动 RDP 连接
   * @param {Object} profile RDP配置
   * @returns {Promise<void>}
   */
  async launchConnection(profile) {
    return await invoke('launch_rdp', { profile });
  }

  /**
   * 创建新的 RDP 配置并保存
   * @param {Object} rdpData 表单数据
   * @returns {Promise<Object>} 保存的配置
   */
  async createProfile(rdpData) {
    const profile = {
      id: rdpData.id || `rdp-${Date.now()}-${Math.random().toString(36).substring(2, 8)}`,
      host: rdpData.host,
      port: Number(rdpData.port) || 3389,
      username: rdpData.username || null,
      domain: rdpData.domain || null,
      name: rdpData.name || null,
      group: rdpData.group || null,
      tags: rdpData.tags || [],
      fullscreen: rdpData.fullscreen !== undefined ? rdpData.fullscreen : true,
      width: rdpData.width ? Number(rdpData.width) : null,
      height: rdpData.height ? Number(rdpData.height) : null,
      admin_mode: rdpData.adminMode || false,
    };

    await this.saveProfile(profile);
    return profile;
  }

  /**
   * 更新已有的 RDP 配置
   * @param {Object} rdpData 表单数据（包含id）
   * @returns {Promise<Object>} 更新后的配置
   */
  async updateProfile(rdpData) {
    const profile = {
      id: rdpData.id,
      host: rdpData.host,
      port: Number(rdpData.port) || 3389,
      username: rdpData.username || null,
      domain: rdpData.domain || null,
      name: rdpData.name || null,
      group: rdpData.group || null,
      tags: rdpData.tags || [],
      fullscreen: rdpData.fullscreen !== undefined ? rdpData.fullscreen : true,
      width: rdpData.width ? Number(rdpData.width) : null,
      height: rdpData.height ? Number(rdpData.height) : null,
      admin_mode: rdpData.adminMode || false,
    };

    await this.saveProfile(profile);
    return profile;
  }
}

export default new RdpService();
