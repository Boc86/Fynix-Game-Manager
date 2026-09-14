use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

/// Builder for constructing UMU (umu-launcher) commands to launch
/// Windows games through Steam's Proton compatibility layer on Linux.
/// 
/// UMU requires these environment variables:
/// - `STEAM_COMPAT_CLIENT_VERSION`: Steam client version to report to Proton
/// - `STEAM_COMPAT_DATA_PATH`: Path to the proton prefix (compatdata/APPID)
/// - `UMU_ID`: App ID string (used for logging and prefix management)
/// - `PROTONPATH`: Path to the Proton installation directory
/// - `GAME_EXECUTABLE`: The Windows executable to launch
pub struct UmuCommandBuilder {
    pub steam_path: PathBuf,
    pub env_vars: HashMap<String, String>,
}

impl UmuCommandBuilder {
    pub fn new(steam_path: impl Into<PathBuf>) -> Self {
        Self {
            steam_path: steam_path.into(),
            env_vars: HashMap::new(),
        }
    }

    /// Set the Proton installation to use (e.g., "Proton 8.0", "GE-Proton9-26")
    pub fn with_proton(mut self, proton_name: &str) -> Self {
        let proton_path = self.steam_path
            .join("steamapps/common")
            .join(proton_name);
        self.env_vars.insert(
            "PROTONPATH".to_string(),
            proton_path.to_string_lossy().to_string(),
        );
        self
    }

    /// Set the Steam compatdata prefix path (e.g., steamapps/compatdata/730)
    pub fn with_prefix(mut self, prefix: impl Into<PathBuf>) -> Self {
        let prefix = prefix.into();
        self.env_vars.insert(
            "STEAM_COMPAT_DATA_PATH".to_string(),
            prefix.to_string_lossy().to_string(),
        );
        self.env_vars.insert(
            "STEAM_COMPAT_CLIENT_VERSION".to_string(),
            "public-regular".to_string(),
        );
        self
    }

    /// Set the Steam App ID (used for UMU_ID and logging)
    pub fn with_app_id(mut self, app_id: &str) -> Self {
        self.env_vars.insert("UMU_ID".to_string(), app_id.to_string());
        self
    }

    /// Add an arbitrary environment variable
    pub fn with_env(mut self, key: &str, val: &str) -> Self {
        self.env_vars.insert(key.to_string(), val.to_string());
        self
    }

    /// Set Wine prefix override
    pub fn with_wine_prefix(mut self, path: impl Into<PathBuf>) -> Self {
        self.env_vars.insert(
            "WINEPREFIX".to_string(),
            path.into().to_string_lossy().to_string(),
        );
        self
    }

    /// Build the final command that will be executed
    pub fn build(&self, executable: &str) -> BuiltUmuCommand {
        BuiltUmuCommand {
            program: executable.to_string(),
            env_vars: self.env_vars.clone(),
            steam_path: self.steam_path.clone(),
        }
    }
}

/// A built UMU command ready for execution.
pub struct BuiltUmuCommand {
    pub program: String,
    pub env_vars: HashMap<String, String>,
    pub steam_path: PathBuf,
}

impl BuiltUmuCommand {
    /// Resolve the path to the Wine binary within the Proton installation
    pub fn wine_path(&self) -> Option<PathBuf> {
        let proton_path = self.env_vars.get("PROTONPATH")?;
        let wine = PathBuf::from(proton_path)
            .join("dist")
            .join("bin")
            .join("wine");
        if wine.exists() { Some(wine) } else { None }
    }

    /// Convert this into a std::process::Command ready for .spawn()
    pub fn to_command(&self) -> Command {
        let umu_runner = std::env::var("UMU_LAUNCHER")
            .ok()
            .unwrap_or_else(|| "/usr/bin/umu-run".to_string());
        
        let mut cmd = Command::new(&umu_runner);
        for (key, val) in &self.env_vars {
            cmd.env(key, val);
        }
        cmd.arg(&self.program);
        cmd
    }

    /// Execute the command (blocking)
    pub fn execute(&self) -> anyhow::Result<()> {
        let mut cmd = self.to_command();
        let _status = cmd.status()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_umu_command_builder_sets_env() {
        let builder = UmuCommandBuilder::new("/home/boc/.steam/steam")
            .with_proton("Proton 8.0")
            .with_prefix("/home/boc/.steam/steam/steamapps/compatdata/730")
            .with_app_id("730");
        let cmd = builder.build("/path/to/game.exe");
        assert!(cmd.env_vars.contains_key("STEAM_COMPAT_CLIENT_VERSION"));
        assert!(cmd.env_vars.contains_key("STEAM_COMPAT_DATA_PATH"));
        assert!(cmd.env_vars.contains_key("UMU_ID"));
        assert!(cmd.env_vars.contains_key("PROTONPATH"));
        assert_eq!(cmd.program, "/path/to/game.exe");
    }

    #[test]
    fn test_umu_command_builder_program() {
        let builder = UmuCommandBuilder::new("/home/boc/.steam/steam");
        let cmd = builder.build("/games/app/Game.exe");
        assert_eq!(cmd.program, "/games/app/Game.exe");
    }
}
