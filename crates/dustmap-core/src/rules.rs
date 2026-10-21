use serde::{Deserialize, Serialize};

/// Risk level for a cleanup rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RiskLevel {
    Safe,
    Review,
    Caution,
    ReportOnly,
}

/// A single cleanup rule definition.
pub struct Rule {
    pub id: String,
    pub name: String,
    pub risk: RiskLevel,
}
