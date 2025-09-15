use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileItem {
  pub name: String,
  pub path: String,
  pub is_directory: bool,
  pub size: Option<u64>,
  pub modified: Option<String>,
  pub is_hidden: bool,
}

#[tauri::command]
pub fn list_files(path: String, show_hidden: bool) -> Result<Vec<FileItem>, String> {
  let target_path = if path.is_empty() {
    // 如果路径为空，返回根目录或用户目录
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"))
  } else {
    PathBuf::from(&path)
  };

  if !target_path.exists() {
    return Err("路径不存在".into());
  }

  if !target_path.is_dir() {
    return Err("不是一个目录".into());
  }

  let mut files = Vec::new();
  
  match fs::read_dir(&target_path) {
    Ok(entries) => {
      for entry in entries {
        if let Ok(entry) = entry {
          let entry_path = entry.path();
          let name = entry.file_name().to_string_lossy().to_string();
          
          // 检查是否为隐藏文件
          let is_hidden = name.starts_with('.');
          
          // 如果不显示隐藏文件且是隐藏文件，跳过
          if !show_hidden && is_hidden {
            continue;
          }
          
          let metadata = entry.metadata().ok();
          let is_directory = entry_path.is_dir();
          
          let size = if is_directory {
            None
          } else {
            metadata.as_ref().map(|m| m.len())
          };
          
          let modified = metadata
            .and_then(|m| m.modified().ok())
            .and_then(|time| {
              time.duration_since(SystemTime::UNIX_EPOCH)
                .ok()
                .map(|duration| {
                  // 转换为ISO字符串
                  let secs = duration.as_secs();
                  chrono::DateTime::from_timestamp(secs as i64, 0)
                    .map(|dt| dt.to_rfc3339())
                })
            })
            .flatten();
          
          files.push(FileItem {
            name,
            path: entry_path.to_string_lossy().to_string(),
            is_directory,
            size,
            modified,
            is_hidden,
          });
        }
      }
    }
    Err(e) => return Err(format!("读取目录失败: {}", e)),
  }
  
  // 排序：目录在前，然后按名称排序
  files.sort_by(|a, b| {
    match (a.is_directory, b.is_directory) {
      (true, false) => std::cmp::Ordering::Less,
      (false, true) => std::cmp::Ordering::Greater,
      _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    }
  });
  
  Ok(files)
}

#[tauri::command]
pub fn get_home_dir() -> Result<String, String> {
  dirs::home_dir()
    .map(|path| path.to_string_lossy().to_string())
    .ok_or_else(|| "无法获取用户主目录".into())
}

#[tauri::command]
pub fn get_parent_dir(path: String) -> Result<Option<String>, String> {
  let path_buf = PathBuf::from(path);
  Ok(path_buf.parent().map(|p| p.to_string_lossy().to_string()))
}

#[tauri::command]
pub fn open_file_explorer(path: String) -> Result<(), String> {
  let path_buf = PathBuf::from(path);
  
  #[cfg(target_os = "windows")]
  {
    std::process::Command::new("explorer")
      .arg(path_buf)
      .spawn()
      .map_err(|e| format!("无法打开文件管理器: {}", e))?;
  }
  
  #[cfg(target_os = "macos")]
  {
    std::process::Command::new("open")
      .arg(path_buf)
      .spawn()
      .map_err(|e| format!("无法打开文件管理器: {}", e))?;
  }
  
  #[cfg(target_os = "linux")]
  {
    std::process::Command::new("xdg-open")
      .arg(path_buf)
      .spawn()
      .map_err(|e| format!("无法打开文件管理器: {}", e))?;
  }
  
  Ok(())
}
