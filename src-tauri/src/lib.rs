use fdrop_net::ConnectionManager;
use std::sync::Mutex;
use tauri::{Manager, WindowEvent};
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
            fdrop_config::commands::get_details_from_config,
            fdrop_config::commands::check_first_launch,
            fdrop_config::commands::initial_setup,
            fdrop_config::commands::generate_keys,
            fdrop_net::commands::enable_networking,
        ])
        .setup(|app| {
            let connection_manager = fdrop_net::ConnectionManager::new()?;
            app.manage(connection_manager);

            if !tauri::async_runtime::block_on(fdrop_config::check_first_launch(&app.handle())) {
                let user_config = Mutex::new(fdrop_config::get_details_from_config(&app.handle())?);
                app.manage(user_config);
            }
            let main_window = app.get_webview_window("main").unwrap();
            let main_window2 = main_window.clone();
            main_window.on_window_event(move |event| {
                if matches!(event, WindowEvent::CloseRequested { .. }) {
                    let cm_lock = main_window2.state::<Mutex<ConnectionManager>>();
                    let connection_manager = cm_lock.lock().unwrap();
                    connection_manager.shutdown().unwrap();
                    info!("shutdown mdns daemon");
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
