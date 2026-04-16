use std::fs;
use std::path::PathBuf;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

/// RDP 配置文件结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RdpProfileMeta {
  pub id: String,
  pub host: String,
  pub port: u16,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub username: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub domain: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub group: Option<String>,
  #[serde(skip_serializing_if = "Vec::is_empty", default)]
  pub tags: Vec<String>,
  #[serde(default = "default_true")]
  pub fullscreen: bool,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub width: Option<u32>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub height: Option<u32>,
  #[serde(default)]
  pub admin_mode: bool,
}

fn default_true() -> bool {
  true
}

/// 获取 RDP 配置文件存储目录
pub fn rdp_profiles_dir(_app: &AppHandle) -> Result<PathBuf, String> {
  let proj = ProjectDirs::from("com", "Termlink", "Termlink").ok_or("no project dirs")?;
  let base = proj.config_dir().to_path_buf();
  let dir = base.join("rdp_profiles");
  fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
  Ok(dir)
}

/// 保存 RDP 配置
#[tauri::command]
pub fn save_rdp_profile(app: AppHandle, profile: RdpProfileMeta) -> Result<(), String> {
  let dir = rdp_profiles_dir(&app)?;
  let path = dir.join(format!("{}.json", profile.id));
  let data = serde_json::to_string_pretty(&profile).map_err(|e| e.to_string())?;
  fs::write(path, data).map_err(|e| e.to_string())?;
  Ok(())
}

/// 列出所有 RDP 配置
#[tauri::command]
pub fn list_rdp_profiles(app: AppHandle) -> Result<Vec<RdpProfileMeta>, String> {
  let dir = rdp_profiles_dir(&app)?;
  let mut out = Vec::new();
  for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
    if let Ok(ent) = entry {
      if ent.path().extension().and_then(|s| s.to_str()) == Some("json") {
        if let Ok(txt) = fs::read_to_string(ent.path()) {
          if let Ok(meta) = serde_json::from_str::<RdpProfileMeta>(&txt) {
            out.push(meta);
          }
        }
      }
    }
  }
  Ok(out)
}

/// 删除 RDP 配置
#[tauri::command]
pub fn delete_rdp_profile(app: AppHandle, profile_id: String) -> Result<(), String> {
  let dir = rdp_profiles_dir(&app)?;
  let path = dir.join(format!("{}.json", profile_id));
  if path.exists() {
    fs::remove_file(path).map_err(|e| e.to_string())?;
  }
  Ok(())
}

/// 生成 .rdp 文件内容
fn generate_rdp_file_content(profile: &RdpProfileMeta) -> String {
  let mut lines = Vec::new();

  // 全屏模式: 2 = 全屏, 1 = 窗口
  let screen_mode = if profile.fullscreen { 2 } else { 1 };
  lines.push(format!("screen mode id:i:{}", screen_mode));

  // 窗口分辨率（仅在窗口模式下有效）
  if let Some(w) = profile.width {
    lines.push(format!("desktopwidth:i:{}", w));
  }
  if let Some(h) = profile.height {
    lines.push(format!("desktopheight:i:{}", h));
  }

  // 色深 32 位
  lines.push("session bpp:i:32".to_string());

  // 连接地址
  if profile.port != 3389 {
    lines.push(format!("full address:s:{}:{}", profile.host, profile.port));
  } else {
    lines.push(format!("full address:s:{}", profile.host));
  }

  // 用户名
  if let Some(ref username) = profile.username {
    if let Some(ref domain) = profile.domain {
      if !domain.is_empty() {
        lines.push(format!("username:s:{}\\{}", domain, username));
      } else {
        lines.push(format!("username:s:{}", username));
      }
    } else {
      lines.push(format!("username:s:{}", username));
    }
  }

  // 管理模式
  if profile.admin_mode {
    lines.push("administrative session:i:1".to_string());
  }

  // 一些合理的默认设置
  lines.push("compression:i:1".to_string());
  lines.push("keyboardhook:i:2".to_string());
  lines.push("audiocapturemode:i:0".to_string());
  lines.push("videoplaybackmode:i:1".to_string());
  lines.push("connection type:i:7".to_string());
  lines.push("networkautodetect:i:1".to_string());
  lines.push("bandwidthautodetect:i:1".to_string());
  lines.push("displayconnectionbar:i:1".to_string());
  lines.push("enableworkspacereconnect:i:0".to_string());
  lines.push("disable wallpaper:i:0".to_string());
  lines.push("allow font smoothing:i:1".to_string());
  lines.push("allow desktop composition:i:1".to_string());
  lines.push("disable full window drag:i:0".to_string());
  lines.push("disable menu anims:i:0".to_string());
  lines.push("disable themes:i:0".to_string());
  lines.push("disable cursor setting:i:0".to_string());
  lines.push("bitmapcachepersistenable:i:1".to_string());
  lines.push("audiomode:i:0".to_string());
  lines.push("redirectprinters:i:1".to_string());
  lines.push("redirectcomports:i:0".to_string());
  lines.push("redirectsmartcards:i:1".to_string());
  lines.push("redirectclipboard:i:1".to_string());
  lines.push("redirectposdevices:i:0".to_string());
  lines.push("autoreconnection enabled:i:1".to_string());
  lines.push("authentication level:i:2".to_string());
  lines.push("prompt for credentials:i:1".to_string());
  lines.push("negotiate security layer:i:1".to_string());
  lines.push("remoteapplicationmode:i:0".to_string());
  lines.push("alternate shell:s:".to_string());
  lines.push("shell working directory:s:".to_string());
  lines.push("gatewayhostname:s:".to_string());
  lines.push("gatewayusagemethod:i:4".to_string());
  lines.push("gatewaycredentialssource:i:4".to_string());
  lines.push("gatewayprofileusagemethod:i:0".to_string());
  lines.push("promptcredentialonce:i:0".to_string());
  lines.push("use redirection server name:i:0".to_string());

  lines.join("\r\n")
}

/// 启动 RDP 连接 - 生成临时 .rdp 文件并调用 mstsc
#[tauri::command]
pub fn launch_rdp(_app: AppHandle, profile: RdpProfileMeta) -> Result<(), String> {
  // 生成 .rdp 文件内容
  let content = generate_rdp_file_content(&profile);

  // 写入临时文件
  let temp_dir = std::env::temp_dir();
  let rdp_file_path = temp_dir.join(format!("termlink_rdp_{}.rdp", profile.id));
  fs::write(&rdp_file_path, content).map_err(|e| format!("写入RDP文件失败: {}", e))?;

  // 启动 mstsc
  let rdp_path_str = rdp_file_path.to_string_lossy().to_string();
  std::process::Command::new("mstsc")
    .arg(&rdp_path_str)
    .spawn()
    .map_err(|e| format!("启动mstsc失败: {}", e))?;

  Ok(())
}
