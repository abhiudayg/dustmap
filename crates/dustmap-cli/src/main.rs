use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use dustmap_core::audit::{AuditAction, AuditEntry, AuditLog};
use dustmap_core::cleaner::{self, ItemOutcome, Preview, RejectReason};
use dustmap_core::data::{self, APP_SUPPORT_RELATIVE};
use dustmap_core::journal::{self, ItemState};
use dustmap_core::remnants;
use dustmap_core::rules::RiskLevel;
use dustmap_core::snapshots;
use dustmap_core::trash::move_to_trash;
use dustmap_rules::{collect_targets, load_rules_with_fallback, validate, Rule, RuleSource};
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::{mpsc, Arc};

#[derive(Parser)]
#[command(
    name = "dustmap",
    about = "DustMap: blazing-fast disk space analyzer",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan a directory and report disk usage
    Scan {
        path: PathBuf,
        #[arg(long, default_value_t = false)]
        json: bool,
        #[arg(long, default_value = "logical")]
        metric: String,
    },
    /// Print a summary of disk usage for a path
    Summary { path: PathBuf },
    /// Preview or execute cleanup rules (dry run unless --execute)
    Clean {
        #[arg(long, default_value = "rules")]
        rules: PathBuf,
        #[arg(long, default_value = "rules")]
        fallback_rules: PathBuf,
        #[arg(long = "rule")]
        rule_ids: Vec<String>,
        #[arg(long, default_value_t = false)]
        execute: bool,
        #[arg(long, default_value_t = false)]
        yes: bool,
    },
    /// List or validate cleanup rules
    Rules {
        #[command(subcommand)]
        command: RulesCommand,
    },
    /// Show the local audit log
    Audit {
        #[arg(long)]
        since: Option<String>,
    },
    /// Report local Time Machine snapshots (report only)
    Snapshots { volume: PathBuf },
    /// Dry-run remnant detection for an uninstalled app
    Remnants {
        bundle_id: String,
        #[arg(long = "apps")]
        app_dirs: Vec<PathBuf>,
    },
    /// Journal recovery and quarantine handling
    Journal {
        #[command(subcommand)]
        command: JournalCommand,
    },
    /// List or remove DustMap's own data
    Data {
        #[command(subcommand)]
        command: DataCommand,
    },
}

#[derive(Subcommand)]
enum RulesCommand {
    /// List loaded rules
    List {
        #[arg(long, default_value = "rules")]
        rules: PathBuf,
    },
    /// Validate every rule file in a directory
    Validate {
        #[arg(long, default_value = "rules")]
        rules: PathBuf,
        #[arg(long)]
        repo_root: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum JournalCommand {
    /// Reconcile unfinished cleanup items from filesystem state
    Reconcile,
    /// Acknowledge quarantined journals so cleanup can resume
    Ack,
}

#[derive(Subcommand)]
enum DataCommand {
    /// List every file DustMap has written, with sizes
    List,
    /// Remove the named data entries
    Remove {
        labels: Vec<String>,
        #[arg(long, default_value_t = false)]
        yes: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Scan {
            path,
            json,
            metric: _,
        } => scan_command(&path, json)?,
        Commands::Summary { path } => summary_command(&path)?,
        Commands::Clean {
            rules,
            fallback_rules,
            rule_ids,
            execute,
            yes,
        } => clean_command(&rules, &fallback_rules, &rule_ids, execute, yes)?,
        Commands::Rules { command } => rules_command(command)?,
        Commands::Audit { since } => audit_command(since.as_deref())?,
        Commands::Snapshots { volume } => snapshots_command(&volume)?,
        Commands::Remnants {
            bundle_id,
            app_dirs,
        } => remnants_command(&bundle_id, app_dirs)?,
        Commands::Journal { command } => journal_command(command)?,
        Commands::Data { command } => data_command(command)?,
    }

    Ok(())
}

fn home_dir() -> Result<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .context("HOME is not set")
}

fn support_dir(home: &Path) -> PathBuf {
    home.join(APP_SUPPORT_RELATIVE)
}

fn scan_command(path: &Path, json: bool) -> Result<()> {
    eprintln!("Scanning {}...", path.display());
    let cancel = Arc::new(AtomicBool::new(false));
    let (tx, rx) = mpsc::channel();

    let options = dustmap_core::scanner::ScanOptions::default();
    let cancel_clone = cancel.clone();
    let path_clone = path.to_path_buf();

    let handle = std::thread::spawn(move || {
        dustmap_core::scanner::scan(&path_clone, options, cancel_clone, tx)
    });

    while let Ok(progress) = rx.recv() {
        eprint!(
            "\r  {} entries | {} | {:.1}s",
            progress.entries_seen,
            dustmap_core::metrics::format_size(progress.bytes_accounted),
            progress.elapsed_secs
        );
    }

    let result = handle
        .join()
        .map_err(|_| anyhow::anyhow!("Scanner thread panicked"))??;
    let root_agg = result.index.aggregate(dustmap_core::index::ROOT_ID);

    if json {
        let summary = serde_json::json!({
            "root": path.display().to_string(),
            "total_logical": root_agg.map(|a| a.logical_total).unwrap_or(0),
            "total_allocated": root_agg.map(|a| a.allocated_total).unwrap_or(0),
            "total_items": root_agg.map(|a| a.item_count).unwrap_or(0),
            "errors": result.errors.len(),
            "duration_secs": result.duration.as_secs_f64(),
            "cancelled": result.cancelled,
        });
        println!("\n{}", serde_json::to_string_pretty(&summary)?);
    } else {
        let agg = root_agg.cloned().unwrap_or_default();
        println!(
            "\n\nRoot: {}\nTotal size: {}\nItems: {}\nErrors: {}\nDuration: {:.1}s",
            path.display(),
            dustmap_core::metrics::format_size(agg.logical_total),
            agg.item_count,
            result.errors.len(),
            result.duration.as_secs_f64(),
        );
    }
    Ok(())
}

fn summary_command(path: &Path) -> Result<()> {
    println!("Summary for: {}", path.display());
    let cancel = Arc::new(AtomicBool::new(false));
    let (tx, _rx) = mpsc::channel();
    let options = dustmap_core::scanner::ScanOptions::default();
    let result = dustmap_core::scanner::scan(path, options, cancel, tx)?;
    let agg = result
        .index
        .aggregate(dustmap_core::index::ROOT_ID)
        .cloned()
        .unwrap_or_default();
    println!(
        "Total Size: {}",
        dustmap_core::metrics::format_size(agg.logical_total)
    );
    println!("Total Items: {}", agg.item_count);
    Ok(())
}

fn load_rules(rules: &Path, fallback: &Path) -> Result<Vec<Rule>> {
    let (rules, source) = load_rules_with_fallback(rules, fallback, None)?;
    if matches!(source, RuleSource::Fallback) {
        eprintln!("Warning: rules bundle is invalid; using fallback rules.");
    }
    Ok(rules)
}

fn clean_command(
    rules_dir: &Path,
    fallback_dir: &Path,
    rule_ids: &[String],
    execute: bool,
    yes: bool,
) -> Result<()> {
    let home = home_dir()?;
    let support = support_dir(&home);
    let journal_dir = support.join("journal");
    let audit = AuditLog::new(support.join("audit.jsonl"));

    let reconciled = journal::reconcile(&journal_dir, &home)?;
    for item in reconciled.items.iter().filter(|i| i.needs_user_review) {
        println!(
            "Needs review from an earlier run: {} item {} ({:?})",
            item.operation_id, item.item_id, item.state
        );
    }
    let blocked = journal::unacknowledged_quarantine(&journal_dir)?;
    if !blocked.is_empty() {
        bail!(
            "{} corrupt journal(s) are quarantined; run `dustmap journal ack` after reviewing them",
            blocked.len()
        );
    }

    let rules: Vec<Rule> = load_rules(rules_dir, fallback_dir)?
        .into_iter()
        .filter(|rule| rule_ids.is_empty() || rule_ids.contains(&rule.id))
        .collect();
    let (candidates, tools) = collect_targets(&rules, &home)?;
    let preview = cleaner::preview(
        &format!("cli-{}", chrono::Utc::now().timestamp()),
        candidates,
        tools,
        &home,
    );
    print_preview(&preview);

    if !execute {
        println!("\nDry run. Re-run with --execute to move these items to Trash.");
        return Ok(());
    }
    if preview.targets.is_empty() && preview.tool_targets.is_empty() {
        println!("Nothing to clean.");
        return Ok(());
    }
    if !yes
        && !confirm(&format!(
            "Move {} item(s) and run {} tool command(s)?",
            preview.targets.len(),
            preview.tool_targets.len()
        ))?
    {
        println!("Cancelled.");
        return Ok(());
    }

    let operation_id = uuid::Uuid::new_v4().to_string();
    let results = cleaner::execute(
        &operation_id,
        &preview,
        &journal_dir,
        &audit,
        &home,
        move_to_trash,
    )?;
    println!("\nOperation {operation_id}");
    for result in &results {
        println!(
            "  {} {} -> {}",
            result.item_id,
            result
                .path
                .as_deref()
                .unwrap_or(Path::new(&result.rule_id))
                .display(),
            describe_outcome(&result.outcome)
        );
    }
    Ok(())
}

fn print_preview(preview: &Preview) {
    println!("Preview ({} item(s)):", preview.targets.len());
    let mut total = 0u64;
    for candidate in &preview.targets {
        total += candidate.logical_bytes;
        println!(
            "  [{}] {} ({}) {}",
            risk_label(candidate.risk),
            candidate.path.display(),
            dustmap_core::metrics::format_size(candidate.logical_bytes),
            candidate.rule_id
        );
    }
    for target in &preview.tool_targets {
        println!(
            "  [{}] tool: {} {}",
            risk_label(target.risk),
            target.argv.join(" "),
            target.rule_id
        );
    }
    println!(
        "Estimated size: {}",
        dustmap_core::metrics::format_size(total)
    );
    for rejection in &preview.rejected {
        println!(
            "  rejected {}: {}",
            rejection.label,
            match rejection.reason {
                RejectReason::Denylisted => "protected path",
                RejectReason::ReportOnly => "report only",
            }
        );
    }
}

fn risk_label(risk: RiskLevel) -> &'static str {
    match risk {
        RiskLevel::Safe => "safe",
        RiskLevel::Review => "review",
        RiskLevel::Caution => "caution",
        RiskLevel::ReportOnly => "report-only",
    }
}

fn describe_outcome(outcome: &ItemOutcome) -> String {
    match outcome {
        ItemOutcome::MovedToTrash => "moved to Trash".to_string(),
        ItemOutcome::RemovedByTool => "removed by tool".to_string(),
        ItemOutcome::TimedOut => "timed out".to_string(),
        ItemOutcome::SkippedStale => "skipped: changed since preview".to_string(),
        ItemOutcome::SkippedProtected => "skipped: protected path".to_string(),
        ItemOutcome::Failed { reason } => format!("failed: {reason}"),
    }
}

fn confirm(question: &str) -> Result<bool> {
    print!("{question} Type 'yes' to continue: ");
    io::stdout().flush()?;
    let mut answer = String::new();
    io::stdin().lock().read_line(&mut answer)?;
    Ok(answer.trim() == "yes")
}

fn rules_command(command: RulesCommand) -> Result<()> {
    match command {
        RulesCommand::List { rules } => {
            for rule in load_rules(&rules, &rules)? {
                println!("{}\t{}\t{}", rule.id, risk_label(rule.risk), rule.name);
            }
        }
        RulesCommand::Validate { rules, repo_root } => {
            let mut failures = 0;
            let mut files: Vec<PathBuf> = std::fs::read_dir(&rules)?
                .map(|entry| entry.map(|entry| entry.path()))
                .collect::<std::io::Result<_>>()?;
            files.retain(|path| path.extension().is_some_and(|ext| ext == "toml"));
            files.sort();
            for file in files {
                let text = std::fs::read_to_string(&file)?;
                let errors = match dustmap_rules::parse_rule(&text) {
                    Ok(rule) => validate(&rule, repo_root.as_deref()),
                    Err(err) => vec![format!("{err:#}")],
                };
                if errors.is_empty() {
                    println!("ok    {}", file.display());
                } else {
                    failures += 1;
                    for error in errors {
                        println!("FAIL  {}: {error}", file.display());
                    }
                }
            }
            if failures > 0 {
                bail!("{failures} rule file(s) failed validation");
            }
        }
    }
    Ok(())
}

fn audit_command(since: Option<&str>) -> Result<()> {
    let home = home_dir()?;
    let path = support_dir(&home).join("audit.jsonl");
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            println!("No audit entries.");
            return Ok(());
        }
        Err(err) => return Err(err.into()),
    };
    let since = since
        .map(|date| chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d"))
        .transpose()
        .context("--since must be YYYY-MM-DD")?;
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let entry: AuditEntry = serde_json::from_str(line)?;
        if since.is_some_and(|since| entry.timestamp.date_naive() < since) {
            continue;
        }
        println!(
            "{}  dustmap {}  {} target(s)",
            entry.timestamp.to_rfc3339(),
            entry.dustmap_version,
            entry.targets.len()
        );
        for target in &entry.targets {
            println!("    {}  {}", audit_label(&target.action), target.path);
        }
    }
    Ok(())
}

fn audit_label(action: &AuditAction) -> &'static str {
    match action {
        AuditAction::MovedToTrash => "moved_to_trash",
        AuditAction::RemovedByTool => "removed_by_tool",
        AuditAction::DeletedPermanently => "deleted_permanently",
        AuditAction::Skipped => "skipped",
        AuditAction::Stale => "stale",
        AuditAction::Failed { .. } => "failed",
    }
}

fn snapshots_command(volume: &Path) -> Result<()> {
    let names = snapshots::list_local_snapshots(volume)?;
    if names.is_empty() {
        println!("No local Time Machine snapshots on {}.", volume.display());
    }
    for name in &names {
        match snapshots::snapshot_date(name) {
            Some(date) => println!("{name}  ({date})"),
            None => println!("{name}"),
        }
    }
    println!("\n{}", snapshots::REMOVAL_GUIDANCE);
    Ok(())
}

fn remnants_command(bundle_id: &str, app_dirs: Vec<PathBuf>) -> Result<()> {
    let home = home_dir()?;
    let app_dirs = if app_dirs.is_empty() {
        vec![PathBuf::from("/Applications"), home.join("Applications")]
    } else {
        app_dirs
    };
    let installed = remnants::installed_bundle_ids(&app_dirs)?;
    let found = remnants::find_remnants(bundle_id, &home, &installed)?;
    if found.is_empty() {
        println!("No remnants offered for {bundle_id}.");
    }
    for remnant in &found {
        println!(
            "[{}] {}  ({})\n    {}",
            risk_label(remnant.risk),
            remnant.path.display(),
            remnant.location,
            remnant.note
        );
    }
    println!("\nDry run: nothing was removed.");
    Ok(())
}

fn journal_command(command: JournalCommand) -> Result<()> {
    let home = home_dir()?;
    let journal_dir = support_dir(&home).join("journal");
    match command {
        JournalCommand::Reconcile => {
            let report = journal::reconcile(&journal_dir, &home)?;
            for item in &report.items {
                println!(
                    "{} {}: {}{}",
                    item.operation_id,
                    item.item_id,
                    describe_state(item.state),
                    if item.needs_user_review {
                        " (needs review)"
                    } else {
                        ""
                    }
                );
            }
            for path in &report.quarantined {
                println!("quarantined: {}", path.display());
            }
            if report.items.is_empty() && report.quarantined.is_empty() {
                println!("Nothing to reconcile.");
            }
        }
        JournalCommand::Ack => {
            journal::acknowledge_quarantine(&journal_dir)?;
            println!("Quarantine acknowledged.");
        }
    }
    Ok(())
}

fn describe_state(state: ItemState) -> &'static str {
    match state {
        ItemState::MovedToTrash => "moved to Trash",
        ItemState::NotMoved => "not moved",
        ItemState::Unknown => "unknown",
    }
}

fn data_command(command: DataCommand) -> Result<()> {
    let home = home_dir()?;
    match command {
        DataCommand::List => {
            for entry in data::data_entries(&home) {
                println!(
                    "{}\t{}\t{}",
                    entry.label,
                    dustmap_core::metrics::format_size(entry.bytes),
                    entry.path.display()
                );
            }
        }
        DataCommand::Remove { labels, yes } => {
            if labels.is_empty() {
                bail!("name at least one data label to remove (see `dustmap data list`)");
            }
            let labels_ref: Vec<&str> = labels.iter().map(String::as_str).collect();
            if !yes && !confirm(&format!("Remove {}?", labels.join(", ")))? {
                println!("Cancelled.");
                return Ok(());
            }
            let removed = data::remove_data(&home, &labels_ref)?;
            println!("Removed: {}", removed.join(", "));
        }
    }
    Ok(())
}
