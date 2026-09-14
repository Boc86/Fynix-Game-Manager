use crate::models::game::Game;
use crate::core::plugin::StorePlugin;
use async_trait::async_trait;
use std::path::{Path, PathBuf};

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
///     "LastPlayed"    "1700000000"
/// }
/// ```
pub fn parse_appmanifest(content: &str, steamapps_dir: &Path) -> anyhow::Result<Game> {
    let app_id = extract_acf_value(content, "appid")
        .ok_or_else(|| anyhow::anyhow!("Could not parse appid from appmanifest"))?;
    let name = extract_acf_value(content, "name")
        .unwrap_or_else(|| "Unknown".to_string());

    let mut game = Game::new(&format!("steam:{}", app_id), &name, "Valve");

    // installdir (optional)
    if let Some(installdir) = extract_acf_value(content, "installdir") {
        let install_path = steamapps_dir.join("common").join(&installdir);
        game = game.with_install_path(install_path.to_string_lossy().to_string());
    }

    // LastPlayed (optional — 0 means never played)
    if let Some(last_played_str) = extract_acf_value(content, "LastPlayed") {
        if let Ok(ts) = last_played_str.parse::<u64>() {
            if ts > 0 {
                game = game.with_last_played(ts);
            }
        }
    }

    Ok(game)
}

/// Extract a key-value pair from ACF/VDF content.
/// Matches: "key"    "value"
fn extract_acf_value(content: &str, key: &str) -> Option<String> {
    let re = regex::Regex::new(&format!(r#""{}"\s+"([^"]+)""#, key)).unwrap();
    re.captures(content).and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
}

/// Plugin for detecting games installed via Steam.
///
/// Reads `appmanifest_*.acf` files from `steamapps/` directories.
/// Auto-discovers Steam path and all library folders across all HDDs.
pub struct SteamPlugin {
    steam_path: PathBuf,
}

impl SteamPlugin {
    pub fn new(steam_path: PathBuf) -> Self {
        Self { steam_path }
    }

    /// Auto-detect the Steam installation path from common locations.
    pub fn detect_steam_path() -> Option<PathBuf> {
        let common_paths = [
            "/home/boc/.local/share/Steam",
            "/home/boc/.steam/steam",
            "/home/boc/.steam/debian",
            "/var/lib/flatpak/exports/share/Steam",
            "/usr/share/steam",
        ];

        for path in &common_paths {
            let p = PathBuf::from(path);
            if p.exists() && p.join("steamapps").exists() {
                return Some(p);
            }
        }

        // Check environment variable
        if let Some(steam_home) = std::env::var_os("STEAM_HOME") {
            let p = PathBuf::from(steam_home);
            if p.join("steamapps").exists() {
                return Some(p);
            }
        }

        // Check for steam binary and use --dir flag
        if let Ok(output) = std::process::Command::new("steam")
            .arg("--dir")
            .output()
        {
            let path = String::from_utf8_lossy(&output.stdout);
            let path = path.trim();
            if !path.is_empty() {
                let p = PathBuf::from(path);
                if p.join("steamapps").exists() {
                    return Some(p);
                }
            }
        }

        None
    }

    fn steamapps_dir(&self) -> PathBuf {
        self.steam_path.join("steamapps")
    }

    /// Discover all library folders by parsing `libraryfolders.vdf`.
    /// Checks both `steamapps/libraryfolders.vdf` and `config/libraryfolders.vdf`.
    fn library_folders(&self) -> Vec<PathBuf> {
        let mut folders = vec![self.steamapps_dir()];

        // Try multiple locations for libraryfolders.vdf
        let vdf_paths = [
            self.steam_path.join("steamapps/libraryfolders.vdf"),
            self.steam_path.join("config/libraryfolders.vdf"),
        ];

        for vdf_path in &vdf_paths {
            if let Ok(content) = std::fs::read_to_string(vdf_path) {
                if let Some(paths) = extract_vdf_paths(&content) {
                    for path_str in paths {
                        let path = PathBuf::from(&path_str).join("steamapps");
                        if path.exists() && !folders.contains(&path) {
                            folders.push(path);
                        }
                    }
                }
            }
        }

        folders
    }

    /// Find the executable for a Steam game
    fn find_executable(&self, app_id: &str) -> Option<String> {
        let manifest = self.steamapps_dir().join(format!("appmanifest_{}.acf", app_id));
        if let Ok(content) = std::fs::read_to_string(&manifest) {
            if let Some(exec) = extract_acf_value(&content, "LaunchExePath") {
                return Some(exec);
            }
        }
        // Also check in library folders
        for folder in self.library_folders() {
            let manifest = folder.join(format!("appmanifest_{}.acf", app_id));
            if let Ok(content) = std::fs::read_to_string(&manifest) {
                if let Some(exec) = extract_acf_value(&content, "LaunchExePath") {
                    return Some(exec);
                }
            }
        }
        None
    }
}

/// Extract all "path" values from a Steam VDF file.
/// VDF format: "path"    "/some/path"
fn extract_vdf_paths(content: &str) -> Option<Vec<String>> {
    let re = regex::Regex::new(r#""path"\s+"([^"]+)""#).unwrap();
    let paths: Vec<String> = re
        .captures_iter(content)
        .filter_map(|c| c.get(1).map(|m| m.as_str().to_string()))
        .collect();
    if paths.is_empty() { None } else { Some(paths) }
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

        for folder in self.library_folders() {
            let manifest = folder.join(format!("appmanifest_{}.acf", app_id));
            if manifest.exists() {
                let content = std::fs::read_to_string(&manifest)?;
                let mut game = parse_appmanifest(&content, &folder)?;
                if let Some(exec) = self.find_executable(app_id) {
                    game = game.with_executable(exec);
                }
                return Ok(Some(game));
            }
        }
        Ok(None)
    }

    async fn launch_game(&self, game_id: &str) -> anyhow::Result<()> {
        let app_id = game_id.strip_prefix("steam:").ok_or_else(|| anyhow::anyhow!("invalid steam id"))?;
        // Use Steam URL protocol to launch — works regardless of install location
        let steam_exe = self.steam_path.join("steam");
        if steam_exe.exists() {
            std::process::Command::new(&steam_exe)
                .arg(format!("steam://run/{}", app_id))
                .spawn()?;
            Ok(())
        } else {
            if let Ok(_) = std::process::Command::new("xdg-open")
                .arg(format!("steam://run/{}", app_id))
                .spawn()
            {
                Ok(())
            } else {
                Err(anyhow::anyhow!("Steam executable not found at {:?}", steam_exe))
            }
        }
    }

    fn is_authenticated(&self) -> bool {
        self.steam_path.exists()
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

    #[test]
    fn test_extract_vdf_paths() {
        let vdf = r#""libraryfolders"
{
    "0"
    {
        "path" "/home/boc/.local/share/Steam"
        "apps"
        {
            "730" "12345"
        }
    }
    "1"
    {
        "path" "/mnt/Games1/SteamLibrary"
    }
}"#;
        let paths = extract_vdf_paths(vdf).unwrap();
        assert_eq!(paths.len(), 2);
        assert_eq!(paths[0], "/home/boc/.local/share/Steam");
        assert_eq!(paths[1], "/mnt/Games1/SteamLibrary");
    }

    #[test]
    fn test_steam_plugin_detect_steam_path() {
        if let Some(path) = SteamPlugin::detect_steam_path() {
            assert!(path.join("steamapps").exists());
        }
    }
}