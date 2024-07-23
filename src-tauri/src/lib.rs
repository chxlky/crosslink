use structs::Device;
use tauri::{AppHandle, Manager, Runtime, Window};

mod structs;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            #[cfg(debug_assertions)]
            {
                window.open_devtools();
            }
            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![discover_devices])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn discover_devices<R: Runtime>(
    app: AppHandle<R>,
    window: Window<R>,
) -> Vec<Device> {
    Vec::new()
}
