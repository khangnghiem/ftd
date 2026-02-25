use std::fs;
use zed::LanguageServerId;
use zed_extension_api::{self as zed, settings::LspSettings, Result};

/// Resolved binary location and arguments.
struct LspBinary {
    path: String,
    args: Option<Vec<String>>,
}

/// FD language extension — provides Tree-sitter grammar and LSP support.
struct FdExtension {
    cached_binary_path: Option<String>,
}

/// Platform-specific details for downloading a GitHub release asset.
struct ReleaseAsset {
    asset_name: String,
    file_type: zed::DownloadedFileType,
    directory: String,
    binary_path: String,
}

impl ReleaseAsset {
    fn new(platform: zed::Os, arch: zed::Architecture, version: &str) -> Self {
        let target = format!(
            "{arch}-{os}",
            arch = match arch {
                zed::Architecture::Aarch64 => "aarch64",
                zed::Architecture::X86 => "x86",
                zed::Architecture::X8664 => "x86_64",
            },
            os = match platform {
                zed::Os::Mac => "apple-darwin",
                zed::Os::Linux => "unknown-linux-gnu",
                zed::Os::Windows => "pc-windows-msvc",
            }
        );

        let asset_name = format!("fd-lsp-{target}.tar.gz");
        let directory = format!("fd-lsp-{version}");
        let binary_path = format!("{directory}/fd-lsp-{target}/fd-lsp");

        Self {
            asset_name,
            file_type: zed::DownloadedFileType::GzipTar,
            directory,
            binary_path,
        }
    }
}

impl FdExtension {
    /// Resolve the fd-lsp binary path, downloading if necessary.
    fn resolve_binary(
        &mut self,
        server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<LspBinary> {
        let settings = LspSettings::for_worktree(server_id.as_ref(), worktree)
            .ok()
            .and_then(|s| s.binary);

        let args = settings
            .as_ref()
            .and_then(|s| s.arguments.clone());

        // User-specified binary path
        if let Some(path) = settings.and_then(|s| s.path) {
            return Ok(LspBinary { path, args });
        }

        // Binary on PATH
        if let Some(path) = worktree.which("fd-lsp") {
            return Ok(LspBinary { path, args });
        }

        // Previously downloaded binary
        if let Some(path) = &self.cached_binary_path {
            if fs::metadata(path).is_ok_and(|stat| stat.is_file()) {
                return Ok(LspBinary {
                    path: path.clone(),
                    args,
                });
            }
        }

        // Download from GitHub Releases
        zed::set_language_server_installation_status(
            server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let release = zed::latest_github_release(
            "khangnghiem/fast-draft",
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let (platform, arch) = zed::current_platform();
        let asset_info = ReleaseAsset::new(platform, arch, &release.version);

        let download_url = release
            .assets
            .iter()
            .find(|a| a.name == asset_info.asset_name)
            .ok_or_else(|| {
                format!(
                    "No release asset for this platform: {}",
                    asset_info.asset_name
                )
            })?
            .download_url
            .clone();

        if !fs::metadata(&asset_info.binary_path).is_ok_and(|stat| stat.is_file()) {
            zed::set_language_server_installation_status(
                server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );

            zed::download_file(&download_url, &asset_info.directory, asset_info.file_type)
                .map_err(|e| format!("Failed to download fd-lsp: {e}"))?;

            // Clean up old versions
            let entries = fs::read_dir(".")
                .map_err(|e| format!("Failed to list directory: {e}"))?;

            for entry in entries.flatten() {
                if entry.file_name().to_str() != Some(&asset_info.directory) {
                    fs::remove_dir_all(entry.path()).ok();
                }
            }
        }

        self.cached_binary_path = Some(asset_info.binary_path.clone());

        Ok(LspBinary {
            path: asset_info.binary_path,
            args,
        })
    }
}

impl zed::Extension for FdExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let binary = self.resolve_binary(server_id, worktree)?;
        Ok(zed::Command {
            command: binary.path,
            args: binary.args.unwrap_or_default(),
            env: vec![],
        })
    }

    fn language_server_initialization_options(
        &mut self,
        server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<zed::serde_json::Value>> {
        let settings = LspSettings::for_worktree(server_id.as_ref(), worktree)
            .ok()
            .and_then(|s| s.initialization_options.clone())
            .unwrap_or_default();
        Ok(Some(settings))
    }
}

zed::register_extension!(FdExtension);
