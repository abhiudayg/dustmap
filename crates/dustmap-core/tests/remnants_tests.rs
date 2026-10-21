use dustmap_core::remnants::{find_remnants, installed_bundle_ids, MatchKind};
use dustmap_core::rules::RiskLevel;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

fn temp_dir(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "dustmap_remnants_{}_{}_{}",
        name,
        std::process::id(),
        nanos
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn touch(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, b"data").unwrap();
}

#[test]
fn finds_exact_and_suffixed_entries_and_ignores_name_only_matches() {
    let home = temp_dir("matches");
    let id = "com.example.app";
    touch(&home.join("Library/Caches/com.example.app/cache.db"));
    touch(&home.join("Library/Preferences/com.example.app.plist"));
    touch(&home.join("Library/Application Support/com.example.app/store.sqlite"));
    touch(&home.join("Library/Logs/com.example.app/run.log"));
    touch(&home.join("Library/Caches/com.example.appother/cache.db"));
    touch(&home.join("Library/Caches/Notes/cache.db"));

    let remnants = find_remnants(id, &home, &HashSet::new()).unwrap();

    let locations: Vec<&str> = remnants.iter().map(|r| r.location).collect();
    assert_eq!(remnants.len(), 4);
    assert!(locations.contains(&"Library/Caches"));
    assert!(locations.contains(&"Library/Preferences"));
    assert!(locations.contains(&"Library/Application Support"));
    assert!(locations.contains(&"Library/Logs"));
    assert!(remnants
        .iter()
        .all(|r| r.path.to_string_lossy().contains("com.example.app")));
    assert!(remnants
        .iter()
        .all(|r| !r.path.to_string_lossy().contains("appother") && !r.path.ends_with("Notes")));
}

#[test]
fn assigns_caution_to_application_support_and_review_to_caches() {
    let home = temp_dir("risk");
    let id = "com.example.app";
    touch(&home.join("Library/Application Support/com.example.app/store.sqlite"));
    touch(&home.join("Library/Caches/com.example.app/cache.db"));

    let remnants = find_remnants(id, &home, &HashSet::new()).unwrap();

    let support = remnants
        .iter()
        .find(|r| r.location == "Library/Application Support")
        .unwrap();
    let cache = remnants
        .iter()
        .find(|r| r.location == "Library/Caches")
        .unwrap();
    assert_eq!(support.risk, RiskLevel::Caution);
    assert_eq!(cache.risk, RiskLevel::Review);
    assert!(!support.note.is_empty());
    assert_eq!(support.match_kind, MatchKind::ExactId);
}

#[test]
fn matches_preferences_plist_as_identifier_with_suffix() {
    let home = temp_dir("plist");
    touch(&home.join("Library/Preferences/com.example.app.plist"));

    let remnants = find_remnants("com.example.app", &home, &HashSet::new()).unwrap();

    assert_eq!(remnants.len(), 1);
    assert_eq!(remnants[0].match_kind, MatchKind::IdWithSuffix);
    assert_eq!(remnants[0].risk, RiskLevel::Review);
}

#[test]
fn never_offers_remnants_of_an_app_that_is_still_installed() {
    let home = temp_dir("installed");
    touch(&home.join("Library/Caches/com.example.app/cache.db"));
    let installed: HashSet<String> = ["com.example.app".to_string()].into_iter().collect();

    let remnants = find_remnants("com.example.app", &home, &installed).unwrap();

    assert!(remnants.is_empty());
}

#[test]
fn never_offers_apple_identifiers() {
    let home = temp_dir("apple");
    touch(&home.join("Library/Caches/com.apple.Notes/cache.db"));

    let remnants = find_remnants("com.apple.Notes", &home, &HashSet::new()).unwrap();

    assert!(remnants.is_empty());
}

#[test]
fn missing_library_directories_report_nothing_without_error() {
    let home = temp_dir("empty");

    let remnants = find_remnants("com.example.app", &home, &HashSet::new()).unwrap();

    assert!(remnants.is_empty());
}

#[test]
fn reads_bundle_identifiers_from_installed_app_info_plists() {
    let apps = temp_dir("apps");
    let info = apps.join("Fake.app/Contents/Info.plist");
    fs::create_dir_all(info.parent().unwrap()).unwrap();
    fs::write(
        &info,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleIdentifier</key><string>com.example.fake</string>
</dict></plist>"#,
    )
    .unwrap();

    let ids = installed_bundle_ids(&[apps.clone(), apps.join("missing")]).unwrap();

    assert!(ids.contains("com.example.fake"));
    assert_eq!(ids.len(), 1);
}
