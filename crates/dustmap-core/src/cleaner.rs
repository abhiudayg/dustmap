/// Cleaner — stub implementation (post-M1).
/// Orchestrates rule matching and deletion for a set of paths.
pub struct CleanerConfig {
    pub dry_run: bool,
    pub permanent: bool,
}

impl Default for CleanerConfig {
    fn default() -> Self {
        Self {
            dry_run: true,
            permanent: false,
        }
    }
}
