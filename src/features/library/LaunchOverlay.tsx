import { Icon } from "../../components/Icon";
import { asset } from "../../lib/assets";
import { useActionHandler } from "../../input/bus";
import type { Game } from "../../lib/types";

/** Tela de carregamento enquanto o jogo abre. Fechar (B / Esc) só esconde a tela; o jogo continua abrindo. */
export function LaunchOverlay({ game, onDismiss }: { game: Game; onDismiss: () => void }) {
  const bg = asset(game.backgroundPath ?? game.heroPath ?? game.coverPath);
  const cover = asset(game.coverPath);
  const logo = asset(game.logoPath);

  useActionHandler((a) => {
    if (a === "back") onDismiss();
  });

  return (
    <div className="launch" role="status" aria-live="polite">
      <div className="launch-bg" style={bg ? { backgroundImage: `url("${bg}")` } : undefined} />
      <div className="launch-body">
        {cover ? <img className="launch-cover" src={cover} alt="" /> : <div className="launch-cover ph">{game.name}</div>}
        {logo ? <img className="launch-logo" src={logo} alt={game.name} /> : <h2>{game.name}</h2>}
        <div className="launch-bar"><span /></div>
        <p className="muted">Iniciando…</p>
        <button className="round" title="Fechar" aria-label="Fechar" onClick={onDismiss}><Icon name="close" /></button>
      </div>
    </div>
  );
}
