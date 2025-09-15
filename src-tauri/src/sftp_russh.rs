use once_cell::sync::Lazy;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use russh::*;
use russh_keys::*;
use russh_sftp::client::SftpSession;

// SFTP文件信息
#[derive(Serialize, Deserialize, Debug)]
pub struct SftpFileInfo {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: Option<u64>,
    pub permissions: String,
}

// SFTP连接管理
struct SftpConnection {
    session: Arc<SftpSession>,
}

static SFTP_CONNECTIONS: Lazy<Mutex<HashMap<String, SftpConnection>>> = 
    Lazy::new(|| Mutex::new(HashMap::new()));

// SSH客户端处理器
struct SftpClient;

#[async_trait::async_trait]
impl client::Handler for SftpClient {
    type Error = russh::Error;

    async fn check_server_key(
        self,
        _server_public_key: &key::PublicKey,
    ) -> Result<(Self, bool), Self::Error> {
        // 接受所有服务器密钥 (在生产环境中应该验证)
        Ok((self, true))
    }
}

// 连接到SFTP服务器
#[tauri::command]
pub async fn connect_sftp(
    connection_id: String, 
    host: String, 
    port: u16, 
    username: String, 
    password: Option<String>
) -> Result<(), String> {
    println!("连接SFTP服务器: {}@{}:{}", username, host, port);
    
    // 创建客户端配置
    let config = client::Config {
        inactivity_timeout: Some(std::time::Duration::from_secs(300)),
        ..<_>::default()
    };
    
    println!("连接到SSH服务器...");
    
    // 连接到服务器
    let mut session = match client::connect(Arc::new(config), (&host[..], port), SftpClient).await {
        Ok(session) => {
            println!("✓ SSH连接成功");
            session
        },
        Err(e) => {
            return Err(format!("SSH连接失败: {}", e));
        }
    };
    
    // 进行认证
    if let Some(pwd) = password {
        println!("开始密码认证...");
        match session.authenticate_password(&username, &pwd).await {
            Ok(true) => {
                println!("✓ 密码认证成功");
            },
            Ok(false) => {
                return Err("密码认证失败：用户名或密码错误".to_string());
            },
            Err(e) => {
                return Err(format!("密码认证过程失败: {}", e));
            }
        }
    } else {
        return Err("当前仅支持密码认证".to_string());
    }
    
    println!("创建SFTP通道...");
    
    // 创建SFTP通道
    let channel = match session.channel_open_session().await {
        Ok(channel) => channel,
        Err(e) => {
            return Err(format!("创建通道失败: {}", e));
        }
    };
    
    // 请求SFTP子系统
    if let Err(e) = channel.request_subsystem(true, "sftp").await {
        return Err(format!("请求SFTP子系统失败: {}", e));
    }
    
    // 创建SFTP会话
    let sftp_session = match SftpSession::new(channel.into_stream()).await {
        Ok(session) => {
            println!("✓ SFTP会话创建成功");
            session
        },
        Err(e) => {
            return Err(format!("创建SFTP会话失败: {}", e));
        }
    };
    
    // 保存连接
    let connection = SftpConnection {
        session: Arc::new(sftp_session),
    };
    
    SFTP_CONNECTIONS.lock().insert(connection_id, connection);
    println!("SFTP连接建立成功");
    Ok(())
}

// 断开SFTP连接
#[tauri::command]
pub fn disconnect_sftp(connection_id: String) -> Result<(), String> {
    let mut connections = SFTP_CONNECTIONS.lock();
    if let Some(_connection) = connections.remove(&connection_id) {
        println!("SFTP连接已断开: {}", connection_id);
        Ok(())
    } else {
        Err("SFTP连接不存在".to_string())
    }
}

// 列出SFTP目录文件
#[tauri::command]
pub async fn list_sftp_files(connection_id: String, path: String) -> Result<Vec<SftpFileInfo>, String> {
    let session = {
        let connections = SFTP_CONNECTIONS.lock();
        let connection = match connections.get(&connection_id) {
            Some(conn) => conn,
            None => return Err("SFTP连接不存在".to_string()),
        };
        connection.session.clone()
    }; // 锁在这里自动释放
    
    println!("列出目录: {}", path);
    
    match session.read_dir(&path).await {
            Ok(entries) => {
                let mut files = Vec::new();
                
                for entry in entries {
                    let metadata = entry.metadata();
                    let file_info = SftpFileInfo {
                        name: entry.file_name().to_string(),
                        is_dir: entry.file_type().is_dir(),
                        size: metadata.len(),
                        modified: metadata.modified().ok().and_then(|t| 
                            t.duration_since(std::time::UNIX_EPOCH).ok()
                        ).map(|d| d.as_secs()),
                        permissions: {
                            let perms = metadata.permissions();
                            // 构建完整的权限字符串，使用可用的字段
                            let mut perm_str = String::new();
                            
                            // 用户权限 (如果字段不存在，显示为 "--x")
                            perm_str.push_str("---");
                            
                            // 组权限 (使用group_字段)
                            perm_str.push(if perms.group_read { 'r' } else { '-' });
                            perm_str.push(if perms.group_write { 'w' } else { '-' });
                            perm_str.push(if perms.group_exec { 'x' } else { '-' });
                            
                            // 其他权限 (使用other_字段)
                            perm_str.push(if perms.other_read { 'r' } else { '-' });
                            perm_str.push(if perms.other_write { 'w' } else { '-' });
                            perm_str.push(if perms.other_exec { 'x' } else { '-' });
                            
                            perm_str
                        },
                    };
                    files.push(file_info);
                }
                
                // 按名称排序，目录在前
                files.sort_by(|a, b| {
                    match (a.is_dir, b.is_dir) {
                        (true, false) => std::cmp::Ordering::Less,
                        (false, true) => std::cmp::Ordering::Greater,
                        _ => a.name.cmp(&b.name),
                    }
                });
                
                Ok(files)
            },
            Err(e) => Err(format!("列出目录失败: {}", e)),
        }
}

// 下载SFTP文件
#[tauri::command]
pub async fn download_sftp_file(
    connection_id: String, 
    remote_path: String, 
    local_path: String
) -> Result<(), String> {
    let session = {
        let connections = SFTP_CONNECTIONS.lock();
        let connection = match connections.get(&connection_id) {
            Some(conn) => conn,
            None => return Err("SFTP连接不存在".to_string()),
        };
        connection.session.clone()
    }; // 锁在这里自动释放
        
        println!("下载文件: {} -> {}", remote_path, local_path);
        
        // 读取远程文件
        let data = match session.read(&remote_path).await {
            Ok(data) => data,
            Err(e) => return Err(format!("读取远程文件失败: {}", e)),
        };
        
        // 写入本地文件
        match tokio::fs::write(&local_path, data).await {
            Ok(_) => {
                println!("文件下载成功");
                Ok(())
            },
            Err(e) => Err(format!("写入本地文件失败: {}", e)),
        }
}

// 上传文件到SFTP
#[tauri::command]
pub async fn upload_sftp_file(
    connection_id: String, 
    local_path: String, 
    remote_path: String
) -> Result<(), String> {
    let session = {
        let connections = SFTP_CONNECTIONS.lock();
        let connection = match connections.get(&connection_id) {
            Some(conn) => conn,
            None => return Err("SFTP连接不存在".to_string()),
        };
        connection.session.clone()
    }; // 锁在这里自动释放
        
        println!("上传文件: {} -> {}", local_path, remote_path);
        
        // 读取本地文件
        let data = match tokio::fs::read(&local_path).await {
            Ok(data) => data,
            Err(e) => return Err(format!("读取本地文件失败: {}", e)),
        };
        
        // 写入远程文件
        match session.write(&remote_path, &data).await {
            Ok(_) => {
                println!("文件上传成功");
                Ok(())
            },
            Err(e) => Err(format!("写入远程文件失败: {}", e)),
        }
}

// 读取SFTP文件内容
#[tauri::command]
pub async fn read_sftp_file(connection_id: String, path: String) -> Result<String, String> {
    let session = {
        let connections = SFTP_CONNECTIONS.lock();
        let connection = match connections.get(&connection_id) {
            Some(conn) => conn,
            None => return Err("SFTP连接不存在".to_string()),
        };
        connection.session.clone()
    }; // 锁在这里自动释放
        
        println!("读取文件: {}", path);
        
        match session.read(&path).await {
            Ok(data) => {
                match String::from_utf8(data) {
                    Ok(content) => Ok(content),
                    Err(_) => Err("文件内容不是有效的UTF-8文本".to_string()),
                }
            },
            Err(e) => Err(format!("读取文件失败: {}", e)),
        }
}

// 写入SFTP文件内容
#[tauri::command]
pub async fn write_sftp_file(
    connection_id: String, 
    path: String, 
    content: String
) -> Result<(), String> {
    let session = {
        let connections = SFTP_CONNECTIONS.lock();
        let connection = match connections.get(&connection_id) {
            Some(conn) => conn,
            None => return Err("SFTP连接不存在".to_string()),
        };
        connection.session.clone()
    }; // 锁在这里自动释放
        
        println!("写入文件: {}", path);
        
        match session.write(&path, &content.into_bytes()).await {
            Ok(_) => {
                println!("文件写入成功");
                Ok(())
            },
            Err(e) => Err(format!("写入文件失败: {}", e)),
        }
}

// 删除SFTP文件
#[tauri::command]
pub async fn delete_sftp_file(connection_id: String, path: String) -> Result<(), String> {
    let session = {
        let connections = SFTP_CONNECTIONS.lock();
        let connection = match connections.get(&connection_id) {
            Some(conn) => conn,
            None => return Err("SFTP连接不存在".to_string()),
        };
        connection.session.clone()
    }; // 锁在这里自动释放
        
        println!("删除文件: {}", path);
        
        match session.remove_file(&path).await {
            Ok(_) => {
                println!("文件删除成功");
                Ok(())
            },
            Err(e) => Err(format!("删除文件失败: {}", e)),
        }
}

// 创建SFTP目录
#[tauri::command]
pub async fn create_sftp_directory(connection_id: String, path: String) -> Result<(), String> {
    let session = {
        let connections = SFTP_CONNECTIONS.lock();
        let connection = match connections.get(&connection_id) {
            Some(conn) => conn,
            None => return Err("SFTP连接不存在".to_string()),
        };
        connection.session.clone()
    }; // 锁在这里自动释放
        
        println!("创建目录: {}", path);
        
        match session.create_dir(&path).await {
            Ok(_) => {
                println!("目录创建成功");
                Ok(())
            },
            Err(e) => Err(format!("创建目录失败: {}", e)),
        }
}

// 从SSH会话创建SFTP连接（新实现）
#[tauri::command]
pub fn create_sftp_from_ssh(_connection_id: String) -> Result<(), String> {
    // TODO: 实现从russh会话创建SFTP连接
    // 这需要在SSH终端模块中暴露会话，然后在这里复用
    Err("从SSH会话创建SFTP连接功能正在开发中".to_string())
}
