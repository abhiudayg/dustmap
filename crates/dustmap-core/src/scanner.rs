use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use walkdir::WalkDir;

use crate::index::{Coverage, Entry, EntryId, EntryKind, Index, ROOT_ID};
use crate::metadata::read_metadata;

#[derive(Debug, Clone)]
pub struct ScanProgress {
    pub entries_seen: u64,
    pub bytes_accounted: u64,
    pub errors: u64,
    pub elapsed_secs: f64,
    pub current_path: String,
}

#[derive(Debug, Clone)]
pub struct ScanError {
    pub path: String,
    pub reason: String,
}

#[derive(Debug, Clone, Default)]
pub struct ScanOptions {
    pub follow_symlinks: bool,
    pub cross_filesystems: bool,
    pub exclusions: Vec<PathBuf>,
}

pub struct ScanResult {
    pub index: Index,
    pub root: PathBuf,
    pub errors: Vec<ScanError>,
    pub cancelled: bool,
    pub duration: Duration,
}

pub fn scan(
    root: &Path,
    options: ScanOptions,
    cancel: Arc<AtomicBool>,
    progress_tx: std::sync::mpsc::Sender<ScanProgress>,
) -> Result<ScanResult> {
    let start = Instant::now();
    let mut index = Index::new();
    let mut errors: Vec<ScanError> = Vec::new();

    // Insert root entry
    let root_meta = read_metadata(
        root,
        root.file_name().and_then(|n| n.to_str()).unwrap_or("/"),
    )
    .unwrap_or(crate::metadata::FileMeta {
        logical_bytes: 0,
        allocated_bytes: Some(0),
        device: 0,
        inode: 0,
        link_count: 1,
        modified_secs: 0,
        is_hidden: false,
    });

    let root_entry = Entry {
        id: ROOT_ID,
        parent: ROOT_ID,
        name: root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("/")
            .to_string(),
        kind: EntryKind::Directory,
        logical_bytes: root_meta.logical_bytes,
        allocated_bytes: root_meta.allocated_bytes,
        device: root_meta.device,
        inode: root_meta.inode,
        link_count: root_meta.link_count,
        modified_secs: root_meta.modified_secs,
        is_hidden: false,
        is_sparse: false,
        is_symlink_target: None,
        permission_denied: false,
        coverage: Coverage::Complete,
    };
    index.add_entry(root_entry);

    // Path → EntryId mapping for parent lookup
    let path_to_id: Arc<Mutex<std::collections::HashMap<PathBuf, EntryId>>> =
        Arc::new(Mutex::new(std::collections::HashMap::new()));
    {
        let mut map = path_to_id.lock().unwrap();
        map.insert(root.to_path_buf(), ROOT_ID);
    }

    let entries_seen = Arc::new(AtomicU64::new(0));
    let bytes_accounted = Arc::new(AtomicU64::new(0));

    let mut exclusions = options.exclusions.clone();
    if root == Path::new("/") {
        // Exclude APFS volume group helper mounts to avoid scanning duplicate trees (Data firmlink),
        // external mounted volumes, and special device nodes.
        exclusions.push(PathBuf::from("/System/Volumes"));
        exclusions.push(PathBuf::from("/Volumes"));
        exclusions.push(PathBuf::from("/dev"));
        exclusions.push(PathBuf::from("/.fseventsd"));
    }

    let exclusions_clone = exclusions.clone();
    let walker = WalkDir::new(root)
        .follow_links(options.follow_symlinks)
        .same_file_system(!options.cross_filesystems)
        .into_iter()
        .filter_entry(move |entry| {
            let p = entry.path();
            if p == root {
                return true;
            }
            !exclusions_clone.iter().any(|ex| p.starts_with(ex))
        });

    let mut last_progress = Instant::now();

    for result in walker {
        if cancel.load(Ordering::Relaxed) {
            index.compute_aggregates();
            return Ok(ScanResult {
                index,
                root: root.to_path_buf(),
                errors,
                cancelled: true,
                duration: start.elapsed(),
            });
        }

        match result {
            Err(e) => {
                let path = e
                    .path()
                    .map(|p| p.display().to_string())
                    .unwrap_or_default();
                errors.push(ScanError {
                    path,
                    reason: e.to_string(),
                });
            }
            Ok(dir_entry) => {
                let path = dir_entry.path().to_path_buf();

                // Skip root (already inserted)
                if path == root {
                    continue;
                }

                // Check exclusions
                let excluded = options.exclusions.iter().any(|ex| path.starts_with(ex));
                if excluded {
                    continue;
                }

                let file_name = dir_entry.file_name().to_string_lossy().to_string();
                let kind = if dir_entry.file_type().is_dir() {
                    EntryKind::Directory
                } else if dir_entry.file_type().is_symlink() {
                    EntryKind::Symlink
                } else if dir_entry.file_type().is_file() {
                    EntryKind::File
                } else {
                    EntryKind::Other
                };

                let (meta, permission_denied) = match read_metadata(&path, &file_name) {
                    Ok(m) => (m, false),
                    Err(_) => (
                        crate::metadata::FileMeta {
                            logical_bytes: 0,
                            allocated_bytes: Some(0),
                            device: 0,
                            inode: 0,
                            link_count: 1,
                            modified_secs: 0,
                            is_hidden: file_name.starts_with('.'),
                        },
                        true,
                    ),
                };

                let parent_path = path.parent().unwrap_or(root).to_path_buf();
                let parent_id = {
                    let map = path_to_id.lock().unwrap();
                    *map.get(&parent_path).unwrap_or(&ROOT_ID)
                };

                let entry_id = EntryId(index.len() as u32);
                let symlink_target = if dir_entry.file_type().is_symlink() {
                    std::fs::read_link(&path)
                        .ok()
                        .map(|t| t.display().to_string())
                } else {
                    None
                };
                let bytes = meta.logical_bytes;
                let entry = Entry {
                    id: entry_id,
                    parent: parent_id,
                    name: file_name,
                    kind,
                    logical_bytes: meta.logical_bytes,
                    allocated_bytes: meta.allocated_bytes,
                    device: meta.device,
                    inode: meta.inode,
                    link_count: meta.link_count,
                    modified_secs: meta.modified_secs,
                    is_hidden: meta.is_hidden,
                    is_sparse: false,
                    is_symlink_target: symlink_target,
                    permission_denied,
                    coverage: if permission_denied {
                        Coverage::Partial
                    } else {
                        Coverage::Complete
                    },
                };

                index.add_entry(entry);

                {
                    let mut map = path_to_id.lock().unwrap();
                    map.insert(path.clone(), entry_id);
                }

                entries_seen.fetch_add(1, Ordering::Relaxed);
                bytes_accounted.fetch_add(bytes, Ordering::Relaxed);

                // Progress every 200ms
                if last_progress.elapsed() > Duration::from_millis(200) {
                    let _ = progress_tx.send(ScanProgress {
                        entries_seen: entries_seen.load(Ordering::Relaxed),
                        bytes_accounted: bytes_accounted.load(Ordering::Relaxed),
                        errors: errors.len() as u64,
                        elapsed_secs: start.elapsed().as_secs_f64(),
                        current_path: path.display().to_string(),
                    });
                    last_progress = Instant::now();
                }
            }
        }
    }

    index.compute_aggregates();

    Ok(ScanResult {
        index,
        root: root.to_path_buf(),
        errors,
        cancelled: false,
        duration: start.elapsed(),
    })
}
