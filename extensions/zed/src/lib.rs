use zed_extension_api as zed;

/// Project Mars ships a Tree-sitter grammar and language configuration for `.mars`
/// notebooks, plus an MCP context server (`mars mcp serve`) that exposes
/// notebook/cell/kernel tools to Zed's Agent Panel.
///
/// Notebook cell execution is intentionally not wired here: the
/// `mars kernel serve` protocol is not LSP, and Zed has no notebook UI, so
/// running cells is left to the Project Mars desktop app, the VS Code extension, or
/// the `mars` CLI (`mars notebook run <file.mars>`).
pub struct MarsExtension;

/// Builds the command for a named context server. Split out from the
/// `Extension` trait method so it can be unit-tested without a `Project`.
fn context_server_command_for(id: &str) -> zed::Result<zed::Command> {
    match id {
        "mars" => Ok(zed::Command::new("mars").args(["mcp", "serve"])),
        other => Err(format!("unknown context server: {other}")),
    }
}

impl zed::Extension for MarsExtension {
    fn new() -> Self {
        Self
    }

    /// Starts the Project Mars MCP server (`mars mcp serve`) as a Zed context server,
    /// giving the Agent Panel tools for reading, editing, and executing
    /// `.mars` notebooks. Requires the `mars` CLI on PATH.
    #[rustfmt::skip]
    fn context_server_command(&mut self, id: &zed::ContextServerId, _p: &zed::Project) -> zed::Result<zed::Command> { context_server_command_for(id.as_ref()) }
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

    #[test]
    fn mars_context_server_launches_mcp_serve() {
        let command = context_server_command_for("mars").expect("mars command");
        assert_eq!(command.command, "mars");
        assert_eq!(command.args, vec!["mcp".to_owned(), "serve".to_owned()]);
    }

    #[test]
    fn unknown_context_server_is_rejected() {
        let error = context_server_command_for("other").expect_err("unknown id");
        assert_eq!(error, "unknown context server: other");
    }
}
