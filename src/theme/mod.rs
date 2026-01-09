mod palette;

pub use palette::Palette;

use iced::theme;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;

/// Custom theme for the application
#[derive(Debug, Clone)]
pub struct CustomTheme {
    pub name: String,
    pub palette: Palette,
}

impl CustomTheme {
    /// Create a new custom theme
    pub fn new(name: String, palette: Palette) -> Self {
        Self { name, palette }
    }

    /// Built-in dark theme
    pub fn dark() -> Self {
        Self {
            name: "Dark".to_string(),
            palette: Palette::dark(),
        }
    }

    /// Built-in light theme
    pub fn light() -> Self {
        Self {
            name: "Light".to_string(),
            palette: Palette::light(),
        }
    }

    /// Load a theme from a JSON file
    pub async fn load_from_file(path: &Path) -> Result<Self, ThemeError> {
        let content = fs::read_to_string(path)
            .await
            .map_err(|e| ThemeError::Io(e.to_string()))?;

        let theme_file: ThemeFile = serde_json::from_str(&content)
            .map_err(|e| ThemeError::Parse(e.to_string()))?;

        Ok(Self {
            name: theme_file.name,
            palette: theme_file.palette,
        })
    }

    /// Convert to iced Theme
    pub fn to_iced_theme(&self) -> iced::Theme {
        let iced_palette = theme::Palette {
            background: (&self.palette.background).into(),
            text: (&self.palette.text).into(),
            primary: (&self.palette.primary).into(),
            success: (&self.palette.success).into(),
            warning: (&self.palette.warning).into(),
            danger: (&self.palette.error).into(),
        };

        iced::Theme::custom(self.name.clone(), iced_palette)
    }

    /// Get theme by name
    pub fn by_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "light" => Self::light(),
            _ => Self::dark(), // Default to dark
        }
    }
}

/// Theme file format for loading from JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeFile {
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub author: String,
    pub palette: Palette,
}

/// Errors that can occur when loading themes
#[derive(Debug, thiserror::Error)]
pub enum ThemeError {
    #[error("IO error: {0}")]
    Io(String),

    #[error("Parse error: {0}")]
    Parse(String),
}

/// Get list of available themes (built-in + user themes)
pub async fn available_themes(user_themes_dir: &Path) -> Vec<String> {
    let mut themes = vec!["Dark".to_string(), "Light".to_string()];

    // Try to read user themes directory
    if let Ok(mut entries) = fs::read_dir(user_themes_dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".json") {
                    // Try to load and get the theme name
                    if let Ok(theme) = CustomTheme::load_from_file(&entry.path()).await {
                        themes.push(theme.name);
                    }
                }
            }
        }
    }

    themes
}
