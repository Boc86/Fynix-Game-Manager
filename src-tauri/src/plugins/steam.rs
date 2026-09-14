use crate::models::game::Game;
use crate::core::plugin::StorePlugin;
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use once_cell::sync::Lazy;

/// Regex patterns for parsing Steam appmanifest .acf files.
/// Each pattern captures the value for a specific ACF key.
static ACF_PATTERNS: Lazy<Vec<regex::Regex>> = Lazy::new(|| {
    vec![
        regex::Regex::new(r#""appid"\s+"([^"]+)""#).unwrap(),
        regex::Regex::new(r#""name"\s+"([^"]+)""#).unwrap(),
        regex::Regex::new(r#""installdir"\s+"([^"]+)""#).unwrap(),
        regex::Regex::new(r#""LastPlayed"\s+"([^"]+)""#).unwrap(),
        regex::Regex::new(r#""SizeOnDisk"\s+"([^"]+)""#).unwrap(),
    ]
});

/// Parse a single Steam appmanifest .acf file into a Game.
///
/// ACF files are Valve's key-value format used for Steam game manifests.
/// Example:
/// ```text
/// "appmanifest_730.acf"
/// {
///     "appid"        "730"
///     "name"        "Counter-Strike 2"
///     "installdir"    "Counter-Strike Global Offensive"
///     "LastUpdated"    "1700000000"
///     "LastPlayed"    "1700000000"
/// }
/// ```
pub fn parse_appmanifest(content: &str, steamapps_dir: &Path) -> anyhow::Result<Game> {
    // ACF_PATTERNS captures are in order: appid, name, installdir, LastPlayed, SizeOnDisk
    let app_id = ACF_PATTERNS[0]
        .captures(content)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str())
        .ok_or_else(|| anyhow::anyhow!("Could not parse appid from appmanifest"))?;

    let name = ACF_PATTERNS[1]
        .captures(content)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str())
        .unwrap_or("Unknown");

    let mut game = Game::new(&format!("steam:{}", app_id), name, "Valve");

    // installdir (optional, present in some manifests)
    if let Some(m) = ACF_PATTERNS[2].captures(content).and_then(|c| c.get(1)) {
        let install_path = steamapps_dir.join("common").join(m.as_str());
        game = game.with_install_path(install_path.to_string_lossy().to_string());
    }

    // LastPlayed (optional — 0 means never played)
    if let Some(m) = ACF_PATTERNS[3].captures(content).and_then(|c| c.get(1)) {
        if let Ok(ts) = m.as_str().parse::<u64>() {
            if ts > 0 {
                game = game.with_last_played(ts);
            }
        }
    }

    Ok(game)
}

/// Plugin for detecting games installed via Steam.
///
/// Reads `appmanifest_*.acf` files from `steamapps/` directories.
/// Steam does not require authentication — all installed games are local.
pub struct SteamPlugin {
    steam_path: PathBuf,
}

impl SteamPlugin {
    pub fn new(steam_path: PathBuf) -> Self {
        Self { steam_path }
    }

    fn steamapps_dir(&self) -> PathBuf {
        self.steam_path.join("steamapps")
    }

    /// Discover all library folders by parsing `libraryfolders.vdf`
    fn library_folders(&self) -> Vec<PathBuf> {
        let mut folders = vec![self.steamapps_dir()];

        let vdf_path = self.steam_path.join("steamapps/libraryfolders.vdf");
        if let Ok(content) = std::fs::read_to_string(&vdf_path) {
            // Look for "path" entries in the VDF
            for line in content.lines() {
                if let Some(path_str) = extract_vdf_path(line) {
                    let path = PathBuf::from(path_str).join("steamapps");
                    if path.exists() {
                        folders.push(path);
                    }
                }
            }
        }
        folders
    }

    /// Find the executable for a Steam game
    fn find_executable(&self, app_id: &str) -> Option<String> {
        // Check appmanifest for the launch executable
        let manifest = self.steamapps_dir().join(format!("appmanifest_{}.acf", app_id));
        if let Ok(content) = std::fs::read_to_string(&manifest) {
            if let Some(exec) = extract_acf_value(&content, "LaunchExePath") {
                return Some(exec);
            }
        }
        None
    }
}

fn extract_vdf_path(line: &str) -> Option<String> {
    if line.contains("\"path\"") {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 2 {
            let path = parts[1].trim().trim_matches('"');
            return Some(path.to_string());
        }
    }
    None
}

fn extract_acf_value(content: &str, key: &str) -> Option<String> {
    let re = regex::Regex::new(&format!(r#""{}"\s+"([^"]+)""#, key)).unwrap();
    re.captures(content).and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
}

#[async_trait]
impl StorePlugin for SteamPlugin {
    fn name(&self) -> &str {
        "Steam"
    }

    fn store_id(&self) -> &str {
        "steam"
    }

    async fn detect_installed_games(&self) -> anyhow::Result<Vec<Game>> {
        let mut games = Vec::new();
        let folders = self.library_folders();

        for folder in folders {
            if let Ok(entries) = std::fs::read_dir(&folder) {
                for entry in entries.flatten() {
                    let name = entry.file_name();
                    let name = name.to_string_lossy();
                    if name.starts_with("appmanifest_") && name.ends_with(".acf") {
                        if let Ok(content) = std::fs::read_to_string(entry.path()) {
                            if let Ok(mut game) = parse_appmanifest(&content, &folder) {
                                // Try to find executable
                                let app_id = game.id.strip_prefix("steam:").unwrap_or("");
                                if let Some(exec) = self.find_executable(app_id) {
                                    game = game.with_executable(exec);
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
        let app_id = game_id.strip_prefix("steam:").ok_or_else(|| anyhow::anyhow!("invalid steam id"))?;
        let manifest = self.steamapps_dir().join(format!("appmanifest_{}.acf", app_id));
        if manifest.exists() {
            let content = std::fs::read_to_string(&manifest)?;
            let mut game = parse_appmanifest(&content, &self.steamapps_dir())?;
            if let Some(exec) = self.find_executable(app_id) {
                game = game.with_executable(exec);
            }
            Ok(Some(game))
        } else {
            Ok(None)
        }
    }

    async fn launch_game(&self, game_id: &str) -> anyhow::Result<()> {
        let app_id = game_id.strip_prefix("steam:").ok_or_else(|| anyhow::anyhow!("invalid steam id"))?;
        // Use Steam URL protocol to launch
        let steam_exe = self.steam_path.join("steam");
        if steam_exe.exists() {
            std::process::Command::new(&steam_exe)
                .arg(format!("steam://run/{}", app_id))
                .spawn()?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Steam executable not found at {:?}", steam_exe))
        }
    }

    fn is_authenticated(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_appmanifest_basic() {
        let acf = r#"{
            "appid" "730"
            "name" "Counter-Strike 2"
            "installdir" "Counter-Strike Global Offensive"
            "LastUpdated" "1700000000"
            "LastPlayed" "1700000000"
        }"#;
        let steamapps = std::path::Path::new("/home/user/.steam/steam/steamapps");
        let game = parse_appmanifest(acf, steamapps).unwrap();
        assert_eq!(game.id, "steam:730");
        assert_eq!(game.name, "Counter-Strike 2");
        assert_eq!(game.publisher, "Valve");
        assert_eq!(game.store_id, "steam");
    }

    #[test]
    fn test_parse_appmanifest_with_played_time() {
        let acf = r#"{
            "appid" "730"
            "name" "Counter-Strike 2"
            "LastPlayed" "1700000000"
        }"#;
        let steamapps = std::path::Path::new("/tmp");
        let game = parse_appmanifest(acf, steamapps).unwrap();
        assert_eq!(game.last_played, Some(1700000000));
    }

    #[test]
    fn test_parse_appmanifest_no_played_time() {
        let acf = r#"{
            "appid" "123"
            "name" "Some Game"
        }"#;
        let steamapps = std::path::Path::new("/tmp");
        let game = parse_appmanifest(acf, steamapps).unwrap();
        assert_eq!(game.id, "steam:123");
        assert_eq!(game.name, "Some Game");
        assert!(game.last_played.is_none());
    }
}
