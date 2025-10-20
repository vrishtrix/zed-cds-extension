use std::fs;

use zed_extension_api::{
    node_binary_path, npm_install_package, npm_package_installed_version,
    npm_package_latest_version, register_extension, set_language_server_installation_status,
    Command, Extension, LanguageServerId, LanguageServerInstallationStatus, Result, Worktree,
};

#[derive(Default)]
struct CdsExtension {
    cached_binary_path: Option<String>,
}

impl CdsExtension {
    fn language_server_binary_path(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<String> {
        if let Some(path) = &self.cached_binary_path {
            if fs::metadata(path).map_or(false, |stat| stat.is_file()) {
                return Ok(path.clone());
            }
        }

        if let Some(path) = worktree.which("cds-lsp") {
            self.cached_binary_path = Some(path.clone());
            return Ok(path.clone());
        }

        let local_lsp_path = format!(
            "{}/node_modules/@sap/cds-lsp/bin/cds-lsp",
            worktree.root_path()
        );
        if fs::metadata(&local_lsp_path).map_or(false, |stat| stat.is_file()) {
            self.cached_binary_path = Some(local_lsp_path.clone());
            return Ok(local_lsp_path);
        }

        set_language_server_installation_status(
            language_server_id,
            &LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let version = npm_package_latest_version("@sap/cds-lsp")?;
        if let Some(installed_version) = npm_package_installed_version("@sap/cds-lsp")? {
            if installed_version == version {
                set_language_server_installation_status(
                    language_server_id,
                    &LanguageServerInstallationStatus::None,
                );

                let server_path = std::env::current_dir()
                    .map_err(|e| format!("Failed to get current directory: {}", e))?
                    .join("node_modules/@sap/cds-lsp/bin/cds-lsp")
                    .to_string_lossy()
                    .to_string();

                if fs::metadata(&server_path).map_or(false, |stat| stat.is_file()) {
                    self.cached_binary_path = Some(server_path.clone());
                    return Ok(server_path);
                }
            }
        }

        set_language_server_installation_status(
            language_server_id,
            &LanguageServerInstallationStatus::Downloading,
        );

        let result = npm_install_package("@sap/cds-lsp", &version);
        match result {
            Ok(()) => {
                set_language_server_installation_status(
                    language_server_id,
                    &LanguageServerInstallationStatus::None,
                );

                let server_path = std::env::current_dir()
                    .map_err(|e| format!("Failed to get current directory: {}", e))?
                    .join("node_modules/.bin/cds-lsp")
                    .to_string_lossy()
                    .to_string();

                if !fs::metadata(&server_path).map_or(false, |stat| stat.is_file()) {
                    return Err(format!(
                        "cds-lsp package installed but binary not found at: {}",
                        server_path
                    ));
                }

                self.cached_binary_path = Some(server_path.clone());
                Ok(server_path)
            }
            Err(error) => {
                set_language_server_installation_status(
                    language_server_id,
                    &LanguageServerInstallationStatus::Failed(error.clone()),
                );

                Err(error)
            }
        }
    }
}

impl Extension for CdsExtension {
    fn new() -> Self {
        Self::default()
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed_extension_api::LanguageServerId,
        worktree: &zed_extension_api::Worktree,
    ) -> zed_extension_api::Result<zed_extension_api::Command> {
        let server_path = self.language_server_binary_path(language_server_id, worktree)?;

        Ok(Command {
            command: node_binary_path()?,
            args: vec![server_path, "--stdio".to_string()],
            env: Default::default(),
        })
    }
}

register_extension!(CdsExtension);
