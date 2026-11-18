import { useEffect, useState, type ReactNode } from "react";
import { Hints } from "../../components/Hints";
import { Icon, type IconName } from "../../components/Icon";
import type { Action } from "../../input/bus";
import { DownloadChip } from "../store/DownloadChip";
import type { DownloadSummary } from "../store/useStore";
import { asset } from "../../lib/assets";
import { fmtPlaytime } from "../../lib/format";
import { platformLabel } from "../../lib/platform";
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
  storeContent: ReactNode;
  downloads: DownloadSummary | null;
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
export function FullscreenShell({ games, hasAny, selected, onSelect, onOpen, onPlay, tab, pad, storeContent, downloads }: Props) {
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
          {downloads && <DownloadChip d={downloads} />}
          {pad && <span>{pad === "ps" ? "PlayStation" : "Xbox"}</span>}
          <Clock />
        </div>
      </header>

      {tab === "store" ? (
        <div className="fs-store">{storeContent}</div>
      ) : (
      <>
      <section className="fs-hero" key={selected?.id ?? "none"}>
        {selected ? (
          <>
            {logo ? <img className="fs-logo" src={logo} alt={selected.name} /> : <h2>{selected.name}</h2>}
            <p className="fs-sub">
              {[selected.category, platformLabel(selected.platform), fmtPlaytime(selected.playtimeMinutes)].filter(Boolean).join("  ·  ")}
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
      </>
      )}

      <Hints
        pad={pad}
        items={tab === "store" ? ([
          ["ok", "download", "Baixar, abrir ou jogar"],
          ["x", "star", "Desejo ou remover"],
          ["y", "search", "Buscar ou gerenciar"],
          ["lt", "skip", "Trocar painel"],
          ["lb", "layers", "Trocar de seção"],
          ["menu", "sliders", "Ajustes"],
          ["mode", "shrink", "Sair da tela cheia"],
        ] as [Action, IconName, string][]) : [
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
