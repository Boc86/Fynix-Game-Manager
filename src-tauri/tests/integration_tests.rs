use fynix_gm::models::game::Game;
use fynix_gm::core::plugin::StorePlugin;
use fynix_gm::plugins::steam::SteamPlugin;
use fynix_gm::plugins::heroic::HeroicPlugin;
use std::path::PathBuf;

#[test]
fn test_game_creation() {
    let g = Game::new("steam:730", "Counter-Strike 2", "Valve");
    assert_eq!(g.id, "steam:730");
    assert_eq!(g.name, "Counter-Strike 2");
    assert_eq!(g.publisher, "Valve");
    assert_eq!(g.store_id, "steam");
}

#[test]
fn test_game_with_install_path() {
    let g = Game::new("steam:730", "CS2", "Valve")
        .with_install_path("/home/boc/.steam/steam/steamapps/common/Counter-Strike Global Offensive".to_string())
        .with_last_played(1700000000)
        .with_cover("https://example.com/cover.jpg".to_string())
        .with_playtime(42.5);
    assert_eq!(g.install_path, Some("/home/boc/.steam/steam/steamapps/common/Counter-Strike Global Offensive".to_string()));
    assert_eq!(g.last_played, Some(1700000000));
    assert_eq!(g.cover_url, Some("https://example.com/cover.jpg".to_string()));
    assert_eq!(g.playtime_hours, Some(42.5));
}

#[test]
fn test_parse_appmanifest() {
    let acf = r#""appmanifest_730.acf"
    {
        "appid"        "730"
        "name"        "Counter-Strike 2"
        "installdir"    "Counter-Strike Global Offensive"
        "LastUpdated"    "1700000000"
        "LastPlayed"    "1700000000"
        "SizeOnDisk"    "12345678"
    }"#;
    let steamapps = PathBuf::from("/home/boc/.steam/steam/steamapps");
    let game = fynix_gm::plugins::steam::parse_appmanifest(acf, &steamapps).unwrap();
    assert_eq!(game.id, "steam:730");
    assert_eq!(game.name, "Counter-Strike 2");
    assert!(game.install_path.is_some());
}

#[test]
fn test_parse_appmanifest_no_lastplayed() {
    let acf = r#""appmanifest_123.acf"
    {
        "appid"        "123"
        "name"        "Test Game"
        "installdir"    "TestGame"
    }"#;
    let steamapps = PathBuf::from("/home/boc/.steam/steam/steamapps");
    let game = fynix_gm::plugins::steam::parse_appmanifest(acf, &steamapps).unwrap();
    assert_eq!(game.id, "steam:123");
    assert_eq!(game.last_played, None);
}

#[test]
fn test_parse_heroic_games() {
    let json = r#"[
        {"id": "abc-123", "name": "Hades", "executable": "/home/boc/Games/Heroic/Hades/Hades.exe"},
        {"id": "def-456", "name": "Hollow Knight", "executable": "/home/boc/Games/Heroic/HollowKnight/hollow_knight.exe"}
    ]"#;
    let games = fynix_gm::plugins::heroic::parse_heroic_games(json).unwrap();
    assert_eq!(games.len(), 2);
    assert_eq!(games[0].id, "heroic:abc-123");
    assert_eq!(games[0].name, "Hades");
    assert_eq!(games[1].id, "heroic:def-456");
    assert_eq!(games[1].name, "Hollow Knight");
}

#[test]
fn test_parse_heroic_games_empty() {
    let games = fynix_gm::plugins::heroic::parse_heroic_games("[]").unwrap();
    assert_eq!(games.len(), 0);
}

#[test]
fn test_steam_plugin_trait() {
    let plugin = SteamPlugin::new(PathBuf::from("/home/boc/.steam/steam"));
    assert_eq!(plugin.name(), "Steam");
    assert_eq!(plugin.store_id(), "steam");
    assert!(plugin.is_authenticated());
}

#[test]
fn test_heroic_plugin_trait() {
    let plugin = HeroicPlugin::new();
    assert_eq!(plugin.name(), "Heroic Games");
    assert_eq!(plugin.store_id(), "heroic");
}

#[test]
fn test_umu_command_builder() {
    let builder = fynix_gm::core::umu::UmuCommandBuilder::new("/home/boc/.steam/steam")
        .with_proton("Proton 8.0")
        .with_prefix("/home/boc/.steam/steam/steamapps/compatdata/730")
        .with_app_id("730");
    let cmd = builder.build("/path/to/game.exe");
    assert!(cmd.env_vars.contains_key("STEAM_COMPAT_CLIENT_VERSION"));
    assert!(cmd.env_vars.contains_key("STEAM_COMPAT_DATA_PATH"));
    assert!(cmd.env_vars.contains_key("UMU_ID"));
    assert_eq!(cmd.program, "/path/to/game.exe");
}

#[tokio::test]
async fn test_game_library_aggregation() {
    use fynix_gm::core::library::GameLibrary;

    let lib = GameLibrary::new();
    lib.register_plugin(Box::new(SteamPlugin::new(PathBuf::from("/nonexistent")))).await;
    lib.register_plugin(Box::new(HeroicPlugin::new())).await;

    // No games installed in test paths — should not crash
    let count = lib.refresh().await.unwrap();
    assert_eq!(count, 0);

    let games = lib.all_games().await;
    assert!(games.is_empty());
}
