use std::fs;
use zed_extension_api::{self as zed, settings::LspSettings, LanguageServerId, Result};

const REPOSITORY: &str = "fels-lang/fels";

// Release archives are `fels-<version>-x86_64-windows.zip` holding `bin/fels.exe`. Releases up to
// 0.3.1, from before the language was renamed from Gabbro, use `gabbro` for both.
const NAMES: [&str; 2] = ["fels", "gabbro"];

// The language server is `fels lsp`, part of the compiler. It needs the standard library that sits
// next to the executable, so the extension downloads the whole release archive and runs the compiler
// from there.
struct FelsExtension {
    cached_binary_path: Option<String>,
}

fn is_file(path: &str) -> bool {
    fs::metadata(path).map_or(false, |meta| meta.is_file())
}

// The compiler inside an unpacked release archive, for a download folder `fels-<version>`.
fn binary_in(folder: &str, version: &str) -> Option<String> {
    NAMES.iter().find_map(|name| {
        let binary = format!("{folder}/{name}-{version}-x86_64-windows/bin/{name}.exe");
        is_file(&binary).then_some(binary)
    })
}

impl FelsExtension {
    // 1. the path from the Zed settings (Zed uses it on its own, but the check is cheap and keeps
    //    the order in one place), 2. `fels` (or the old `gabbro`) on the PATH, 3. a copy
    //    downloaded by this extension.
    fn binary_path(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<String> {
        if let Ok(settings) = LspSettings::for_worktree("fels", worktree) {
            if let Some(path) = settings.binary.and_then(|binary| binary.path) {
                return Ok(path);
            }
        }
        for name in NAMES {
            if let Some(path) = worktree.which(name) {
                return Ok(path);
            }
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
                "Fels releases exist for 64 bit Windows only. Build fels yourself and put it on the PATH, \
                 or set lsp.fels.binary.path in the Zed settings."
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
                return Err(format!("could not look up the latest Fels release: {error}"));
            }
        };

        let version = release.version.trim_start_matches('v').to_string();
        let asset = NAMES
            .iter()
            .find_map(|name| {
                let archive_name = format!("{name}-{version}-x86_64-windows.zip");
                release.assets.iter().find(|asset| asset.name == archive_name)
            })
            .ok_or_else(|| format!("the Fels release {version} has no fels-{version}-x86_64-windows.zip"))?;

        let version_dir = format!("fels-{version}");
        let binary = match binary_in(&version_dir, &version) {
            Some(binary) => binary,
            None => {
                zed::set_language_server_installation_status(
                    language_server_id,
                    &zed::LanguageServerInstallationStatus::Downloading,
                );
                zed::download_file(&asset.download_url, &version_dir, zed::DownloadedFileType::Zip)
                    .map_err(|error| format!("could not download {}: {error}", asset.name))?;
                let binary = binary_in(&version_dir, &version)
                    .ok_or_else(|| format!("{} did not contain the compiler", asset.name))?;
                if let Ok(entries) = fs::read_dir(".") {
                    for entry in entries.flatten() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        let ours = NAMES.iter().any(|n| name.starts_with(&format!("{n}-")));
                        if ours && name != version_dir {
                            fs::remove_dir_all(entry.path()).ok();
                        }
                    }
                }
                binary
            }
        };

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
            let version = NAMES.iter().find_map(|n| name.strip_prefix(&format!("{n}-")));
            if let Some(binary) = version.and_then(|v| binary_in(&name, v)) {
                if best.as_ref().map_or(true, |current| binary > *current) {
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

impl zed::Extension for FelsExtension {
    fn new() -> Self {
        FelsExtension {
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

zed::register_extension!(FelsExtension);
