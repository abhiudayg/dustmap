use crate::rules::RiskLevel;
use crate::safety::is_denylisted_in;
use anyhow::{Context, Result};
use std::collections::HashSet;
use std::ffi::OsString;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::Command;
use sysinfo::{ProcessesToUpdate, System};

const PLUTIL: &str = "/usr/bin/plutil";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchKind {
    ExactId,
    IdWithSuffix,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Remnant {
    pub bundle_id: String,
    pub path: PathBuf,
    pub location: &'static str,
    pub match_kind: MatchKind,
    pub risk: RiskLevel,
    pub note: &'static str,
}

struct Location {
    dir: &'static str,
    risk: RiskLevel,
    note: &'static str,
}

const LOCATIONS: &[Location] = &[
    Location {
        dir: "Library/Application Support",
        risk: RiskLevel::Caution,
        note: "May contain user data or activation records; removing it may require reactivation.",
    },
    Location {
        dir: "Library/Caches",
        risk: RiskLevel::Review,
        note: "Cache data that the app regenerates.",
    },
    Location {
        dir: "Library/Preferences",
        risk: RiskLevel::Review,
        note: "Settings will be lost.",
    },
    Location {
        dir: "Library/Containers",
        risk: RiskLevel::Caution,
        note: "Sandboxed app data; may contain user data or activation records.",
    },
    Location {
        dir: "Library/Logs",
        risk: RiskLevel::Review,
        note: "Log files.",
    },
    Location {
        dir: "Library/Saved Application State",
        risk: RiskLevel::Review,
        note: "Window state; open-window layout will be lost.",
    },
    Location {
        dir: "Library/HTTPStorages",
        risk: RiskLevel::Review,
        note: "HTTP cookies and storage.",
    },
    Location {
        dir: "Library/WebKit",
        risk: RiskLevel::Review,
        note: "WebKit data.",
    },
];

pub fn find_remnants(
    bundle_id: &str,
    home: &Path,
    installed: &HashSet<String>,
) -> Result<Vec<Remnant>> {
    if bundle_id.starts_with("com.apple.")
        || installed.contains(bundle_id)
        || is_bundle_running(bundle_id)
    {
        return Ok(Vec::new());
    }
    let mut remnants = Vec::new();
    for location in LOCATIONS {
        let dir = home.join(location.dir);
        let entries = match fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(err) if err.kind() == ErrorKind::NotFound => continue,
            Err(err) => return Err(err.into()),
        };
        for entry in entries {
            let entry = entry?;
            let name = entry.file_name();
            let Some(match_kind) = match_kind(&name.to_string_lossy(), bundle_id) else {
                continue;
            };
            let path = entry.path();
            if is_denylisted_in(&path, home) {
                continue;
            }
            remnants.push(Remnant {
                bundle_id: bundle_id.to_string(),
                path,
                location: location.dir,
                match_kind,
                risk: location.risk,
                note: location.note,
            });
        }
    }
    remnants.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(remnants)
}

fn match_kind(name: &str, bundle_id: &str) -> Option<MatchKind> {
    if name == bundle_id {
        Some(MatchKind::ExactId)
    } else if name
        .strip_prefix(bundle_id)
        .and_then(|rest| rest.strip_prefix('.'))
        .is_some()
    {
        Some(MatchKind::IdWithSuffix)
    } else {
        None
    }
}

pub fn installed_bundle_ids(app_dirs: &[PathBuf]) -> Result<HashSet<String>> {
    let mut ids = HashSet::new();
    for app_dir in app_dirs {
        let entries = match fs::read_dir(app_dir) {
            Ok(entries) => entries,
            Err(err) if err.kind() == ErrorKind::NotFound => continue,
            Err(err) => return Err(err.into()),
        };
        for entry in entries {
            let info_plist = entry?.path().join("Contents/Info.plist");
            if let Some(id) = bundle_identifier(&info_plist)? {
                ids.insert(id);
            }
        }
    }
    Ok(ids)
}

fn bundle_identifier(info_plist: &Path) -> Result<Option<String>> {
    if !info_plist.exists() {
        return Ok(None);
    }
    let output = Command::new(PLUTIL)
        .args(["-extract", "CFBundleIdentifier", "raw", "-o", "-"])
        .arg(info_plist)
        .output()
        .with_context(|| format!("failed to run plutil on {}", info_plist.display()))?;
    if !output.status.success() {
        return Ok(None);
    }
    let id = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok((!id.is_empty()).then_some(id))
}

pub fn is_bundle_running(bundle_id: &str) -> bool {
    let mut system = System::new();
    system.refresh_processes(ProcessesToUpdate::All, true);
    system
        .processes()
        .values()
        .any(|process| process_matches(process.exe(), process.cmd(), bundle_id))
}

fn process_matches(exe: Option<&Path>, cmd: &[OsString], bundle_id: &str) -> bool {
    exe.is_some_and(|path| mentions_identifier(&path.to_string_lossy(), bundle_id))
        || cmd
            .iter()
            .any(|arg| mentions_identifier(&arg.to_string_lossy(), bundle_id))
}

fn mentions_identifier(text: &str, bundle_id: &str) -> bool {
    text.match_indices(bundle_id).any(|(start, _)| {
        text[start + bundle_id.len()..]
            .chars()
            .next()
            .is_none_or(|next| !next.is_alphanumeric())
    })
}

pub fn recently_uninstalled(previous: &HashSet<String>, current: &HashSet<String>) -> Vec<String> {
    let mut gone: Vec<String> = previous.difference(current).cloned().collect();
    gone.sort();
    gone
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifier_match_requires_a_word_boundary() {
        assert!(mentions_identifier(
            "/Applications/Example.app/Contents/MacOS/com.example.app",
            "com.example.app"
        ));
        assert!(mentions_identifier(
            "com.example.app --helper",
            "com.example.app"
        ));
        assert!(!mentions_identifier(
            "/tmp/com.example.appother",
            "com.example.app"
        ));
        assert!(!mentions_identifier("/tmp/notes", "com.example.app"));
    }

    #[test]
    fn reports_apps_present_last_scan_but_gone_now() {
        let previous: HashSet<String> = ["com.a.one", "com.b.two", "com.c.three"]
            .into_iter()
            .map(str::to_string)
            .collect();
        let current: HashSet<String> = ["com.b.two"].into_iter().map(str::to_string).collect();

        assert_eq!(
            recently_uninstalled(&previous, &current),
            vec!["com.a.one".to_string(), "com.c.three".to_string()]
        );
    }
}
