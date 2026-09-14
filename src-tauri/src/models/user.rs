use serde::{Deserialize, Serialize};

/// User configuration for the game manager.
/// Stored in ~/.config/fynix-gm/config.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    pub theme: String,
    pub steam_path: Option<String>,
    pub heroic_path: Option<String>,
    pub lutris_path: Option<String>,
    pub umu_path: Option<String>,
    pub show_uninstalled: bool,
    pub group_by_store: bool,
    pub last_scan: Option<u64>,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            theme: "netflix".to_string(),
            steam_path: None,
            heroic_path: None,
            lutris_path: None,
            umu_path: Some("/usr/bin/umu-launcher".to_string()),
            show_uninstalled: false,
            group_by_store: true,
            last_scan: None,
        }
    }
}

impl UserSettings {
    pub fn load() -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("~/.config"))
            .join("fynix-gm");
        let config_file = config_dir.join("config.toml");
        
        if config_file.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_file) {
                if let Ok(settings) = toml::from_str::<UserSettings>(&content) {
                    return settings;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("~/.config"))
            .join("fynix-gm");
        std::fs::create_dir_all(&config_dir)?;
        let content = toml::to_string_pretty(self)?;
        std::fs::write(config_dir.join("config.toml"), content)?;
        Ok(())
    }
}
