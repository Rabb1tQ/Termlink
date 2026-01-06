use russh::client;
use russh_keys::key;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use async_trait::async_trait;
use tauri::command;
use crate::ssh_terminal_russh;

// SSH连接池，复用现有连接
lazy_static::lazy_static! {
    static ref SSH_SESSIONS: Arc<Mutex<HashMap<String, Arc<Mutex<client::Handle<Client>>>>>> = 
        Arc::new(Mutex::new(HashMap::new()));
}

struct Client;

#[async_trait]
impl client::Handler for Client {
    type Error = russh::Error;

    async fn check_server_key(
        self,
        _server_public_key: &key::PublicKey,
    ) -> Result<(Self, bool), Self::Error> {
        Ok((self, true))
    }
}

// 执行SSH命令
#[command]
pub async fn execute_ssh_command(connection_id: String, command: String) -> Result<String, String> {
    // 优先使用独立的SSH监控连接池（避免影响Terminal session）
    let session_handle = {
        let sessions = SSH_SESSIONS.lock().await;
        sessions.get(&connection_id).cloned()
    };

    if let Some(session_handle) = session_handle {
        // 使用异步锁，可以安全地跨await持有
        let session = session_handle.lock().await;
        
        match session.channel_open_session().await {
            Ok(mut channel) => {
                // 执行命令
                if let Err(e) = channel.exec(true, command.as_bytes()).await {
                    return Err(format!("执行命令失败: {}", e));
                }

                // 读取输出
                let mut output = String::new();
                let mut code = None;
                
                while let Some(msg) = channel.wait().await {
                    match msg {
                        russh::ChannelMsg::Data { data } => {
                            output.push_str(&String::from_utf8_lossy(&data));
                        },
                        russh::ChannelMsg::ExitStatus { exit_status } => {
                            code = Some(exit_status);
                        },
                        russh::ChannelMsg::Eof => {
                            break;
                        },
                        _ => {}
                    }
                }

                if code == Some(0) || code.is_none() {
                    Ok(output.trim().to_string())
                } else {
                    Err(format!("命令执行失败，退出码: {:?}", code))
                }
            },
            Err(e) => Err(format!("创建通道失败: {}", e))
        }
    } else {
        Err("SSH连接不存在".to_string())
    }
}

// 注册SSH会话（由SSH连接模块调用）
pub async fn register_ssh_session(connection_id: String, session: Arc<Mutex<client::Handle<Client>>>) {
    let mut sessions = SSH_SESSIONS.lock().await;
    sessions.insert(connection_id, session);
}

// 移除SSH会话
pub async fn unregister_ssh_session(connection_id: &str) {
    let mut sessions = SSH_SESSIONS.lock().await;
    sessions.remove(connection_id);
}

// 为系统监控创建SSH连接
#[command]
pub async fn connect_ssh_for_monitoring(
    connection_id: String,
    host: String,
    port: u16,
    username: String,
    password: Option<String>
) -> Result<(), String> {
    println!("为系统监控创建/检查SSH连接: {}@{}:{}", username, host, port);
    
    // 检查是否已存在连接
    let exists = {
        let sessions = SSH_SESSIONS.lock().await;
        sessions.contains_key(&connection_id)
    };
    
    if exists {
        println!("✓ SSH监控连接已存在，复用连接");
        return Ok(());
    }
    
    // 创建客户端配置
    let config = client::Config {
        inactivity_timeout: Some(std::time::Duration::from_secs(600)),
        ..<_>::default()
    };
    
    // 连接到服务器
    let mut session = match client::connect(Arc::new(config), (&host[..], port), Client).await {
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
    
    // 保存会话
    let session_handle = Arc::new(Mutex::new(session));
    register_ssh_session(connection_id, session_handle).await;
    
    println!("✓ SSH监控连接建立成功");
    Ok(())
}

// 断开系统监控的SSH连接
#[command]
pub async fn disconnect_ssh_monitoring(connection_id: String) -> Result<(), String> {
    unregister_ssh_session(&connection_id).await;
    println!("SSH监控连接已断开: {}", connection_id);
    Ok(())
}
