use dustmap_core::{
    index::{EntryId, ROOT_ID},
    ipc::{IpcChildrenResponse, IpcDeleteResult, IpcEntry, IpcScanSummary, VolumeInfo},
    safety::validate_delete_target,
    scanner::{scan, ScanOptions, ScanResult},
    trash::{delete_permanently, move_to_trash},
    volumes::list_volumes,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{mpsc, Arc, Mutex};
use tauri::State;

pub struct ScanStore {
    pub scans: HashMap<String, ScanResult>,
    pub cancel_flags: HashMap<String, Arc<AtomicBool>>,
}

pub type SharedScanStore = Arc<Mutex<ScanStore>>;

fn entry_full_path(result: &ScanResult, entry_id: EntryId) -> PathBuf {
    if entry_id == ROOT_ID {
        return result.root.clone();
    }
    let mut segments = Vec::new();
    let mut curr = entry_id;
    while curr != ROOT_ID {
        if let Some(entry) = result.index.entry(curr) {
            segments.push(entry.name.clone());
            curr = entry.parent;
        } else {
            break;
        }
    }
    segments.reverse();
    let mut p = result.root.clone();
    for seg in segments {
        p.push(seg);
    }
    p
}

fn entry_to_ipc(result: &dustmap_core::scanner::ScanResult, entry_id: EntryId) -> Option<IpcEntry> {
    let entry = result.index.entry(entry_id)?;
    let agg = result.index.aggregate(entry_id);
    let child_count = if matches!(entry.kind, dustmap_core::index::EntryKind::Directory) {
        Some(result.index.children(entry_id).len())
    } else {
        None
    };
    let full_path = entry_full_path(result, entry_id);
    Some(IpcEntry {
        id: entry_id.0,
        path: full_path.display().to_string(),
        name: entry.name.clone(),
        kind: entry.kind,
        logical_bytes: entry.logical_bytes,
        allocated_bytes: entry.allocated_bytes,
        is_hidden: entry.is_hidden,
        modified_secs: entry.modified_secs,
        coverage: entry.coverage,
        child_count,
        total_logical: agg.map(|a| a.logical_total),
        total_allocated: agg.map(|a| a.allocated_total),
    })
}

#[tauri::command]
pub async fn list_volumes_cmd() -> Result<Vec<VolumeInfo>, String> {
    list_volumes().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn start_scan(path: String, state: State<'_, SharedScanStore>) -> Result<String, String> {
    let scan_id = uuid::Uuid::new_v4().to_string();
    let cancel = Arc::new(AtomicBool::new(false));

    let root = PathBuf::from(&path);
    let options = ScanOptions::default();
    let cancel_clone = cancel.clone();

    let (tx, rx) = mpsc::channel();

    // Run scan in blocking thread
    let result = tokio::task::spawn_blocking(move || scan(&root, options, cancel_clone, tx))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())?;

    // Drain any remaining progress messages
    while rx.try_recv().is_ok() {}

    let mut store = state.lock().map_err(|e| e.to_string())?;
    store.scans.insert(scan_id.clone(), result);
    store.cancel_flags.insert(scan_id.clone(), cancel);

    Ok(scan_id)
}

#[tauri::command]
pub async fn scan_summary(
    scan_id: String,
    state: State<'_, SharedScanStore>,
) -> Result<IpcScanSummary, String> {
    let store = state.lock().map_err(|e| e.to_string())?;
    let result = store.scans.get(&scan_id).ok_or("scan not found")?;
    let agg = result.index.aggregate(ROOT_ID).cloned().unwrap_or_default();
    Ok(IpcScanSummary {
        scan_id: scan_id.clone(),
        root: result.root.display().to_string(),
        total_logical: agg.logical_total,
        total_allocated: agg.allocated_total,
        total_items: agg.item_count,
        error_count: result.errors.len() as u64,
        duration_secs: result.duration.as_secs_f64(),
        is_partial: result.cancelled,
    })
}

#[tauri::command]
pub async fn list_children(
    scan_id: String,
    entry_id: u32,
    state: State<'_, SharedScanStore>,
) -> Result<IpcChildrenResponse, String> {
    let store = state.lock().map_err(|e| e.to_string())?;
    let result = store.scans.get(&scan_id).ok_or("scan not found")?;
    let parent = EntryId(entry_id);
    let sorted = result.index.children_sorted_by_size(parent);
    let total_count = sorted.len();
    let entries = sorted
        .into_iter()
        .take(500) // page size
        .filter_map(|id| entry_to_ipc(result, id))
        .collect();
    Ok(IpcChildrenResponse {
        parent_id: entry_id,
        entries,
        total_count,
    })
}

#[tauri::command]
pub async fn reveal_in_finder(path: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("-R")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        let p = std::path::PathBuf::from(&path);
        let dir = if p.is_dir() {
            p
        } else {
            p.parent().unwrap_or(&p).to_path_buf()
        };
        open::that(dir).map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[tauri::command]
pub async fn list_cache_entries(
    scan_id: String,
    state: State<'_, SharedScanStore>,
) -> Result<Vec<IpcEntry>, String> {
    let store = state.lock().map_err(|e| e.to_string())?;
    let result = store.scans.get(&scan_id).ok_or("scan not found")?;
    let cache_ids = result.index.find_cache_entries();
    let entries = cache_ids
        .into_iter()
        .take(100)
        .filter_map(|id| entry_to_ipc(result, id))
        .collect();
    Ok(entries)
}

#[tauri::command]
pub async fn delete_path(
    path: String,
    permanent: bool,
    scan_id: Option<String>,
    entry_id: Option<u32>,
    parent_id: Option<u32>,
    state: State<'_, SharedScanStore>,
) -> Result<IpcDeleteResult, String> {
    let p = PathBuf::from(&path);

    // Validate path against safety rules (no deleting /System, root, SIP, or outside boundaries)
    if let Err(safety_err) = validate_delete_target(&p, None) {
        return Ok(IpcDeleteResult {
            path,
            success: false,
            error: Some(format!("Safety Protection: {}", safety_err)),
        });
    }

    let result = if permanent {
        delete_permanently(&p)
    } else {
        move_to_trash(&p)
    };
    match result {
        Ok(()) => {
            // Remove entry from parent's children in memory if active scan
            if let (Some(sid), Some(eid), Some(pid)) = (scan_id, entry_id, parent_id) {
                if let Ok(mut store) = state.lock() {
                    if let Some(scan_res) = store.scans.get_mut(&sid) {
                        scan_res.index.remove_child(EntryId(pid), EntryId(eid));
                    }
                }
            }
            Ok(IpcDeleteResult {
                path,
                success: true,
                error: None,
            })
        }
        Err(e) => Ok(IpcDeleteResult {
            path,
            success: false,
            error: Some(e.to_string()),
        }),
    }
}
