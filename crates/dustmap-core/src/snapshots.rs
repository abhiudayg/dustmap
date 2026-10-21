use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

const TMUTIL: &str = "/usr/bin/tmutil";
const SNAPSHOT_PREFIX: &str = "com.apple.TimeMachine.";

pub const REMOVAL_GUIDANCE: &str = "Deleting local Time Machine snapshots requires administrator \
privileges and cannot be undone. DustMap does not run this command. To remove one yourself, \
run 'tmutil deletelocalsnapshots <date>' in Terminal.";

pub fn list_local_snapshots(volume: &Path) -> Result<Vec<String>> {
    let output = Command::new(TMUTIL)
        .arg("listlocalsnapshots")
        .arg(volume)
        .output()
        .context("failed to run tmutil")?;
    if !output.status.success() {
        bail!("tmutil listlocalsnapshots exited with {}", output.status);
    }
    Ok(parse_local_snapshots(&String::from_utf8_lossy(
        &output.stdout,
    )))
}

pub fn parse_local_snapshots(output: &str) -> Vec<String> {
    output
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with(SNAPSHOT_PREFIX))
        .map(str::to_string)
        .collect()
}

pub fn snapshot_date(name: &str) -> Option<&str> {
    name.strip_prefix(SNAPSHOT_PREFIX)?.split('.').next()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_snapshot_names_and_ignores_the_header() {
        let output = "Snapshots for disk /:\n\
com.apple.TimeMachine.2026-10-20-101010.local\n\
com.apple.TimeMachine.2026-10-21-090000.local\n";

        let names = parse_local_snapshots(output);

        assert_eq!(
            names,
            vec![
                "com.apple.TimeMachine.2026-10-20-101010.local",
                "com.apple.TimeMachine.2026-10-21-090000.local",
            ]
        );
    }

    #[test]
    fn reports_no_snapshots_for_empty_or_header_only_output() {
        assert!(parse_local_snapshots("").is_empty());
        assert!(parse_local_snapshots("Snapshots for disk /:\n").is_empty());
    }

    #[test]
    fn extracts_snapshot_date_when_present() {
        assert_eq!(
            snapshot_date("com.apple.TimeMachine.2026-10-20-101010.local"),
            Some("2026-10-20-101010")
        );
        assert_eq!(snapshot_date("something-else"), None);
    }
}
