//! Import system for detecting games from various launchers

mod steam;
mod epic;
mod gog;
mod manual;

// Re-exports - will be used when import UI is connected
#[allow(unused_imports)]
pub use steam::SteamImporter;
#[allow(unused_imports)]
pub use epic::EpicImporter;
#[allow(unused_imports)]
pub use gog::GOGImporter;

use crate::data::{Game, GameSource};
use std::path::PathBuf;
use thiserror::Error;

/// A game detected during import
#[derive(Debug, Clone)]
pub struct DetectedGame {
    pub name: String,
    pub source_id: String,
    pub executable_path: PathBuf,
    pub install_path: PathBuf,
    pub icon_path: Option<PathBuf>,
}

/// Trait for game importers
pub trait GameImporter {
    /// Get the source type for this importer
    fn source(&self) -> GameSource;

    /// Check if the launcher is installed and available
    fn is_available(&self) -> bool;

    /// Scan for installed games
    fn scan_games(&self) -> Result<Vec<DetectedGame>, ImportError>;
}

/// Convert a detected game to a library game
impl DetectedGame {
    pub fn into_game(self, source: GameSource) -> Game {
        Game::from_import(
            self.name,
            self.executable_path,
            self.install_path,
            source,
            self.source_id,
        )
    }
}

/// Errors that can occur during import
#[derive(Debug, Error)]
pub enum ImportError {
    #[error("Launcher not installed")]
    NotInstalled,

    #[error("Could not find launcher path")]
    PathNotFound,

    #[error("Failed to parse configuration: {0}")]
    ParseError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Registry error: {0}")]
    RegistryError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),
}
