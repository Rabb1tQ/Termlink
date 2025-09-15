use once_cell::sync::Lazy;
use parking_lot::Mutex;
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::{collections::HashMap, io::Read, thread};
use tauri::Emitter;

enum PtyMsg {
  Write(String),
  Resize { cols: u16, rows: u16 },
}

static PTY_SENDERS: Lazy<Mutex<HashMap<String, crossbeam_channel::Sender<PtyMsg>>>> =
  Lazy::new(|| Mutex::new(HashMap::new()));

#[tauri::command]
pub fn start_pty(
  window: tauri::Window,
  id: String,
  cols: u16,
  rows: u16,
  program: Option<String>,
  args: Option<Vec<String>>,
  cwd: Option<String>,
) -> Result<(), String> {
  let (tx, rx) = crossbeam_channel::unbounded::<PtyMsg>();
  PTY_SENDERS.lock().insert(id.clone(), tx);

  thread::spawn(move || {
    let pty_system = native_pty_system();
    let pair = match pty_system.openpty(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 }) {
      Ok(p) => p,
      Err(e) => {
        let _ = window.emit("pty_error", format!("{}: {}", id, e));
        return;
      }
    };

    let mut cmd = if let Some(p) = program.clone() {
      let mut c = CommandBuilder::new(p);
      if let Some(a) = args.clone() { for s in a { c.arg(s); } }
      c
    } else if cfg!(windows) {
      let mut c = CommandBuilder::new("cmd.exe");
      c.arg("/K");
      c
    } else {
      CommandBuilder::new("/bin/bash")
    };
    if let Some(dir) = cwd { cmd.cwd(dir); }

    let _child = match pair.slave.spawn_command(cmd) {
      Ok(c) => c,
      Err(e) => {
        let _ = window.emit("pty_error", format!("{}: {}", id, e));
        return;
      }
    };

    let mut reader = match pair.master.try_clone_reader() {
      Ok(r) => r,
      Err(e) => {
        let _ = window.emit("pty_error", format!("{}: {}", id, e));
        return;
      }
    };

    let mut writer = match pair.master.take_writer() {
      Ok(w) => w,
      Err(e) => {
        let _ = window.emit("pty_error", format!("{}: {}", id, e));
        return;
      }
    };

    // Reader loop
    let win_clone = window.clone();
    let id_clone = id.clone();
    let reader_thread = thread::spawn(move || {
      let mut buf = [0u8; 4096];
      loop {
        match reader.read(&mut buf) {
          Ok(0) => break,
          Ok(n) => {
            let chunk = String::from_utf8_lossy(&buf[..n]).to_string();
            let _ = win_clone.emit(&format!("pty://{}", id_clone), chunk);
          }
          Err(_) => break,
        }
      }
    });

    // Writer/resize loop
    while let Ok(msg) = rx.recv() {
      match msg {
        PtyMsg::Write(data) => {
          // Write user input as-is; Windows shells handle CR/LF themselves
          let _ = std::io::Write::write_all(&mut writer, data.as_bytes());
        }
        PtyMsg::Resize { cols, rows } => {
          let _ = pair.master.resize(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 });
        }
      }
    }

    let _ = reader_thread.join();
    let _ = window.emit(&format!("pty_exit://{}", id), "");
  });

  Ok(())
}

#[tauri::command]
pub fn write_pty(id: String, data: String) -> Result<(), String> {
  if let Some(tx) = PTY_SENDERS.lock().get(&id) {
    tx.send(PtyMsg::Write(data)).map_err(|e| e.to_string())
  } else {
    Err("PTY not found".into())
  }
}

#[tauri::command]
pub fn resize_pty(id: String, cols: u16, rows: u16) -> Result<(), String> {
  if let Some(tx) = PTY_SENDERS.lock().get(&id) {
    tx.send(PtyMsg::Resize { cols, rows }).map_err(|e| e.to_string())
  } else {
    Err("PTY not found".into())
  }
}

#[tauri::command]
pub fn close_pty(id: String) -> Result<(), String> {
  PTY_SENDERS.lock().remove(&id);
  Ok(())
}