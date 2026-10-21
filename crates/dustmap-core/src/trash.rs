use crate::safety::validate_delete_target;
use anyhow::{Context, Result};
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

/// Move a path to the macOS Trash after strict safety validation.
pub fn move_to_trash(path: &Path) -> Result<()> {
    let safe_path = validate_delete_target(path, None)
        .with_context(|| format!("Safety policy rejected trash move: {}", path.display()))?;
    trash::delete(&safe_path)
        .with_context(|| format!("Failed to move to trash: {}", safe_path.display()))
}

/// Permanently delete a path (file or directory) after strict safety validation.
pub fn delete_permanently(path: &Path) -> Result<()> {
    let safe_path = validate_delete_target(path, None).with_context(|| {
        format!(
            "Safety policy rejected permanent deletion: {}",
            path.display()
        )
    })?;
    let meta = std::fs::symlink_metadata(&safe_path)
        .with_context(|| format!("Cannot stat: {}", safe_path.display()))?;
    if meta.is_dir() {
        std::fs::remove_dir_all(&safe_path)
            .with_context(|| format!("Failed to remove directory: {}", safe_path.display()))
    } else {
        std::fs::remove_file(&safe_path)
            .with_context(|| format!("Failed to remove file: {}", safe_path.display()))
    }
}

pub fn find_in_trash(home: &Path, device: u64, inode: u64) -> Option<PathBuf> {
    trash_roots(home).into_iter().find_map(|root| {
        fs::read_dir(&root)
            .ok()?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .find(|path| {
                fs::symlink_metadata(path)
                    .map(|meta| meta.dev() == device && meta.ino() == inode)
                    .unwrap_or(false)
            })
    })
}

fn trash_roots(home: &Path) -> Vec<PathBuf> {
    let mut roots = vec![home.join(".Trash")];
    let Ok(volumes) = fs::read_dir("/Volumes") else {
        return roots;
    };
    for volume in volumes.filter_map(Result::ok) {
        let trashes = volume.path().join(".Trashes");
        if let Ok(uid_dirs) = fs::read_dir(&trashes) {
            roots.extend(uid_dirs.filter_map(Result::ok).map(|dir| dir.path()));
        }
    }
    roots
}
