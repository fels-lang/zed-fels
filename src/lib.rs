use std::fs;
use zed_extension_api::{self as zed, settings::LspSettings, LanguageServerId, Result};

const REPOSITORY: &str = "gabbro-lang/gabbro";

// The language server is `gabbro lsp`, part of the compiler. It needs the standard library that sits
// next to the executable, so the extension downloads the whole release archive and runs the compiler
// from there.
struct GabbroExtension {
    cached_binary_path: Option<String>,
}

fn is_file(path: &str) -> bool {
    fs::metadata(path).map_or(false, |meta| meta.is_file())
}

impl GabbroExtension {
    // 1. the path from the Zed settings (Zed uses it on its own, but the check is cheap and keeps
    //    the order in one place), 2. `gabbro` on the PATH, 3. a copy downloaded by this extension.
    fn binary_path(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<String> {
        if let Ok(settings) = LspSettings::for_worktree("gabbro", worktree) {
            if let Some(path) = settings.binary.and_then(|binary| binary.path) {
                return Ok(path);
            }
        }
        if let Some(path) = worktree.which("gabbro") {
            return Ok(path);
        }
        if let Some(path) = &self.cached_binary_path {
            if is_file(path) {
                return Ok(path.clone());
            }
        }
        let path = self.install(language_server_id)?;
        self.cached_binary_path = Some(path.clone());
        Ok(path)
    }

    // The newest release is downloaded once into a folder named after its version. Older folders
    // are removed. If GitHub cannot be reached, a copy from an earlier run is used.
    fn install(&mut self, language_server_id: &LanguageServerId) -> Result<String> {
        let (os, arch) = zed::current_platform();
        if os != zed::Os::Windows || arch != zed::Architecture::X8664 {
            return Err(
                "Gabbro releases exist for 64 bit Windows only. Build gabbro yourself and put it on the PATH, \
                 or set lsp.gabbro.binary.path in the Zed settings."
                    .to_string(),
            );
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );
        let release = match zed::latest_github_release(
            REPOSITORY,
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        ) {
            Ok(release) => release,
            Err(error) => {
                if let Some(path) = self.existing_copy() {
                    zed::set_language_server_installation_status(
                        language_server_id,
                        &zed::LanguageServerInstallationStatus::None,
                    );
                    return Ok(path);
                }
                return Err(format!("could not look up the latest Gabbro release: {error}"));
            }
        };

        let version = release.version.trim_start_matches('v').to_string();
        let archive_name = format!("gabbro-{version}-x86_64-windows.zip");
        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == archive_name)
            .ok_or_else(|| format!("the Gabbro release {version} has no asset named {archive_name}"))?;

        let version_dir = format!("gabbro-{version}");
        let binary = format!("{version_dir}/gabbro-{version}-x86_64-windows/bin/gabbro.exe");

        if !is_file(&binary) {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );
            zed::download_file(
                &asset.download_url,
                &version_dir,
                zed::DownloadedFileType::Zip,
            )
            .map_err(|error| format!("could not download {archive_name}: {error}"))?;
            if !is_file(&binary) {
                return Err(format!("{archive_name} did not contain {binary}"));
            }
            if let Ok(entries) = fs::read_dir(".") {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.starts_with("gabbro-") && name != version_dir {
                        fs::remove_dir_all(entry.path()).ok();
                    }
                }
            }
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::None,
        );
        absolute(&binary)
    }

    // A copy that an earlier run downloaded (the one with the highest version name).
    fn existing_copy(&self) -> Option<String> {
        let mut best: Option<String> = None;
        for entry in fs::read_dir(".").ok()?.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if let Some(version) = name.strip_prefix("gabbro-") {
                let binary = format!("{name}/gabbro-{version}-x86_64-windows/bin/gabbro.exe");
                if is_file(&binary) && best.as_ref().map_or(true, |current| binary > *current) {
                    best = Some(binary);
                }
            }
        }
        best.and_then(|binary| absolute(&binary).ok())
    }
}

fn absolute(relative: &str) -> Result<String> {
    let dir = std::env::current_dir().map_err(|error| error.to_string())?;
    Ok(dir.join(relative).to_string_lossy().to_string())
}

impl zed::Extension for GabbroExtension {
    fn new() -> Self {
        GabbroExtension {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        Ok(zed::Command {
            command: self.binary_path(language_server_id, worktree)?,
            args: vec!["lsp".to_string()],
            env: worktree.shell_env(),
        })
    }
}

zed::register_extension!(GabbroExtension);
