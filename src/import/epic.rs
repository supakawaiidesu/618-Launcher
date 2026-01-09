use std::path::PathBuf;

use super::{DetectedGame, GameImporter, ImportError};
use crate::data::GameSource;

/// Importer for Epic Games Store games
pub struct EpicImporter {
    manifests_path: Option<PathBuf>,
}

impl EpicImporter {
    pub fn new() -> Self {
        Self {
            manifests_path: Self::find_manifests_path(),
        }
    }

    /// Find Epic Games manifests path
    #[cfg(target_os = "windows")]
    fn find_manifests_path() -> Option<PathBuf> {
        // Default location for Epic Games manifests
        let path = PathBuf::from("C:\\ProgramData\\Epic\\EpicGamesLauncher\\Data\\Manifests");
        if path.exists() {
            return Some(path);
        }

        // Try registry
        use winreg::enums::*;
        use winreg::RegKey;

        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        if let Ok(key) = hklm.open_subkey("SOFTWARE\\WOW6432Node\\Epic Games\\EpicGamesLauncher") {
            if let Ok(app_data_path) = key.get_value::<String, _>("AppDataPath") {
                let manifests = PathBuf::from(app_data_path).join("Manifests");
                if manifests.exists() {
                    return Some(manifests);
                }
            }
        }

        None
    }

    #[cfg(not(target_os = "windows"))]
    fn find_manifests_path() -> Option<PathBuf> {
        // Epic Games is primarily Windows-only
        // On Linux, games would be through compatibility layers
        None
    }

    /// Parse an Epic Games manifest (.item) file
    fn parse_manifest(&self, path: &PathBuf) -> Option<DetectedGame> {
        let content = std::fs::read_to_string(path).ok()?;

        // Parse JSON
        let manifest: serde_json::Value = serde_json::from_str(&content).ok()?;

        let name = manifest.get("DisplayName")?.as_str()?.to_string();
        let install_location = manifest.get("InstallLocation")?.as_str()?;
        let app_name = manifest.get("AppName")?.as_str()?.to_string();
        let launch_executable = manifest.get("LaunchExecutable")?.as_str()?;

        let install_path = PathBuf::from(install_location);
        let executable_path = install_path.join(launch_executable);

        if !executable_path.exists() {
            return None;
        }

        Some(DetectedGame {
            name,
            source_id: app_name,
            executable_path,
            install_path,
            icon_path: None,
        })
    }
}

impl Default for EpicImporter {
    fn default() -> Self {
        Self::new()
    }
}

impl GameImporter for EpicImporter {
    fn source(&self) -> GameSource {
        GameSource::Epic
    }

    fn is_available(&self) -> bool {
        self.manifests_path.is_some()
    }

    fn scan_games(&self) -> Result<Vec<DetectedGame>, ImportError> {
        let manifests_path = self.manifests_path.as_ref().ok_or(ImportError::NotInstalled)?;
        let mut games = Vec::new();

        if let Ok(entries) = std::fs::read_dir(manifests_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "item").unwrap_or(false) {
                    if let Some(game) = self.parse_manifest(&path) {
                        games.push(game);
                    }
                }
            }
        }

        tracing::info!("Found {} Epic Games", games.len());
        Ok(games)
    }
}
