// Fynix Game Manager — Rust backend for Tauri
// Lightweight cross-store game launcher for Linux

pub mod models;
pub mod core;
pub mod plugins;
pub mod tauri_commands;

pub use models::game::Game;
pub use core::plugin::StorePlugin;
pub use core::library::GameLibrary;
pub use core::umu::UmuCommandBuilder;
pub use core::plugin_loader::PluginLoader;
pub use plugins::steam::{SteamPlugin, parse_appmanifest};
pub use plugins::heroic::{HeroicPlugin, parse_heroic_games};
pub use plugins::lutris::{LutrisPlugin, parse_lutris_yaml};

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            tauri_commands::get_all_games,
            tauri_commands::get_recent_games,
            tauri_commands::search_games,
            tauri_commands::launch_game,
            tauri_commands::refresh_library,
            tauri_commands::get_plugin_names,
            tauri_commands::get_artwork,
            tauri_commands::fetch_missing_artwork,
        ])
        .setup(|app| {
            let library = GameLibrary::new();
            
            // Get plugin directory from app data dir
            let plugin_dir = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("./plugins"))
                .join("plugins");
            
            // Ensure plugin directory exists
            let _ = std::fs::create_dir_all(&plugin_dir);
            
            // Load all plugins synchronously (blocking in setup is fine for initial load)
            let loader = PluginLoader::new(plugin_dir);
            
            tauri::async_runtime::block_on(async {
                let _ = loader.load_all(&library).await;
                let _ = library.refresh().await;
            });
            
            app.manage(library);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
