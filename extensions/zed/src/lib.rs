use zed_extension_api as zed;
use zed_extension_api::{Command, LanguageServerId, Worktree};

const DEFAULT_CLI_PATH: &str = "mars";
#[allow(dead_code)]
const DEFAULT_EXECUTION_TIMEOUT_MILLIS: u64 = 30_000;

pub struct MarsExtension;

impl MarsExtension {
    pub fn cli_path() -> String {
        DEFAULT_CLI_PATH.to_string()
    }

    pub fn build_command(cli_path: &str) -> Command {
        Command {
            command: cli_path.to_string(),
            args: vec!["kernel".to_string(), "serve".to_string()],
            env: vec![],
        }
    }
}

impl zed::Extension for MarsExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> zed::Result<Command> {
        let cli_path = Self::cli_path();

        // Check if the Mars CLI is available on PATH
        if worktree.which(&cli_path).is_none() {
            return Err(format!(
                "Mars CLI not found at '{}'. Install the Mars CLI and ensure it is on your PATH.",
                cli_path
            ));
        }

        Ok(Self::build_command(&cli_path))
    }
}

zed::register_extension!(MarsExtension);

#[cfg(test)]
mod tests {
    use super::*;
    use zed_extension_api::Extension;

    #[test]
    fn cli_path_defaults_to_mars() {
        assert_eq!(MarsExtension::cli_path(), "mars");
    }

    #[test]
    fn build_command_has_expected_structure() {
        let command = MarsExtension::build_command("mars");
        assert_eq!(command.command, "mars");
        assert_eq!(command.args, vec!["kernel", "serve"]);
        assert!(command.env.is_empty());
    }

    #[test]
    fn new_creates_extension() {
        let _ext = MarsExtension::new();
    }
}
