import { useState, useEffect, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Game } from './types/Game';
import { GameCard } from './components/GameCard';
import './App.css';

function App() {
  const [games, setGames] = useState<Game[]>([]);
  const [search, setSearch] = useState('');
  const [loading, setLoading] = useState(true);
  const [activeTab, setActiveTab] = useState('all');

  useEffect(() => {
    const loadGames = async () => {
      try {
        // Fetch games first
        const result = await invoke<Game[]>('get_all_games');
        setGames(result || []);

        // Fetch missing artwork for games without covers
        await invoke('fetch_missing_artwork');
        // Reload to get updated games with covers
        const refreshed = await invoke<Game[]>('get_all_games');
        setGames(refreshed || []);
      } catch (e) {
        console.error('Failed to load games:', e);
      } finally {
        setLoading(false);
      }
    };
    loadGames();
  }, []);

  const handleSearch = (e: React.ChangeEvent<HTMLInputElement>) => {
    setSearch(e.target.value);
  };

  const handleLaunch = (gameId: string) => {
    invoke('launch_game', { game_id: gameId })
      .catch(err => console.error('Launch failed:', err));
  };

  const handleRefresh = async () => {
    setLoading(true);
    try {
      await invoke('refresh_library');
      const result = await invoke<Game[]>('get_all_games');
      setGames(result || []);
    } catch (e) {
      console.error('Refresh failed:', e);
    } finally {
      setLoading(false);
    }
  };

  // Group games by store for carousel rows
  const storeGroups = useMemo(() => {
    const groups: Record<string, Game[]> = {};
    games.forEach(g => {
      const store = g.store_id || 'unknown';
      if (!groups[store]) groups[store] = [];
      groups[store].push(g);
    });
    return groups;
  }, [games]);

  // Filter games by search query
  const filtered = useMemo(() => {
    if (!search) return games;
    const q = search.toLowerCase();
    return games.filter(g =>
      g.name.toLowerCase().includes(q) ||
      (g.publisher || '').toLowerCase().includes(q)
    );
  }, [games, search]);

  // Games for the "Recently Played" tab
  const recentGames = useMemo(() => {
    return [...filtered]
      .filter(g => g.last_played && g.last_played > 0)
      .sort((a, b) => (b.last_played || 0) - (a.last_played || 0))
      .slice(0, 12);
  }, [filtered]);

  // Games for the "All Games" tab (split into rows of 12 for carousels)
  const allGamesRows = useMemo(() => {
    const rows: Game[][] = [];
    for (let i = 0; i < filtered.length; i += 12) {
      rows.push(filtered.slice(i, i + 12));
    }
    return rows;
  }, [filtered]);

  const displayedGames = activeTab === 'recent' ? recentGames : filtered;
  const displayedRows = activeTab === 'recent'
    ? displayedGames.length > 0 ? [displayedGames] : []
    : allGamesRows;

  // Hero game (first game for the hero banner)
  const heroGame = games.find(g => g.cover_url);

  return (
    <div className="flix-app">
      {/* Netflix-style fixed header */}
      <header className="flix-header">
        <div style={{ display: 'flex', alignItems: 'center', gap: '28px' }}>
          <a href="#" className="flix-logo">Fynix<span className="flix-logo-red">GM</span></a>
          <nav className="flix-header-nav">
            <a href="#" className={activeTab === 'all' ? '' : ''} onClick={() => setActiveTab('all')}>Games</a>
            <a href="#" className={activeTab === 'recent' ? '' : ''} onClick={() => setActiveTab('recent')}>Recently Played</a>
            <a href="#">Stores</a>
          </nav>
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '16px' }}>
          <div className="flix-search">
            <span className="flix-search-icon">🔍</span>
            <input
              type="text"
              placeholder="Search games..."
              value={search}
              onChange={handleSearch}
            />
          </div>
          <button onClick={handleRefresh} className="flix-btn flix-btn-secondary">
            ↻ Refresh
          </button>
        </div>
      </header>

      {/* Hero banner */}
      {heroGame && (
        <section className="flix-hero">
          <img
            src={heroGame.cover_url}
            alt={heroGame.name}
            className="flix-hero-image"
          />
          <div className="flix-hero-overlay">
            <h1 className="flix-hero-title">{heroGame.name}</h1>
            <div className="flix-hero-meta">
              <span>{heroGame.publisher || 'Unknown publisher'}</span>
            </div>
            <div className="flix-hero-actions">
              <button
                className="flix-btn flix-btn-primary"
                onClick={() => handleLaunch(heroGame.id)}
              >
                ▶ Play
              </button>
              <button className="flix-btn flix-btn-secondary">
                ⭐ My List
              </button>
            </div>
          </div>
        </section>
      )}

      {/* Content rows */}
      <div className="flix-rows">
        {loading ? (
          <div className="flix-loading">Loading games…</div>
        ) : displayedRows.length === 0 ? (
          <div className="flix-empty">
            <h3>No games found</h3>
            <p>Install games in Steam, Heroic, or Lutris to see them here.</p>
          </div>
        ) : (
          displayedRows.map((row, i) => (
            <div className="flix-row" key={i}>
              {activeTab === 'recent' && i === 0 && (
                <h2 className="flix-row-title">Recently Played</h2>
              )}
              {activeTab === 'all' && i === 0 && (
                <h2 className="flix-row-title">All Games</h2>
              )}
              <div className="flix-card-container">
                {row.map(game => (
                  <GameCard key={game.id} game={game} onLaunch={handleLaunch} />
                ))}
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}

export default App;
