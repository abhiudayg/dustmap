use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleFile {
    pub id: String,
    pub name: String,
    pub owner: String,
    pub category: String,
    pub risk: String,
    pub removal: String,
    pub regenerable: bool,
    #[serde(default)]
    pub regeneration_note: Option<String>,
}

pub fn load_rule(toml_str: &str) -> anyhow::Result<RuleFile> {
    let rule: RuleFile = toml::from_str(toml_str)?;
    Ok(rule)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_rule() {
        let sample = r#"
id = "test-rule"
name = "Test Rule"
owner = "Test Owner"
category = "developer"
risk = "safe"
removal = "trash"
regenerable = true
"#;
        let rule = load_rule(sample).unwrap();
        assert_eq!(rule.id, "test-rule");
        assert_eq!(rule.risk, "safe");
    }
}
