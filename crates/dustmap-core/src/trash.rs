use crate::safety::validate_delete_target;
use anyhow::{Context, Result};
use std::path::Path;

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
