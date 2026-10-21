pub mod commands;
pub mod maintenance;
use commands::{ScanStore, SharedScanStore};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let store: SharedScanStore = Arc::new(Mutex::new(ScanStore {
        scans: HashMap::new(),
        cancel_flags: HashMap::new(),
        active_cancel: None,
    }));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(store)
        .setup(|app| {
            app.manage(commands::run_startup_checks());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_volumes_cmd,
            commands::start_scan,
            commands::scan_summary,
            commands::list_children,
            commands::list_cache_entries,
            commands::reveal_in_finder,
            commands::delete_path,
            commands::clear_folder_contents,
            commands::delete_paths,
            commands::save_report_file,
            commands::startup_report,
            commands::cancel_scan,
            commands::dismiss_interrupted_scan,
            commands::load_cached_scan,
            maintenance::cleanup_preview,
            maintenance::cleanup_execute,
            maintenance::remnant_search,
            maintenance::remnant_remove,
            maintenance::data_list,
            maintenance::data_remove,
            maintenance::acknowledge_quarantine,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
