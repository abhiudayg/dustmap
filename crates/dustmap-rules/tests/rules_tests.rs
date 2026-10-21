use dustmap_rules::{
    load_rules_with_fallback, load_valid_rules, matched_paths, parse_rule, validate, RuleSource,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn temp_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "dustmap_rules_{}_{}_{}",
        name,
        std::process::id(),
        nanos
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn trash_rule(paths: &str, extra: &str) -> String {
    format!(
        r#"
id = "example-cache"
name = "Example Cache"
owner = "Example"
category = "developer"
risk = "review"
removal = "trash"
regenerable = true
regeneration_note = "Rebuilt on next use."
{extra}
[match]
paths = {paths}
type = "directory"

[explanation]
summary = "Cached data."
consequence = "Rebuilt on next use."
undo = "Restore from Trash."

[verification]
tests = ["fixtures/homebrew-cache/basic"]
last_checked_against = "1.0"
"#
    )
}

#[test]
fn shipped_homebrew_rule_parses_and_validates() {
    let text = fs::read_to_string(repo_root().join("rules/homebrew-cache.toml")).unwrap();

    let rule = parse_rule(&text).unwrap();

    assert!(validate(&rule, Some(&repo_root())).is_empty());
}

#[test]
fn shipped_homebrew_rule_matches_its_fixture_layout() {
    let text = fs::read_to_string(repo_root().join("rules/homebrew-cache.toml")).unwrap();
    let rule = parse_rule(&text).unwrap();
    let home = repo_root().join("fixtures/homebrew-cache/basic");

    let matches = matched_paths(&rule, &home, SystemTime::now());

    assert_eq!(matches, vec![home.join("Library/Caches/Homebrew")]);
}

#[test]
fn rejects_traversal_wildcards_and_relative_paths() {
    let rule = parse_rule(&trash_rule(
        r#"["~/Library/../Documents", "~/Caches/*", "relative/dir"]"#,
        "",
    ))
    .unwrap();

    let errors = validate(&rule, Some(&repo_root()));

    assert!(errors.iter().any(|e| e.contains("traversal")));
    assert!(errors.iter().any(|e| e.contains("wildcards")));
    assert!(errors
        .iter()
        .any(|e| e.contains("absolute or start with ~/")));
}

#[test]
fn rejects_paths_on_the_protected_denylist() {
    let rule = parse_rule(&trash_rule(
        r#"["~/Documents/projects", "/System/Library/Caches"]"#,
        "",
    ))
    .unwrap();

    let errors = validate(&rule, Some(&repo_root()));

    assert_eq!(errors.iter().filter(|e| e.contains("denylist")).count(), 2);
}

#[test]
fn rejects_missing_fixtures_and_safe_rules_without_review() {
    let text = trash_rule(r#"["~/Library/Caches/Example"]"#, "")
        .replace("risk = \"review\"", "risk = \"safe\"")
        .replace("fixtures/homebrew-cache/basic", "fixtures/does-not-exist");
    let rule = parse_rule(&text).unwrap();

    let errors = validate(&rule, Some(&repo_root()));

    assert!(errors.iter().any(|e| e.contains("does not exist")));
    assert!(errors.iter().any(|e| e.contains("reviewed_by")));
}

#[test]
fn rejects_unsupported_schema_major_version() {
    let rule = parse_rule(&trash_rule(
        r#"["~/Library/Caches/Example"]"#,
        "schema = \"2.0\"\n",
    ))
    .unwrap();

    let errors = validate(&rule, Some(&repo_root()));

    assert!(errors.iter().any(|e| e.contains("unsupported schema")));
}

#[test]
fn tool_command_rules_need_absolute_program_paths_and_timeouts() {
    let good = trash_rule(r#"["~/Library/Caches/Example"]"#, "")
        .replace("removal = \"trash\"", "removal = \"tool-command\"")
        .replace(
            "[match]",
            "[[tool_command]]\ndescription = \"clean\"\nargv = [\"/usr/bin/true\"]\ntimeout_seconds = 30\n\n[match]",
        );
    assert!(validate(&parse_rule(&good).unwrap(), Some(&repo_root())).is_empty());

    let bad = good.replace("argv = [\"/usr/bin/true\"]", "argv = [\"true\"]");
    let errors = validate(&parse_rule(&bad).unwrap(), Some(&repo_root()));
    assert!(errors.iter().any(|e| e.contains("absolute program path")));

    let mismatched = trash_rule(r#"["~/Library/Caches/Example"]"#, "")
        .replace(
            "[match]",
            "[[tool_command]]\ndescription = \"clean\"\nargv = [\"/usr/bin/true\"]\ntimeout_seconds = 30\n\n[match]",
        );
    let errors = validate(&parse_rule(&mismatched).unwrap(), Some(&repo_root()));
    assert!(errors
        .iter()
        .any(|e| e.contains("must not declare tool_command")));
}

#[test]
fn min_age_excludes_recently_modified_directories() {
    let home = temp_dir("min_age");
    fs::create_dir_all(home.join("Library/Caches/Example")).unwrap();
    let text = trash_rule(r#"["~/Library/Caches/Example"]"#, "").replace(
        "type = \"directory\"",
        "type = \"directory\"\nmin_age_days = 30",
    );
    let rule = parse_rule(&text).unwrap();

    assert!(matched_paths(&rule, &home, SystemTime::now()).is_empty());
}

#[test]
fn invalid_primary_bundle_falls_back_to_bundled_rules() {
    let primary = temp_dir("primary_bundle");
    fs::write(primary.join("broken.toml"), "id = 3\n").unwrap();
    let fallback = repo_root().join("rules");

    let (rules, source) =
        load_rules_with_fallback(&primary, &fallback, Some(&repo_root())).unwrap();

    assert!(matches!(source, RuleSource::Fallback));
    assert!(rules.iter().any(|rule| rule.id == "homebrew-cache"));
}

#[test]
fn load_valid_rules_reports_every_invalid_rule_in_one_error() {
    let dir = temp_dir("invalid_bundle");
    fs::write(dir.join("a.toml"), trash_rule(r#"["/usr/bin"]"#, "")).unwrap();
    fs::write(dir.join("b.toml"), "not = [valid").unwrap();

    let err = load_valid_rules(&dir, Some(&repo_root()))
        .unwrap_err()
        .to_string();

    assert!(err.contains("a.toml"));
    assert!(err.contains("b.toml"));
}
