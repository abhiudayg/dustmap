use chrono::{DateTime, Utc};
use dustmap_core::journal::{
    reconcile, replay, Action, Event, Identity, ItemState, Journal, JournalError, Outcome,
};
use dustmap_core::rules::RiskLevel;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

fn temp_dir(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "dustmap_journal_{}_{}_{}",
        name,
        std::process::id(),
        nanos
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn make_file(path: &Path) {
    fs::write(path, b"cache").unwrap();
}

fn identity_of(path: &Path) -> Identity {
    let meta = fs::symlink_metadata(path).unwrap();
    Identity {
        device: meta.dev(),
        inode: meta.ino(),
        modified: DateTime::<Utc>::from(meta.modified().unwrap()),
    }
}

fn planned(item_id: &str, path: &Path) -> Event {
    let meta = fs::symlink_metadata(path).unwrap();
    Event::ItemPlanned {
        item_id: item_id.into(),
        rule_id: "homebrew-cache".into(),
        path: Some(path.to_path_buf()),
        device: Some(meta.dev()),
        inode: Some(meta.ino()),
        logical_bytes: meta.len(),
        allocated_bytes: None,
        risk: RiskLevel::Safe,
        action: Action::Trash,
    }
}

fn intent(item_id: &str, identity: Identity) -> Event {
    Event::ItemIntent {
        item_id: item_id.into(),
        expected_identity: Some(identity),
        action: Action::Trash,
    }
}

fn append_raw(path: &Path, bytes: &[u8]) {
    let mut file = OpenOptions::new().append(true).open(path).unwrap();
    file.write_all(bytes).unwrap();
}

#[test]
fn journal_round_trips_records_in_sequence() {
    let dir = temp_dir("round_trip");
    let target = dir.join("cache.bin");
    make_file(&target);

    let mut journal = Journal::create(&dir, "op-1").unwrap();
    journal
        .record(Event::OperationStarted {
            scan_snapshot_id: "scan-1".into(),
            rule_ids: vec!["homebrew-cache".into()],
            risk_ceiling: RiskLevel::Safe,
            selected_count: 1,
            estimated_bytes: 5,
        })
        .unwrap();
    journal.record(planned("item-1", &target)).unwrap();
    journal
        .record(intent("item-1", identity_of(&target)))
        .unwrap();
    journal
        .record(Event::ItemOutcome {
            item_id: "item-1".into(),
            outcome: Outcome::MovedToTrash,
            trash_location: None,
            error_code: None,
        })
        .unwrap();

    let records = replay(&dir.join("op-1.jsonl")).unwrap().records;

    let seqs: Vec<u64> = records.iter().map(|r| r.seq).collect();
    assert_eq!(seqs, vec![1, 2, 3, 4]);
    assert!(records
        .iter()
        .all(|r| r.schema == "1.0" && r.operation_id == "op-1"));
    assert!(matches!(
        records[3].event,
        Event::ItemOutcome {
            outcome: Outcome::MovedToTrash,
            ..
        }
    ));
}

#[test]
fn replay_ignores_partial_final_line() {
    let dir = temp_dir("torn_tail");
    let target = dir.join("cache.bin");
    make_file(&target);
    let path = dir.join("op-2.jsonl");

    let mut journal = Journal::create(&dir, "op-2").unwrap();
    journal.record(planned("item-1", &target)).unwrap();
    drop(journal);
    let valid_len = fs::metadata(&path).unwrap().len();
    append_raw(&path, b"{\"schema\":\"1.0\",\"seq\":3");

    let replay = replay(&path).unwrap();

    assert_eq!(replay.records.len(), 1);
    assert!(replay.torn_tail);
    assert_eq!(replay.valid_len, valid_len);
}

#[test]
fn replay_rejects_corrupt_line_before_the_end() {
    let dir = temp_dir("corrupt_middle");
    let target = dir.join("cache.bin");
    make_file(&target);
    let path = dir.join("op-3.jsonl");

    let mut journal = Journal::create(&dir, "op-3").unwrap();
    journal.record(planned("item-1", &target)).unwrap();
    append_raw(&path, b"not json\n");
    journal.record(planned("item-2", &target)).unwrap();

    let err = replay(&path).unwrap_err();

    assert!(matches!(err, JournalError::Corrupt { line: 2, .. }));
}

#[test]
fn reconcile_marks_pending_item_not_moved_when_path_still_exists() {
    let root = temp_dir("not_moved_root");
    let journals = temp_dir("not_moved_journals");
    let target = root.join("cache.bin");
    make_file(&target);

    let mut journal = Journal::create(&journals, "op-4").unwrap();
    journal.record(planned("item-1", &target)).unwrap();
    journal
        .record(intent("item-1", identity_of(&target)))
        .unwrap();
    drop(journal);

    let report = reconcile(&journals, &no_home()).unwrap();

    assert_eq!(report.items.len(), 1);
    let item = &report.items[0];
    assert_eq!(item.operation_id, "op-4");
    assert_eq!(item.item_id, "item-1");
    assert_eq!(item.state, ItemState::NotMoved);
    assert!(!item.needs_user_review);
    assert!(target.exists());
}

#[test]
fn reconcile_marks_pending_item_unknown_when_path_is_gone() {
    let root = temp_dir("gone_root");
    let journals = temp_dir("gone_journals");
    let target = root.join("cache.bin");
    make_file(&target);

    let mut journal = Journal::create(&journals, "op-5").unwrap();
    journal.record(planned("item-1", &target)).unwrap();
    journal
        .record(intent("item-1", identity_of(&target)))
        .unwrap();
    drop(journal);
    fs::remove_file(&target).unwrap();

    let report = reconcile(&journals, &no_home()).unwrap();

    assert_eq!(report.items.len(), 1);
    assert_eq!(report.items[0].state, ItemState::Unknown);
    assert!(report.items[0].needs_user_review);
}

#[test]
fn reconcile_marks_pending_item_unknown_when_identity_changed() {
    let root = temp_dir("identity_root");
    let journals = temp_dir("identity_journals");
    let target = root.join("cache.bin");
    let other = root.join("other.bin");
    make_file(&target);
    make_file(&other);

    let mut journal = Journal::create(&journals, "op-6").unwrap();
    journal.record(planned("item-1", &target)).unwrap();
    journal
        .record(intent("item-1", identity_of(&other)))
        .unwrap();
    drop(journal);

    let report = reconcile(&journals, &no_home()).unwrap();

    assert_eq!(report.items.len(), 1);
    assert_eq!(report.items[0].state, ItemState::Unknown);
    assert!(report.items[0].needs_user_review);
}

#[test]
fn reconcile_never_reports_tool_command_items_as_not_moved() {
    let root = temp_dir("tool_root");
    let journals = temp_dir("tool_journals");
    let target = root.join("cache");
    fs::create_dir_all(&target).unwrap();

    let mut journal = Journal::create(&journals, "op-7").unwrap();
    journal.record(planned("item-1", &target)).unwrap();
    journal
        .record(Event::ItemIntent {
            item_id: "item-1".into(),
            expected_identity: Some(identity_of(&target)),
            action: Action::ToolCommand,
        })
        .unwrap();
    drop(journal);

    let report = reconcile(&journals, &no_home()).unwrap();

    assert_eq!(report.items.len(), 1);
    assert_eq!(report.items[0].state, ItemState::Unknown);
    assert!(report.items[0].needs_user_review);
}

#[test]
fn reconcile_skips_items_with_an_outcome() {
    let root = temp_dir("settled_root");
    let journals = temp_dir("settled_journals");
    let target = root.join("cache.bin");
    make_file(&target);

    let mut journal = Journal::create(&journals, "op-8").unwrap();
    journal.record(planned("item-1", &target)).unwrap();
    journal
        .record(intent("item-1", identity_of(&target)))
        .unwrap();
    journal
        .record(Event::ItemOutcome {
            item_id: "item-1".into(),
            outcome: Outcome::Failed,
            trash_location: None,
            error_code: Some("EPERM".into()),
        })
        .unwrap();
    drop(journal);

    let report = reconcile(&journals, &no_home()).unwrap();

    assert!(report.items.is_empty());
}

#[test]
fn reconcile_does_not_reconcile_the_same_item_twice() {
    let root = temp_dir("twice_root");
    let journals = temp_dir("twice_journals");
    let target = root.join("cache.bin");
    make_file(&target);

    let mut journal = Journal::create(&journals, "op-9").unwrap();
    journal.record(planned("item-1", &target)).unwrap();
    journal
        .record(intent("item-1", identity_of(&target)))
        .unwrap();
    drop(journal);

    assert_eq!(reconcile(&journals, &no_home()).unwrap().items.len(), 1);
    assert!(reconcile(&journals, &no_home()).unwrap().items.is_empty());
}

#[test]
fn reconcile_repairs_partial_final_line_before_appending() {
    let root = temp_dir("repair_root");
    let journals = temp_dir("repair_journals");
    let target = root.join("cache.bin");
    make_file(&target);
    let path = journals.join("op-10.jsonl");

    let mut journal = Journal::create(&journals, "op-10").unwrap();
    journal.record(planned("item-1", &target)).unwrap();
    journal
        .record(intent("item-1", identity_of(&target)))
        .unwrap();
    drop(journal);
    append_raw(&path, b"{\"schema\":\"1.0\",\"se");

    let report = reconcile(&journals, &no_home()).unwrap();

    assert_eq!(report.items.len(), 1);
    let repaired = replay(&path).unwrap();
    assert!(!repaired.torn_tail);
    assert!(repaired
        .records
        .iter()
        .any(|r| matches!(r.event, Event::JournalRepaired { .. })));
    assert!(repaired
        .records
        .iter()
        .any(|r| matches!(r.event, Event::ItemReconciled { .. })));
}

#[test]
fn reconcile_quarantines_corrupt_journal_without_blocking_others() {
    let root = temp_dir("quarantine_root");
    let journals = temp_dir("quarantine_journals");
    let target = root.join("cache.bin");
    make_file(&target);

    let mut broken = Journal::create(&journals, "op-broken").unwrap();
    broken.record(planned("item-x", &target)).unwrap();
    append_raw(&journals.join("op-broken.jsonl"), b"garbage\n");
    broken.record(planned("item-y", &target)).unwrap();
    drop(broken);

    let mut healthy = Journal::create(&journals, "op-healthy").unwrap();
    healthy.record(planned("item-1", &target)).unwrap();
    healthy
        .record(intent("item-1", identity_of(&target)))
        .unwrap();
    drop(healthy);

    let report = reconcile(&journals, &no_home()).unwrap();

    assert_eq!(report.quarantined.len(), 1);
    assert!(report.quarantined[0].exists());
    assert!(!journals.join("op-broken.jsonl").exists());
    assert_eq!(report.items.len(), 1);
    assert_eq!(report.items[0].operation_id, "op-healthy");
}

#[test]
fn reconcile_on_missing_journal_directory_reports_nothing() {
    let missing = std::env::temp_dir().join("dustmap_journal_never_created_xyz");
    let _ = fs::remove_dir_all(&missing);

    let report = reconcile(&missing, &no_home()).unwrap();

    assert!(report.items.is_empty());
    assert!(report.quarantined.is_empty());
}

fn no_home() -> PathBuf {
    std::env::temp_dir().join("dustmap_journal_no_home_directory")
}

#[test]
fn reconcile_recovers_item_moved_into_the_home_trash() {
    let home = temp_dir("trash_home");
    let journals = temp_dir("trash_journals");
    let target = home.join("Downloads/cache.bin");
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    make_file(&target);

    let mut journal = Journal::create(&journals, "op-trash").unwrap();
    journal.record(planned("item-1", &target)).unwrap();
    journal
        .record(intent("item-1", identity_of(&target)))
        .unwrap();
    drop(journal);
    let trashed = home.join(".Trash/cache.bin");
    fs::create_dir_all(trashed.parent().unwrap()).unwrap();
    fs::rename(&target, &trashed).unwrap();

    let report = reconcile(&journals, &home).unwrap();

    assert_eq!(report.items.len(), 1);
    assert_eq!(report.items[0].state, ItemState::MovedToTrash);
    assert_eq!(
        report.items[0].trash_location.as_deref(),
        Some(trashed.as_path())
    );
    assert!(!report.items[0].needs_user_review);
}

#[test]
fn replay_keeps_unknown_event_types_without_failing() {
    let dir = temp_dir("unknown_event");
    let target = dir.join("cache.bin");
    make_file(&target);
    let path = dir.join("op-future.jsonl");
    let mut journal = Journal::create(&dir, "op-future").unwrap();
    journal.record(planned("item-1", &target)).unwrap();
    drop(journal);
    append_raw(
        &path,
        b"{\"schema\":\"1.1\",\"seq\":2,\"operation_id\":\"op-future\",\"timestamp\":\"2026-10-21T09:14:03Z\",\"app_version\":\"9.9.9\",\"event\":\"future_event\",\"payload\":1}\n",
    );

    let replay = replay(&path).unwrap();

    assert_eq!(replay.records.len(), 2);
    assert!(matches!(replay.records[1].event, Event::Unknown));
    assert!(!replay.torn_tail);
}

#[test]
fn quarantine_blocks_until_acknowledged() {
    let journals = temp_dir("ack_journals");
    let target = temp_dir("ack_target").join("cache.bin");
    make_file(&target);
    let mut journal = Journal::create(&journals, "op-bad").unwrap();
    journal.record(planned("item-1", &target)).unwrap();
    append_raw(&journals.join("op-bad.jsonl"), b"garbage\n");
    journal.record(planned("item-2", &target)).unwrap();
    drop(journal);

    reconcile(&journals, &no_home()).unwrap();
    assert_eq!(
        dustmap_core::journal::unacknowledged_quarantine(&journals)
            .unwrap()
            .len(),
        1
    );

    dustmap_core::journal::acknowledge_quarantine(&journals).unwrap();
    assert!(dustmap_core::journal::unacknowledged_quarantine(&journals)
        .unwrap()
        .is_empty());
}
