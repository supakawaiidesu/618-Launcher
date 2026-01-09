use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::io::AsyncWriteExt;

use super::{Category, CategoryId, Game, GameId};
use crate::message::SortOrder;

/// The game library containing all games and categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Library {
    /// All games indexed by their ID
    pub games: HashMap<GameId, Game>,

    /// All categories indexed by their ID
    pub categories: HashMap<CategoryId, Category>,
}

impl Default for Library {
    fn default() -> Self {
        Self::new()
    }
}

impl Library {
    /// Create a new empty library with default categories
    pub fn new() -> Self {
        let default_cats = super::category::default_categories();
        let categories = default_cats
            .into_iter()
            .map(|c| (c.id, c))
            .collect();

        Self {
            games: HashMap::new(),
            categories,
        }
    }

    /// Add a game to the library
    pub fn add_game(&mut self, game: Game) {
        self.games.insert(game.id, game);
    }

    /// Remove a game from the library
    pub fn remove_game(&mut self, id: &GameId) -> Option<Game> {
        self.games.remove(id)
    }

    /// Get a game by ID
    pub fn get_game(&self, id: &GameId) -> Option<&Game> {
        self.games.get(id)
    }

    /// Get a mutable game by ID
    pub fn get_game_mut(&mut self, id: &GameId) -> Option<&mut Game> {
        self.games.get_mut(id)
    }

    /// Get all games as a vector
    pub fn all_games(&self) -> Vec<&Game> {
        self.games.values().collect()
    }

    /// Get games sorted by the specified order
    pub fn games_sorted(&self, order: SortOrder) -> Vec<&Game> {
        let mut games: Vec<&Game> = self.games.values().collect();

        match order {
            SortOrder::NameAsc => {
                games.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
            }
            SortOrder::NameDesc => {
                games.sort_by(|a, b| b.name.to_lowercase().cmp(&a.name.to_lowercase()));
            }
            SortOrder::LastPlayed => {
                games.sort_by(|a, b| b.last_played.cmp(&a.last_played));
            }
            SortOrder::RecentlyAdded => {
                games.sort_by(|a, b| b.added_date.cmp(&a.added_date));
            }
            SortOrder::MostPlayed => {
                games.sort_by(|a, b| b.playtime_minutes.cmp(&a.playtime_minutes));
            }
        }

        games
    }

    /// Get games filtered by search query
    pub fn search_games(&self, query: &str) -> Vec<&Game> {
        let query_lower = query.to_lowercase();
        self.games
            .values()
            .filter(|g| g.name.to_lowercase().contains(&query_lower))
            .collect()
    }

    /// Get games in a specific category
    pub fn games_in_category(&self, category_id: &CategoryId) -> Vec<&Game> {
        self.games
            .values()
            .filter(|g| g.has_category(category_id))
            .collect()
    }

    /// Get favorite games
    pub fn favorite_games(&self) -> Vec<&Game> {
        self.games.values().filter(|g| g.favorite).collect()
    }

    /// Get total number of games
    pub fn game_count(&self) -> usize {
        self.games.len()
    }

    /// Add a category to the library
    pub fn add_category(&mut self, category: Category) {
        self.categories.insert(category.id, category);
    }

    /// Remove a category from the library
    pub fn remove_category(&mut self, id: &CategoryId) -> Option<Category> {
        // Also remove this category from all games
        for game in self.games.values_mut() {
            game.remove_category(id);
        }
        self.categories.remove(id)
    }

    /// Get a category by ID
    pub fn get_category(&self, id: &CategoryId) -> Option<&Category> {
        self.categories.get(id)
    }

    /// Get all categories as a vector
    pub fn all_categories(&self) -> Vec<&Category> {
        self.categories.values().collect()
    }

    /// Save the library to a JSON file
    pub async fn save_to_file(&self, path: &Path) -> Result<(), LibraryError> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| LibraryError::Serialization(e.to_string()))?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| LibraryError::Io(e.to_string()))?;
        }

        let mut file = fs::File::create(path)
            .await
            .map_err(|e| LibraryError::Io(e.to_string()))?;

        file.write_all(json.as_bytes())
            .await
            .map_err(|e| LibraryError::Io(e.to_string()))?;

        tracing::info!("Library saved to {:?}", path);
        Ok(())
    }

    /// Load the library from a JSON file
    pub async fn load_from_file(path: &Path) -> Result<Self, LibraryError> {
        let content = fs::read_to_string(path)
            .await
            .map_err(|e| LibraryError::Io(e.to_string()))?;

        let library: Library = serde_json::from_str(&content)
            .map_err(|e| LibraryError::Deserialization(e.to_string()))?;

        tracing::info!("Library loaded from {:?} ({} games)", path, library.game_count());
        Ok(library)
    }

    /// Load from file or create new if file doesn't exist
    pub async fn load_or_create(path: &Path) -> Self {
        match Self::load_from_file(path).await {
            Ok(library) => library,
            Err(e) => {
                tracing::warn!("Could not load library: {}. Creating new library.", e);
                Self::new()
            }
        }
    }
}

/// Errors that can occur with library operations
#[derive(Debug, thiserror::Error)]
pub enum LibraryError {
    #[error("IO error: {0}")]
    Io(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),
}
