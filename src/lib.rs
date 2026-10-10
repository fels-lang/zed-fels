use zed_extension_api::{self as zed, settings::LspSettings, LanguageServerId, Result};

struct GabbroExtension;

impl GabbroExtension {
    // The server is `gabbro lsp`, a subcommand of the compiler. The path can be set in the Zed
    // settings under lsp.gabbro.binary.path; otherwise `gabbro` is looked up on the PATH.
    fn binary_path(&self, worktree: &zed::Worktree) -> Result<String> {
        if let Ok(settings) = LspSettings::for_worktree("gabbro", worktree) {
            if let Some(path) = settings.binary.and_then(|binary| binary.path) {
                return Ok(path);
            }
        }
        worktree.which("gabbro").ok_or_else(|| {
            "gabbro was not found on the PATH. Install Gabbro, or set lsp.gabbro.binary.path in the Zed settings."
                .to_string()
        })
    }
}

impl zed::Extension for GabbroExtension {
    fn new() -> Self {
        GabbroExtension
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        Ok(zed::Command {
            command: self.binary_path(worktree)?,
            args: vec!["lsp".to_string()],
            env: worktree.shell_env(),
        })
    }
}

zed::register_extension!(GabbroExtension);
