use crate::models::game::Game;
use crate::core::plugin::StorePlugin;
use async_trait::async_trait;
use std::path::PathBuf;

/// Parse a Lutris game YAML config file.
/// Lutris stores game configs in ~/.config/lutris/games/*.yml
pub fn parse_lutris_yaml(content: &str) -> anyhow::Result<Game> {
    let name = extract_yaml_value(content, "name").unwrap_or_else(|| "Unknown Game".to_string());
    let executable = extract_yaml_value(content, "exe");

    let mut game = Game::new(&format!("lutris:{}", sanitize_name(&name)), &name, "Lutris");
    if let Some(exec) = executable {
        game = game.with_executable(exec);
    }
    Ok(game)
}

fn extract_yaml_value(content: &str, key: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with(&format!("{}:", key)) || line.starts_with(&format!("{} :", key)) {
            let value = line.split(':').nth(1)?.trim().to_string();
            return Some(value.trim_matches('"').to_string());
        }
    }
    None
}

fn sanitize_name(name: &str) -> String {
    name.chars().map(|c| if c.is_alphanumeric() { c } else { '_' }).collect()
}

/// Plugin for detecting games installed via Lutris.
///
/// Reads game YAML config files from ~/.config/lutris/games/
pub struct LutrisPlugin {
    config_path: PathBuf,
}

impl LutrisPlugin {
    pub fn new() -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("lutris");
        Self { config_path: config_dir }
    }

    fn games_dir(&self) -> PathBuf {
        self.config_path.join("games")
    }
}

impl Default for LutrisPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StorePlugin for LutrisPlugin {
    fn name(&self) -> &str {
        "Lutris"
    }

    fn store_id(&self) -> &str {
        "lutris"
    }

    async fn detect_installed_games(&self) -> anyhow::Result<Vec<Game>> {
        let mut games = Vec::new();
        let games_dir = self.games_dir();

        if games_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&games_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let ext = path.extension().and_then(|e| e.to_str());
                    if ext == Some("yml") || ext == Some("yaml") {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if let Ok(game) = parse_lutris_yaml(&content) {
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
        let _id = game_id.strip_prefix("lutris:").ok_or_else(|| anyhow::anyhow!("invalid lutris id"))?;
        std::process::Command::new("lutris")
            .arg("-l")
            .arg(game_id)
            .spawn()?;
        Ok(())
    }

    fn is_authenticated(&self) -> bool {
        self.games_dir().exists()
    }
}
