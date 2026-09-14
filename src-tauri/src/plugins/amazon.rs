use crate::models::game::Game;
use crate::core::plugin::StorePlugin;
use async_trait::async_trait;
use std::path::PathBuf;

/// Plugin for connecting to Amazon Luna / Prime Gaming via Heroic's Nile integration.
pub struct AmazonPlugin {
    config_path: PathBuf,
}

impl AmazonPlugin {
    pub fn new() -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("heroic");
        Self { config_path: config_dir }
    }

    fn library_file(&self) -> PathBuf {
        self.config_path.join("store_cache").join("nile_library.json")
    }
}

impl Default for AmazonPlugin {
    fn default() -> Self {
        Self::new()
    }
}

pub fn parse_nile_library(content: &str) -> anyhow::Result<Vec<Game>> {
    let parsed: serde_json::Value = serde_json::from_str(content)?;
    let mut games = Vec::new();

    if let Some(arr) = parsed.get("library").and_then(|v| v.as_array()) {
        for entry in arr {
            let title = entry.get("title").and_then(|v| v.as_str()).unwrap_or("Unknown Game");
            let app_name = entry.get("app_name").and_then(|v| v.as_str()).unwrap_or("");

            let mut game = Game::new(&format!("amazon:{}", app_name), title, "Amazon");

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
impl StorePlugin for AmazonPlugin {
    fn name(&self) -> &str { "Amazon Prime" }
    fn store_id(&self) -> &str { "amazon" }

    async fn detect_installed_games(&self) -> anyhow::Result<Vec<Game>> {
        let library_file = self.library_file();
        if !library_file.exists() { return Ok(Vec::new()); }
        let content = std::fs::read_to_string(&library_file)?;
        let all = parse_nile_library(&content)?;
        Ok(all.into_iter().filter(|g| g.install_path.is_some()).collect())
    }

    async fn list_user_library(&self) -> anyhow::Result<Vec<Game>> {
        let library_file = self.library_file();
        if !library_file.exists() { return Ok(Vec::new()); }
        let content = std::fs::read_to_string(&library_file)?;
        parse_nile_library(&content)
    }

    async fn get_game_details(&self, game_id: &str) -> anyhow::Result<Option<Game>> {
        let all = self.list_user_library().await?;
        Ok(all.into_iter().find(|g| g.id == game_id))
    }

    async fn launch_game(&self, game_id: &str) -> anyhow::Result<()> {
        let _id = game_id.strip_prefix("amazon:")
            .ok_or_else(|| anyhow::anyhow!("invalid amazon game id"))?;
        std::process::Command::new("heroic").arg("run").arg(game_id).spawn()?;
        Ok(())
    }

    fn is_authenticated(&self) -> bool {
        self.library_file().exists()
    }

    async fn get_artwork(&self, game_id: &str) -> anyhow::Result<Option<String>> {
        let app_name = game_id.strip_prefix("amazon:")
            .ok_or_else(|| anyhow::anyhow!("invalid amazon game id"))?;
        let library_file = self.library_file();
        if !library_file.exists() { return Ok(None); }
        let content = std::fs::read_to_string(&library_file)?;
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(arr) = parsed.get("library").and_then(|v| v.as_array()) {
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
    fn test_parse_nile_library() {
        let json = r#"{"library": [{"app_name": "eeb2fd4c-u7aa-4e73-a1d4-071b90e1060d", "title": "Pumped BMX Pro", "art_square": "https://m.media-amazon.com/cover.jpg", "art_cover": "https://m.media-amazon.com/cover2.jpg", "developer": "Yeah Us! Games", "is_installed": false, "install": {}}], "__timestamp": "2024-01-01"}"#;
        let games = parse_nile_library(json).unwrap();
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].id, "amazon:eeb2fd4c-u7aa-4e73-a1d4-071b90e1060d");
        assert_eq!(games[0].name, "Pumped BMX Pro");
        assert_eq!(games[0].publisher, "Yeah Us! Games");
        assert_eq!(games[0].cover_url, Some("https://m.media-amazon.com/cover.jpg".to_string()));
    }

    #[test]
    fn test_parse_nile_library_empty() {
        let json = r#"{"library": [], "__timestamp": "2024-01-01"}"#;
        let games = parse_nile_library(json).unwrap();
        assert!(games.is_empty());
    }
}
