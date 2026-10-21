use crate::trash::delete_permanently;
use anyhow::Result;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub const APP_SUPPORT_RELATIVE: &str = "Library/Application Support/com.dustmap.desktop";

pub const DATA_ENTRIES: &[(&str, &str)] = &[
    (
        "settings",
        "Library/Application Support/com.dustmap.desktop/settings.json",
    ),
    (
        "audit-log",
        "Library/Application Support/com.dustmap.desktop/audit.jsonl",
    ),
    (
        "journal",
        "Library/Application Support/com.dustmap.desktop/journal",
    ),
    (
        "scan-cache",
        "Library/Application Support/com.dustmap.desktop/scan-cache",
    ),
    (
        "storage-history",
        "Library/Application Support/com.dustmap.desktop/storage-history",
    ),
    (
        "rules-cache",
        "Library/Application Support/com.dustmap.desktop/rules-cache",
    ),
    ("logs", "Library/Logs/com.dustmap.desktop"),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataEntry {
    pub label: &'static str,
    pub path: PathBuf,
    pub bytes: u64,
}

pub fn data_entries(home: &Path) -> Vec<DataEntry> {
    DATA_ENTRIES
        .iter()
        .map(|(label, relative)| {
            let path = home.join(relative);
            DataEntry {
                label,
                bytes: size_of(&path),
                path,
            }
        })
        .filter(|entry| entry.path.exists())
        .collect()
}

pub fn remove_data(home: &Path, labels: &[&str]) -> Result<Vec<&'static str>> {
    let mut removed = Vec::new();
    for entry in data_entries(home) {
        if labels.contains(&entry.label) {
            delete_permanently(&entry.path)?;
            removed.push(entry.label);
        }
    }
    Ok(removed)
}

fn size_of(path: &Path) -> u64 {
    WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter_map(|entry| fs::symlink_metadata(entry.path()).ok())
        .filter(|meta| meta.is_file())
        .map(|meta| meta.len())
        .sum()
}

pub fn interrupted_scan_path(state_dir: &Path) -> PathBuf {
    state_dir.join("scan.active")
}

pub fn mark_scan_started(state_dir: &Path, root: &Path) -> Result<()> {
    fs::create_dir_all(state_dir)?;
    fs::write(
        interrupted_scan_path(state_dir),
        root.to_string_lossy().as_bytes(),
    )?;
    Ok(())
}

pub fn mark_scan_finished(state_dir: &Path) -> Result<()> {
    match fs::remove_file(interrupted_scan_path(state_dir)) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err.into()),
    }
}

pub fn interrupted_scan(state_dir: &Path) -> Result<Option<PathBuf>> {
    match fs::read_to_string(interrupted_scan_path(state_dir)) {
        Ok(root) => Ok(Some(PathBuf::from(root))),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err.into()),
    }
}
