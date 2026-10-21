use anyhow::{bail, Context, Result};
use dustmap_core::cleaner::{self, Candidate, ToolTarget};
use dustmap_core::rules::RiskLevel;
use dustmap_core::safety::is_denylisted_in;
use serde::Deserialize;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::time::{Duration, SystemTime};

const SUPPORTED_SCHEMA_MAJOR: &str = "1";
const VALIDATION_HOME: &str = "/dustmap-rule-validation-home";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Removal {
    Trash,
    ToolCommand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MatchKind {
    Directory,
    File,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Match {
    pub paths: Vec<String>,
    #[serde(rename = "type")]
    pub kind: MatchKind,
    #[serde(default)]
    pub min_age_days: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Explanation {
    pub summary: String,
    pub consequence: String,
    pub undo: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Verification {
    pub tests: Vec<String>,
    #[serde(default)]
    pub reviewed_by: Vec<String>,
    #[serde(default)]
    pub reviewed_on: Option<String>,
    pub last_checked_against: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ToolCommand {
    pub description: String,
    pub argv: Vec<String>,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    #[serde(default)]
    pub schema: Option<String>,
    pub id: String,
    pub name: String,
    pub owner: String,
    pub category: String,
    pub risk: RiskLevel,
    pub removal: Removal,
    pub regenerable: bool,
    #[serde(default)]
    pub regeneration_note: Option<String>,
    #[serde(rename = "match")]
    pub matcher: Match,
    pub explanation: Explanation,
    pub verification: Verification,
    #[serde(default)]
    pub tool_command: Vec<ToolCommand>,
}

pub fn parse_rule(toml_str: &str) -> Result<Rule> {
    toml::from_str(toml_str).context("failed to parse rule")
}

pub fn validate(rule: &Rule, repo_root: Option<&Path>) -> Vec<String> {
    let mut errors = Vec::new();
    let mut require = |field: &str, value: &str| {
        if value.trim().is_empty() {
            errors.push(format!("{field} must not be empty"));
        }
    };
    require("id", &rule.id);
    require("name", &rule.name);
    require("owner", &rule.owner);
    require("category", &rule.category);
    require("explanation.summary", &rule.explanation.summary);
    require("explanation.consequence", &rule.explanation.consequence);
    require("explanation.undo", &rule.explanation.undo);
    require(
        "verification.last_checked_against",
        &rule.verification.last_checked_against,
    );

    if !rule
        .id
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        errors.push("id must use lowercase letters, digits and hyphens".to_string());
    }
    if let Some(schema) = &rule.schema {
        if schema.split('.').next() != Some(SUPPORTED_SCHEMA_MAJOR) {
            errors.push(format!("unsupported schema version {schema}"));
        }
    }
    if rule.regenerable
        && rule
            .regeneration_note
            .as_deref()
            .unwrap_or("")
            .trim()
            .is_empty()
    {
        errors.push("regenerable rules need a regeneration_note".to_string());
    }
    if rule.risk == RiskLevel::Safe
        && (rule.verification.reviewed_by.is_empty() || rule.verification.reviewed_on.is_none())
    {
        errors.push("safe rules need reviewed_by and reviewed_on".to_string());
    }
    validate_paths(rule, &mut errors);
    validate_tests(rule, repo_root, &mut errors);
    validate_removal(rule, &mut errors);
    errors
}

fn validate_paths(rule: &Rule, errors: &mut Vec<String>) {
    if rule.matcher.paths.is_empty() {
        errors.push("match.paths must list at least one path".to_string());
    }
    for raw in &rule.matcher.paths {
        if !(raw.starts_with("~/") || raw.starts_with('/')) {
            errors.push(format!("path '{raw}' must be absolute or start with ~/"));
            continue;
        }
        if raw.contains('*') {
            errors.push(format!(
                "path '{raw}' must be literal; wildcards are not supported"
            ));
        }
        if Path::new(raw)
            .components()
            .any(|c| matches!(c, Component::ParentDir))
        {
            errors.push(format!("path '{raw}' contains a traversal segment"));
        }
        let expanded = expand(raw, Path::new(VALIDATION_HOME));
        if is_denylisted_in(&expanded, Path::new(VALIDATION_HOME)) {
            errors.push(format!("path '{raw}' is on the protected-path denylist"));
        }
    }
}

fn validate_tests(rule: &Rule, repo_root: Option<&Path>, errors: &mut Vec<String>) {
    if rule.verification.tests.is_empty() {
        errors.push("verification.tests must name at least one fixture".to_string());
    }
    for fixture in &rule.verification.tests {
        if repo_root.is_some_and(|root| !root.join(fixture).is_dir()) {
            errors.push(format!("fixture '{fixture}' does not exist"));
        }
    }
}

fn validate_removal(rule: &Rule, errors: &mut Vec<String>) {
    match rule.removal {
        Removal::Trash if !rule.tool_command.is_empty() => {
            errors.push("trash removal must not declare tool_command entries".to_string());
        }
        Removal::ToolCommand if rule.tool_command.is_empty() => {
            errors.push("tool-command removal needs at least one tool_command entry".to_string());
        }
        Removal::ToolCommand => {
            for command in &rule.tool_command {
                if !command
                    .argv
                    .first()
                    .is_some_and(|program| program.starts_with('/'))
                {
                    errors.push(format!(
                        "tool command '{}' must start with an absolute program path",
                        command.description
                    ));
                }
                if command.timeout_seconds == 0 {
                    errors.push(format!(
                        "tool command '{}' needs a timeout",
                        command.description
                    ));
                }
                if command.argv.iter().any(|arg| arg.contains('\0')) {
                    errors.push(format!(
                        "tool command '{}' has a NUL byte",
                        command.description
                    ));
                }
            }
        }
        Removal::Trash => {}
    }
}

fn expand(raw: &str, home: &Path) -> PathBuf {
    match raw.strip_prefix("~/") {
        Some(rest) => home.join(rest),
        None => PathBuf::from(raw),
    }
}

pub fn matched_paths(rule: &Rule, home: &Path, now: SystemTime) -> Vec<PathBuf> {
    let min_age = Duration::from_secs(rule.matcher.min_age_days * 24 * 60 * 60);
    rule.matcher
        .paths
        .iter()
        .map(|raw| expand(raw, home))
        .filter(|path| matches_kind(path, rule.matcher.kind))
        .filter(|path| old_enough(path, min_age, now))
        .collect()
}

fn matches_kind(path: &Path, kind: MatchKind) -> bool {
    let Ok(meta) = fs::symlink_metadata(path) else {
        return false;
    };
    match kind {
        MatchKind::Directory => meta.is_dir(),
        MatchKind::File => meta.is_file(),
    }
}

fn old_enough(path: &Path, min_age: Duration, now: SystemTime) -> bool {
    if min_age.is_zero() {
        return true;
    }
    fs::symlink_metadata(path)
        .and_then(|meta| meta.modified())
        .ok()
        .and_then(|modified| now.duration_since(modified).ok())
        .is_some_and(|age| age >= min_age)
}

pub fn load_valid_rules(dir: &Path, repo_root: Option<&Path>) -> Result<Vec<Rule>> {
    let mut rules = Vec::new();
    let mut errors = Vec::new();
    for path in rule_files(dir)? {
        let text = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        match parse_rule(&text) {
            Ok(rule) => {
                for error in validate(&rule, repo_root) {
                    errors.push(format!("{}: {error}", path.display()));
                }
                rules.push(rule);
            }
            Err(err) => errors.push(format!("{}: {err:#}", path.display())),
        }
    }
    if !errors.is_empty() {
        bail!("invalid rules bundle: {}", errors.join("; "));
    }
    Ok(rules)
}

pub enum RuleSource {
    Primary,
    Fallback,
}

pub fn load_rules_with_fallback(
    primary: &Path,
    fallback: &Path,
    repo_root: Option<&Path>,
) -> Result<(Vec<Rule>, RuleSource)> {
    match load_valid_rules(primary, repo_root) {
        Ok(rules) => Ok((rules, RuleSource::Primary)),
        Err(_) => Ok((load_valid_rules(fallback, repo_root)?, RuleSource::Fallback)),
    }
}

fn rule_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut paths: Vec<PathBuf> = fs::read_dir(dir)
        .with_context(|| format!("failed to read rules directory {}", dir.display()))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<std::io::Result<_>>()?;
    paths.retain(|path| path.extension().is_some_and(|ext| ext == "toml"));
    paths.sort();
    Ok(paths)
}

pub fn collect_targets(rules: &[Rule], home: &Path) -> Result<(Vec<Candidate>, Vec<ToolTarget>)> {
    let now = SystemTime::now();
    let mut candidates = Vec::new();
    let mut tools = Vec::new();
    for rule in rules {
        match rule.removal {
            Removal::Trash => {
                for path in matched_paths(rule, home, now) {
                    candidates.push(cleaner::candidate_for(&path, &rule.id, rule.risk)?);
                }
            }
            Removal::ToolCommand => {
                for command in &rule.tool_command {
                    tools.push(ToolTarget {
                        rule_id: rule.id.clone(),
                        risk: rule.risk,
                        description: command.description.clone(),
                        argv: command.argv.clone(),
                        timeout: Duration::from_secs(command.timeout_seconds),
                    });
                }
            }
        }
    }
    Ok((candidates, tools))
}
