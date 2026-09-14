use serde::{Deserialize, Serialize};

/// A game discovered from any store.
/// 
/// `id` is a composite ID: `store:appid` (e.g., "steam:730", "heroic:abc-123")
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub id: String,
    pub name: String,
    pub publisher: String,
    pub install_path: Option<String>,
    pub cover_url: Option<String>,
    pub last_played: Option<u64>,
    pub playtime_hours: Option<f64>,
    pub executable: Option<String>,
    pub launch_args: Option<String>,
    pub store_id: String,
}

impl Game {
    pub fn new(id: &str, name: &str, publisher: &str) -> Self {
        let store_id = id.split(':').next().unwrap_or("unknown").to_string();
        Self {
            id: id.to_string(),
            name: name.to_string(),
            publisher: publisher.to_string(),
            install_path: None,
            cover_url: None,
            last_played: None,
            playtime_hours: None,
            executable: None,
            launch_args: None,
            store_id,
        }
    }

    pub fn with_install_path(mut self, path: String) -> Self {
        self.install_path = Some(path);
        self
    }

    pub fn with_last_played(mut self, ts: u64) -> Self {
        self.last_played = Some(ts);
        self
    }

    pub fn with_cover(mut self, url: String) -> Self {
        self.cover_url = Some(url);
        self
    }

    pub fn with_playtime(mut self, hours: f64) -> Self {
        self.playtime_hours = Some(hours);
        self
    }

    pub fn with_executable(mut self, path: String) -> Self {
        self.executable = Some(path);
        self
    }

    pub fn with_launch_args(mut self, args: String) -> Self {
        self.launch_args = Some(args);
        self
    }
}
