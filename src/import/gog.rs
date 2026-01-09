use std::path::PathBuf;

use super::{DetectedGame, GameImporter, ImportError};
use crate::data::GameSource;

/// Importer for GOG Galaxy games
pub struct GOGImporter {
    database_path: Option<PathBuf>,
}

impl GOGImporter {
    pub fn new() -> Self {
        Self {
            database_path: Self::find_database_path(),
        }
    }

    /// Find GOG Galaxy database path
    #[cfg(target_os = "windows")]
    fn find_database_path() -> Option<PathBuf> {
        let path = PathBuf::from("C:\\ProgramData\\GOG.com\\Galaxy\\storage\\galaxy-2.0.db");
        if path.exists() {
            Some(path)
        } else {
            None
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn find_database_path() -> Option<PathBuf> {
        // GOG Galaxy is Windows-only
        None
    }
}

impl Default for GOGImporter {
    fn default() -> Self {
        Self::new()
    }
}

impl GameImporter for GOGImporter {
    fn source(&self) -> GameSource {
        GameSource::GOG
    }

    fn is_available(&self) -> bool {
        self.database_path.is_some()
    }

    fn scan_games(&self) -> Result<Vec<DetectedGame>, ImportError> {
        // GOG import requires the rusqlite feature
        #[cfg(feature = "gog")]
        {
            use rusqlite::Connection;

            let db_path = self.database_path.as_ref().ok_or(ImportError::NotInstalled)?;
            let conn = Connection::open(db_path)
                .map_err(|e| ImportError::DatabaseError(e.to_string()))?;

            let mut stmt = conn
                .prepare(
                    "SELECT productId, localPath FROM InstalledBaseProducts WHERE localPath IS NOT NULL"
                )
                .map_err(|e| ImportError::DatabaseError(e.to_string()))?;

            let mut games = Vec::new();

            let rows = stmt
                .query_map([], |row| {
                    let product_id: i64 = row.get(0)?;
                    let local_path: String = row.get(1)?;
                    Ok((product_id, local_path))
                })
                .map_err(|e| ImportError::DatabaseError(e.to_string()))?;

            for row in rows {
                if let Ok((product_id, local_path)) = row {
                    let install_path = PathBuf::from(&local_path);
                    if install_path.exists() {
                        // Try to find the game name and executable
                        if let Some(game) = self.find_game_in_folder(&install_path, product_id) {
                            games.push(game);
                        }
                    }
                }
            }

            tracing::info!("Found {} GOG games", games.len());
            Ok(games)
        }

        #[cfg(not(feature = "gog"))]
        {
            tracing::warn!("GOG import requires the 'gog' feature to be enabled");
            Err(ImportError::NotInstalled)
        }
    }
}

impl GOGImporter {
    #[cfg(feature = "gog")]
    fn find_game_in_folder(&self, install_path: &PathBuf, product_id: i64) -> Option<DetectedGame> {
        // Try to find goggame-*.info file
        if let Ok(entries) = std::fs::read_dir(install_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("goggame-") && name.ends_with(".info") {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if let Ok(info) = serde_json::from_str::<serde_json::Value>(&content) {
                                let game_name = info.get("name")?.as_str()?.to_string();
                                let play_tasks = info.get("playTasks")?.as_array()?;

                                // Get the primary play task
                                if let Some(task) = play_tasks.first() {
                                    let exe_path = task.get("path")?.as_str()?;
                                    let executable_path = install_path.join(exe_path);

                                    if executable_path.exists() {
                                        return Some(DetectedGame {
                                            name: game_name,
                                            source_id: product_id.to_string(),
                                            executable_path,
                                            install_path: install_path.clone(),
                                            icon_path: None,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        None
    }
}
