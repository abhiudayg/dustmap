use crate::audit::{AuditAction, AuditEntry, AuditLog, AuditTarget};
use crate::journal::{self, Action, Event, Identity, Journal, Outcome};
use crate::rules::RiskLevel;
use crate::safety::is_denylisted_in;
use crate::toolcmd;
use crate::trash::find_in_trash;
use anyhow::{bail, Result};
use chrono::{DateTime, Utc};
use std::collections::BTreeMap;
use std::fs;
use std::io::ErrorKind;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

const STARTUP_VOLUME: &str = "/";

#[derive(Debug, Clone)]
pub struct Candidate {
    pub rule_id: String,
    pub risk: RiskLevel,
    pub path: PathBuf,
    pub device: u64,
    pub inode: u64,
    pub logical_bytes: u64,
    pub allocated_bytes: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct ToolTarget {
    pub rule_id: String,
    pub risk: RiskLevel,
    pub description: String,
    pub argv: Vec<String>,
    pub timeout: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RejectReason {
    Denylisted,
    ReportOnly,
}

#[derive(Debug, Clone)]
pub struct Rejection {
    pub label: String,
    pub reason: RejectReason,
}

#[derive(Debug, Clone)]
pub struct Preview {
    pub scan_snapshot_id: String,
    pub targets: Vec<Candidate>,
    pub tool_targets: Vec<ToolTarget>,
    pub rejected: Vec<Rejection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemOutcome {
    MovedToTrash,
    RemovedByTool,
    TimedOut,
    SkippedStale,
    SkippedProtected,
    Failed { reason: String },
}

#[derive(Debug, Clone)]
pub struct ItemResult {
    pub item_id: String,
    pub rule_id: String,
    pub path: Option<PathBuf>,
    pub outcome: ItemOutcome,
    pub trash_location: Option<PathBuf>,
}

pub fn candidate_for(path: &Path, rule_id: &str, risk: RiskLevel) -> std::io::Result<Candidate> {
    let meta = fs::symlink_metadata(path)?;
    Ok(Candidate {
        rule_id: rule_id.to_string(),
        risk,
        path: path.to_path_buf(),
        device: meta.dev(),
        inode: meta.ino(),
        logical_bytes: meta.len(),
        allocated_bytes: None,
    })
}

pub fn preview(
    scan_snapshot_id: &str,
    candidates: Vec<Candidate>,
    tool_targets: Vec<ToolTarget>,
    home: &Path,
) -> Preview {
    let mut targets = Vec::new();
    let mut rejected = Vec::new();
    for candidate in candidates {
        let reason = if candidate.risk == RiskLevel::ReportOnly {
            Some(RejectReason::ReportOnly)
        } else if is_denylisted_in(&candidate.path, home) {
            Some(RejectReason::Denylisted)
        } else {
            None
        };
        match reason {
            Some(reason) => rejected.push(Rejection {
                label: candidate.path.display().to_string(),
                reason,
            }),
            None => targets.push(candidate),
        }
    }
    let mut planned_tools = Vec::new();
    for target in tool_targets {
        if target.risk == RiskLevel::ReportOnly {
            rejected.push(Rejection {
                label: target.description,
                reason: RejectReason::ReportOnly,
            });
        } else {
            planned_tools.push(target);
        }
    }
    Preview {
        scan_snapshot_id: scan_snapshot_id.to_string(),
        targets,
        tool_targets: planned_tools,
        rejected,
    }
}

pub fn execute(
    operation_id: &str,
    preview: &Preview,
    journal_dir: &Path,
    audit: &AuditLog,
    home: &Path,
    mut mover: impl FnMut(&Path) -> Result<()>,
) -> Result<Vec<ItemResult>> {
    let blocked = journal::unacknowledged_quarantine(journal_dir)?;
    if !blocked.is_empty() {
        bail!(
            "cleanup refused: {} corrupt journal(s) need acknowledgement",
            blocked.len()
        );
    }
    fs::create_dir_all(journal_dir)?;
    let mut journal = Journal::create(journal_dir, operation_id)?;
    journal.record(Event::OperationStarted {
        scan_snapshot_id: preview.scan_snapshot_id.clone(),
        rule_ids: rule_ids(preview),
        risk_ceiling: risk_ceiling(preview),
        selected_count: preview.targets.len() + preview.tool_targets.len(),
        estimated_bytes: preview.targets.iter().map(|c| c.logical_bytes).sum(),
    })?;

    let items = plan_items(&mut journal, preview)?;
    let used_before = used_bytes_on_startup_volume();

    let mut results = Vec::new();
    let mut audit_targets = Vec::new();
    for item in items {
        let (outcome, trash_location) = match &item.target {
            Planned::Trash(candidate) => {
                act_trash(&mut journal, &item.item_id, candidate, home, &mut mover)?
            }
            Planned::Tool(target) => (act_tool(&mut journal, &item.item_id, target)?, None),
        };
        journal.record(Event::ItemOutcome {
            item_id: item.item_id.clone(),
            outcome: journal_outcome(&outcome),
            trash_location: trash_location.clone(),
            error_code: failure_reason(&outcome).map(str::to_string),
        })?;
        audit_targets.push(AuditTarget {
            path: item.label.clone(),
            logical_bytes: item.logical_bytes,
            allocated_bytes: item.allocated_bytes,
            action: audit_action(&outcome),
            rule_id: Some(item.rule_id.clone()),
        });
        results.push(ItemResult {
            item_id: item.item_id,
            rule_id: item.rule_id,
            path: item.path,
            outcome,
            trash_location,
        });
    }

    let used_after = used_bytes_on_startup_volume();
    if let (Some(used_before), Some(used_after)) = (used_before, used_after) {
        journal.record(Event::Measurement {
            volume: STARTUP_VOLUME.to_string(),
            used_before,
            used_after,
            method: "sysinfo_disks".to_string(),
            caveats: vec!["purgeable space and local snapshots are not included".to_string()],
        })?;
    }
    journal.record(Event::OperationCompleted {
        summary: summarize(&results),
        incomplete: false,
    })?;
    audit.append(&AuditEntry {
        timestamp: Utc::now(),
        dustmap_version: env!("CARGO_PKG_VERSION").to_string(),
        targets: audit_targets,
        bytes_before: used_before,
        bytes_after: used_after,
    })?;
    Ok(results)
}

enum Planned<'a> {
    Trash(&'a Candidate),
    Tool(&'a ToolTarget),
}

struct PlannedItem<'a> {
    item_id: String,
    rule_id: String,
    path: Option<PathBuf>,
    label: String,
    logical_bytes: u64,
    allocated_bytes: Option<u64>,
    target: Planned<'a>,
}

fn plan_items<'a>(journal: &mut Journal, preview: &'a Preview) -> Result<Vec<PlannedItem<'a>>> {
    let mut items = Vec::new();
    for candidate in &preview.targets {
        let item_id = format!("item-{}", items.len() + 1);
        journal.record(Event::ItemPlanned {
            item_id: item_id.clone(),
            rule_id: candidate.rule_id.clone(),
            path: Some(candidate.path.clone()),
            device: Some(candidate.device),
            inode: Some(candidate.inode),
            logical_bytes: candidate.logical_bytes,
            allocated_bytes: candidate.allocated_bytes,
            risk: candidate.risk,
            action: Action::Trash,
        })?;
        items.push(PlannedItem {
            item_id,
            rule_id: candidate.rule_id.clone(),
            path: Some(candidate.path.clone()),
            label: candidate.path.display().to_string(),
            logical_bytes: candidate.logical_bytes,
            allocated_bytes: candidate.allocated_bytes,
            target: Planned::Trash(candidate),
        });
    }
    for target in &preview.tool_targets {
        let item_id = format!("item-{}", items.len() + 1);
        journal.record(Event::ItemPlanned {
            item_id: item_id.clone(),
            rule_id: target.rule_id.clone(),
            path: None,
            device: None,
            inode: None,
            logical_bytes: 0,
            allocated_bytes: None,
            risk: target.risk,
            action: Action::ToolCommand,
        })?;
        items.push(PlannedItem {
            item_id,
            rule_id: target.rule_id.clone(),
            path: None,
            label: target.description.clone(),
            logical_bytes: 0,
            allocated_bytes: None,
            target: Planned::Tool(target),
        });
    }
    Ok(items)
}

fn act_trash(
    journal: &mut Journal,
    item_id: &str,
    candidate: &Candidate,
    home: &Path,
    mover: &mut impl FnMut(&Path) -> Result<()>,
) -> Result<(ItemOutcome, Option<PathBuf>)> {
    if is_denylisted_in(&candidate.path, home) {
        return Ok((ItemOutcome::SkippedProtected, None));
    }
    let meta = match fs::symlink_metadata(&candidate.path) {
        Ok(meta) if meta.dev() == candidate.device && meta.ino() == candidate.inode => meta,
        _ => return Ok((ItemOutcome::SkippedStale, None)),
    };
    journal.record(Event::ItemIntent {
        item_id: item_id.to_string(),
        expected_identity: Some(Identity {
            device: meta.dev(),
            inode: meta.ino(),
            modified: DateTime::<Utc>::from(meta.modified()?),
        }),
        action: Action::Trash,
    })?;

    if let Err(err) = mover(&candidate.path) {
        return Ok((
            ItemOutcome::Failed {
                reason: format!("{err:#}"),
            },
            None,
        ));
    }
    match fs::symlink_metadata(&candidate.path) {
        Err(err) if err.kind() == ErrorKind::NotFound => Ok((
            ItemOutcome::MovedToTrash,
            find_in_trash(home, meta.dev(), meta.ino()),
        )),
        _ => Ok((
            ItemOutcome::Failed {
                reason: "path still present after move".to_string(),
            },
            None,
        )),
    }
}

fn act_tool(journal: &mut Journal, item_id: &str, target: &ToolTarget) -> Result<ItemOutcome> {
    journal.record(Event::ItemIntent {
        item_id: item_id.to_string(),
        expected_identity: None,
        action: Action::ToolCommand,
    })?;
    let run = toolcmd::run(
        &target.argv,
        target.timeout,
        |child_pid, child_start_time| {
            Ok(journal.record(Event::ItemExecuting {
                item_id: item_id.to_string(),
                child_pid,
                child_start_time,
            })?)
        },
    );
    Ok(match run {
        Err(err) => ItemOutcome::Failed {
            reason: format!("{err:#}"),
        },
        Ok(run) if run.timed_out => ItemOutcome::TimedOut,
        Ok(run) if run.succeeded() => ItemOutcome::RemovedByTool,
        Ok(run) => ItemOutcome::Failed {
            reason: format!("exit code {:?}: {}", run.exit_code, run.stderr.trim()),
        },
    })
}

fn used_bytes_on_startup_volume() -> Option<u64> {
    let disks = sysinfo::Disks::new_with_refreshed_list();
    disks
        .list()
        .iter()
        .find(|disk| disk.mount_point() == Path::new(STARTUP_VOLUME))
        .map(|disk| disk.total_space().saturating_sub(disk.available_space()))
}

fn journal_outcome(outcome: &ItemOutcome) -> Outcome {
    match outcome {
        ItemOutcome::MovedToTrash => Outcome::MovedToTrash,
        ItemOutcome::RemovedByTool => Outcome::RemovedByTool,
        ItemOutcome::TimedOut => Outcome::TimedOut,
        ItemOutcome::SkippedStale => Outcome::SkippedStale,
        ItemOutcome::SkippedProtected => Outcome::SkippedProtected,
        ItemOutcome::Failed { .. } => Outcome::Failed,
    }
}

fn audit_action(outcome: &ItemOutcome) -> AuditAction {
    match outcome {
        ItemOutcome::MovedToTrash => AuditAction::MovedToTrash,
        ItemOutcome::RemovedByTool => AuditAction::RemovedByTool,
        ItemOutcome::TimedOut => AuditAction::Failed {
            reason: "timed out".to_string(),
        },
        ItemOutcome::SkippedStale => AuditAction::Stale,
        ItemOutcome::SkippedProtected => AuditAction::Skipped,
        ItemOutcome::Failed { reason } => AuditAction::Failed {
            reason: reason.clone(),
        },
    }
}

fn failure_reason(outcome: &ItemOutcome) -> Option<&str> {
    match outcome {
        ItemOutcome::Failed { reason } => Some(reason),
        ItemOutcome::TimedOut => Some("timed out"),
        _ => None,
    }
}

fn summarize(results: &[ItemResult]) -> BTreeMap<String, u64> {
    let mut summary = BTreeMap::new();
    for result in results {
        let key = match result.outcome {
            ItemOutcome::MovedToTrash => "moved_to_trash",
            ItemOutcome::RemovedByTool => "removed_by_tool",
            ItemOutcome::TimedOut => "timed_out",
            ItemOutcome::SkippedStale => "skipped_stale",
            ItemOutcome::SkippedProtected => "skipped_protected",
            ItemOutcome::Failed { .. } => "failed",
        };
        *summary.entry(key.to_string()).or_insert(0) += 1;
    }
    summary
}

fn rule_ids(preview: &Preview) -> Vec<String> {
    let mut ids: Vec<String> = preview
        .targets
        .iter()
        .map(|c| c.rule_id.clone())
        .chain(preview.tool_targets.iter().map(|t| t.rule_id.clone()))
        .collect();
    ids.sort();
    ids.dedup();
    ids
}

fn risk_ceiling(preview: &Preview) -> RiskLevel {
    let rank = |risk: &RiskLevel| match risk {
        RiskLevel::Safe => 0,
        RiskLevel::Review => 1,
        RiskLevel::Caution => 2,
        RiskLevel::ReportOnly => 3,
    };
    preview
        .targets
        .iter()
        .map(|c| c.risk)
        .chain(preview.tool_targets.iter().map(|t| t.risk))
        .max_by_key(rank)
        .unwrap_or(RiskLevel::Safe)
}
