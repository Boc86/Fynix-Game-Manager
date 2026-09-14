// GameCard — Netflix-style cover-only hover card with install state indicator
import { Game } from '../types/Game';

interface GameCardProps {
  game: Game;
  onLaunch: (gameId: string) => void;
}

export function GameCard({ game, onLaunch }: GameCardProps) {
  const hasImage = game.cover_url && game.cover_url.length > 0;
  const isInstalled = !!game.install_path;

  return (
    <div className={`flix-card ${isInstalled ? 'installed' : 'not-installed'}`} data-game-id={game.id}>
      <div className="flix-card-media">
        {hasImage ? (
          <img
            src={game.cover_url}
            alt={game.name}
            className="flix-card-image"
          />
        ) : (
          <div className="flix-card-placeholder">
            <span className="flix-card-placeholder-text">{game.name}</span>
          </div>
        )}
        <div className="flix-play-overlay">
          <div className="flix-play-icon"></div>
        </div>
        {!isInstalled && (
          <div className="flix-card-badge">Not Installed</div>
        )}
      </div>
      <div className="flix-card-stats">
        <h3 className="flix-card-title" title={game.name}>{game.name}</h3>
        {game.playtime_hours && isInstalled && (
          <span className="flix-card-meta">{Math.round(game.playtime_hours)}h</span>
        )}
      </div>
    </div>
  );
}
