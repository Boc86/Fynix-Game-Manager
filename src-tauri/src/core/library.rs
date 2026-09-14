use crate::models::game::Game;
use crate::core::plugin::StorePlugin;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Aggregates games from all registered store plugins into a unified library.
/// 
/// Thread-safe for concurrent access from Tauri commands.
/// Uses a Mutex-wrapped HashMap for game storage, keyed by composite ID.
pub struct GameLibrary {
    pub plugins: Arc<Mutex<Vec<Box<dyn StorePlugin>>>>,
    pub games: Arc<Mutex<HashMap<String, Game>>>,
}

impl Default for GameLibrary {
    fn default() -> Self {
        Self::new()
    }
}

impl GameLibrary {
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(Mutex::new(vec![])),
            games: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register a plugin to contribute games to this library.
    pub async fn register_plugin(&self, plugin: Box<dyn StorePlugin>) {
        let mut plugins = self.plugins.lock().await;
        plugins.push(plugin);
    }

    /// Trigger a full re-scan of all plugins.
    /// Calls `list_user_library()` on each plugin to get ALL games (installed + uninstalled).
    /// Clears and repopulates the library. Safe to call multiple times.
    pub async fn refresh(&self) -> anyhow::Result<usize> {
        let mut games = self.games.lock().await;
        games.clear();
        
        let plugins = self.plugins.lock().await;
        for plugin in plugins.iter() {
            if plugin.is_authenticated() {
                match plugin.list_user_library().await {
                    Ok(found) => {
                        for game in found {
                            games.insert(game.id.clone(), game);
                        }
                    }
                    Err(e) => {
                        eprintln!("Plugin '{}' error during scan: {}", plugin.name(), e);
                    }
                }
            }
        }
        
        Ok(games.len())
    }

    /// Get all games currently in the library.
    pub async fn all_games(&self) -> Vec<Game> {
        let games = self.games.lock().await;
        games.values().cloned().collect()
    }

    /// Get games sorted by last_played (most recent first).
    pub async fn recent_games(&self) -> Vec<Game> {
        let mut games: Vec<Game> = self.games.lock().await.values().cloned().collect();
        games.sort_by(|a, b| {
            b.last_played.cmp(&a.last_played)
        });
        games.into_iter().take(12).collect()
    }

    /// Search games by name (case-insensitive substring match).
    pub async fn search(&self, query: &str) -> Vec<Game> {
        let q = query.to_lowercase();
        let games = self.games.lock().await;
        games
            .values()
            .filter(|g| g.name.to_lowercase().contains(&q) || g.publisher.to_lowercase().contains(&q))
            .cloned()
            .collect()
    }

    /// Get a single game by ID.
    pub async fn get(&self, game_id: &str) -> Option<Game> {
        let games = self.games.lock().await;
        games.get(game_id).cloned()
    }

    /// Get all registered plugin names.
    pub async fn plugin_names(&self) -> Vec<String> {
        let plugins = self.plugins.lock().await;
        plugins.iter().map(|p| p.name().to_string()).collect()
    }

    /// Launch a game by its composite ID (e.g., "steam:730").
    ///
    /// Looks up the game's store_id, finds the matching plugin,
    /// and delegates the launch to that plugin.
    pub async fn launch_game(&self, game_id: &str) -> anyhow::Result<()> {
        let store_id = game_id
            .split(':')
            .next()
            .unwrap_or("unknown");

        let plugins = self.plugins.lock().await;
        for plugin in plugins.iter() {
            if plugin.store_id() == store_id {
                return plugin.launch_game(game_id).await;
            }
        }

        Err(anyhow::anyhow!(
            "No plugin found for store_id '{}'",
            store_id
        ))
    }

    /// Fetch artwork (cover URL) for a game from its store plugin.
    /// Falls back to searching other plugins if the game's own store can't provide art.
    pub async fn fetch_artwork(&self, game_id: &str) -> anyhow::Result<Option<String>> {
        let store_id = game_id
            .split(':')
            .next()
            .unwrap_or("unknown");

        let plugins = self.plugins.lock().await;

        // 1. Try the game's own store plugin first
        for plugin in plugins.iter() {
            if plugin.store_id() == store_id {
                if let Ok(Some(url)) = plugin.get_artwork(game_id).await {
                    return Ok(Some(url));
                }
            }
        }

        // 2. Fall back to any other plugin that might have artwork
        for plugin in plugins.iter() {
            if plugin.store_id() != store_id {
                if let Ok(Some(url)) = plugin.get_artwork(game_id).await {
                    return Ok(Some(url));
                }
            }
        }

        Ok(None)
    }
}
