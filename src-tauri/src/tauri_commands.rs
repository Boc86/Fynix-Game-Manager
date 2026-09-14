use tauri::State;
use crate::models::game::Game;
use crate::core::library::GameLibrary;

/// Return all games currently in the library.
///
/// Each game carries composite IDs prefixed with the store (e.g., "steam:730").
#[tauri::command]
pub async fn get_all_games(
    state: State<'_, GameLibrary>,
) -> Result<Vec<Game>, String> {
    Ok(state.all_games().await)
}

/// Return the 12 most recently played games (sorted by last_played descending).
#[tauri::command]
pub async fn get_recent_games(
    state: State<'_, GameLibrary>,
) -> Result<Vec<Game>, String> {
    Ok(state.recent_games().await)
}

/// Search games by name or publisher (case-insensitive substring match).
#[tauri::command]
pub async fn search_games(
    state: State<'_, GameLibrary>,
    query: String,
) -> Result<Vec<Game>, String> {
    Ok(state.search(&query).await)
}

/// Launch a game by its composite ID (e.g., "steam:730").
///
/// Delegates to the matching store plugin. Returns a status message.
#[tauri::command]
pub async fn launch_game(
    state: State<'_, GameLibrary>,
    game_id: String,
) -> Result<String, String> {
    let game = state.get(&game_id).await
        .ok_or_else(|| format!("Game not found: {}", game_id))?;

    state.launch_game(&game_id).await
        .map(|_| format!("Launched {}", game.name))
        .map_err(|e| e.to_string())
}

/// Trigger a full library refresh — re-scans all plugins for installed games.
/// Returns the number of games detected.
#[tauri::command]
pub async fn refresh_library(
    state: State<'_, GameLibrary>,
) -> Result<String, String> {
    let count = state.refresh().await.map_err(|e| e.to_string())?;
    Ok(format!("Found {} games", count))
}

/// Return all registered plugin store names.
#[tauri::command]
pub async fn get_plugin_names(
    state: State<'_, GameLibrary>,
) -> Result<Vec<String>, String> {
    Ok(state.plugin_names().await)
}
