use fdrop_net::{ConnectionInfo, ConnectionManager};
use std::str::FromStr;
use tauri::{AppHandle, Manager, WebviewUrl, WindowEvent};
use tokio::sync::Mutex;
use tracing::info;
use tracing_subscriber::{filter::Directive, EnvFilter};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let envfilter = EnvFilter::builder()
        .with_regex(false)
        .with_env_var("FDROP_LOG")
        .with_default_directive(Directive::from_str("info").unwrap())
        .from_env_lossy();

    tracing_subscriber::fmt()
        .compact()
        .with_env_filter(envfilter)
        .init();
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
            open_link_device_window,
            get_available_connections,
            fdrop_net::commands::enable_networking,
            fdrop_net::commands::send_text_message,
            fdrop_net::commands::link_device_by_name,
            fdrop_net::commands::send_files,
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
                    let connection_manager = tauri::async_runtime::block_on(cm_lock.lock());
                    connection_manager.shutdown().unwrap();
                    info!("shutdown mdns daemon");
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn get_available_connections(handle: AppHandle) -> String {
    let cm_lock = handle.state::<Mutex<ConnectionManager>>();
    let connection_manager = cm_lock.lock().await;
    serde_json::to_string(
        &connection_manager
            .get_connectionss()
            .collect::<Vec<&ConnectionInfo>>(),
    )
    .unwrap()
}

#[tauri::command]
fn open_link_device_window(handle: AppHandle) {
    let main = handle.get_webview_window("main").unwrap();
    tauri::WebviewWindowBuilder::new(
        &handle,
        "link-device",
        WebviewUrl::App("/link-device".into()),
    )
    .title("Link Devices")
    .inner_size(600.0, 450.0)
    .resizable(false)
    .parent(&main)
    .unwrap()
    .build()
    .unwrap();
}
