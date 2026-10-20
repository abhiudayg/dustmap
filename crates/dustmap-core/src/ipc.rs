/// Shared IPC types — serialized to/from frontend.
/// These types are stable; any change requires a version bump.
use serde::{Deserialize, Serialize};

use crate::index::{Coverage, EntryKind};
pub use crate::volumes::VolumeInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcEntry {
    pub id: u32,
    pub path: String,
    pub name: String,
    pub kind: EntryKind,
    pub logical_bytes: u64,
    pub allocated_bytes: Option<u64>,
    pub is_hidden: bool,
    pub modified_secs: i64,
    pub coverage: Coverage,
    /// Only present for directories
    pub child_count: Option<usize>,
    /// Only present for directories (aggregated)
    pub total_logical: Option<u64>,
    pub total_allocated: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcScanProgress {
    pub scan_id: String,
    pub entries_seen: u64,
    pub bytes_accounted: u64,
    pub errors: u64,
    pub elapsed_secs: f64,
    pub current_path: String,
    pub is_complete: bool,
    pub was_cancelled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcScanSummary {
    pub scan_id: String,
    pub root: String,
    pub total_logical: u64,
    pub total_allocated: u64,
    pub total_items: u64,
    pub error_count: u64,
    pub duration_secs: f64,
    pub is_partial: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcChildrenResponse {
    pub parent_id: u32,
    pub entries: Vec<IpcEntry>,
    pub total_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcDeleteRequest {
    pub path: String,
    pub permanent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcDeleteResult {
    pub path: String,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcError {
    pub code: String,
    pub message: String,
}
