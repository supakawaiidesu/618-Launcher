use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CategoryId(pub Uuid);

impl CategoryId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for CategoryId {
    fn default() -> Self {
        Self::new()
    }
}

/// A category/tag for organizing games
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    /// Unique identifier
    pub id: CategoryId,

    /// Display name
    pub name: String,

    /// Optional color (hex format, e.g., "#FF5733")
    pub color: Option<String>,

    /// Optional icon name
    pub icon: Option<String>,
}

impl Category {
    /// Create a new category with just a name
    pub fn new(name: String) -> Self {
        Self {
            id: CategoryId::new(),
            name,
            color: None,
            icon: None,
        }
    }

    /// Create a new category with a name and color
    pub fn with_color(name: String, color: String) -> Self {
        Self {
            id: CategoryId::new(),
            name,
            color: Some(color),
            icon: None,
        }
    }
}

/// Default categories provided with a fresh library
pub fn default_categories() -> Vec<Category> {
    vec![
        Category::with_color("Action".to_string(), "#E74C3C".to_string()),
        Category::with_color("RPG".to_string(), "#9B59B6".to_string()),
        Category::with_color("Strategy".to_string(), "#3498DB".to_string()),
        Category::with_color("Puzzle".to_string(), "#2ECC71".to_string()),
        Category::with_color("Simulation".to_string(), "#F39C12".to_string()),
        Category::with_color("Sports".to_string(), "#1ABC9C".to_string()),
        Category::with_color("Indie".to_string(), "#E91E63".to_string()),
    ]
}
