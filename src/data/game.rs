use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

use super::CategoryId;

/// Unique identifier for a game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GameId(pub Uuid);

impl GameId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for GameId {
    fn default() -> Self {
        Self::new()
    }
}

/// Source/origin of a game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GameSource {
    Steam,
    Epic,
    GOG,
    Manual,
}

impl GameSource {
    pub fn label(&self) -> &'static str {
        match self {
            GameSource::Steam => "Steam",
            GameSource::Epic => "Epic Games",
            GameSource::GOG => "GOG Galaxy",
            GameSource::Manual => "Manual",
        }
    }

    pub fn all() -> &'static [GameSource] {
        &[
            GameSource::Steam,
            GameSource::Epic,
            GameSource::GOG,
            GameSource::Manual,
        ]
    }
}

/// A game in the library
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    /// Unique identifier
    pub id: GameId,

    /// Display name
    pub name: String,

    /// Path to the executable
    pub executable_path: PathBuf,

    /// Installation directory (if known)
    pub install_path: Option<PathBuf>,

    /// Where this game came from
    pub source: GameSource,

    /// ID from the source platform (Steam AppID, Epic manifest ID, etc.)
    pub source_id: Option<String>,

    /// Assigned categories/tags
    pub categories: Vec<CategoryId>,

    /// Whether this game is a favorite
    pub favorite: bool,

    /// Path to the game icon
    pub icon_path: Option<PathBuf>,

    /// Path to the game banner/cover image
    pub banner_path: Option<PathBuf>,

    /// Last time the game was played
    pub last_played: Option<DateTime<Utc>>,

    /// Total playtime in minutes
    pub playtime_minutes: u64,

    /// When the game was added to the library
    pub added_date: DateTime<Utc>,

    /// Additional launch arguments
    pub launch_args: Option<String>,
}

impl Game {
    /// Create a new game with minimal required fields
    pub fn new(name: String, executable_path: PathBuf, source: GameSource) -> Self {
        Self {
            id: GameId::new(),
            name,
            executable_path,
            install_path: None,
            source,
            source_id: None,
            categories: Vec::new(),
            favorite: false,
            icon_path: None,
            banner_path: None,
            last_played: None,
            playtime_minutes: 0,
            added_date: Utc::now(),
            launch_args: None,
        }
    }

    /// Create a game from an import source
    pub fn from_import(
        name: String,
        executable_path: PathBuf,
        install_path: PathBuf,
        source: GameSource,
        source_id: String,
    ) -> Self {
        Self {
            id: GameId::new(),
            name,
            executable_path,
            install_path: Some(install_path),
            source,
            source_id: Some(source_id),
            categories: Vec::new(),
            favorite: false,
            icon_path: None,
            banner_path: None,
            last_played: None,
            playtime_minutes: 0,
            added_date: Utc::now(),
            launch_args: None,
        }
    }

    /// Update the last played time to now
    pub fn mark_played(&mut self) {
        self.last_played = Some(Utc::now());
    }

    /// Add playtime in minutes
    pub fn add_playtime(&mut self, minutes: u64) {
        self.playtime_minutes += minutes;
    }

    /// Toggle favorite status
    pub fn toggle_favorite(&mut self) {
        self.favorite = !self.favorite;
    }

    /// Check if game has a specific category
    pub fn has_category(&self, category_id: &CategoryId) -> bool {
        self.categories.contains(category_id)
    }

    /// Add a category to the game
    pub fn add_category(&mut self, category_id: CategoryId) {
        if !self.has_category(&category_id) {
            self.categories.push(category_id);
        }
    }

    /// Remove a category from the game
    pub fn remove_category(&mut self, category_id: &CategoryId) {
        self.categories.retain(|c| c != category_id);
    }

    /// Get formatted playtime string
    pub fn playtime_display(&self) -> String {
        let hours = self.playtime_minutes / 60;
        let mins = self.playtime_minutes % 60;
        if hours > 0 {
            format!("{}h {}m", hours, mins)
        } else {
            format!("{}m", mins)
        }
    }
}
