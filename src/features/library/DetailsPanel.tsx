import { useState } from "react";
import { Hints } from "../../components/Hints";
import { Icon, type IconName } from "../../components/Icon";
import { asset } from "../../lib/assets";
import { fmtPlaytime } from "../../lib/format";
import { useActionHandler } from "../../input/bus";
import type { PadKind } from "../../input/glyphs";
import type { Game } from "../../lib/types";

interface Props {
  game: Game;
  pad: PadKind | null;
  onClose: () => void;
  onLaunch: (g: Game) => void;
  onFavorite: (g: Game) => void;
  onEdit: (g: Game) => void;
}

interface Act {
  key: string;
  label: string;
  icon: IconName;
  run: () => void;
  cta?: boolean;
  on?: boolean;
}

/** Detalhes do jogo: descrição, informações e ações (ícones, navegável por controle). */
export function DetailsPanel({ game, pad, onClose, onLaunch, onFavorite, onEdit }: Props) {
  const [idx, setIdx] = useState(0);
  const cover = asset(game.coverPath);
  const logo = asset(game.logoPath);
  const hero = asset(game.heroPath);

  const actions: Act[] = [
    { key: "play", label: game.installState === "not_installed" ? "Instalar na Steam" : "Jogar", icon: "play", run: () => onLaunch(game), cta: true },
    { key: "fav", label: game.favorite ? "Remover dos favoritos" : "Favoritar", icon: "star", run: () => onFavorite(game), on: game.favorite },
    { key: "edit", label: "Editar", icon: "pencil", run: () => onEdit(game) },
    { key: "close", label: "Fechar", icon: "close", run: onClose },
  ];

  useActionHandler((a) => {
    if (a === "back") onClose();
    else if (a === "left") setIdx((i) => Math.max(0, i - 1));
    else if (a === "right") setIdx((i) => Math.min(actions.length - 1, i + 1));
    else if (a === "ok") actions[idx].run();
    else if (a === "x") onFavorite(game);
  });

  return (
    <div className="details-backdrop" onClick={(e) => e.target === e.currentTarget && onClose()}>
      <article className="details" role="dialog" aria-label={game.name}>
        <div className="cover big">{cover ? <img src={cover} alt="" /> : <span>{game.name}</span>}</div>
        <div className="details-body">
          {logo ? <img className="details-logo" src={logo} alt={game.name} /> : <h2>{game.name}</h2>}
          <p className="fs-sub">
            {[game.category, game.platform === "steam" ? "Steam" : game.platform, game.installState === "not_installed" ? "Não instalado" : null, fmtPlaytime(game.playtimeMinutes)]
              .filter(Boolean)
              .join("  ·  ")}
          </p>
          {hero && <img className="details-hero" src={hero} alt="" />}
          <p className="details-desc">{game.description ?? "Sem descrição ainda. Toque no lápis para adicionar uma."}</p>
          <p className="fs-sub">
            {game.lastPlayedAt ? `Última execução: ${new Date(game.lastPlayedAt * 1000).toLocaleString("pt-BR")}` : "Nunca executado pelo ALEXANDRIA"}
          </p>
          <div className="actions">
            {actions.map((a, i) =>
              a.cta ? (
                <button key={a.key} className={`btn primary${i === idx ? " focus" : ""}`} onMouseEnter={() => setIdx(i)} onClick={a.run}>
                  <Icon name={a.icon} size={16} /> {a.label}
                </button>
              ) : (
                <button
                  key={a.key}
                  className={`round${i === idx ? " focus" : ""}${a.on ? " on" : ""}`}
                  title={a.label}
                  aria-label={a.label}
                  onMouseEnter={() => setIdx(i)}
                  onClick={a.run}
                >
                  <Icon name={a.icon} filled={a.on} />
                </button>
              ),
            )}
          </div>
          {pad && <Hints pad={pad} items={[["ok", "play", "Selecionar"], ["back", "close", "Voltar"], ["x", "star", "Favoritar"]]} />}
        </div>
      </article>
    </div>
  );
}
