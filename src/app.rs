use std::path::PathBuf;

use directories::ProjectDirs;
use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Element, Length, Subscription, Task, Theme};

use crate::constants::{
    APP_APPLICATION, APP_ORGANIZATION, APP_QUALIFIER, CONFIG_FILE, LIBRARY_FILE,
};
use crate::data::{Category, CategoryId, Config, Game, GameId, GameSource, Library};
use crate::message::{Message, SortOrder, View, ViewMode};
use crate::theme::CustomTheme;

/// Main application state
pub struct App {
    // Data
    library: Library,
    config: Config,

    // Theme
    theme: CustomTheme,

    // UI State
    current_view: View,
    search_query: String,
    selected_category: Option<CategoryId>,
    selected_game: Option<GameId>,
    sort_order: SortOrder,
    view_mode: ViewMode,

    // Form state for adding games
    new_game_name: String,
    new_game_path: String,

    // Paths
    data_dir: PathBuf,
}

impl Default for App {
    fn default() -> Self {
        // Get application directories
        let project_dirs = ProjectDirs::from(APP_QUALIFIER, APP_ORGANIZATION, APP_APPLICATION)
            .expect("Could not determine project directories");

        let data_dir = project_dirs.data_dir().to_path_buf();

        tracing::info!("Data directory: {:?}", data_dir);

        Self {
            library: Library::new(),
            config: Config::default(),
            theme: CustomTheme::dark(),
            current_view: View::Library,
            search_query: String::new(),
            selected_category: None,
            selected_game: None,
            sort_order: SortOrder::NameAsc,
            view_mode: ViewMode::Grid,
            new_game_name: String::new(),
            new_game_path: String::new(),
            data_dir,
        }
    }
}

impl App {
    /// Create a new application instance with initial task
    pub fn new() -> (Self, Task<Message>) {
        let app = Self::default();
        // Load library and config asynchronously
        let load_task = Task::perform(async {}, |_| Message::LoadLibrary);
        (app, load_task)
    }

    /// Get the library file path
    fn library_path(&self) -> PathBuf {
        self.data_dir.join(LIBRARY_FILE)
    }

    /// Get the config file path
    fn config_path(&self) -> PathBuf {
        self.data_dir.join(CONFIG_FILE)
    }

    /// Handle messages and update state
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // Navigation
            Message::NavigateTo(view) => {
                self.current_view = view;
                Task::none()
            }

            // Game selection and launching
            Message::GameSelected(id) => {
                self.selected_game = Some(id);
                Task::none()
            }

            Message::LaunchGame(id) => {
                if let Some(game) = self.library.get_game_mut(&id) {
                    game.mark_played();
                    let exe_path = game.executable_path.clone();
                    let launch_args = game.launch_args.clone();

                    return Task::perform(
                        async move {
                            crate::launcher::launch_game(&exe_path, launch_args.as_deref())
                                .map_err(|e| e.to_string())
                        },
                        Message::GameLaunched,
                    );
                }
                Task::none()
            }

            Message::GameLaunched(result) => {
                match &result {
                    Ok(()) => tracing::info!("Game launched successfully"),
                    Err(e) => tracing::error!("Failed to launch game: {}", e),
                }
                // Save library to persist the last_played update
                self.save_library()
            }

            // Search and filtering
            Message::SearchChanged(query) => {
                self.search_query = query;
                Task::none()
            }

            Message::CategorySelected(category) => {
                self.selected_category = category;
                Task::none()
            }

            Message::SortChanged(order) => {
                self.sort_order = order;
                Task::none()
            }

            Message::ViewModeChanged(mode) => {
                self.view_mode = mode;
                Task::none()
            }

            // Game management
            Message::AddGamePressed => {
                self.current_view = View::AddGame;
                self.new_game_name.clear();
                self.new_game_path.clear();
                Task::none()
            }

            Message::AddGame(game) => {
                self.library.add_game(game);
                self.current_view = View::Library;
                self.save_library()
            }

            Message::RemoveGame(id) => {
                self.library.remove_game(&id);
                if self.selected_game == Some(id) {
                    self.selected_game = None;
                }
                self.save_library()
            }

            Message::EditGame(id) => {
                self.current_view = View::GameDetail(id);
                Task::none()
            }

            Message::UpdateGame(id, update) => {
                if let Some(game) = self.library.get_game_mut(&id) {
                    if let Some(name) = update.name {
                        game.name = name;
                    }
                    if let Some(path) = update.executable_path {
                        game.executable_path = path;
                    }
                    if let Some(args) = update.launch_args {
                        game.launch_args = Some(args);
                    }
                    if let Some(icon) = update.icon_path {
                        game.icon_path = Some(icon);
                    }
                }
                self.save_library()
            }

            Message::ToggleFavorite(id) => {
                if let Some(game) = self.library.get_game_mut(&id) {
                    game.toggle_favorite();
                }
                self.save_library()
            }

            // Add Game Form
            Message::NewGameNameChanged(name) => {
                self.new_game_name = name;
                Task::none()
            }

            Message::NewGamePathChanged(path) => {
                self.new_game_path = path;
                Task::none()
            }

            // Category management
            Message::AddCategory(name) => {
                let category = Category::new(name);
                self.library.add_category(category);
                self.save_library()
            }

            Message::RemoveCategory(id) => {
                self.library.remove_category(&id);
                if self.selected_category == Some(id) {
                    self.selected_category = None;
                }
                self.save_library()
            }

            Message::AssignCategory(game_id, category_id) => {
                if let Some(game) = self.library.get_game_mut(&game_id) {
                    game.add_category(category_id);
                }
                self.save_library()
            }

            Message::UnassignCategory(game_id, category_id) => {
                if let Some(game) = self.library.get_game_mut(&game_id) {
                    game.remove_category(&category_id);
                }
                self.save_library()
            }

            // Settings
            Message::ThemeChanged(theme_name) => {
                self.theme = CustomTheme::by_name(&theme_name);
                self.config.theme = theme_name;
                self.save_config()
            }

            Message::SettingChanged(key, value) => {
                use crate::message::{SettingKey, SettingValue};
                match (key, value) {
                    (SettingKey::StartMinimized, SettingValue::Bool(v)) => {
                        self.config.start_minimized = v;
                    }
                    (SettingKey::CloseToTray, SettingValue::Bool(v)) => {
                        self.config.close_to_tray = v;
                    }
                    _ => {}
                }
                self.save_config()
            }

            // File dialogs (placeholder - requires native dialog integration)
            Message::SelectExecutable => {
                // TODO: Implement native file dialog
                tracing::info!("File dialog requested");
                Task::none()
            }

            Message::ExecutableSelected(path) => {
                if let Some(p) = path {
                    self.new_game_path = p.to_string_lossy().to_string();
                }
                Task::none()
            }

            // Persistence
            Message::SaveLibrary => self.save_library(),

            Message::LibrarySaved(result) => {
                match &result {
                    Ok(()) => tracing::debug!("Library saved"),
                    Err(e) => tracing::error!("Failed to save library: {}", e),
                }
                Task::none()
            }

            Message::LoadLibrary => {
                let library_path = self.library_path();
                let config_path = self.config_path();

                Task::perform(
                    async move {
                        let library = Library::load_or_create(&library_path).await;
                        let config = Config::load_or_create(&config_path).await;
                        (library, config)
                    },
                    |_| Message::LibraryLoaded(Ok(())),
                )
            }

            Message::LibraryLoaded(result) => {
                // This is a simplified handler - in a real impl we'd pass the loaded data
                tracing::info!("Library load completed: {:?}", result);
                Task::none()
            }

            // Import (placeholder)
            Message::StartImport(source) => {
                tracing::info!("Starting import from {:?}", source);
                // TODO: Implement import
                Task::none()
            }

            Message::ImportProgress(_progress) => Task::none(),

            Message::ImportComplete(result) => {
                match result {
                    Ok(games) => {
                        for game in games {
                            self.library.add_game(game);
                        }
                        self.save_library()
                    }
                    Err(e) => {
                        tracing::error!("Import failed: {}", e);
                        Task::none()
                    }
                }
            }

            // Misc
            Message::Tick => Task::none(),
            Message::None => Task::none(),
        }
    }

    /// Save library to disk
    fn save_library(&self) -> Task<Message> {
        let library = self.library.clone();
        let path = self.library_path();

        Task::perform(
            async move { library.save_to_file(&path).await.map_err(|e| e.to_string()) },
            Message::LibrarySaved,
        )
    }

    /// Save config to disk
    fn save_config(&self) -> Task<Message> {
        let config = self.config.clone();
        let path = self.config_path();

        Task::perform(
            async move {
                config.save_to_file(&path).await.map_err(|e| e.to_string())
            },
            |_| Message::None,
        )
    }

    /// Render the UI
    pub fn view(&self) -> Element<'_, Message> {
        let content = match &self.current_view {
            View::Library => self.view_library(),
            View::GameDetail(id) => self.view_game_detail(*id),
            View::Settings => self.view_settings(),
            View::Import => self.view_import(),
            View::AddGame => self.view_add_game(),
        };

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(0)
            .into()
    }

    /// View: Main library
    fn view_library(&self) -> Element<'_, Message> {
        // Header
        let header = self.view_header();

        // Sidebar
        let sidebar = self.view_sidebar();

        // Game grid/list
        let games = self.get_filtered_games();
        let game_grid = self.view_game_grid(&games);

        // Status bar
        let status = self.view_status_bar();

        // Layout
        let main_content = row![
            sidebar,
            container(scrollable(game_grid))
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(20),
        ];

        column![header, main_content, status]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// View: Header bar
    fn view_header(&self) -> Element<'_, Message> {
        let title = text("618-Launcher").size(24);

        let search = text_input("Search games...", &self.search_query)
            .on_input(Message::SearchChanged)
            .width(300);

        let settings_btn = button(text("Settings"))
            .on_press(Message::NavigateTo(View::Settings));

        row![
            title,
            Space::new().width(Length::Fill),
            search,
            settings_btn,
        ]
        .spacing(20)
        .padding(15)
        .into()
    }

    /// View: Sidebar with categories
    fn view_sidebar(&self) -> Element<'_, Message> {
        let all_games_btn = button(
            text(format!("All Games ({})", self.library.game_count())),
        )
        .width(Length::Fill)
        .on_press(Message::CategorySelected(None));

        let favorites_btn = button(
            text(format!("Favorites ({})", self.library.favorite_games().len())),
        )
        .width(Length::Fill)
        .on_press(Message::CategorySelected(None)); // TODO: Filter favorites

        let mut category_buttons: Vec<Element<Message>> = self
            .library
            .all_categories()
            .iter()
            .map(|cat| {
                button(text(&cat.name))
                    .width(Length::Fill)
                    .on_press(Message::CategorySelected(Some(cat.id)))
                    .into()
            })
            .collect();

        let add_game_btn = button(text("+ Add Game"))
            .width(Length::Fill)
            .on_press(Message::AddGamePressed);

        let import_btn = button(text("Import Games"))
            .width(Length::Fill)
            .on_press(Message::NavigateTo(View::Import));

        let mut sidebar_items = vec![
            all_games_btn.into(),
            favorites_btn.into(),
        ];
        sidebar_items.append(&mut category_buttons);
        sidebar_items.push(add_game_btn.into());
        sidebar_items.push(import_btn.into());

        container(
            scrollable(
                column(sidebar_items)
                    .spacing(5)
                    .padding(10)
                    .width(Length::Fill),
            ),
        )
        .width(200)
        .height(Length::Fill)
        .into()
    }

    /// View: Game grid
    fn view_game_grid(&self, games: &[&Game]) -> Element<'_, Message> {
        if games.is_empty() {
            return container(
                text("No games in library. Add some games to get started!")
                    .size(18),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into();
        }

        let game_cards: Vec<Element<Message>> = games
            .iter()
            .map(|game| self.view_game_card(game))
            .collect();

        // Simple vertical list for now (grid layout will come later)
        column(game_cards)
            .spacing(10)
            .width(Length::Fill)
            .into()
    }

    /// View: Individual game card
    fn view_game_card(&self, game: &Game) -> Element<'_, Message> {
        let name = text(game.name.clone()).size(16);
        let source = text(game.source.label()).size(12);

        let game_id = game.id;
        let play_btn = button(text("Play"))
            .on_press(Message::LaunchGame(game_id));

        let fav_icon = if game.favorite { "★" } else { "☆" };
        let fav_btn = button(text(fav_icon))
            .on_press(Message::ToggleFavorite(game_id));

        let card_content = row![
            column![name, source].spacing(5),
            Space::new().width(Length::Fill),
            fav_btn,
            play_btn,
        ]
        .spacing(10)
        .padding(15)
        .align_y(iced::Alignment::Center);

        container(card_content)
            .width(Length::Fill)
            .into()
    }

    /// View: Status bar
    fn view_status_bar(&self) -> Element<'_, Message> {
        let game_count = text(format!("{} games", self.library.game_count())).size(12);

        row![game_count]
            .padding(10)
            .into()
    }

    /// View: Game detail page
    fn view_game_detail(&self, id: GameId) -> Element<'_, Message> {
        if let Some(game) = self.library.get_game(&id) {
            let title = text(&game.name).size(28);
            let source = text(format!("Source: {}", game.source.label()));
            let path = text(format!("Path: {:?}", game.executable_path)).size(12);
            let playtime = text(format!("Playtime: {}", game.playtime_display()));

            let back_btn = button(text("Back"))
                .on_press(Message::NavigateTo(View::Library));

            let play_btn = button(text("Play"))
                .on_press(Message::LaunchGame(id));

            column![
                back_btn,
                title,
                source,
                path,
                playtime,
                play_btn,
            ]
            .spacing(15)
            .padding(20)
            .into()
        } else {
            column![
                text("Game not found"),
                button(text("Back")).on_press(Message::NavigateTo(View::Library)),
            ]
            .spacing(15)
            .padding(20)
            .into()
        }
    }

    /// View: Settings page
    fn view_settings(&self) -> Element<'_, Message> {
        let title = text("Settings").size(24);

        let back_btn = button(text("Back"))
            .on_press(Message::NavigateTo(View::Library));

        let theme_section = column![
            text("Theme").size(18),
            row![
                button(text("Dark")).on_press(Message::ThemeChanged("dark".to_string())),
                button(text("Light")).on_press(Message::ThemeChanged("light".to_string())),
            ]
            .spacing(10),
        ]
        .spacing(10);

        column![
            row![back_btn, title].spacing(20),
            theme_section,
        ]
        .spacing(20)
        .padding(20)
        .into()
    }

    /// View: Import page
    fn view_import(&self) -> Element<'_, Message> {
        let title = text("Import Games").size(24);

        let back_btn = button(text("Back"))
            .on_press(Message::NavigateTo(View::Library));

        let steam_btn = button(text("Import from Steam"))
            .on_press(Message::StartImport(GameSource::Steam));

        let epic_btn = button(text("Import from Epic Games"))
            .on_press(Message::StartImport(GameSource::Epic));

        let gog_btn = button(text("Import from GOG Galaxy"))
            .on_press(Message::StartImport(GameSource::GOG));

        column![
            row![back_btn, title].spacing(20),
            text("Select a source to import games from:"),
            steam_btn,
            epic_btn,
            gog_btn,
        ]
        .spacing(15)
        .padding(20)
        .into()
    }

    /// View: Add game form
    fn view_add_game(&self) -> Element<'_, Message> {
        let title = text("Add Game").size(24);

        let back_btn = button(text("Back"))
            .on_press(Message::NavigateTo(View::Library));

        let name_input = text_input("Game name", &self.new_game_name)
            .on_input(Message::NewGameNameChanged)
            .padding(10);

        let path_input = text_input("Executable path (e.g., C:\\Games\\game.exe)", &self.new_game_path)
            .on_input(Message::NewGamePathChanged)
            .padding(10);

        let can_add = !self.new_game_name.trim().is_empty()
            && !self.new_game_path.trim().is_empty();

        let add_btn = button(text("Add Game")).on_press_maybe(
            if can_add {
                Some(Message::AddGame(Game::new(
                    self.new_game_name.trim().to_string(),
                    PathBuf::from(self.new_game_path.trim()),
                    GameSource::Manual,
                )))
            } else {
                None
            },
        );

        column![
            row![back_btn, title].spacing(20),
            text("Game Name:"),
            name_input,
            text("Executable Path:"),
            path_input,
            add_btn,
        ]
        .spacing(15)
        .padding(20)
        .into()
    }

    /// Get filtered and sorted games based on current filters
    fn get_filtered_games(&self) -> Vec<&Game> {
        let mut games = if let Some(category_id) = &self.selected_category {
            self.library.games_in_category(category_id)
        } else if !self.search_query.is_empty() {
            self.library.search_games(&self.search_query)
        } else {
            self.library.all_games()
        };

        // Apply sorting
        match self.sort_order {
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

    /// Get the current theme
    pub fn theme(&self) -> Theme {
        self.theme.to_iced_theme()
    }

    /// Handle subscriptions (for async events, timers, etc.)
    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }
}
