pub mod commands;
use commands::{ScanStore, SharedScanStore};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let store: SharedScanStore = Arc::new(Mutex::new(ScanStore {
        scans: HashMap::new(),
        cancel_flags: HashMap::new(),
    }));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(store)
        .invoke_handler(tauri::generate_handler![
            commands::list_volumes_cmd,
            commands::start_scan,
            commands::scan_summary,
            commands::list_children,
            commands::delete_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
