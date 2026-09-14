// TypeScript interfaces matching the Rust Game struct
export interface Game {
  id: string;          // "steam:730", "heroic:abc-123", "lutris:SuperTux"
  name: string;
  publisher: string;
  install_path?: string;
  cover_url?: string;
  last_played?: number;
  playtime_hours?: number;
  executable?: string;
  launch_args?: string;
  store_id: string;    // "steam", "heroic", "lutris"
}
