import { memo, type CSSProperties } from "react";
import { asset } from "../../lib/assets";
import type { Game } from "../../lib/types";

interface Props {
  game: Game;
  index: number;
  selected: boolean;
  onSelect: (id: number) => void;
  onOpen: (id: number) => void;
}

function GameCardBase({ game, index, selected, onSelect, onOpen }: Props) {
  const cover = asset(game.coverPath);
  return (
    <li
      className={`card${selected ? " sel" : ""}${game.installState === "not_installed" ? " off" : ""}`}
      style={{ "--i": Math.min(index, 14) } as CSSProperties}
      data-id={game.id}
      onMouseEnter={() => onSelect(game.id)}
      onClick={() => {
        onSelect(game.id);
        onOpen(game.id);
      }}
    >
      <div className="cover">
        {cover ? <img src={cover} alt="" loading="lazy" decoding="async" draggable={false} /> : <span>{game.name}</span>}
        {game.favorite && <i className="star" aria-label="Favorito">★</i>}
      </div>
      <div className="meta">
        <span className="name">{game.name}</span>
        <span>{game.installState === "not_installed" ? "Não instalado" : game.category ?? game.platform}</span>
      </div>
    </li>
  );
}

export const GameCard = memo(GameCardBase);
