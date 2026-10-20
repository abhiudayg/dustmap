/// Tool command integration — stub implementation (post-M1).
/// Handles launching external tools (Finder, Terminal, etc.).
pub struct ToolCmd {
    pub app: String,
    pub args: Vec<String>,
}

impl ToolCmd {
    pub fn new(app: impl Into<String>) -> Self {
        Self {
            app: app.into(),
            args: Vec::new(),
        }
    }
}
