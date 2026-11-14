import { useEffect, useState } from "react";
import { Hints } from "../../components/Hints";
import { Icon } from "../../components/Icon";
import { asset } from "../../lib/assets";
import { fmtPlaytime } from "../../lib/format";
import { glyph, type PadKind } from "../../input/glyphs";
import { TABS, type Tab } from "../browse/useBrowse";
import { GameCard } from "./GameCard";
import type { Game } from "../../lib/types";

interface Props {
  games: Game[];
  hasAny: boolean;
  selected: Game | null;
  onSelect: (id: number) => void;
  onOpen: (id: number) => void;
  onPlay: (g: Game) => void;
  tab: Tab;
  pad: PadKind | null;
}

const fmtTime = () => new Date().toLocaleTimeString("pt-BR", { hour: "2-digit", minute: "2-digit" });

/** Relógio leve: atualiza a cada 20 s e só existe no fullscreen. */
function Clock() {
  const [t, setT] = useState(fmtTime);
  useEffect(() => {
    const id = window.setInterval(() => setT(fmtTime()), 20_000);
    return () => window.clearInterval(id);
  }, []);
  return <span className="fs-clock">{t}</span>;
}

/** Modo fullscreen: experiência de console. Confirmar já inicia o jogo selecionado. */
export function FullscreenShell({ games, hasAny, selected, onSelect, onOpen, onPlay, tab, pad }: Props) {
  const logo = asset(selected?.logoPath);

  useEffect(() => {
    if (selected) {
      document
        .querySelector(`[data-id="${selected.id}"]`)
        ?.scrollIntoView({ block: "nearest", inline: "center", behavior: "smooth" });
    }
  }, [selected?.id]); // eslint-disable-line react-hooks/exhaustive-deps

  return (
    <div className="shell fs shell-in">
      <header className="fs-top">
        <nav className="fs-tabs">
          {TABS.map((t) => (
            <span key={t.id} className={t.id === tab ? "on" : ""}>{t.label}</span>
          ))}
        </nav>
        <div className="fs-status">
          {pad && <span>{pad === "ps" ? "PlayStation" : "Xbox"}</span>}
          <Clock />
        </div>
      </header>

      <section className="fs-hero" key={selected?.id ?? "none"}>
        {selected ? (
          <>
            {logo ? <img className="fs-logo" src={logo} alt={selected.name} /> : <h2>{selected.name}</h2>}
            <p className="fs-sub">
              {[selected.category, selected.platform === "steam" ? "Steam" : selected.platform, fmtPlaytime(selected.playtimeMinutes)].filter(Boolean).join("  ·  ")}
            </p>
            {selected.description && <p className="fs-desc">{selected.description}</p>}
            <button className="play" onClick={() => onPlay(selected)}>
              <Icon name="play" size={18} />
              {selected.installState === "not_installed" ? "Instalar na Steam" : "Jogar"}
              <kbd>{glyph("ok", pad)}</kbd>
            </button>
          </>
        ) : (
          <>
            <h2>{hasAny ? "Nada por aqui" : "Biblioteca vazia"}</h2>
            <p className="fs-sub">
              {hasAny ? "Mude de seção com os botões de ombro." : "Importe da Steam ou adicione jogos pelos ajustes."}
            </p>
          </>
        )}
      </section>

      <ul className="fs-shelf">
        {games.map((g, i) => (
          <GameCard key={g.id} game={g} index={i} selected={g.id === selected?.id} onSelect={onSelect} onOpen={onOpen} />
        ))}
      </ul>

      <Hints
        pad={pad}
        items={[
          ["ok", "play", "Jogar"],
          ["x", "star", "Favoritar"],
          ["y", "info", "Detalhes"],
          ["lb", "layers", "Trocar de seção"],
          ["lt", "skip", "Pular"],
          ["menu", "sliders", "Ajustes"],
          ["mode", "shrink", "Sair da tela cheia"],
        ]}
      />
    </div>
  );
}
