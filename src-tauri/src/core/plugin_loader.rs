use libloading::{Library, Symbol};
use crate::core::plugin::StorePlugin;
use crate::plugins::steam::SteamPlugin;
use crate::plugins::heroic::HeroicPlugin;
use crate::plugins::lutris::LutrisPlugin;
use std::path::PathBuf;

/// Plugin loader — discovers and loads both built-in and external plugins.
/// 
/// External plugins are Rust shared libraries (.so) that export a `plugin_create`
/// symbol returning a boxed `StorePlugin`.
pub struct PluginLoader {
    plugin_dir: PathBuf,
}

impl PluginLoader {
    pub fn new(plugin_dir: PathBuf) -> Self {
        Self { plugin_dir }
    }

    /// Get all built-in plugins (Steam, Heroic, Lutris).
    /// These are always available without external dependencies.
    pub fn builtin_plugins(&self) -> Vec<Box<dyn StorePlugin>> {
        // Auto-detect Steam path
        let steam_path = std::env::var("STEAM_PATH")
            .map(PathBuf::from)
            .ok()
            .or_else(|| SteamPlugin::detect_steam_path())
            .unwrap_or_else(|| PathBuf::from("/home/boc/.local/share/Steam"));

        vec![
            Box::new(SteamPlugin::new(steam_path)),
            Box::new(HeroicPlugin::new()),
            Box::new(LutrisPlugin::new()),
        ]
    }

    /// Load external plugins from the plugins/ directory.
    /// Each .so file must export `extern "C" fn plugin_create() -> *mut dyn StorePlugin`.
    pub fn load_external_plugins(&self) -> anyhow::Result<Vec<Box<dyn StorePlugin>>> {
        let mut plugins: Vec<Box<dyn StorePlugin>> = Vec::new();
        
        if !self.plugin_dir.exists() {
            return Ok(plugins);
        }

        for entry in std::fs::read_dir(&self.plugin_dir)? {
            let entry = entry?;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            
            if name.ends_with(".so") {
                let lib_path = entry.path();
                match unsafe { Library::new(&lib_path) } {
                    Ok(lib) => {
                        let symbol_result: Result<Symbol<unsafe extern "C" fn() -> *mut dyn StorePlugin>, _> =
                            unsafe { lib.get(b"plugin_create") };
                        
                        match symbol_result {
                            Ok(plugin_create) => {
                                let plugin_ptr = unsafe { plugin_create() };
                                if !plugin_ptr.is_null() {
                                    plugins.push(unsafe { Box::from_raw(plugin_ptr) });
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to get plugin_create symbol from {}: {}", name, e);
                            }
                        }
                        // Keep library loaded for the lifetime of the plugin
                        std::mem::forget(lib);
                    }
                    Err(e) => {
                        eprintln!("Failed to load plugin {}: {}", name, e);
                    }
                }
            }
        }
        
        Ok(plugins)
    }

    /// Load all plugins (builtin + external) and register them with the library.
    pub async fn load_all(&self, library: &crate::core::library::GameLibrary) -> anyhow::Result<()> {
        // Register built-in plugins
        for plugin in self.builtin_plugins() {
            library.register_plugin(plugin).await;
        }

        // Register external plugins
        match self.load_external_plugins() {
            Ok(external) => {
                for plugin in external {
                    library.register_plugin(plugin).await;
                }
            }
            Err(e) => {
                eprintln!("Warning: failed to load external plugins: {}", e);
            }
        }

        Ok(())
    }
}
