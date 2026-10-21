use anyhow::anyhow;
use dustmap_core::audit::{AuditAction, AuditEntry, AuditLog};
use dustmap_core::cleaner::{execute, preview, Candidate, ItemOutcome, RejectReason};
use dustmap_core::journal::{replay, Event};
use dustmap_core::rules::RiskLevel;
use std::cell::Cell;
use std::fs;
use std::os::unix::fs::{symlink, MetadataExt};
use std::path::{Path, PathBuf};

fn temp_dir(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "dustmap_cleaner_{}_{}_{}",
        name,
        std::process::id(),
        nanos
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn make_file(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, b"cache-bytes").unwrap();
}

fn candidate(path: &Path, risk: RiskLevel) -> Candidate {
    let meta = fs::symlink_metadata(path).unwrap();
    Candidate {
        rule_id: "homebrew-cache".into(),
        risk,
        path: path.to_path_buf(),
        device: meta.dev(),
        inode: meta.ino(),
        logical_bytes: meta.len(),
        allocated_bytes: None,
    }
}

fn remove_like_trash(path: &Path) -> anyhow::Result<()> {
    fs::remove_file(path).map_err(Into::into)
}

#[test]
fn preview_rejects_denylisted_and_report_only_candidates() {
    let root = temp_dir("preview");
    let home = root.join("home");
    let docs = home.join("Documents/a.bin");
    let downloads = home.join("Downloads/b.bin");
    let report_only = home.join("Downloads/c.bin");
    make_file(&docs);
    make_file(&downloads);
    make_file(&report_only);

    let preview = preview(
        "scan-1",
        vec![
            candidate(&docs, RiskLevel::Safe),
            candidate(&downloads, RiskLevel::Safe),
            candidate(&report_only, RiskLevel::ReportOnly),
        ],
        Vec::new(),
        &home,
    );

    assert_eq!(preview.targets.len(), 1);
    assert_eq!(preview.targets[0].path, downloads);
    let reasons: Vec<RejectReason> = preview.rejected.iter().map(|r| r.reason).collect();
    assert_eq!(
        reasons,
        vec![RejectReason::Denylisted, RejectReason::ReportOnly]
    );
}

#[test]
fn execute_writes_intent_before_acting_and_records_audit_entry() {
    let root = temp_dir("intent_order");
    let home = root.join("home");
    let target = home.join("Downloads/cache.bin");
    make_file(&target);
    let journal_dir = root.join("journal");
    let audit = AuditLog::new(root.join("audit.jsonl"));
    let preview = preview(
        "scan-2",
        vec![candidate(&target, RiskLevel::Safe)],
        Vec::new(),
        &home,
    );

    let intent_seen_before_action = Cell::new(false);
    let results = execute("op-intent", &preview, &journal_dir, &audit, &home, |path| {
        let records = replay(&journal_dir.join("op-intent.jsonl"))
            .unwrap()
            .records;
        intent_seen_before_action.set(
            records
                .iter()
                .any(|r| matches!(r.event, Event::ItemIntent { .. })),
        );
        remove_like_trash(path)
    })
    .unwrap();

    assert!(intent_seen_before_action.get());
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].outcome, ItemOutcome::MovedToTrash);
    assert!(!target.exists());

    let records = replay(&journal_dir.join("op-intent.jsonl"))
        .unwrap()
        .records;
    assert!(matches!(records[0].event, Event::OperationStarted { .. }));
    assert!(matches!(
        records.last().unwrap().event,
        Event::OperationCompleted { .. }
    ));
    assert!(records.iter().any(|r| matches!(
        r.event,
        Event::ItemOutcome {
            outcome: dustmap_core::journal::Outcome::MovedToTrash,
            ..
        }
    )));

    let audit_text = fs::read_to_string(root.join("audit.jsonl")).unwrap();
    let entry: AuditEntry = serde_json::from_str(audit_text.trim()).unwrap();
    assert_eq!(entry.targets.len(), 1);
    assert!(matches!(entry.targets[0].action, AuditAction::MovedToTrash));
}

#[test]
fn execute_skips_stale_target_without_calling_mover() {
    let root = temp_dir("stale");
    let home = root.join("home");
    let target = home.join("Downloads/cache.bin");
    make_file(&target);
    let mut stale = candidate(&target, RiskLevel::Safe);
    stale.inode += 1;
    let preview = preview("scan-3", vec![stale], Vec::new(), &home);
    let calls = Cell::new(0);
    let audit = AuditLog::new(root.join("audit.jsonl"));

    let results = execute(
        "op-stale",
        &preview,
        &root.join("journal"),
        &audit,
        &home,
        |_| {
            calls.set(calls.get() + 1);
            Ok(())
        },
    )
    .unwrap();

    assert_eq!(calls.get(), 0);
    assert_eq!(results[0].outcome, ItemOutcome::SkippedStale);
    assert!(target.exists());
}

#[test]
fn execute_never_moves_target_that_became_protected_after_preview() {
    let root = temp_dir("protected_later");
    let home = root.join("home");
    let target = home.join("Documents/cache.bin");
    make_file(&target);
    let preview_home = root.join("elsewhere");
    let preview = preview(
        "scan-4",
        vec![candidate(&target, RiskLevel::Safe)],
        Vec::new(),
        &preview_home,
    );
    assert_eq!(preview.targets.len(), 1);
    let calls = Cell::new(0);
    let audit = AuditLog::new(root.join("audit.jsonl"));

    let results = execute(
        "op-protected",
        &preview,
        &root.join("journal"),
        &audit,
        &home,
        |_| {
            calls.set(calls.get() + 1);
            Ok(())
        },
    )
    .unwrap();

    assert_eq!(results[0].outcome, ItemOutcome::SkippedProtected);
    assert_eq!(calls.get(), 0);
    assert!(target.exists());
}

#[test]
fn execute_reports_failed_when_mover_errors() {
    let root = temp_dir("mover_err");
    let home = root.join("home");
    let target = home.join("Downloads/cache.bin");
    make_file(&target);
    let preview = preview(
        "scan-5",
        vec![candidate(&target, RiskLevel::Safe)],
        Vec::new(),
        &home,
    );
    let audit = AuditLog::new(root.join("audit.jsonl"));

    let results = execute(
        "op-err",
        &preview,
        &root.join("journal"),
        &audit,
        &home,
        |_| Err(anyhow!("EPERM")),
    )
    .unwrap();

    match &results[0].outcome {
        ItemOutcome::Failed { reason } => assert!(reason.contains("EPERM")),
        other => panic!("expected Failed, got {other:?}"),
    }
    assert!(target.exists());
}

#[test]
fn execute_reports_failed_when_target_is_still_present_after_move() {
    let root = temp_dir("still_there");
    let home = root.join("home");
    let target = home.join("Downloads/cache.bin");
    make_file(&target);
    let preview = preview(
        "scan-6",
        vec![candidate(&target, RiskLevel::Safe)],
        Vec::new(),
        &home,
    );
    let audit = AuditLog::new(root.join("audit.jsonl"));

    let results = execute(
        "op-still",
        &preview,
        &root.join("journal"),
        &audit,
        &home,
        |_| Ok(()),
    )
    .unwrap();

    match &results[0].outcome {
        ItemOutcome::Failed { reason } => assert!(reason.contains("still present")),
        other => panic!("expected Failed, got {other:?}"),
    }
}

#[test]
fn execute_reports_every_item_in_a_partial_batch() {
    let root = temp_dir("partial");
    let home = root.join("home");
    let first = home.join("Downloads/first.bin");
    let second = home.join("Downloads/second.bin");
    make_file(&first);
    make_file(&second);
    let preview = preview(
        "scan-7",
        vec![
            candidate(&first, RiskLevel::Safe),
            candidate(&second, RiskLevel::Safe),
        ],
        Vec::new(),
        &home,
    );
    let audit = AuditLog::new(root.join("audit.jsonl"));

    let results = execute(
        "op-partial",
        &preview,
        &root.join("journal"),
        &audit,
        &home,
        |path| {
            if path == first {
                Err(anyhow!("busy"))
            } else {
                remove_like_trash(path)
            }
        },
    )
    .unwrap();

    assert_eq!(results.len(), 2);
    assert!(matches!(results[0].outcome, ItemOutcome::Failed { .. }));
    assert_eq!(results[1].outcome, ItemOutcome::MovedToTrash);
    assert!(first.exists());
    assert!(!second.exists());
}

#[test]
fn execute_moves_a_symlink_itself_and_never_follows_it() {
    let root = temp_dir("symlink");
    let home = root.join("home");
    let real = home.join("Downloads/real.bin");
    let link = home.join("Downloads/link.bin");
    make_file(&real);
    symlink(&real, &link).unwrap();
    let preview = preview(
        "scan-8",
        vec![candidate(&link, RiskLevel::Safe)],
        Vec::new(),
        &home,
    );
    let audit = AuditLog::new(root.join("audit.jsonl"));

    let results = execute(
        "op-link",
        &preview,
        &root.join("journal"),
        &audit,
        &home,
        remove_like_trash,
    )
    .unwrap();

    assert_eq!(results[0].outcome, ItemOutcome::MovedToTrash);
    assert!(!link.exists() && fs::symlink_metadata(&link).is_err());
    assert!(real.exists());
}

fn tool(argv: &[&str], timeout_ms: u64) -> dustmap_core::cleaner::ToolTarget {
    dustmap_core::cleaner::ToolTarget {
        rule_id: "tool-rule".into(),
        risk: RiskLevel::Review,
        description: argv.join(" "),
        argv: argv.iter().map(|part| part.to_string()).collect(),
        timeout: std::time::Duration::from_millis(timeout_ms),
    }
}

#[test]
fn execute_runs_tool_command_and_journals_child_process() {
    let root = temp_dir("tool_ok");
    let home = root.join("home");
    fs::create_dir_all(&home).unwrap();
    let preview = preview(
        "scan-9",
        Vec::new(),
        vec![tool(&["/usr/bin/true"], 10_000)],
        &home,
    );
    let audit = AuditLog::new(root.join("audit.jsonl"));
    let journal_dir = root.join("journal");

    let results = execute("op-tool", &preview, &journal_dir, &audit, &home, |_| Ok(())).unwrap();

    assert_eq!(results[0].outcome, ItemOutcome::RemovedByTool);
    let records = replay(&journal_dir.join("op-tool.jsonl")).unwrap().records;
    assert!(records
        .iter()
        .any(|r| matches!(r.event, Event::ItemExecuting { child_pid, .. } if child_pid > 0)));
    assert!(records.iter().any(|r| matches!(
        r.event,
        Event::ItemOutcome {
            outcome: dustmap_core::journal::Outcome::RemovedByTool,
            ..
        }
    )));
}

#[test]
fn execute_reports_failed_for_non_zero_tool_exit_and_timeout_for_slow_tool() {
    let root = temp_dir("tool_bad");
    let home = root.join("home");
    fs::create_dir_all(&home).unwrap();
    let preview = preview(
        "scan-10",
        Vec::new(),
        vec![
            tool(&["/usr/bin/false"], 10_000),
            tool(&["/bin/sleep", "30"], 100),
        ],
        &home,
    );
    let audit = AuditLog::new(root.join("audit.jsonl"));

    let results = execute(
        "op-tool-bad",
        &preview,
        &root.join("journal"),
        &audit,
        &home,
        |_| Ok(()),
    )
    .unwrap();

    assert!(matches!(results[0].outcome, ItemOutcome::Failed { .. }));
    assert_eq!(results[1].outcome, ItemOutcome::TimedOut);
}

#[test]
fn execute_records_trash_location_for_moved_items() {
    let root = temp_dir("trash_loc");
    let home = root.join("home");
    let target = home.join("Downloads/cache.bin");
    make_file(&target);
    let trashed = home.join(".Trash/cache.bin");
    let preview = preview(
        "scan-11",
        vec![candidate(&target, RiskLevel::Safe)],
        Vec::new(),
        &home,
    );
    let audit = AuditLog::new(root.join("audit.jsonl"));
    let journal_dir = root.join("journal");

    let results = execute("op-loc", &preview, &journal_dir, &audit, &home, |path| {
        fs::create_dir_all(trashed.parent().unwrap())?;
        fs::rename(path, &trashed).map_err(Into::into)
    })
    .unwrap();

    assert_eq!(results[0].outcome, ItemOutcome::MovedToTrash);
    assert_eq!(
        results[0].trash_location.as_deref(),
        Some(trashed.as_path())
    );
    let records = replay(&journal_dir.join("op-loc.jsonl")).unwrap().records;
    assert!(matches!(
        records.last().unwrap().event,
        Event::OperationCompleted { .. }
    ));
    assert!(records.iter().any(|r| matches!(
        &r.event,
        Event::OperationCompleted { summary, .. } if summary.get("moved_to_trash") == Some(&1)
    )));
}

#[test]
fn execute_refuses_to_run_while_a_corrupt_journal_is_unacknowledged() {
    let root = temp_dir("quarantine_block");
    let home = root.join("home");
    let target = home.join("Downloads/cache.bin");
    make_file(&target);
    let journal_dir = root.join("journal");
    fs::create_dir_all(&journal_dir).unwrap();
    fs::write(journal_dir.join("op-old.jsonl.quarantined"), b"garbage\n").unwrap();
    let preview = preview(
        "scan-12",
        vec![candidate(&target, RiskLevel::Safe)],
        Vec::new(),
        &home,
    );
    let audit = AuditLog::new(root.join("audit.jsonl"));
    let calls = Cell::new(0);

    let err = execute("op-blocked", &preview, &journal_dir, &audit, &home, |_| {
        calls.set(calls.get() + 1);
        Ok(())
    })
    .unwrap_err();

    assert!(err.to_string().contains("acknowledgement"));
    assert_eq!(calls.get(), 0);
    assert!(target.exists());
}
