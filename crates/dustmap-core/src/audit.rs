use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditAction {
    MovedToTrash,
    DeletedPermanently,
    Skipped,
    Stale,
    Failed { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTarget {
    pub path: String,
    pub logical_bytes: u64,
    pub allocated_bytes: Option<u64>,
    pub action: AuditAction,
    pub rule_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: DateTime<Utc>,
    pub dustmap_version: String,
    pub targets: Vec<AuditTarget>,
    pub bytes_before: Option<u64>,
    pub bytes_after: Option<u64>,
}

pub struct AuditLog {
    path: PathBuf,
}

impl AuditLog {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn append(&self, entry: &AuditEntry) -> Result<()> {
        let line = serde_json::to_string(entry)?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        writeln!(file, "{}", line)?;
        Ok(())
    }
}
