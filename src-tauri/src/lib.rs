mod terminal;
mod ssh;
mod sftp_russh;
mod fs;
mod ssh_terminal_russh;
mod system_monitor;
mod download_manager;
mod ssh_command;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      // Terminal commands
      terminal::start_pty,
      terminal::write_pty,
      terminal::resize_pty,
      terminal::close_pty,
      
      // SSH Terminal commands
      ssh_terminal_russh::start_ssh_terminal,
      ssh_terminal_russh::write_ssh_terminal,
      ssh_terminal_russh::resize_ssh_terminal,
      ssh_terminal_russh::close_ssh_terminal,
      
      // SSH profile commands
      ssh::save_ssh_profile,
      ssh::list_ssh_profiles,
      ssh::get_ssh_password,
      ssh::restart_ssh_connection,
      ssh::delete_ssh_profile,
      ssh::get_profiles_dir,
      
      // Local filesystem commands
      fs::list_files,
      fs::get_home_dir,
      fs::get_parent_dir,
      fs::open_file_explorer,
      
      // SFTP commands
      sftp_russh::connect_sftp,
      sftp_russh::create_sftp_from_ssh,
      sftp_russh::disconnect_sftp,
      sftp_russh::list_sftp_files,
      sftp_russh::download_sftp_file,
      sftp_russh::upload_sftp_file,
      sftp_russh::read_sftp_file,
      sftp_russh::write_sftp_file,
      sftp_russh::delete_sftp_file,
      sftp_russh::delete_sftp_directory,
      sftp_russh::create_sftp_directory,
      sftp_russh::rename_sftp_file,
      sftp_russh::get_sftp_file_metadata,
      
      // System monitor commands
      system_monitor::get_system_info,
      system_monitor::get_cpu_info,
      system_monitor::get_memory_info,
      system_monitor::get_disk_info,
      system_monitor::get_network_info,
      system_monitor::get_process_info,
      
      // Download manager commands
      download_manager::select_download_location,
      download_manager::get_sftp_file_info,
      download_manager::download_sftp_file_with_progress,
      download_manager::cancel_download,
      download_manager::open_file_location,
      
      // SSH command execution
      ssh_command::execute_ssh_command,
      ssh_command::connect_ssh_for_monitoring,
      ssh_command::disconnect_ssh_monitoring
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}