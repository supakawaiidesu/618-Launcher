use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::message::{SortOrder, ViewMode};

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Name of the current theme
    pub theme: String,

    /// Whether to start the app minimized
    pub start_minimized: bool,

    /// Whether to minimize to tray on close
    pub close_to_tray: bool,

    /// Default sort order for the library
    pub default_sort: SortOrder,

    /// Default view mode (grid or list)
    pub default_view_mode: ViewMode,

    /// Size of game cards in grid view (small, medium, large)
    pub card_size: CardSize,

    /// Whether to show game sources in the library
    pub show_sources: bool,

    /// Paths to additional Steam library folders (for manual configuration)
    pub steam_library_paths: Vec<PathBuf>,

    /// Last import sync time for each source
    pub last_sync: LastSyncTimes,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            start_minimized: false,
            close_to_tray: false,
            default_sort: SortOrder::NameAsc,
            default_view_mode: ViewMode::Grid,
            card_size: CardSize::Medium,
            show_sources: true,
            steam_library_paths: Vec::new(),
            last_sync: LastSyncTimes::default(),
        }
    }
}

impl Config {
    /// Save config to a JSON file
    pub async fn save_to_file(&self, path: &Path) -> Result<(), ConfigError> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| ConfigError::Serialization(e.to_string()))?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| ConfigError::Io(e.to_string()))?;
        }

        let mut file = fs::File::create(path)
            .await
            .map_err(|e| ConfigError::Io(e.to_string()))?;

        file.write_all(json.as_bytes())
            .await
            .map_err(|e| ConfigError::Io(e.to_string()))?;

        tracing::debug!("Config saved to {:?}", path);
        Ok(())
    }

    /// Load config from a JSON file
    pub async fn load_from_file(path: &Path) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)
            .await
            .map_err(|e| ConfigError::Io(e.to_string()))?;

        let config: Config = serde_json::from_str(&content)
            .map_err(|e| ConfigError::Deserialization(e.to_string()))?;

        tracing::debug!("Config loaded from {:?}", path);
        Ok(config)
    }

    /// Load from file or create default if file doesn't exist
    pub async fn load_or_create(path: &Path) -> Self {
        match Self::load_from_file(path).await {
            Ok(config) => config,
            Err(e) => {
                tracing::warn!("Could not load config: {}. Using defaults.", e);
                Self::default()
            }
        }
    }
}

/// Card size options for grid view
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CardSize {
    Small,
    #[default]
    Medium,
    Large,
}

impl CardSize {
    pub fn width(&self) -> f32 {
        match self {
            CardSize::Small => 120.0,
            CardSize::Medium => 180.0,
            CardSize::Large => 240.0,
        }
    }

    pub fn height(&self) -> f32 {
        match self {
            CardSize::Small => 160.0,
            CardSize::Medium => 240.0,
            CardSize::Large => 320.0,
        }
    }
}

/// Timestamps for last sync with each game source
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LastSyncTimes {
    pub steam: Option<chrono::DateTime<chrono::Utc>>,
    pub epic: Option<chrono::DateTime<chrono::Utc>>,
    pub gog: Option<chrono::DateTime<chrono::Utc>>,
}

/// Errors that can occur with config operations
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),
}
