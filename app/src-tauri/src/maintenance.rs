use crate::commands::app_support_dir;
use chrono::Utc;
use dustmap_core::audit::{AuditAction, AuditEntry, AuditLog, AuditTarget};
use dustmap_core::cleaner::{self, ItemOutcome, Preview, RejectReason};
use dustmap_core::data;
use dustmap_core::journal;
use dustmap_core::remnants::{self, MatchKind, Remnant};
use dustmap_core::rules::RiskLevel;
use dustmap_core::trash::move_to_trash;
use dustmap_rules::{collect_targets, load_rules_with_fallback, Rule};
use serde::Serialize;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

fn home() -> Result<PathBuf, String> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| "HOME is not set".to_string())
}

fn rules_dir(app: &AppHandle) -> Result<PathBuf, String> {
    if let Some(dir) = std::env::var_os("DUSTMAP_RULES_DIR") {
        return Ok(PathBuf::from(dir));
    }
    app.path()
        .resource_dir()
        .map(|dir| dir.join("rules"))
        .map_err(|e| e.to_string())
}

fn load_rules(app: &AppHandle) -> Result<Vec<Rule>, String> {
    let dir = rules_dir(app)?;
    let (rules, _) = load_rules_with_fallback(&dir, &dir, None).map_err(|e| e.to_string())?;
    Ok(rules)
}

fn build_preview(app: &AppHandle) -> Result<(Vec<Rule>, Preview), String> {
    let home = home()?;
    let rules = load_rules(app)?;
    let (candidates, tools) = collect_targets(&rules, &home).map_err(|e| e.to_string())?;
    let scan_id = format!("gui-{}", chrono::Utc::now().timestamp_millis());
    Ok((rules, cleaner::preview(&scan_id, candidates, tools, &home)))
}

#[derive(Serialize)]
pub struct RuleInfoDto {
    id: String,
    name: String,
    category: String,
    risk: RiskLevel,
    summary: String,
    consequence: String,
    undo: String,
}

#[derive(Serialize)]
pub struct CleanupTargetDto {
    rule_id: String,
    risk: RiskLevel,
    path: String,
    logical_bytes: u64,
}

#[derive(Serialize)]
pub struct ToolTargetDto {
    rule_id: String,
    risk: RiskLevel,
    description: String,
}

#[derive(Serialize)]
pub struct RejectedDto {
    label: String,
    reason: &'static str,
}

#[derive(Serialize)]
pub struct CleanupPreviewDto {
    rules: Vec<RuleInfoDto>,
    targets: Vec<CleanupTargetDto>,
    tool_targets: Vec<ToolTargetDto>,
    rejected: Vec<RejectedDto>,
    estimated_bytes: u64,
}

#[derive(Serialize)]
pub struct CleanupResultDto {
    item_id: String,
    rule_id: String,
    path: Option<String>,
    outcome: &'static str,
    detail: Option<String>,
}

#[derive(Serialize)]
pub struct RemnantDto {
    path: String,
    location: &'static str,
    risk: RiskLevel,
    note: &'static str,
    match_kind: &'static str,
}

#[derive(Serialize)]
pub struct DataDto {
    label: &'static str,
    path: String,
    bytes: u64,
}

#[tauri::command]
pub fn cleanup_preview(app: AppHandle) -> Result<CleanupPreviewDto, String> {
    let (rules, preview) = build_preview(&app)?;
    let estimated_bytes = preview.targets.iter().map(|c| c.logical_bytes).sum();
    Ok(CleanupPreviewDto {
        rules: rules
            .into_iter()
            .map(|rule| RuleInfoDto {
                id: rule.id,
                name: rule.name,
                category: rule.category,
                risk: rule.risk,
                summary: rule.explanation.summary,
                consequence: rule.explanation.consequence,
                undo: rule.explanation.undo,
            })
            .collect(),
        targets: preview
            .targets
            .iter()
            .map(|c| CleanupTargetDto {
                rule_id: c.rule_id.clone(),
                risk: c.risk,
                path: c.path.display().to_string(),
                logical_bytes: c.logical_bytes,
            })
            .collect(),
        tool_targets: preview
            .tool_targets
            .iter()
            .map(|t| ToolTargetDto {
                rule_id: t.rule_id.clone(),
                risk: t.risk,
                description: t.description.clone(),
            })
            .collect(),
        rejected: preview
            .rejected
            .iter()
            .map(|r| RejectedDto {
                label: r.label.clone(),
                reason: match r.reason {
                    RejectReason::Denylisted => "protected path",
                    RejectReason::ReportOnly => "report only",
                },
            })
            .collect(),
        estimated_bytes,
    })
}

#[tauri::command]
pub fn cleanup_execute(
    app: AppHandle,
    rule_ids: Vec<String>,
    confirmed: bool,
) -> Result<Vec<CleanupResultDto>, String> {
    if !confirmed {
        return Err("Confirmation is required before anything is moved to Trash.".to_string());
    }
    let home = home()?;
    let (_, preview) = build_preview(&app)?;
    let selected: HashSet<&str> = rule_ids.iter().map(String::as_str).collect();
    let preview = Preview {
        scan_snapshot_id: preview.scan_snapshot_id,
        targets: preview
            .targets
            .into_iter()
            .filter(|c| selected.contains(c.rule_id.as_str()))
            .collect(),
        tool_targets: preview
            .tool_targets
            .into_iter()
            .filter(|t| selected.contains(t.rule_id.as_str()))
            .collect(),
        rejected: preview.rejected,
    };

    let support = app_support_dir()?;
    let audit = AuditLog::new(support.join("audit.jsonl"));
    let operation_id = uuid::Uuid::new_v4().to_string();
    let results = cleaner::execute(
        &operation_id,
        &preview,
        &support.join("journal"),
        &audit,
        &home,
        move_to_trash,
    )
    .map_err(|e| e.to_string())?;

    Ok(results
        .into_iter()
        .map(|result| {
            let (outcome, detail) = describe(&result.outcome);
            CleanupResultDto {
                item_id: result.item_id,
                rule_id: result.rule_id,
                path: result.path.map(|p| p.display().to_string()),
                outcome,
                detail,
            }
        })
        .collect())
}

fn describe(outcome: &ItemOutcome) -> (&'static str, Option<String>) {
    match outcome {
        ItemOutcome::MovedToTrash => ("moved_to_trash", None),
        ItemOutcome::RemovedByTool => ("removed_by_tool", None),
        ItemOutcome::TimedOut => ("timed_out", None),
        ItemOutcome::SkippedStale => (
            "skipped_stale",
            Some("The item changed since the preview; rescan and try again.".to_string()),
        ),
        ItemOutcome::SkippedProtected => ("skipped_protected", None),
        ItemOutcome::Failed { reason } => ("failed", Some(reason.clone())),
    }
}

fn found_remnants(bundle_id: &str, home: &Path) -> Result<Vec<Remnant>, String> {
    let app_dirs = vec![PathBuf::from("/Applications"), home.join("Applications")];
    let installed = remnants::installed_bundle_ids(&app_dirs).map_err(|e| e.to_string())?;
    remnants::find_remnants(bundle_id, home, &installed).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remnant_search(bundle_id: String) -> Result<Vec<RemnantDto>, String> {
    let home = home()?;
    Ok(found_remnants(&bundle_id, &home)?
        .into_iter()
        .map(|remnant| RemnantDto {
            path: remnant.path.display().to_string(),
            location: remnant.location,
            risk: remnant.risk,
            note: remnant.note,
            match_kind: match remnant.match_kind {
                MatchKind::ExactId => "exact identifier",
                MatchKind::IdWithSuffix => "identifier with suffix",
            },
        })
        .collect())
}

#[tauri::command]
pub fn remnant_remove(
    bundle_id: String,
    paths: Vec<String>,
    confirm_caution: bool,
) -> Result<Vec<String>, String> {
    let home = home()?;
    let selected: Vec<Remnant> = found_remnants(&bundle_id, &home)?
        .into_iter()
        .filter(|remnant| paths.contains(&remnant.path.display().to_string()))
        .collect();
    if selected.len() != paths.len() {
        return Err("Some selected items are no longer offered. Search again.".to_string());
    }
    if selected.iter().any(|remnant| remnant.risk == RiskLevel::ReportOnly) {
        return Err("Report-only items cannot be removed.".to_string());
    }
    if selected.iter().any(|remnant| remnant.risk == RiskLevel::Caution) && !confirm_caution {
        return Err("Caution items need explicit confirmation before they are moved.".to_string());
    }

    let audit = AuditLog::new(app_support_dir()?.join("audit.jsonl"));
    let mut targets = Vec::new();
    let mut failure = None;
    for remnant in &selected {
        let logical_bytes = fs::symlink_metadata(&remnant.path)
            .map(|meta| meta.len())
            .unwrap_or(0);
        if let Err(err) = move_to_trash(&remnant.path) {
            failure = Some(format!("{}: {err}", remnant.path.display()));
            break;
        }
        targets.push(AuditTarget {
            path: remnant.path.display().to_string(),
            logical_bytes,
            allocated_bytes: None,
            action: AuditAction::MovedToTrash,
            rule_id: None,
        });
    }

    let moved: Vec<String> = targets.iter().map(|target| target.path.clone()).collect();
    if !targets.is_empty() {
        audit
            .append(&AuditEntry {
                timestamp: Utc::now(),
                dustmap_version: env!("CARGO_PKG_VERSION").to_string(),
                targets,
                bytes_before: None,
                bytes_after: None,
            })
            .map_err(|e| e.to_string())?;
    }
    match failure {
        Some(err) => Err(format!("Moved {} item(s); stopped at {err}", moved.len())),
        None => Ok(moved),
    }
}

#[tauri::command]
pub fn data_list() -> Result<Vec<DataDto>, String> {
    let home = home()?;
    Ok(data::data_entries(&home)
        .into_iter()
        .map(|entry| DataDto {
            label: entry.label,
            path: entry.path.display().to_string(),
            bytes: entry.bytes,
        })
        .collect())
}

#[tauri::command]
pub fn data_remove(labels: Vec<String>) -> Result<Vec<String>, String> {
    let home = home()?;
    let refs: Vec<&str> = labels.iter().map(String::as_str).collect();
    data::remove_data(&home, &refs)
        .map(|removed| removed.into_iter().map(str::to_string).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn acknowledge_quarantine() -> Result<(), String> {
    let support = app_support_dir()?;
    journal::acknowledge_quarantine(&support.join("journal")).map_err(|e| e.to_string())
}
