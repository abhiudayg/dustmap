use crate::rules::RiskLevel;
use crate::trash::find_in_trash;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::ffi::OsStr;
use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind, Write};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub const SCHEMA_VERSION: &str = "1.0";
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const QUARANTINE_EXTENSION: &str = "jsonl.quarantined";
const ACKNOWLEDGED_MARKER: &str = "quarantine.acknowledged";

#[derive(Debug, Error)]
pub enum JournalError {
    #[error("journal I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("journal serialization failed: {0}")]
    Serialize(#[from] serde_json::Error),
    #[error("journal '{path}' is corrupt at line {line}")]
    Corrupt { path: PathBuf, line: u64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Trash,
    ToolCommand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    MovedToTrash,
    RemovedByTool,
    SkippedStale,
    SkippedProtected,
    SkippedInUse,
    Failed,
    TimedOut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemState {
    MovedToTrash,
    NotMoved,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub device: u64,
    pub inode: u64,
    pub modified: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum Event {
    OperationStarted {
        scan_snapshot_id: String,
        rule_ids: Vec<String>,
        risk_ceiling: RiskLevel,
        selected_count: usize,
        estimated_bytes: u64,
    },
    ItemPlanned {
        item_id: String,
        rule_id: String,
        path: Option<PathBuf>,
        device: Option<u64>,
        inode: Option<u64>,
        logical_bytes: u64,
        allocated_bytes: Option<u64>,
        risk: RiskLevel,
        action: Action,
    },
    ItemIntent {
        item_id: String,
        expected_identity: Option<Identity>,
        action: Action,
    },
    ItemExecuting {
        item_id: String,
        child_pid: u32,
        child_start_time: DateTime<Utc>,
    },
    ItemOutcome {
        item_id: String,
        outcome: Outcome,
        trash_location: Option<PathBuf>,
        error_code: Option<String>,
    },
    ItemReconciled {
        item_id: String,
        determined_state: ItemState,
        method: String,
        needs_user_review: bool,
        trash_location: Option<PathBuf>,
    },
    Measurement {
        volume: String,
        used_before: u64,
        used_after: u64,
        method: String,
        caveats: Vec<String>,
    },
    OperationInterrupted {
        reason: String,
    },
    OperationCompleted {
        summary: BTreeMap<String, u64>,
        incomplete: bool,
    },
    JournalRepaired {
        truncated_line_seq: u64,
        note: String,
    },
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    pub schema: String,
    pub seq: u64,
    pub operation_id: String,
    pub timestamp: DateTime<Utc>,
    pub app_version: String,
    #[serde(flatten)]
    pub event: Event,
}

pub struct Journal {
    file: File,
    operation_id: String,
    next_seq: u64,
}

impl Journal {
    pub fn create(dir: &Path, operation_id: &str) -> Result<Self, JournalError> {
        let file = OpenOptions::new()
            .create_new(true)
            .append(true)
            .open(journal_path(dir, operation_id))?;
        Ok(Self {
            file,
            operation_id: operation_id.to_string(),
            next_seq: 1,
        })
    }

    fn open_append(path: &Path, operation_id: String, next_seq: u64) -> Result<Self, JournalError> {
        let file = OpenOptions::new().append(true).open(path)?;
        Ok(Self {
            file,
            operation_id,
            next_seq,
        })
    }

    pub fn record(&mut self, event: Event) -> Result<(), JournalError> {
        let record = Record {
            schema: SCHEMA_VERSION.to_string(),
            seq: self.next_seq,
            operation_id: self.operation_id.clone(),
            timestamp: Utc::now(),
            app_version: APP_VERSION.to_string(),
            event,
        };
        let mut line = serde_json::to_vec(&record)?;
        line.push(b'\n');
        self.file.write_all(&line)?;
        self.file.sync_all()?;
        self.next_seq += 1;
        Ok(())
    }
}

fn journal_path(dir: &Path, operation_id: &str) -> PathBuf {
    dir.join(format!("{operation_id}.jsonl"))
}

#[derive(Debug)]
pub struct Replay {
    pub records: Vec<Record>,
    pub valid_len: u64,
    pub torn_tail: bool,
}

impl Replay {
    fn next_seq(&self) -> u64 {
        self.records.last().map_or(1, |record| record.seq + 1)
    }
}

pub fn replay(path: &Path) -> Result<Replay, JournalError> {
    let bytes = fs::read(path)?;
    let segments: Vec<&[u8]> = bytes.split_inclusive(|byte| *byte == b'\n').collect();
    let last = segments.len().saturating_sub(1);
    let mut replay = Replay {
        records: Vec::new(),
        valid_len: 0,
        torn_tail: false,
    };
    for (index, segment) in segments.iter().enumerate() {
        let complete = segment.ends_with(b"\n");
        let body = if complete {
            &segment[..segment.len() - 1]
        } else {
            segment
        };
        match serde_json::from_slice::<Record>(body) {
            Ok(record) if complete => {
                replay.records.push(record);
                replay.valid_len += segment.len() as u64;
            }
            _ if index == last => replay.torn_tail = true,
            _ => {
                return Err(JournalError::Corrupt {
                    path: path.to_path_buf(),
                    line: index as u64 + 1,
                })
            }
        }
    }
    Ok(replay)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconciledItem {
    pub operation_id: String,
    pub item_id: String,
    pub state: ItemState,
    pub needs_user_review: bool,
    pub trash_location: Option<PathBuf>,
}

#[derive(Debug, Default)]
pub struct Reconciliation {
    pub items: Vec<ReconciledItem>,
    pub quarantined: Vec<PathBuf>,
}

pub fn reconcile(dir: &Path, home: &Path) -> Result<Reconciliation, JournalError> {
    let mut report = Reconciliation::default();
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(report),
        Err(err) => return Err(err.into()),
    };
    let mut paths: Vec<PathBuf> = entries
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<_, _>>()?;
    paths.retain(|path| path.extension() == Some(OsStr::new("jsonl")));
    paths.sort();

    for path in paths {
        match reconcile_file(&path, home) {
            Ok(items) => report.items.extend(items),
            Err(JournalError::Corrupt { .. }) => report.quarantined.push(quarantine(&path)?),
            Err(err) => return Err(err),
        }
    }
    Ok(report)
}

fn reconcile_file(path: &Path, home: &Path) -> Result<Vec<ReconciledItem>, JournalError> {
    let replay = replay(path)?;
    let operation_id = path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    if replay.torn_tail {
        OpenOptions::new()
            .write(true)
            .open(path)?
            .set_len(replay.valid_len)?;
    }
    let mut journal = Journal::open_append(path, operation_id.clone(), replay.next_seq())?;
    if replay.torn_tail {
        journal.record(Event::JournalRepaired {
            truncated_line_seq: replay.next_seq(),
            note: "partial final line ignored".to_string(),
        })?;
    }

    let mut items = Vec::new();
    for pending in pending_items(&replay.records) {
        let (state, trash_location) = determine_state(&pending, home);
        let needs_user_review = state == ItemState::Unknown;
        journal.record(Event::ItemReconciled {
            item_id: pending.item_id.clone(),
            determined_state: state,
            method: "filesystem_check".to_string(),
            needs_user_review,
            trash_location: trash_location.clone(),
        })?;
        items.push(ReconciledItem {
            operation_id: operation_id.clone(),
            item_id: pending.item_id,
            state,
            needs_user_review,
            trash_location,
        });
    }
    Ok(items)
}

struct PendingItem {
    item_id: String,
    path: Option<PathBuf>,
    identity: Option<Identity>,
    action: Action,
}

fn pending_items(records: &[Record]) -> Vec<PendingItem> {
    let mut paths: HashMap<&str, &PathBuf> = HashMap::new();
    let mut intents: Vec<PendingItem> = Vec::new();
    let mut settled: HashSet<&str> = HashSet::new();
    for record in records {
        match &record.event {
            Event::ItemPlanned {
                item_id,
                path: Some(path),
                ..
            } => {
                paths.insert(item_id, path);
            }
            Event::ItemIntent {
                item_id,
                expected_identity,
                action,
            } => intents.push(PendingItem {
                item_id: item_id.clone(),
                path: None,
                identity: expected_identity.clone(),
                action: *action,
            }),
            Event::ItemOutcome { item_id, .. } | Event::ItemReconciled { item_id, .. } => {
                settled.insert(item_id);
            }
            _ => {}
        }
    }
    intents
        .into_iter()
        .filter(|intent| !settled.contains(intent.item_id.as_str()))
        .map(|intent| PendingItem {
            path: paths
                .get(intent.item_id.as_str())
                .map(|path| (*path).clone()),
            ..intent
        })
        .collect()
}

fn determine_state(pending: &PendingItem, home: &Path) -> (ItemState, Option<PathBuf>) {
    let (Some(path), Some(identity), Action::Trash) =
        (&pending.path, &pending.identity, pending.action)
    else {
        return (ItemState::Unknown, None);
    };
    if let Ok(meta) = fs::symlink_metadata(path) {
        if meta.dev() == identity.device && meta.ino() == identity.inode {
            return (ItemState::NotMoved, None);
        }
    }
    match find_in_trash(home, identity.device, identity.inode) {
        Some(location) => (ItemState::MovedToTrash, Some(location)),
        None => (ItemState::Unknown, None),
    }
}

fn quarantine(path: &Path) -> Result<PathBuf, JournalError> {
    let target = path.with_extension(QUARANTINE_EXTENSION);
    fs::rename(path, &target)?;
    Ok(target)
}

pub fn quarantined_journals(dir: &Path) -> Result<Vec<PathBuf>, JournalError> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => return Err(err.into()),
    };
    let mut paths: Vec<PathBuf> = entries
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<_, _>>()?;
    paths.retain(|path| path.to_string_lossy().ends_with(QUARANTINE_EXTENSION));
    paths.sort();
    Ok(paths)
}

pub fn unacknowledged_quarantine(dir: &Path) -> Result<Vec<PathBuf>, JournalError> {
    if dir.join(ACKNOWLEDGED_MARKER).exists() {
        return Ok(Vec::new());
    }
    quarantined_journals(dir)
}

pub fn acknowledge_quarantine(dir: &Path) -> Result<(), JournalError> {
    fs::create_dir_all(dir)?;
    fs::write(dir.join(ACKNOWLEDGED_MARKER), b"acknowledged\n")?;
    Ok(())
}
