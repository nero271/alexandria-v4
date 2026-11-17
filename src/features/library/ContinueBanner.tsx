import { Icon } from "../../components/Icon";
import { asset } from "../../lib/assets";
import { fmtPlaytime } from "../../lib/format";
import { timeAgo } from "../../lib/status";
import type { Game } from "../../lib/types";

/** Destaque do último jogo jogado, com botão para continuar. */
export function ContinueBanner({ game, onPlay, onOpen }: { game: Game; onPlay: (g: Game) => void; onOpen: (id: number) => void }) {
  const bg = asset(game.heroPath ?? game.backgroundPath ?? game.coverPath);
  const logo = asset(game.logoPath);
  const meta = [game.lastPlayedAt ? `Jogado ${timeAgo(game.lastPlayedAt)}` : null, fmtPlaytime(game.playtimeMinutes)].filter(Boolean).join("  ·  ");
  return (
    <section className="continue" aria-label="Continuar jogando">
      <div className="continue-bg" style={bg ? { backgroundImage: `url("${bg}")` } : undefined} />
      <div className="continue-body">
        <div>
          <p className="continue-kicker">Continuar</p>
          {logo ? <img className="continue-logo" src={logo} alt={game.name} /> : <h2>{game.name}</h2>}
          {meta && <p className="fs-sub">{meta}</p>}
        </div>
        <div className="row">
          <button className="play" onClick={() => onPlay(game)}><Icon name="play" size={18} /> Jogar</button>
          <button className="round" title="Detalhes" aria-label="Detalhes" onClick={() => onOpen(game.id)}><Icon name="info" /></button>
        </div>
      </div>
    </section>
  );
}
