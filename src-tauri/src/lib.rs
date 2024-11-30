use tracing::info;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt().compact().init();
    info!("logging enabled");
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            fdrop_config::commands::get_device_details,
            fdrop_config::commands::check_first_launch,
            fdrop_config::commands::initial_setup,
            fdrop_config::commands::generate_keys,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
