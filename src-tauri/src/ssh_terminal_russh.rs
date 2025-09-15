use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::{collections::HashMap, thread, sync::Arc};
use tauri::Emitter;
use tokio::runtime::Runtime;
use russh::*;
use russh_keys::*;
use crate::ssh_command;

// SSH终端消息类型
enum SshMsg {
    Write(String),
    Resize { cols: u16, rows: u16 },
    Close,
    ExecuteCommand { command: String, response_sender: tokio::sync::oneshot::Sender<Result<String, String>> },
}

// SSH终端连接
struct SshTerminal {
    sender: crossbeam_channel::Sender<SshMsg>,
    runtime: Option<Arc<Runtime>>,
}

// 全局SSH终端连接管理
static SSH_TERMINALS: Lazy<Mutex<HashMap<String, SshTerminal>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

// SSH客户端处理器
struct Client;

#[async_trait::async_trait]
impl client::Handler for Client {
    type Error = russh::Error;

    async fn check_server_key(
        self,
        _server_public_key: &key::PublicKey,
    ) -> Result<(Self, bool), Self::Error> {
        // 接受所有服务器密钥 (在生产环境中应该验证)
        Ok((self, true))
    }
}

// 启动SSH终端
#[tauri::command]
pub fn start_ssh_terminal(
    window: tauri::Window,
    id: String,
    host: String,
    port: u16,
    username: String,
    password: Option<String>,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    println!("开始SSH连接: {}@{}:{}", username, host, port);
    
    // 创建通信通道
    let (tx, rx) = crossbeam_channel::unbounded::<SshMsg>();
    
    // 创建Tokio运行时
    let runtime = match Runtime::new() {
        Ok(rt) => Arc::new(rt),
        Err(e) => return Err(format!("创建运行时失败: {}", e)),
    };
    
    // 保存发送端和运行时
    SSH_TERMINALS.lock().insert(id.clone(), SshTerminal {
        sender: tx,
        runtime: Some(runtime.clone()),
    });
    
    // 在新线程中处理SSH连接
    let runtime_clone = runtime.clone();
    thread::spawn(move || {
        // 在运行时中执行异步SSH连接
        match runtime_clone.block_on(connect_ssh_russh(&window, &id, &host, port, &username, password, cols, rows, rx)) {
            Ok(_) => {
                println!("SSH连接关闭: {}@{}:{}", username, host, port);
                let _ = window.emit(&format!("ssh_exit://{}", id), "");
            }
            Err(e) => {
                println!("SSH连接错误: {}", e);
                let _ = window.emit("ssh_error", format!("{}: {}", id, e));
                let _ = window.emit(&format!("ssh_exit://{}", id), "");
            }
        }
        
        // 连接结束后移除
        SSH_TERMINALS.lock().remove(&id);
    });
    
    Ok(())
}

// 使用russh连接到SSH服务器
async fn connect_ssh_russh(
    window: &tauri::Window,
    id: &str,
    host: &str,
    port: u16,
    username: &str,
    password: Option<String>,
    cols: u16,
    rows: u16,
    rx: crossbeam_channel::Receiver<SshMsg>,
) -> Result<(), String> {
    println!("使用russh库连接SSH服务器...");
    
    // 输出连接信息
    println!("SSH连接详情:");
    println!("  目标主机: {}:{}", host, port);
    println!("  用户名: {}", username);
    println!("  认证方式: {}", if password.is_some() { "密码认证" } else { "密钥认证" });
    
    // 创建客户端配置
    let config = client::Config {
        inactivity_timeout: Some(std::time::Duration::from_secs(300)), // 5分钟超时
        ..<_>::default()
    };
    
    println!("连接到SSH服务器...");
    
    // 连接到服务器
    let mut session = match client::connect(Arc::new(config), (host, port), Client).await {
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
        match session.authenticate_password(username, &pwd).await {
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
    
    // 这里我们不需要克隆session，因为Handle不支持Clone
    // 我们将采用不同的策略来支持系统监控
    
    println!("创建终端通道...");
    
    // 创建通道
    let channel = match session.channel_open_session().await {
        Ok(channel) => {
            println!("✓ 终端通道创建成功");
            channel
        },
        Err(e) => {
            return Err(format!("创建终端通道失败: {}", e));
        }
    };
    
    // 请求PTY
    if let Err(e) = channel.request_pty(true, "xterm-256color", cols as u32, rows as u32, 0, 0, &[]).await {
        return Err(format!("请求PTY失败: {}", e));
    }
    
    // 启动shell
    if let Err(e) = channel.request_shell(true).await {
        return Err(format!("启动shell失败: {}", e));
    }
    
    println!("✓ SSH终端启动成功");
    
    // 处理终端会话
    handle_russh_session(window, id, session, channel, rx).await
}

// 处理russh会话
async fn handle_russh_session(
    window: &tauri::Window,
    id: &str,
    session: client::Handle<Client>,
    mut channel: Channel<client::Msg>,
    rx: crossbeam_channel::Receiver<SshMsg>,
) -> Result<(), String> {
    use tokio::sync::mpsc;
    
    // 创建异步通道用于线程间通信
    let (tx, mut async_rx) = mpsc::unbounded_channel::<SshMsg>();
    
    // 启动一个任务来监听同步通道并转发到异步通道
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        while let Ok(msg) = rx.recv() {
            if tx_clone.send(msg).is_err() {
                break;
            }
        }
    });
    
    // 主循环处理SSH消息
    loop {
        tokio::select! {
            // 处理来自通道的数据
            Some(msg) = channel.wait() => {
                match msg {
                    ChannelMsg::Data { data } => {
                        let output = String::from_utf8_lossy(&data).to_string();
                        let _ = window.emit(&format!("ssh_data://{}", id), output);
                    },
                    ChannelMsg::Eof => {
                        println!("SSH通道EOF");
                        break;
                    },
                    ChannelMsg::Close => {
                        println!("SSH通道关闭");
                        break;
                    },
                    ChannelMsg::ExitStatus { exit_status } => {
                        println!("SSH进程退出，状态码: {}", exit_status);
                        break;
                    },
                    _ => {}
                }
            },
            
            // 处理用户输入和控制消息
            Some(msg) = async_rx.recv() => {
                match msg {
                    SshMsg::Write(data) => {
                        if let Err(e) = channel.data(data.as_bytes()).await {
                            println!("发送数据失败: {}", e);
                            break;
                        }
                    },
                    SshMsg::Resize { cols, rows } => {
                        if let Err(e) = channel.window_change(cols as u32, rows as u32, 0, 0).await {
                            println!("调整窗口大小失败: {}", e);
                        }
                    },
                    SshMsg::Close => {
                        println!("收到关闭信号");
                        break;
                    },
                    SshMsg::ExecuteCommand { command, response_sender } => {
                        // 执行系统监控命令
                        let result = execute_monitoring_command(&session, &command).await;
                        let _ = response_sender.send(result);
                    }
                }
            },
            
            else => {
                // 所有选项都完成，退出循环
                break;
            }
        }
    }
    
    // 关闭通道
    let _ = channel.eof().await;
    let _ = channel.close().await;
    
    // 断开会话
    let _ = session.disconnect(Disconnect::ByApplication, "", "").await;
    
    Ok(())
}

// 执行系统监控命令
async fn execute_monitoring_command(session: &client::Handle<Client>, command: &str) -> Result<String, String> {
    // 创建新的通道执行命令
    let mut channel = match session.channel_open_session().await {
        Ok(channel) => channel,
        Err(e) => return Err(format!("创建通道失败: {}", e)),
    };

    // 执行命令
    if let Err(e) = channel.exec(true, command.as_bytes()).await {
        return Err(format!("执行命令失败: {}", e));
    }

    // 读取输出
    let mut output = String::new();
    let mut code = None;
    
    while let Some(msg) = channel.wait().await {
        match msg {
            ChannelMsg::Data { data } => {
                output.push_str(&String::from_utf8_lossy(&data));
            },
            ChannelMsg::ExitStatus { exit_status } => {
                code = Some(exit_status);
            },
            ChannelMsg::Eof => {
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
}

// 通过SSH终端执行命令（供系统监控使用）
pub async fn execute_command_via_terminal(terminal_id: &str, command: String) -> Result<String, String> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    
    // 获取终端并发送命令
    {
        let terminals = SSH_TERMINALS.lock();
        if let Some(terminal) = terminals.get(terminal_id) {
            if let Err(e) = terminal.sender.send(SshMsg::ExecuteCommand { 
                command, 
                response_sender: tx 
            }) {
                return Err(format!("发送命令失败: {}", e));
            }
        } else {
            return Err("SSH终端未找到".to_string());
        }
    }
    
    // 等待命令执行结果
    match rx.await {
        Ok(result) => result,
        Err(_) => Err("命令执行超时".to_string()),
    }
}

// 向SSH终端写入数据
#[tauri::command]
pub fn write_ssh_terminal(id: String, data: String) -> Result<(), String> {
    let terminals = SSH_TERMINALS.lock();
    if let Some(terminal) = terminals.get(&id) {
        terminal.sender.send(SshMsg::Write(data)).map_err(|e| e.to_string())
    } else {
        Err("SSH终端未找到".into())
    }
}

// 调整SSH终端大小
#[tauri::command]
pub fn resize_ssh_terminal(id: String, cols: u16, rows: u16) -> Result<(), String> {
    let terminals = SSH_TERMINALS.lock();
    if let Some(terminal) = terminals.get(&id) {
        terminal.sender.send(SshMsg::Resize { cols, rows }).map_err(|e| e.to_string())
    } else {
        Err("SSH终端未找到".into())
    }
}

// 关闭SSH终端
#[tauri::command]
pub fn close_ssh_terminal(id: String) -> Result<(), String> {
    let terminals = SSH_TERMINALS.lock();
    if let Some(terminal) = terminals.get(&id) {
        let _ = terminal.sender.send(SshMsg::Close);
        Ok(())
    } else {
        Err("SSH终端未找到".into())
    }
}
