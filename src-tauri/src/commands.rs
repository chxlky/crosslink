use crate::structs::Device;

#[tauri::command]
pub async fn discover_devices() -> Vec<Device> {
    Vec::new()
}
