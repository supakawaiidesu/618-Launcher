// Allow dead code during development - features will be wired up incrementally
#![allow(dead_code)]

mod app;
mod constants;
mod data;
mod message;
mod theme;

// Views
mod views;

// Components
mod components;

// Import system
mod import;

// Game launching
mod launcher;

// Platform-specific code
mod platform;

use app::App;
use constants::{APP_NAME, DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_WIDTH};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() -> iced::Result {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "launcher_618=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting {}", APP_NAME);

    iced::application(App::new, App::update, App::view)
        .title(APP_NAME)
        .subscription(App::subscription)
        .theme(App::theme)
        .window_size((DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT))
        .run()
}
