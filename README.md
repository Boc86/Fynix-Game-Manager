# Fynix GM

A lightweight Tauri-based game launcher for Linux. Part of the Fynix suite — same dark aesthetic as SteamFlix and Fynix Hub.

## Architecture

- **Frontend**: React + TypeScript + Vite, styled with Netflix-inspired dark theme (charcoal background, red accents)
- **Backend**: Rust + Tauri 2, using a plugin-based architecture for multi-store game detection
- **Bundle targets**: deb, rpm, AppImage

## Store Plugins

Each store implements the `StorePlugin` trait from `src-tauri/src/core/plugin.rs`:

| Plugin | Store | Source |
|--------|-------|--------|
| SteamPlugin | Steam | Parses `appmanifest_*.acf` files |
| HeroicPlugin | Epic/Heroic | Reads `games.txt` / individual JSON files |
| LutrisPlugin | Lutris | Reads YAML configs from `~/.config/lutris/games/` |

External plugins can be dynamically loaded from the app data `plugins/` directory.

## Development

```bash
# Install Rust (if not already)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install npm dependencies
npm install

# Run in development mode
npm run tauri:dev

# Run tests
cd src-tauri && cargo test

# Build for production
npm run tauri:build
```

## TDD

- **Rust unit tests**: inline `#[cfg(test)]` modules in each source file
- **Rust integration tests**: `src-tauri/tests/integration_tests.rs`
- Run with: `cargo test` from `src-tauri/`

## Fynix Branding

Follows the same charcoal-black + Netflix red palette as other Fynix projects.
See `src/App.css` for CSS color variables.
