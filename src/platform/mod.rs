// Platform-specific code

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "macos")]
mod macos;

use std::path::PathBuf;

/// Get the platform name
pub fn platform_name() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "Windows"
    }
    #[cfg(target_os = "linux")]
    {
        "Linux"
    }
    #[cfg(target_os = "macos")]
    {
        "macOS"
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        "Unknown"
    }
}

/// Get default game installation directories for the current platform
pub fn default_game_directories() -> Vec<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        vec![
            PathBuf::from("C:\\Program Files (x86)\\Steam\\steamapps\\common"),
            PathBuf::from("C:\\Program Files\\Epic Games"),
            PathBuf::from("C:\\Games"),
        ]
    }
    #[cfg(target_os = "linux")]
    {
        if let Ok(home) = std::env::var("HOME") {
            vec![
                PathBuf::from(format!("{}/.local/share/Steam/steamapps/common", home)),
                PathBuf::from(format!("{}/Games", home)),
            ]
        } else {
            vec![]
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = std::env::var("HOME") {
            vec![
                PathBuf::from(format!(
                    "{}/Library/Application Support/Steam/steamapps/common",
                    home
                )),
                PathBuf::from("/Applications"),
            ]
        } else {
            vec![]
        }
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        vec![]
    }
}

/// Check if the platform supports a specific feature
pub fn supports_feature(feature: PlatformFeature) -> bool {
    match feature {
        PlatformFeature::SteamImport => true, // All platforms
        PlatformFeature::EpicImport => cfg!(target_os = "windows"),
        PlatformFeature::GOGImport => cfg!(target_os = "windows"),
        PlatformFeature::SystemTray => true, // All platforms via iced
    }
}

/// Platform-specific features
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformFeature {
    SteamImport,
    EpicImport,
    GOGImport,
    SystemTray,
}
