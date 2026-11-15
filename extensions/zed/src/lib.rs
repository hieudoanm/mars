use zed_extension_api as zed;

/// Project Mars ships a Tree-sitter grammar and language configuration for `.mars`
/// notebooks, plus an MCP context server (`mars mcp serve`) that exposes
/// notebook/cell/kernel tools to Zed's Agent Panel.
///
/// Notebook cell execution is intentionally not wired here: the
/// `mars kernel serve` protocol is not LSP, and Zed has no notebook UI, so
/// running cells is left to the Project Mars desktop app, the VS Code extension, or
/// the `mars` CLI (`mars run <file.mars>`).
pub struct MarsExtension;

impl zed::Extension for MarsExtension {
    fn new() -> Self {
        Self
    }

    /// Starts the Project Mars MCP server (`mars mcp serve`) as a Zed context server,
    /// giving the Agent Panel tools for reading, editing, and executing
    /// `.mars` notebooks. Requires the `mars` CLI on PATH.
    fn context_server_command(
        &mut self,
        context_server_id: &zed::ContextServerId,
        _project: &zed::Project,
    ) -> zed::Result<zed::Command> {
        match context_server_id.as_ref() {
            "mars" => Ok(zed::Command::new("mars").args(["mcp", "serve"])),
            other => Err(format!("unknown context server: {other}")),
        }
    }
}

zed::register_extension!(MarsExtension);

#[cfg(test)]
mod tests {
    use super::*;
    use zed_extension_api::Extension;

    #[test]
    fn new_creates_extension() {
        let _ext = MarsExtension::new();
    }
}
