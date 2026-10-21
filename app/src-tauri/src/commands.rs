use chrono::Utc;
use dustmap_core::{
    audit::{AuditAction, AuditEntry, AuditLog, AuditTarget},
    cleaner::{self, ItemOutcome},
    data::{self, APP_SUPPORT_RELATIVE},
    index::{EntryId, ROOT_ID},
    ipc::{IpcChildrenResponse, IpcDeleteResult, IpcEntry, IpcScanSummary, VolumeInfo},
    rules::RiskLevel,
    scanner::{scan, ScanOptions, ScanResult},
    trash::{delete_permanently, move_to_trash},
    volumes::list_volumes,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::{mpsc, Arc, Mutex};
use tauri::{Emitter, State};

pub struct ScanStore {
    pub scans: HashMap<String, ScanResult>,
    pub cancel_flags: HashMap<String, Arc<AtomicBool>>,
    pub active_cancel: Option<Arc<AtomicBool>>,
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
        is_sparse: entry.is_sparse,
        modified_secs: entry.modified_secs,
        coverage: entry.coverage,
        child_count,
        total_logical: agg.map(|a| a.logical_total),
        total_allocated: agg.map(|a| a.allocated_total),
    })
}

pub fn app_support_dir() -> Result<PathBuf, String> {
    std::env::var_os("HOME")
        .map(|home| PathBuf::from(home).join(APP_SUPPORT_RELATIVE))
        .ok_or_else(|| "HOME is not set".to_string())
}

#[tauri::command]
pub async fn list_volumes_cmd() -> Result<Vec<VolumeInfo>, String> {
    list_volumes().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn start_scan(
    app: tauri::AppHandle,
    path: String,
    state: State<'_, SharedScanStore>,
) -> Result<String, String> {
    let scan_id = uuid::Uuid::new_v4().to_string();
    let cancel = Arc::new(AtomicBool::new(false));

    let root = PathBuf::from(&path);
    let options = ScanOptions::default();
    let (tx, rx) = mpsc::channel();
    let state_dir = app_support_dir()?;
    data::mark_scan_started(&state_dir, &root).map_err(|e| e.to_string())?;
    state.lock().map_err(|e| e.to_string())?.active_cancel = Some(cancel.clone());

    std::thread::spawn(move || {
        for progress in rx {
            let _ = app.emit("scan-progress", &progress);
        }
    });

    let scan_cancel = cancel.clone();
    let outcome = tokio::task::spawn_blocking(move || scan(&root, options, scan_cancel, tx))
        .await
        .map_err(|e| e.to_string())
        .and_then(|result| result.map_err(|e| e.to_string()));
    state.lock().map_err(|e| e.to_string())?.active_cancel = None;
    let result = outcome?;
    let cache_dir = state_dir.join("scan-cache");
    if let Err(err) =
        dustmap_core::scan_cache::save(&cache_dir, &result, chrono::Utc::now().timestamp())
    {
        eprintln!("scan cache was not saved: {err:#}");
    }
    data::mark_scan_finished(&state_dir).map_err(|e| e.to_string())?;

    let mut store = state.lock().map_err(|e| e.to_string())?;
    store.scans.insert(scan_id.clone(), result);
    store.cancel_flags.insert(scan_id.clone(), cancel);
    Ok(scan_id)
}

#[tauri::command]
pub fn cancel_scan(state: State<'_, SharedScanStore>) -> Result<(), String> {
    let store = state.lock().map_err(|e| e.to_string())?;
    if let Some(flag) = &store.active_cancel {
        flag.store(true, std::sync::atomic::Ordering::Relaxed);
    }
    Ok(())
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
    state: State<'_, SharedScanStore>,
) -> Result<IpcDeleteResult, String> {
    let home = home_dir()?;
    let result = delete_one(&path, permanent, scan_id.as_deref(), &home)?;
    if let (true, Some(sid), Some(eid)) = (result.success, scan_id.as_deref(), entry_id) {
        forget_entry(&state, sid, eid);
    }
    Ok(result)
}

#[derive(serde::Serialize)]
pub struct IpcClearResult {
    pub removed: usize,
    pub failed: Vec<String>,
}

#[tauri::command]
pub async fn clear_folder_contents(
    path: String,
    permanent: bool,
    scan_id: Option<String>,
    entry_id: Option<u32>,
    state: State<'_, SharedScanStore>,
) -> Result<IpcClearResult, String> {
    let home = home_dir()?;
    let children = std::fs::read_dir(&path).map_err(|e| format!("Cannot read {path}: {e}"))?;
    let mut removed = 0;
    let mut failed = Vec::new();
    for child in children.flatten() {
        let child_path = child.path().display().to_string();
        match delete_one(&child_path, permanent, scan_id.as_deref(), &home) {
            Ok(result) if result.success => removed += 1,
            Ok(result) => failed.push(format!(
                "{child_path}: {}",
                result.error.unwrap_or_default()
            )),
            Err(err) => failed.push(format!("{child_path}: {err}")),
        }
    }
    if let (Some(sid), Some(eid)) = (scan_id.as_deref(), entry_id) {
        prune_missing_children(&state, sid, eid, Path::new(&path));
    }
    Ok(IpcClearResult { removed, failed })
}

fn home_dir() -> Result<PathBuf, String> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| "HOME is not set".to_string())
}

fn delete_one(
    path: &str,
    permanent: bool,
    scan_id: Option<&str>,
    home: &Path,
) -> Result<IpcDeleteResult, String> {
    let candidate =
        cleaner::candidate_for(&PathBuf::from(path), "manual-delete", RiskLevel::Review)
            .map_err(|e| format!("Cannot read {path}: {e}"))?;
    let logical_bytes = candidate.logical_bytes;
    let preview = cleaner::preview(
        scan_id.unwrap_or("manual"),
        vec![candidate],
        Vec::new(),
        home,
    );
    if let Some(rejection) = preview.rejected.first() {
        return Ok(IpcDeleteResult {
            path: path.to_string(),
            success: false,
            error: Some(format!(
                "Safety Protection: {} is protected",
                rejection.label
            )),
        });
    }

    let support = home.join(APP_SUPPORT_RELATIVE);
    let audit = AuditLog::new(support.join("audit.jsonl"));
    if permanent {
        return permanently_delete(path.to_string(), logical_bytes, &audit);
    }
    let operation_id = uuid::Uuid::new_v4().to_string();
    let results = cleaner::execute(
        &operation_id,
        &preview,
        &support.join("journal"),
        &audit,
        home,
        move_to_trash,
    )
    .map_err(|e| e.to_string())?;
    let error = match results.first().map(|result| result.outcome.clone()) {
        Some(ItemOutcome::MovedToTrash) => None,
        Some(other) => Some(describe_failure(&other)),
        None => Some("Nothing was deleted".to_string()),
    };
    Ok(IpcDeleteResult {
        path: path.to_string(),
        success: error.is_none(),
        error,
    })
}

fn forget_entry(state: &State<'_, SharedScanStore>, scan_id: &str, entry_id: u32) {
    if let Ok(mut store) = state.lock() {
        if let Some(scan) = store.scans.get_mut(scan_id) {
            let id = EntryId(entry_id);
            if let Some(parent) = scan.index.entry(id).map(|entry| entry.parent) {
                scan.index.remove_child(parent, id);
            }
        }
    }
}

fn prune_missing_children(
    state: &State<'_, SharedScanStore>,
    scan_id: &str,
    entry_id: u32,
    folder: &Path,
) {
    if let Ok(mut store) = state.lock() {
        if let Some(scan) = store.scans.get_mut(scan_id) {
            let parent = EntryId(entry_id);
            let gone: Vec<EntryId> = scan
                .index
                .children(parent)
                .iter()
                .copied()
                .filter(|id| {
                    scan.index
                        .entry(*id)
                        .is_some_and(|entry| !folder.join(&entry.name).exists())
                })
                .collect();
            for id in gone {
                scan.index.remove_child(parent, id);
            }
        }
    }
}

fn permanently_delete(
    path: String,
    logical_bytes: u64,
    audit: &AuditLog,
) -> Result<IpcDeleteResult, String> {
    delete_permanently(Path::new(&path)).map_err(|e| e.to_string())?;
    audit
        .append(&AuditEntry {
            timestamp: Utc::now(),
            dustmap_version: env!("CARGO_PKG_VERSION").to_string(),
            targets: vec![AuditTarget {
                path: path.clone(),
                logical_bytes,
                allocated_bytes: None,
                action: AuditAction::DeletedPermanently,
                rule_id: None,
            }],
            bytes_before: None,
            bytes_after: None,
        })
        .map_err(|e| e.to_string())?;
    Ok(IpcDeleteResult {
        path,
        success: true,
        error: None,
    })
}

fn describe_failure(outcome: &ItemOutcome) -> String {
    match outcome {
        ItemOutcome::SkippedStale => {
            "The item changed since it was scanned; rescan and try again.".to_string()
        }
        ItemOutcome::SkippedProtected => "Safety Protection: protected path".to_string(),
        ItemOutcome::Failed { reason } => reason.clone(),
        ItemOutcome::TimedOut => "The operation timed out".to_string(),
        ItemOutcome::MovedToTrash | ItemOutcome::RemovedByTool => "Unexpected outcome".to_string(),
    }
}

#[tauri::command]
pub async fn save_report_file(content: String, filename: String) -> Result<String, String> {
    let home = std::env::var("HOME").map_err(|e| e.to_string())?;
    let downloads = PathBuf::from(home).join("Downloads");
    let target = downloads.join(&filename);
    std::fs::write(&target, content).map_err(|e| format!("Failed to write report file: {}", e))?;

    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg("-R")
            .arg(&target)
            .spawn();
    }

    Ok(target.display().to_string())
}

#[derive(Clone, serde::Serialize)]
pub struct StartupReport {
    pub interrupted_scan: Option<String>,
    pub items_needing_review: Vec<String>,
    pub quarantined_journals: Vec<String>,
    pub errors: Vec<String>,
}

pub fn run_startup_checks() -> StartupReport {
    let mut report = StartupReport {
        interrupted_scan: None,
        items_needing_review: Vec::new(),
        quarantined_journals: Vec::new(),
        errors: Vec::new(),
    };
    let support = match app_support_dir() {
        Ok(support) => support,
        Err(err) => {
            report.errors.push(err);
            return report;
        }
    };
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default();
    match dustmap_core::journal::reconcile(&support.join("journal"), &home) {
        Ok(reconciled) => {
            report.items_needing_review = reconciled
                .items
                .iter()
                .filter(|item| item.needs_user_review)
                .map(|item| format!("{} {}", item.operation_id, item.item_id))
                .collect();
            report.quarantined_journals = reconciled
                .quarantined
                .iter()
                .map(|path| path.display().to_string())
                .collect();
        }
        Err(err) => report
            .errors
            .push(format!("journal reconciliation failed: {err}")),
    }
    match data::interrupted_scan(&support) {
        Ok(root) => report.interrupted_scan = root.map(|path| path.display().to_string()),
        Err(err) => report.errors.push(format!("scan state unreadable: {err}")),
    }
    report
}

#[tauri::command]
pub fn startup_report(report: State<'_, StartupReport>) -> StartupReport {
    report.inner().clone()
}

#[tauri::command]
pub fn dismiss_interrupted_scan() -> Result<(), String> {
    data::mark_scan_finished(&app_support_dir()?).map_err(|e| e.to_string())
}

#[derive(serde::Serialize)]
pub struct CachedScanInfo {
    pub scan_id: String,
    pub scanned_at_secs: i64,
}

#[tauri::command]
pub fn load_cached_scan(
    path: String,
    state: State<'_, SharedScanStore>,
) -> Result<Option<CachedScanInfo>, String> {
    let cache_dir = app_support_dir()?.join("scan-cache");
    let Some(cached) = dustmap_core::scan_cache::load(&cache_dir, &PathBuf::from(&path))
        .map_err(|e| format!("{e:#}"))?
    else {
        return Ok(None);
    };
    let scan_id = uuid::Uuid::new_v4().to_string();
    let mut store = state.lock().map_err(|e| e.to_string())?;
    store.scans.insert(scan_id.clone(), cached.result);
    store
        .cancel_flags
        .insert(scan_id.clone(), Arc::new(AtomicBool::new(false)));
    Ok(Some(CachedScanInfo {
        scan_id,
        scanned_at_secs: cached.scanned_at_secs,
    }))
}
