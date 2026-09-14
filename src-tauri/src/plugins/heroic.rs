use crate::models::game::Game;
use crate::core::plugin::StorePlugin;
use async_trait::async_trait;
use std::path::PathBuf;

/// Parse Heroic Games Launcher's `games.txt` file.
/// This is a JSON array of game objects with fields like:
/// `{"id": "...", "name": "...", "executable": "...", "installPath": "..."}`
pub fn parse_heroic_games(content: &str) -> anyhow::Result<Vec<Game>> {
    let parsed: serde_json::Value = serde_json::from_str(content)?;
    let mut games = Vec::new();

    if let Some(arr) = parsed.as_array() {
        for entry in arr {
            let id = entry["id"].as_str().ok_or_else(|| anyhow::anyhow!("no id field"))?;
            let name = entry["name"].as_str().unwrap_or("Unknown");
            let mut game = Game::new(&format!("heroic:{}", id), name, "Unknown");

            if let Some(exec) = entry.get("executable").and_then(|v| v.as_str()) {
                game = game.with_executable(exec.to_string());
            }
            if let Some(install_path) = entry.get("installPath").and_then(|v| v.as_str()) {
                game = game.with_install_path(install_path.to_string());
            }
            if let Some(last_played) = entry.get("lastTimePlayed").and_then(|v| v.as_u64()) {
                game = game.with_last_played(last_played / 1000); // Convert ms to seconds
            }
            if let Some(playtime) = entry.get("playtime").and_then(|v| v.as_f64()) {
                game = game.with_playtime(playtime / 3600.0); // Convert seconds to hours
            }

            games.push(game);
        }
    }
    Ok(games)
}

/// Plugin for detecting games installed via Heroic Games Launcher.
///
/// Heroic stores game data in `~/.config/heroic/games.txt` (legacy) or
/// `~/.local/share/heroic/games/` as individual JSON files.
pub struct HeroicPlugin {
    config_path: PathBuf,
}

impl HeroicPlugin {
    pub fn new() -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("heroic");
        Self { config_path: config_dir }
    }

    fn games_file(&self) -> PathBuf {
        self.config_path.join("games.txt")
    }

    fn games_dir(&self) -> PathBuf {
        self.config_path.join("games")
    }
}

impl Default for HeroicPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StorePlugin for HeroicPlugin {
    fn name(&self) -> &str {
        "Heroic Games"
    }

    fn store_id(&self) -> &str {
        "heroic"
    }

    async fn detect_installed_games(&self) -> anyhow::Result<Vec<Game>> {
        let mut games = Vec::new();

        // Check legacy games.txt file
        let legacy_file = self.games_file();
        if legacy_file.exists() {
            let content = std::fs::read_to_string(&legacy_file)?;
            games.extend(parse_heroic_games(&content)?);
        }

        // Check individual game files in games/ directory
        let games_dir = self.games_dir();
        if games_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&games_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) == Some("json") {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            let parsed: serde_json::Value = serde_json::from_str(&content)?;
                            if let Some(id) = parsed["id"].as_str() {
                                let name = parsed["name"].as_str().unwrap_or("Unknown");
                                let mut game = Game::new(&format!("heroic:{}", id), name, "Unknown");
                                if let Some(exec) = parsed.get("executable").and_then(|v| v.as_str()) {
                                    game = game.with_executable(exec.to_string());
                                }
                                if let Some(install_path) = parsed.get("installPath").and_then(|v| v.as_str()) {
                                    game = game.with_install_path(install_path.to_string());
                                }
                                games.push(game);
                            }
                        }
                    }
                }
            }
        }

        Ok(games)
    }

    async fn list_user_library(&self) -> anyhow::Result<Vec<Game>> {
        self.detect_installed_games().await
    }

    async fn get_game_details(&self, game_id: &str) -> anyhow::Result<Option<Game>> {
        let all = self.detect_installed_games().await?;
        Ok(all.into_iter().find(|g| g.id == game_id))
    }

    async fn launch_game(&self, game_id: &str) -> anyhow::Result<()> {
        let _id = game_id.strip_prefix("heroic:").ok_or_else(|| anyhow::anyhow!("invalid heroic id"))?;
        std::process::Command::new("heroic")
            .arg("run")
            .arg(game_id)
            .spawn()?;
        Ok(())
    }

    fn is_authenticated(&self) -> bool {
        self.games_file().exists() || self.games_dir().exists()
    }

    async fn get_artwork(&self, game_id: &str) -> anyhow::Result<Option<String>> {
        let id = game_id.strip_prefix("heroic:").ok_or_else(|| anyhow::anyhow!("invalid heroic id"))?;
        // Heroic stores cover art in its cache directory, or can use CDN
        // Try local cache first, then fall back to CDN URL format
        let cache_path = self.config_path.join("images-cache").join(format!("{}.jpg", id));
        if cache_path.exists() {
            return Ok(Some(format!("file://{}", cache_path.display())));
        }
        // Heroic CDN URL pattern
        Ok(Some(format!("https://cdn.gamenerdstore.com/heroic/covers/{}.jpg", id)))
    }
}
