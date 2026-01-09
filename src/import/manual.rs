// Manual game import - for games added by the user directly
// This module provides utilities for manual game addition

use std::path::PathBuf;

use crate::data::{Game, GameSource};

/// Create a manual game entry
pub fn create_manual_game(name: String, executable_path: PathBuf) -> Game {
    Game::new(name, executable_path, GameSource::Manual)
}

/// Validate that an executable path exists and is a file
pub fn validate_executable(path: &PathBuf) -> Result<(), ManualImportError> {
    if !path.exists() {
        return Err(ManualImportError::PathNotFound);
    }

    if !path.is_file() {
        return Err(ManualImportError::NotAFile);
    }

    #[cfg(target_os = "windows")]
    {
        if path.extension().map(|e| e != "exe").unwrap_or(true) {
            return Err(ManualImportError::NotExecutable);
        }
    }

    Ok(())
}

/// Errors that can occur during manual import
#[derive(Debug, thiserror::Error)]
pub enum ManualImportError {
    #[error("Path does not exist")]
    PathNotFound,

    #[error("Path is not a file")]
    NotAFile,

    #[error("File is not an executable")]
    NotExecutable,
}
