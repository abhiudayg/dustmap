use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MetricKind {
    #[default]
    Logical,
    Allocated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub schema_version: u32,
    pub default_metric: MetricKind,
    pub scanner_concurrency: Option<usize>,
    pub old_download_age_days: u32,
    pub xcode_archive_age_days: u32,
    pub terminal_app: Option<String>,
    pub telemetry_enabled: bool,
    pub log_verbose: bool,
    pub user_exclusions: Vec<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            schema_version: 1,
            default_metric: MetricKind::Logical,
            scanner_concurrency: None,
            old_download_age_days: 180,
            xcode_archive_age_days: 90,
            terminal_app: None,
            telemetry_enabled: false,
            log_verbose: false,
            user_exclusions: Vec::new(),
        }
    }
}
