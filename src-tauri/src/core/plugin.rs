use crate::models::game::Game;
use async_trait::async_trait;

/// Trait that all store plugins must implement.
/// 
/// Plugins are discovered in two ways:
/// 1. Built-in: statically linked (SteamPlugin, HeroicPlugin)
/// 2. External: dynamically loaded from `plugins/*.so` via PluginLoader
/// 
/// Each plugin identifies itself with a unique `store_id` (e.g., "steam", "heroic", "epic")
/// which is prefixed to game IDs to form composite IDs.
#[async_trait]
pub trait StorePlugin: Send + Sync {
    /// Human-readable display name for UI (e.g., "Steam", "Epic Games")
    fn name(&self) -> &str;
    
    /// Unique identifier used in composite game IDs (e.g., "steam", "epic")
    fn store_id(&self) -> &str;
    
    /// Scan the local filesystem for installed games from this store.
    /// Returns games that are currently installed locally.
    async fn detect_installed_games(&self) -> anyhow::Result<Vec<Game>>;
    
    /// Fetch the full user library from the store (requires API authentication).
    /// For stores without an API, this may return an empty list.
    async fn list_user_library(&self) -> anyhow::Result<Vec<Game>>;
    
    /// Get detailed information about a specific game by its composite ID.
    async fn get_game_details(&self, game_id: &str) -> anyhow::Result<Option<Game>>;
    
    /// Launch a game by its composite ID.
    /// For Windows games, this should use UMU (see UmuCommandBuilder).
    async fn launch_game(&self, game_id: &str) -> anyhow::Result<()>;
    
    /// Uninstall a game by its composite ID (optional — default does nothing)
    async fn uninstall_game(&self, _game_id: &str) -> anyhow::Result<()> {
        Ok(())
    }
    
    /// Check if the plugin is authenticated / has access to the store.
    fn is_authenticated(&self) -> bool;

    /// Fetch artwork (cover URL) for a game that doesn't have one.
    /// Implementations should look up the game in their store's cover art database.
    /// Returns a cover image URL if found, or None if no artwork is available.
    async fn get_artwork(&self, _game_id: &str) -> anyhow::Result<Option<String>> {
        // Default: no artwork lookup available
        Ok(None)
    }
}
