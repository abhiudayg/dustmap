use dustmap_core::{
    audit::{AuditAction, AuditEntry, AuditLog, AuditTarget},
    index::ROOT_ID,
    scanner::{scan, ScanOptions},
    volumes::list_volumes,
};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{mpsc, Arc};

fn create_test_fixture(base_name: &str) -> PathBuf {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let tmp = std::env::temp_dir().join(format!(
        "dustmap_test_{}_{}_{}",
        base_name,
        std::process::id(),
        now
    ));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).expect("Failed to create temp fixture root");
    tmp.canonicalize().unwrap_or(tmp)
}

#[test]
fn test_scanner_directory_hierarchy_and_sizes() {
    let root = create_test_fixture("hierarchy");

    // Create dir A with file 1024 bytes
    let dir_a = root.join("dir_a");
    fs::create_dir_all(&dir_a).unwrap();
    let mut file1 = File::create(dir_a.join("file1.bin")).unwrap();
    file1.write_all(&[0u8; 1024]).unwrap();

    // Create dir B with 2 files (2048 bytes and 512 bytes)
    let dir_b = root.join("dir_b");
    fs::create_dir_all(&dir_b).unwrap();
    let mut file2 = File::create(dir_b.join("file2.bin")).unwrap();
    file2.write_all(&[0u8; 2048]).unwrap();
    let mut file3 = File::create(dir_b.join("file3.bin")).unwrap();
    file3.write_all(&[0u8; 512]).unwrap();

    let (tx, _rx) = mpsc::channel();
    let cancel = Arc::new(AtomicBool::new(false));
    let options = ScanOptions::default();

    let result = scan(&root, options, cancel, tx).expect("Scan should succeed");

    assert!(!result.cancelled);
    assert_eq!(result.errors.len(), 0);

    let root_agg = result
        .index
        .aggregate(ROOT_ID)
        .expect("Root aggregate must exist");
    // Total size should be 1024 + 2048 + 512 = 3584 bytes
    assert_eq!(root_agg.logical_total, 3584);
    // 3 files
    assert_eq!(root_agg.item_count, 3);

    // Clean up
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn test_hidden_files_detection() {
    let root = create_test_fixture("hidden");

    let hidden_dir = root.join(".hidden_cache");
    fs::create_dir_all(&hidden_dir).unwrap();
    let mut hidden_file = File::create(hidden_dir.join(".dotfile")).unwrap();
    hidden_file.write_all(b"secret").unwrap();

    let (tx, _rx) = mpsc::channel();
    let cancel = Arc::new(AtomicBool::new(false));
    let options = ScanOptions::default();

    let result = scan(&root, options, cancel, tx).expect("Scan should succeed");

    let mut found_hidden_dir = false;
    let mut found_hidden_file = false;

    for i in 0..result.index.len() {
        if let Some(entry) = result.index.entry(dustmap_core::index::EntryId(i as u32)) {
            if entry.name == ".hidden_cache" && entry.is_hidden {
                found_hidden_dir = true;
            }
            if entry.name == ".dotfile" && entry.is_hidden {
                found_hidden_file = true;
            }
        }
    }

    assert!(
        found_hidden_dir,
        "Hidden folder must be marked is_hidden=true"
    );
    assert!(
        found_hidden_file,
        "Hidden file must be marked is_hidden=true"
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn test_post_order_aggregation_invariant() {
    let root = create_test_fixture("invariant");

    let sub1 = root.join("sub1");
    let sub2 = sub1.join("sub2");
    fs::create_dir_all(&sub2).unwrap();

    let mut f1 = File::create(root.join("root_file.txt")).unwrap();
    f1.write_all(&[1u8; 100]).unwrap();

    let mut f2 = File::create(sub1.join("sub1_file.txt")).unwrap();
    f2.write_all(&[2u8; 200]).unwrap();

    let mut f3 = File::create(sub2.join("sub2_file.txt")).unwrap();
    f3.write_all(&[3u8; 300]).unwrap();

    let (tx, _rx) = mpsc::channel();
    let cancel = Arc::new(AtomicBool::new(false));
    let result = scan(&root, ScanOptions::default(), cancel, tx).unwrap();

    // Verify root aggregate == 100 + 200 + 300 = 600
    let root_agg = result.index.aggregate(ROOT_ID).unwrap();
    assert_eq!(root_agg.logical_total, 600);
    assert_eq!(root_agg.item_count, 3);

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn test_volume_enumeration_functional() {
    let volumes = list_volumes().expect("list_volumes should succeed on macOS");
    assert!(
        !volumes.is_empty(),
        "Must detect at least 1 volume (Macintosh HD / root)"
    );

    let root_vol = volumes.iter().find(|v| v.mount_point == "/");
    assert!(root_vol.is_some(), "Must have root volume mounted at /");

    let v = root_vol.unwrap();
    assert!(v.total_bytes > 0, "Root volume total_bytes must be > 0");
    assert!(
        v.available_bytes > 0,
        "Root volume available_bytes must be > 0"
    );
}

#[test]
fn test_audit_log_records_deletion_entry() {
    let log_path =
        std::env::temp_dir().join(format!("dustmap_test_audit_{}.jsonl", std::process::id()));
    let _ = fs::remove_file(&log_path);

    let audit = AuditLog::new(log_path.clone());
    let entry = AuditEntry {
        timestamp: chrono::Utc::now(),
        dustmap_version: "0.1.0".to_string(),
        targets: vec![AuditTarget {
            path: "/tmp/sample_cache".to_string(),
            logical_bytes: 52428800,
            allocated_bytes: Some(52428800),
            action: AuditAction::MovedToTrash,
            rule_id: Some("homebrew-cache".to_string()),
        }],
        bytes_before: Some(1000000000),
        bytes_after: Some(947571200),
    };

    audit
        .append(&entry)
        .expect("Append to audit log should succeed");

    let contents = fs::read_to_string(&log_path).expect("Read audit log");
    assert!(contents.contains("sample_cache"));
    assert!(contents.contains("MovedToTrash"));
    assert!(contents.contains("homebrew-cache"));

    let _ = fs::remove_file(&log_path);
}
