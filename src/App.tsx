import { useState, useEffect } from 'react';
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
        const result = await invoke<Game[]>('get_all_games');
        setGames(result || []);
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

  const filtered = search
    ? games.filter(g =>
        g.name.toLowerCase().includes(search.toLowerCase()) ||
        g.publisher.toLowerCase().includes(search.toLowerCase())
      )
    : games;

  const displayedGames = activeTab === 'recent'
    ? [...filtered].sort((a, b) => (b.last_played || 0) - (a.last_played || 0)).slice(0, 12)
    : filtered;

  return (
    <div className="flix-app">
      <header className="flix-header">
        <h1 className="flix-logo">Fynix<span className="flix-logo-red"> GM</span></h1>
        <input
          type="text"
          placeholder="Search games..."
          value={search}
          onChange={handleSearch}
          className="flix-search"
        />
        <button onClick={handleRefresh} className="flix-btn flix-btn-secondary">
          ↻ Refresh
        </button>
      </header>

      <nav className="flix-sidebar">
        <button
          className={`flix-sidebar-item ${activeTab === 'all' ? 'active' : ''}`}
          onClick={() => setActiveTab('all')}
        >
          All Games
        </button>
        <button
          className={`flix-sidebar-item ${activeTab === 'recent' ? 'active' : ''}`}
          onClick={() => setActiveTab('recent')}
        >
          Recently Played
        </button>
      </nav>

      <main>
        {loading ? (
          <div className="flix-game-grid">
            <div className="flix-loading">Loading games…</div>
          </div>
        ) : (
          <div className="flix-game-grid">
            {displayedGames.length === 0 ? (
              <div className="flix-empty">No games found. Try refreshing.</div>
            ) : (
              displayedGames.map(game => (
                <GameCard key={game.id} game={game} onLaunch={handleLaunch} />
              ))
            )}
          </div>
        )}
      </main>
    </div>
  );
}

export default App;
