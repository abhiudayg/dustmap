use dustmap_core::data::{
    data_entries, interrupted_scan, mark_scan_finished, mark_scan_started, remove_data,
    DATA_ENTRIES,
};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

fn temp_dir(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "dustmap_data_{}_{}_{}",
        name,
        std::process::id(),
        nanos
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn cask_zap_paths() -> BTreeSet<String> {
    let text = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packaging/homebrew/Casks/dustmap.rb"),
    )
    .unwrap();
    let block = text
        .split("zap trash: [")
        .nth(1)
        .unwrap()
        .split(']')
        .next()
        .unwrap();
    block
        .split(',')
        .map(|item| item.trim().trim_matches('"').to_string())
        .filter(|item| !item.is_empty())
        .collect()
}

#[test]
fn cask_zap_stanza_matches_in_app_data_list() {
    let expected: BTreeSet<String> = DATA_ENTRIES
        .iter()
        .map(|(_, relative)| format!("~/{relative}"))
        .collect();

    assert_eq!(cask_zap_paths(), expected);
}

#[test]
fn data_entries_list_only_files_that_exist_with_sizes() {
    let home = temp_dir("entries");
    let settings = home.join("Library/Application Support/com.dustmap.desktop/settings.json");
    fs::create_dir_all(settings.parent().unwrap()).unwrap();
    fs::write(&settings, b"{}").unwrap();

    let entries = data_entries(&home);

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].label, "settings");
    assert_eq!(entries[0].bytes, 2);
}

#[test]
fn remove_data_removes_only_the_labels_it_is_given() {
    let home = temp_dir("remove");
    let support = home.join("Library/Application Support/com.dustmap.desktop");
    fs::create_dir_all(support.join("journal")).unwrap();
    fs::write(support.join("settings.json"), b"{}").unwrap();
    fs::write(support.join("audit.jsonl"), b"{}\n").unwrap();

    let removed = remove_data(&home, &["settings"]).unwrap();

    assert_eq!(removed, vec!["settings"]);
    assert!(!support.join("settings.json").exists());
    assert!(support.join("audit.jsonl").exists());
    assert!(support.join("journal").exists());
}

#[test]
fn interrupted_scan_is_reported_until_the_scan_finishes() {
    let state = temp_dir("scan_state");
    assert_eq!(interrupted_scan(&state).unwrap(), None);

    mark_scan_started(&state, Path::new("/Users/someone/Downloads")).unwrap();
    assert_eq!(
        interrupted_scan(&state).unwrap(),
        Some(PathBuf::from("/Users/someone/Downloads"))
    );

    mark_scan_finished(&state).unwrap();
    assert_eq!(interrupted_scan(&state).unwrap(), None);
    mark_scan_finished(&state).unwrap();
}
