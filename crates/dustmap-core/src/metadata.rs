use anyhow::Result;
use std::fs;
use std::path::Path;

pub struct FileMeta {
    pub logical_bytes: u64,
    pub allocated_bytes: Option<u64>,
    pub device: u64,
    pub inode: u64,
    pub link_count: u32,
    pub modified_secs: i64,
    pub is_hidden: bool,
}

pub fn read_metadata(path: &Path, file_name: &str) -> Result<FileMeta> {
    let meta = fs::symlink_metadata(path)?;
    let logical_bytes = meta.len();

    #[cfg(unix)]
    let (allocated_bytes, device, inode, link_count, modified_secs) = {
        use std::os::unix::fs::MetadataExt;
        // blocks are 512-byte units on macOS
        let allocated = (meta.blocks() as u64) * 512;
        (
            Some(allocated),
            meta.dev(),
            meta.ino(),
            meta.nlink() as u32,
            meta.mtime(),
        )
    };

    #[cfg(not(unix))]
    let (allocated_bytes, device, inode, link_count, modified_secs) = {
        let modified = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        (None, 0u64, 0u64, 1u32, modified)
    };

    let is_hidden = file_name.starts_with('.');

    Ok(FileMeta {
        logical_bytes,
        allocated_bytes,
        device,
        inode,
        link_count,
        modified_secs,
        is_hidden,
    })
}
