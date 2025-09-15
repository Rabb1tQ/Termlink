use std::fs;
use std::path::PathBuf;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshProfileMeta {
  pub id: String,
  pub host: String,
  pub port: u16,
  pub username: String,
  pub save_password: bool,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub private_key: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub group: Option<String>, // 分组名称
  #[serde(skip_serializing_if = "Vec::is_empty", default)]
  pub tags: Vec<String>, // 标签列表
}

pub fn profiles_dir(_app: &AppHandle) -> Result<PathBuf, String> {
  let proj = ProjectDirs::from("com", "Termlink", "Termlink").ok_or("no project dirs")?;
  let base = proj.config_dir().to_path_buf();
  let dir = base.join("profiles");
  fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
  Ok(dir)
}

#[tauri::command]
pub fn save_ssh_profile(app: AppHandle, profile: SshProfileMeta, password: Option<String>) -> Result<(), String> {
  let dir = profiles_dir(&app)?;
  let path = dir.join(format!("{}.json", profile.id));
  let data = serde_json::to_string_pretty(&profile).map_err(|e| e.to_string())?;
  fs::write(path, data).map_err(|e| e.to_string())?;
  if profile.save_password {
    if let Some(pw) = password {
      let entry = keyring::Entry::new("Termlink", &profile.id).map_err(|e| e.to_string())?;
      entry.set_password(&pw).map_err(|e| e.to_string())?;
    }
  }
  Ok(())
}

#[tauri::command]
pub fn list_ssh_profiles(app: AppHandle) -> Result<Vec<SshProfileMeta>, String> {
  let dir = profiles_dir(&app)?;
  let mut out = Vec::new();
  for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
    if let Ok(ent) = entry {
      if ent.path().extension().and_then(|s| s.to_str()) == Some("json") {
        if let Ok(txt) = fs::read_to_string(ent.path()) {
          if let Ok(meta) = serde_json::from_str::<SshProfileMeta>(&txt) { out.push(meta) }
        }
      }
    }
  }
  Ok(out)
}

#[tauri::command]
pub fn get_ssh_password(id: String) -> Result<Option<String>, String> {
  let entry = keyring::Entry::new("Termlink", &id).map_err(|e| e.to_string())?;
  match entry.get_password() {
    Ok(pw) => Ok(Some(pw)),
    Err(keyring::Error::NoEntry) => Ok(None),
    Err(e) => Err(e.to_string()),
  }
}

#[tauri::command]
pub fn restart_ssh_connection(_app: AppHandle, id: String, _profile_id: String) -> Result<(), String> {
  // 先关闭当前连接
  crate::terminal::close_pty(id)?;
  
  // TODO: 重新启动SSH连接的逻辑可以在前端实现
  // 这里只是清理资源
  Ok(())
}

#[tauri::command]
pub fn delete_ssh_profile(app: AppHandle, profile_id: String) -> Result<(), String> {
  let dir = profiles_dir(&app)?;
  let path = dir.join(format!("{}.json", profile_id));
  
  // 删除配置文件
  if path.exists() {
    fs::remove_file(path).map_err(|e| e.to_string())?;
  }
  
  // 删除保存的密码
  let entry = keyring::Entry::new("Termlink", &profile_id).map_err(|e| e.to_string())?;
  let _ = entry.delete_password(); // 忽略删除密码的错误，因为可能没有保存密码
  
  Ok(())
}

#[tauri::command]
pub fn get_profiles_dir(app: AppHandle) -> Result<String, String> {
  let dir = profiles_dir(&app)?;
  Ok(dir.to_string_lossy().to_string())
}