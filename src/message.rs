use crate::data::{CategoryId, Game, GameId, GameSource};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// All possible messages/events in the application
#[derive(Debug, Clone)]
pub enum Message {
    // Navigation
    NavigateTo(View),

    // Library
    GameSelected(GameId),
    LaunchGame(GameId),
    GameLaunched(Result<(), String>),

    // Search & Filter
    SearchChanged(String),
    CategorySelected(Option<CategoryId>),
    SortChanged(SortOrder),
    ViewModeChanged(ViewMode),

    // Import
    StartImport(GameSource),
    ImportProgress(ImportProgress),
    ImportComplete(Result<Vec<Game>, String>),

    // Settings
    ThemeChanged(String),
    SettingChanged(SettingKey, SettingValue),

    // Game Management
    AddGamePressed,
    AddGame(Game),
    RemoveGame(GameId),
    EditGame(GameId),
    UpdateGame(GameId, GameUpdate),
    ToggleFavorite(GameId),

    // Add Game Form
    NewGameNameChanged(String),
    NewGamePathChanged(String),

    // Category Management
    AddCategory(String),
    RemoveCategory(CategoryId),
    AssignCategory(GameId, CategoryId),
    UnassignCategory(GameId, CategoryId),

    // File dialogs
    SelectExecutable,
    ExecutableSelected(Option<PathBuf>),

    // Persistence
    SaveLibrary,
    LibrarySaved(Result<(), String>),
    LoadLibrary,
    LibraryLoaded(Result<(), String>),

    // Misc
    Tick,
    None,
}

/// Application views/screens
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum View {
    #[default]
    Library,
    GameDetail(GameId),
    Settings,
    Import,
    AddGame,
}

/// Sort order for game library
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SortOrder {
    #[default]
    NameAsc,
    NameDesc,
    LastPlayed,
    RecentlyAdded,
    MostPlayed,
}

impl SortOrder {
    pub fn label(&self) -> &'static str {
        match self {
            SortOrder::NameAsc => "Name (A-Z)",
            SortOrder::NameDesc => "Name (Z-A)",
            SortOrder::LastPlayed => "Last Played",
            SortOrder::RecentlyAdded => "Recently Added",
            SortOrder::MostPlayed => "Most Played",
        }
    }

    pub fn all() -> &'static [SortOrder] {
        &[
            SortOrder::NameAsc,
            SortOrder::NameDesc,
            SortOrder::LastPlayed,
            SortOrder::RecentlyAdded,
            SortOrder::MostPlayed,
        ]
    }
}

/// View mode for library display
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ViewMode {
    #[default]
    Grid,
    List,
}

/// Import progress information
#[derive(Debug, Clone)]
pub struct ImportProgress {
    pub source: GameSource,
    pub current: usize,
    pub total: usize,
    pub current_game: Option<String>,
}

/// Setting keys for configuration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingKey {
    Theme,
    StartMinimized,
    CloseToTray,
    DefaultView,
}

/// Setting values
#[derive(Debug, Clone)]
pub enum SettingValue {
    String(String),
    Bool(bool),
}

/// Game update fields
#[derive(Debug, Clone, Default)]
pub struct GameUpdate {
    pub name: Option<String>,
    pub executable_path: Option<PathBuf>,
    pub launch_args: Option<String>,
    pub icon_path: Option<PathBuf>,
}
