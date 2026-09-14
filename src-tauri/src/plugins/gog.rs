use crate::models::game::Game;
use crate::core::plugin::StorePlugin;
use async_trait::async_trait;
use std::path::PathBuf;

/// Plugin for connecting to GOG.com via Heroic's gogdl integration.
pub struct GogPlugin {
    config_path: PathBuf,
}

impl GogPlugin {
    pub fn new() -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("heroic");
        Self { config_path: config_dir }
    }

    fn library_file(&self) -> PathBuf {
        self.config_path.join("store_cache").join("gog_library.json")
    }
}

impl Default for GogPlugin {
    fn default() -> Self {
        Self::new()
    }
}

pub fn parse_gog_library(content: &str) -> anyhow::Result<Vec<Game>> {
    let parsed: serde_json::Value = serde_json::from_str(content)?;
    let mut games = Vec::new();

    if let Some(arr) = parsed.get("games").and_then(|v| v.as_array()) {
        for entry in arr {
            let title = entry.get("title").and_then(|v| v.as_str()).unwrap_or("Unknown Game");
            let app_name = entry.get("app_name").and_then(|v| v.as_str()).unwrap_or("");

            let mut game = Game::new(&format!("gog:{}", app_name), title, "GOG");

            if let Some(developer) = entry.get("developer").and_then(|v| v.as_str()) {
                game.publisher = developer.to_string();
            }

            if let Some(art) = entry.get("art_square").and_then(|v| v.as_str()) {
                game.cover_url = Some(art.to_string());
            } else if let Some(art) = entry.get("art_cover").and_then(|v| v.as_str()) {
                game.cover_url = Some(art.to_string());
            }

            if let Some(is_installed) = entry.get("is_installed").and_then(|v| v.as_bool()) {
                if is_installed {
                    if let Some(install) = entry.get("install").and_then(|v| v.as_object()) {
                        if let Some(path) = install.get("path").and_then(|v| v.as_str()) {
                            game.install_path = Some(path.to_string());
                        }
                    }
                }
            }

            games.push(game);
        }
    }

    Ok(games)
}

#[async_trait]
impl StorePlugin for GogPlugin {
    fn name(&self) -> &str { "GOG" }
    fn store_id(&self) -> &str { "gog" }

    async fn detect_installed_games(&self) -> anyhow::Result<Vec<Game>> {
        let library_file = self.library_file();
        if !library_file.exists() { return Ok(Vec::new()); }
        let content = std::fs::read_to_string(&library_file)?;
        let all = parse_gog_library(&content)?;
        Ok(all.into_iter().filter(|g| g.install_path.is_some()).collect())
    }

    async fn list_user_library(&self) -> anyhow::Result<Vec<Game>> {
        let library_file = self.library_file();
        if !library_file.exists() { return Ok(Vec::new()); }
        let content = std::fs::read_to_string(&library_file)?;
        parse_gog_library(&content)
    }

    async fn get_game_details(&self, game_id: &str) -> anyhow::Result<Option<Game>> {
        let all = self.list_user_library().await?;
        Ok(all.into_iter().find(|g| g.id == game_id))
    }

    async fn launch_game(&self, game_id: &str) -> anyhow::Result<()> {
        let _id = game_id.strip_prefix("gog:")
            .ok_or_else(|| anyhow::anyhow!("invalid gog game id"))?;
        std::process::Command::new("heroic").arg("run").arg(game_id).spawn()?;
        Ok(())
    }

    fn is_authenticated(&self) -> bool {
        self.library_file().exists()
    }

    async fn get_artwork(&self, game_id: &str) -> anyhow::Result<Option<String>> {
        let app_name = game_id.strip_prefix("gog:")
            .ok_or_else(|| anyhow::anyhow!("invalid gog game id"))?;
        let library_file = self.library_file();
        if !library_file.exists() { return Ok(None); }
        let content = std::fs::read_to_string(&library_file)?;
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(arr) = parsed.get("games").and_then(|v| v.as_array()) {
                for entry in arr {
                    if entry.get("app_name").and_then(|v| v.as_str()) == Some(app_name) {
                        if let Some(art) = entry.get("art_square").and_then(|v| v.as_str()) {
                            return Ok(Some(art.to_string()));
                        }
                        if let Some(art) = entry.get("art_cover").and_then(|v| v.as_str()) {
                            return Ok(Some(art.to_string()));
                        }
                    }
                }
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_gog_library() {
        let json = r#"{"games": [{"app_name": "1310521702", "title": "Heroes of Loot 2", "developer": "Orangepixel", "art_square": "https://images.gog.com/cover.jpg", "art_cover": "https://images.gog.com/cover2.jpg", "is_installed": false, "install": {"is_dlc": false}}], "__timestamp": "2024-01-01"}"#;
        let games = parse_gog_library(json).unwrap();
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].id, "gog:1310521702");
        assert_eq!(games[0].name, "Heroes of Loot 2");
        assert_eq!(games[0].publisher, "Orangepixel");
        assert_eq!(games[0].cover_url, Some("https://images.gog.com/cover.jpg".to_string()));
    }

    #[test]
    fn test_parse_gog_library_empty() {
        let json = r#"{"games": [], "__timestamp": "2024-01-01"}"#;
        let games = parse_gog_library(json).unwrap();
        assert!(games.is_empty());
    }
}
