use crate::scanner::ScanResult;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Serialize)]
struct CachedScanRef<'a> {
    scanned_at_secs: i64,
    result: &'a ScanResult,
}

#[derive(Deserialize)]
pub struct CachedScan {
    pub scanned_at_secs: i64,
    pub result: ScanResult,
}

fn cache_file(cache_dir: &Path, root: &Path) -> PathBuf {
    let readable: String = root
        .to_string_lossy()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    let hash = root
        .as_os_str()
        .as_encoded_bytes()
        .iter()
        .fold(0xcbf29ce484222325u64, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
        });
    cache_dir.join(format!("{readable}-{hash:016x}.json"))
}

pub fn save(cache_dir: &Path, result: &ScanResult, scanned_at_secs: i64) -> Result<()> {
    fs::create_dir_all(cache_dir)
        .with_context(|| format!("failed to create {}", cache_dir.display()))?;
    let path = cache_file(cache_dir, &result.root);
    let temp = path.with_extension("json.tmp");
    let bytes = serde_json::to_vec(&CachedScanRef {
        scanned_at_secs,
        result,
    })?;
    fs::write(&temp, bytes).with_context(|| format!("failed to write {}", temp.display()))?;
    fs::rename(&temp, &path).with_context(|| format!("failed to replace {}", path.display()))?;
    Ok(())
}

pub fn load(cache_dir: &Path, root: &Path) -> Result<Option<CachedScan>> {
    let path = cache_file(cache_dir, root);
    let bytes = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(err).with_context(|| format!("failed to read {}", path.display())),
    };
    let cached: CachedScan = serde_json::from_slice(&bytes)
        .with_context(|| format!("scan cache {} is unreadable", path.display()))?;
    Ok(Some(cached))
}
