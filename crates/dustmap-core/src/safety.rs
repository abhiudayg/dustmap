use std::path::{Component, Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SafetyError {
    #[error("Path traversal detected: '{0}' contains illegal traversal components")]
    PathTraversal(String),
    #[error("Path is inside a protected macOS system directory: '{0}'")]
    SystemProtected(String),
    #[error("Path '{0}' is outside the authorized scan root '{1}'")]
    OutsideRoot(String, String),
    #[error("Symlink target '{0}' points to a protected or out-of-bounds destination")]
    SymlinkEscape(String),
    #[error("Path contains null bytes or illegal characters: '{0}'")]
    InvalidPath(String),
    #[error("Path does not exist: '{0}'")]
    NotFound(String),
}

/// Protected system path prefixes that DustMap must NEVER modify or delete under any circumstances.
const PROTECTED_PREFIXES: &[&str] = &[
    "/System",
    "/usr/bin",
    "/usr/sbin",
    "/usr/lib",
    "/usr/libexec",
    "/bin",
    "/sbin",
    "/etc",
    "/private/etc",
    "/private/var/db",
    "/private/var/root",
    "/var/db",
    "/var/root",
    "/Library/Apple",
    "/Library/Application Support/com.apple",
    "/dev",
    "/proc",
    "/Volumes/Recovery",
];

/// Exact protected root or top-level critical paths.
const PROTECTED_EXACT: &[&str] = &[
    "/",
    "/System",
    "/Applications",
    "/Library",
    "/Users",
    "/Volumes",
    "/cores",
    "/bin",
    "/sbin",
    "/usr",
    "/etc",
    "/var",
    "/private",
    "/private/var",
    "/private/etc",
];

/// Check if a raw path string attempts directory traversal (e.g., `..`, null bytes).
pub fn check_raw_traversal(raw_path: &str) -> Result<(), SafetyError> {
    if raw_path.contains('\0') {
        return Err(SafetyError::InvalidPath("null byte in path".to_string()));
    }

    let p = Path::new(raw_path);
    for component in p.components() {
        if let Component::ParentDir = component {
            return Err(SafetyError::PathTraversal(raw_path.to_string()));
        }
    }
    Ok(())
}

/// Verify if a path falls inside SIP-protected or essential macOS system boundaries.
pub fn is_system_protected(path: &Path) -> bool {
    let raw_str = path.to_string_lossy();
    for exact in PROTECTED_EXACT {
        if raw_str == *exact {
            return true;
        }
    }
    for prefix in PROTECTED_PREFIXES {
        if raw_str.starts_with(prefix) {
            return true;
        }
    }

    if let Ok(canonical) = path.canonicalize() {
        let canonical_str = canonical.to_string_lossy();
        for exact in PROTECTED_EXACT {
            if canonical_str == *exact {
                return true;
            }
        }
        for prefix in PROTECTED_PREFIXES {
            if canonical_str.starts_with(prefix) {
                return true;
            }
        }
    }

    false
}

/// Validate that a path is safe to delete or operate on:
/// 1. No path traversal attacks
/// 2. If target is a symlink, ensures we don't traverse into protected targets
/// 3. Not in SIP/system protected directories
/// 4. If an allowed_root is provided, must be within that root
pub fn validate_delete_target(
    path: &Path,
    allowed_root: Option<&Path>,
) -> Result<PathBuf, SafetyError> {
    let raw_str = path.to_string_lossy();
    check_raw_traversal(&raw_str)?;

    let clean_path = if path.is_relative() {
        std::env::current_dir()
            .map(|cwd| cwd.join(path))
            .unwrap_or_else(|_| path.to_path_buf())
    } else {
        path.to_path_buf()
    };

    // Check if target is a symlink BEFORE canonicalizing the path
    if let Ok(meta) = std::fs::symlink_metadata(&clean_path) {
        if meta.file_type().is_symlink() {
            if let Ok(target) = std::fs::read_link(&clean_path) {
                let resolved_target = if target.is_relative() {
                    clean_path.parent().unwrap_or(&clean_path).join(&target)
                } else {
                    target
                };
                if is_system_protected(&resolved_target) {
                    return Err(SafetyError::SymlinkEscape(clean_path.display().to_string()));
                }
            }
        }
    }

    // System protection check on the path itself
    if is_system_protected(&clean_path) {
        return Err(SafetyError::SystemProtected(
            clean_path.display().to_string(),
        ));
    }

    // Root boundary check
    if let Some(root) = allowed_root {
        let canonical_root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
        let canonical_target = clean_path
            .canonicalize()
            .unwrap_or_else(|_| clean_path.clone());
        if !canonical_target.starts_with(&canonical_root) {
            return Err(SafetyError::OutsideRoot(
                clean_path.display().to_string(),
                root.display().to_string(),
            ));
        }
    }

    Ok(clean_path)
}

const DENYLIST: &[&str] = &[
    "/System",
    "/usr",
    "/bin",
    "/sbin",
    "/Library/Apple",
    "/private/var/db",
    "/private/var/vm",
    "/dev",
    "/Volumes/*/Backups.backupdb",
    "*.photoslibrary",
    "~/Library/Keychains",
    "~/.ssh",
    "~/.gnupg",
    "~/Library/Mail",
    "~/Library/Messages",
    "~/Library/Photos",
    "~/Library/Mobile Documents",
    "~/Library/CloudStorage",
    "~/Library/Containers/com.apple.*",
    "~/Library/Group Containers/*.com.apple.*",
    "~/Library/Application Support/com.apple.TCC",
    "~/Library/Application Support/AddressBook",
    "~/Library/Safari",
    "~/Library/Accounts",
    "~/Documents",
    "~/Desktop",
    "/Applications",
    "~/Applications",
];

const DENYLIST_USR_EXCEPTION: &[&str] = &["usr", "local"];

pub fn is_denylisted(path: &Path) -> bool {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default();
    is_denylisted_in(path, &home)
}

pub fn is_denylisted_in(path: &Path, home: &Path) -> bool {
    let mut candidates = vec![path.to_path_buf()];
    if let Ok(canonical) = path.canonicalize() {
        candidates.push(canonical);
    }
    candidates
        .iter()
        .any(|candidate| denylist_matches(candidate, home))
}

fn denylist_matches(path: &Path, home: &Path) -> bool {
    let path_parts = components(path);
    let under_usr_local = path_parts.starts_with(DENYLIST_USR_EXCEPTION);
    DENYLIST.iter().any(|entry| {
        if *entry == "/usr" && under_usr_local {
            return false;
        }
        if entry.starts_with('*') {
            return path_parts.iter().any(|part| component_matches(entry, part));
        }
        let pattern = match entry.strip_prefix("~/") {
            Some(rest) => {
                let mut parts = components(home);
                parts.extend(rest.split('/'));
                parts
            }
            None => entry.trim_start_matches('/').split('/').collect(),
        };
        has_prefix(&path_parts, &pattern)
    })
}

fn components(path: &Path) -> Vec<&str> {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(part) => part.to_str(),
            _ => None,
        })
        .collect()
}

fn has_prefix(parts: &[&str], pattern: &[&str]) -> bool {
    parts.len() >= pattern.len()
        && pattern
            .iter()
            .zip(parts)
            .all(|(pattern, part)| component_matches(pattern, part))
}

fn component_matches(pattern: &str, part: &str) -> bool {
    let pieces: Vec<&str> = pattern.split('*').collect();
    if pieces.len() == 1 {
        return pattern == part;
    }
    let first = pieces[0];
    let last = pieces[pieces.len() - 1];
    if part.len() < first.len() + last.len() || !part.starts_with(first) || !part.ends_with(last) {
        return false;
    }
    let mut rest = &part[first.len()..part.len() - last.len()];
    for middle in &pieces[1..pieces.len() - 1] {
        match rest.find(middle) {
            Some(index) => rest = &rest[index + middle.len()..],
            None => return false,
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_traversal_detection() {
        assert!(check_raw_traversal("../etc/passwd").is_err());
        assert!(check_raw_traversal("foo/../../bar").is_err());
        assert!(check_raw_traversal("normal/path/to/file").is_ok());
        assert!(check_raw_traversal("path\0with/null").is_err());
    }

    #[test]
    fn test_system_protection() {
        assert!(is_system_protected(Path::new("/System")));
        assert!(is_system_protected(Path::new("/System/Library")));
        assert!(is_system_protected(Path::new("/usr/bin")));
        assert!(is_system_protected(Path::new("/bin/sh")));
        assert!(is_system_protected(Path::new("/etc")));
        assert!(is_system_protected(Path::new("/")));
        assert!(is_system_protected(Path::new("/Applications")));
        assert!(!is_system_protected(Path::new("/Users/john/Downloads")));
    }

    #[test]
    fn test_validate_delete_target_rejects_system() {
        let res = validate_delete_target(Path::new("/System/Library/Caches"), None);
        assert!(matches!(res, Err(SafetyError::SystemProtected(_))));
    }

    #[test]
    fn test_validate_delete_target_outside_root() {
        let root = Path::new("/tmp/test_dustmap_root");
        let target = Path::new("/tmp/other_dir/file.txt");
        let res = validate_delete_target(target, Some(root));
        assert!(matches!(res, Err(SafetyError::OutsideRoot(_, _))));
    }

    #[test]
    fn denylist_blocks_user_documents_and_credentials_under_home() {
        let home = Path::new("/Users/tester");
        assert!(is_denylisted_in(
            Path::new("/Users/tester/Documents/a.txt"),
            home
        ));
        assert!(is_denylisted_in(
            Path::new("/Users/tester/.ssh/id_ed25519"),
            home
        ));
        assert!(is_denylisted_in(
            Path::new("/Users/tester/Library/Keychains/login.keychain-db"),
            home
        ));
        assert!(!is_denylisted_in(
            Path::new("/Users/tester/Downloads/a.dmg"),
            home
        ));
    }

    #[test]
    fn denylist_matches_path_components_not_string_prefixes() {
        let home = Path::new("/Users/tester");
        assert!(!is_denylisted_in(
            Path::new("/Users/tester/dev/project"),
            home
        ));
        assert!(!is_denylisted_in(
            Path::new("/Users/tester/Desktop2/x"),
            home
        ));
        assert!(!is_denylisted_in(
            Path::new("/Users/tester/Documentsbak/x"),
            home
        ));
    }

    #[test]
    fn denylist_allows_usr_local_but_blocks_other_usr_paths() {
        let home = Path::new("/Users/tester");
        assert!(!is_denylisted_in(Path::new("/usr/local/Cellar/x"), home));
        assert!(is_denylisted_in(Path::new("/usr/share/doc"), home));
    }

    #[test]
    fn denylist_supports_wildcards_for_backups_photo_libraries_and_apple_containers() {
        let home = Path::new("/Users/tester");
        assert!(is_denylisted_in(
            Path::new("/Volumes/BackupDisk/Backups.backupdb/mac/2026"),
            home
        ));
        assert!(is_denylisted_in(
            Path::new("/Users/tester/Pictures/Library.photoslibrary/database"),
            home
        ));
        assert!(is_denylisted_in(
            Path::new("/Users/tester/Library/Containers/com.apple.Notes/Data"),
            home
        ));
        assert!(!is_denylisted_in(
            Path::new("/Users/tester/Library/Containers/com.example.app/Data"),
            home
        ));
    }

    #[test]
    fn denylist_blocks_system_paths_at_the_root_only() {
        let home = Path::new("/Users/tester");
        assert!(is_denylisted_in(Path::new("/System/Library/Caches"), home));
        assert!(is_denylisted_in(
            Path::new("/Applications/Safari.app"),
            home
        ));
        assert!(!is_denylisted_in(
            Path::new("/Users/tester/Applications-old/x"),
            home
        ));
    }
}
