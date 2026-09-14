// Component for displaying a single game card in the Netflix-style grid
import { Game } from '../types/Game';

interface GameCardProps {
  game: Game;
  onLaunch: (gameId: string) => void;
}

export function GameCard({ game, onLaunch }: GameCardProps) {
  return (
    <div className="flix-card group">
      <div className="relative pb-[150%]">
        {game.cover_url ? (
          <img
            src={game.cover_url}
            alt={game.name}
            className="absolute inset-0 w-full h-full object-cover rounded"
          />
        ) : (
          <div className="absolute inset-0 bg-secondary rounded flex items-center justify-center">
            <span className="text-muted">{game.name}</span>
          </div>
        )}
        <div className="flix-play-overlay">
          <button
            onClick={() => onLaunch(game.id)}
            className="flix-btn flix-btn-primary"
          >
            ▶ Play
          </button>
        </div>
      </div>
      <div className="flix-card-content">
        <h3 className="flix-card-title">{game.name}</h3>
        <p className="flix-card-meta">{game.publisher}</p>
        {game.playtime_hours && (
          <p className="flix-card-stats">{Math.round(game.playtime_hours)}h played</p>
        )}
      </div>
    </div>
  );
}
