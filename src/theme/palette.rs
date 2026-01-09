use iced::Color;
use serde::{Deserialize, Serialize};

/// Color palette for a theme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Palette {
    /// Main background color
    pub background: HexColor,

    /// Surface/card background color
    pub surface: HexColor,

    /// Primary accent color
    pub primary: HexColor,

    /// Secondary accent color
    pub secondary: HexColor,

    /// Highlight/accent color
    pub accent: HexColor,

    /// Main text color
    pub text: HexColor,

    /// Secondary/muted text color
    pub text_secondary: HexColor,

    /// Success/positive color
    pub success: HexColor,

    /// Warning color
    pub warning: HexColor,

    /// Error/danger color
    pub error: HexColor,
}

impl Palette {
    /// Dark theme palette
    pub fn dark() -> Self {
        Self {
            background: HexColor::from_hex("#1a1a2e"),
            surface: HexColor::from_hex("#16213e"),
            primary: HexColor::from_hex("#0f3460"),
            secondary: HexColor::from_hex("#533483"),
            accent: HexColor::from_hex("#e94560"),
            text: HexColor::from_hex("#eaeaea"),
            text_secondary: HexColor::from_hex("#a0a0a0"),
            success: HexColor::from_hex("#4ade80"),
            warning: HexColor::from_hex("#fbbf24"),
            error: HexColor::from_hex("#ef4444"),
        }
    }

    /// Light theme palette
    pub fn light() -> Self {
        Self {
            background: HexColor::from_hex("#f8f9fa"),
            surface: HexColor::from_hex("#ffffff"),
            primary: HexColor::from_hex("#3b82f6"),
            secondary: HexColor::from_hex("#8b5cf6"),
            accent: HexColor::from_hex("#ec4899"),
            text: HexColor::from_hex("#1f2937"),
            text_secondary: HexColor::from_hex("#6b7280"),
            success: HexColor::from_hex("#22c55e"),
            warning: HexColor::from_hex("#f59e0b"),
            error: HexColor::from_hex("#ef4444"),
        }
    }
}

/// A color stored as a hex string but convertible to iced Color
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct HexColor(pub String);

impl HexColor {
    /// Create from a hex string (with or without #)
    pub fn from_hex(hex: &str) -> Self {
        Self(hex.to_string())
    }

    /// Parse the hex string to RGB values
    fn parse(&self) -> (u8, u8, u8) {
        let hex = self.0.trim_start_matches('#');

        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
            (r, g, b)
        } else {
            // Default to black if parsing fails
            (0, 0, 0)
        }
    }
}

impl From<HexColor> for Color {
    fn from(hex: HexColor) -> Self {
        let (r, g, b) = hex.parse();
        Color::from_rgb8(r, g, b)
    }
}

impl From<&HexColor> for Color {
    fn from(hex: &HexColor) -> Self {
        let (r, g, b) = hex.parse();
        Color::from_rgb8(r, g, b)
    }
}
