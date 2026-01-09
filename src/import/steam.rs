use std::path::PathBuf;

use super::{DetectedGame, GameImporter, ImportError};
use crate::data::GameSource;

/// Importer for Steam games
pub struct SteamImporter {
    steam_path: Option<PathBuf>,
}

impl SteamImporter {
    pub fn new() -> Self {
        Self {
            steam_path: Self::find_steam_path(),
        }
    }

    /// Find Steam installation path
    #[cfg(target_os = "windows")]
    fn find_steam_path() -> Option<PathBuf> {
        use winreg::enums::*;
        use winreg::RegKey;

        // Try 64-bit registry first
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

        // Try WOW6432Node first (64-bit Windows)
        if let Ok(steam_key) = hklm.open_subkey("SOFTWARE\\WOW6432Node\\Valve\\Steam") {
            if let Ok(path) = steam_key.get_value::<String, _>("InstallPath") {
                return Some(PathBuf::from(path));
            }
        }

        // Try regular path (32-bit Windows or 32-bit Steam)
        if let Ok(steam_key) = hklm.open_subkey("SOFTWARE\\Valve\\Steam") {
            if let Ok(path) = steam_key.get_value::<String, _>("InstallPath") {
                return Some(PathBuf::from(path));
            }
        }

        None
    }

    #[cfg(target_os = "linux")]
    fn find_steam_path() -> Option<PathBuf> {
        let home = std::env::var("HOME").ok()?;

        // Check common Steam locations
        let paths = [
            format!("{}/.local/share/Steam", home),
            format!("{}/.steam/steam", home),
            format!("{}/.steam/debian-installation", home),
        ];

        for path in &paths {
            let p = PathBuf::from(path);
            if p.exists() {
                return Some(p);
            }
        }

        None
    }

    #[cfg(target_os = "macos")]
    fn find_steam_path() -> Option<PathBuf> {
        let home = std::env::var("HOME").ok()?;
        let path = PathBuf::from(format!(
            "{}/Library/Application Support/Steam",
            home
        ));

        if path.exists() {
            Some(path)
        } else {
            None
        }
    }

    /// Get all Steam library folders
    fn get_library_folders(&self) -> Result<Vec<PathBuf>, ImportError> {
        let steam_path = self.steam_path.as_ref().ok_or(ImportError::NotInstalled)?;
        let vdf_path = steam_path.join("steamapps").join("libraryfolders.vdf");

        if !vdf_path.exists() {
            return Ok(vec![steam_path.join("steamapps")]);
        }

        // Parse VDF file to find additional library folders
        let content = std::fs::read_to_string(&vdf_path)?;
        let mut folders = vec![steam_path.join("steamapps")];

        // Simple VDF parsing - look for "path" entries
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("\"path\"") {
                if let Some(path_str) = extract_vdf_value(line) {
                    let path = PathBuf::from(path_str);
                    let steamapps = path.join("steamapps");
                    if steamapps.exists() && !folders.contains(&steamapps) {
                        folders.push(steamapps);
                    }
                }
            }
        }

        Ok(folders)
    }

    /// Parse an appmanifest file
    fn parse_app_manifest(&self, path: &PathBuf) -> Option<DetectedGame> {
        let content = std::fs::read_to_string(path).ok()?;

        let app_id = extract_vdf_value_by_key(&content, "appid")?;
        let name = extract_vdf_value_by_key(&content, "name")?;
        let install_dir = extract_vdf_value_by_key(&content, "installdir")?;

        let library_path = path.parent()?;
        let install_path = library_path.join("common").join(&install_dir);

        if !install_path.exists() {
            return None;
        }

        // Try to find the main executable
        let executable_path = find_executable_in_dir(&install_path)?;

        Some(DetectedGame {
            name,
            source_id: app_id,
            executable_path,
            install_path,
            icon_path: None, // Steam icons are handled differently
        })
    }
}

impl Default for SteamImporter {
    fn default() -> Self {
        Self::new()
    }
}

impl GameImporter for SteamImporter {
    fn source(&self) -> GameSource {
        GameSource::Steam
    }

    fn is_available(&self) -> bool {
        self.steam_path.is_some()
    }

    fn scan_games(&self) -> Result<Vec<DetectedGame>, ImportError> {
        let library_folders = self.get_library_folders()?;
        let mut games = Vec::new();

        for folder in library_folders {
            // Find all appmanifest_*.acf files
            if let Ok(entries) = std::fs::read_dir(&folder) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.starts_with("appmanifest_") && name.ends_with(".acf") {
                            if let Some(game) = self.parse_app_manifest(&path) {
                                games.push(game);
                            }
                        }
                    }
                }
            }
        }

        tracing::info!("Found {} Steam games", games.len());
        Ok(games)
    }
}

/// Extract a value from a VDF line like "key" "value"
fn extract_vdf_value(line: &str) -> Option<String> {
    let parts: Vec<&str> = line.split('"').collect();
    if parts.len() >= 4 {
        Some(parts[3].to_string())
    } else {
        None
    }
}

/// Extract a value by key from VDF content
fn extract_vdf_value_by_key(content: &str, key: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        let search = format!("\"{}\"", key);
        if line.to_lowercase().starts_with(&search.to_lowercase()) {
            return extract_vdf_value(line);
        }
    }
    None
}

/// Try to find a main executable in a game directory
fn find_executable_in_dir(dir: &PathBuf) -> Option<PathBuf> {
    // Look for common executable patterns
    if let Ok(entries) = std::fs::read_dir(dir) {
        let mut candidates: Vec<PathBuf> = entries
            .flatten()
            .filter_map(|e| {
                let path = e.path();
                if path.is_file() && is_executable(&path) {
                    Some(path)
                } else {
                    None
                }
            })
            .collect();

        // Sort by name length (shorter names are often the main executable)
        candidates.sort_by_key(|p| p.file_name().map(|n| n.len()).unwrap_or(usize::MAX));

        candidates.into_iter().next()
    } else {
        None
    }
}

/// Check if a file is executable (platform-specific)
#[cfg(target_os = "windows")]
fn is_executable(path: &PathBuf) -> bool {
    path.extension()
        .map(|ext| ext.eq_ignore_ascii_case("exe"))
        .unwrap_or(false)
}

#[cfg(not(target_os = "windows"))]
fn is_executable(path: &PathBuf) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata()
        .map(|meta| meta.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}
