import { useEffect, type CSSProperties } from "react";
import { Icon } from "../../components/Icon";
import { TABS, type Tab } from "../browse/useBrowse";
import { STATUSES } from "../../lib/status";
import { ContinueBanner } from "./ContinueBanner";
import { GameCard } from "./GameCard";
import type { Game } from "../../lib/types";
import type { Progress } from "./useLibrary";

interface Props {
  appName: string;
  games: Game[];
  hasAny: boolean;
  selectedId: number | null;
  onSelect: (id: number) => void;
  onOpen: (id: number) => void;
  tab: Tab;
  onTab: (t: Tab) => void;
  category: string;
  categories: string[];
  onCategory: (c: string) => void;
  query: string;
  onQuery: (q: string) => void;
  statusFilter: string;
  onStatus: (s: string) => void;
  continueGame: Game | null;
  onPlay: (g: Game) => void;
  progress: Progress | null;
  onAdd: () => void;
  onSettings: () => void;
  onMode: () => void;
  onYear: () => void;
}

/** Modo desktop: barra translúcida, abas deslizantes e grade de capas. */
export function DesktopShell(p: Props) {
  useEffect(() => {
    if (p.selectedId !== null) {
      document.querySelector(`[data-id="${p.selectedId}"]`)?.scrollIntoView({ block: "nearest" });
    }
  }, [p.selectedId]);

  const tabIndex = TABS.findIndex((t) => t.id === p.tab);

  return (
    <div className="shell shell-in">
      <header className="top glass">
        <h1 className="wordmark">{p.appName}</h1>
        <nav className="seg" style={{ "--i": tabIndex, "--n": TABS.length } as CSSProperties}>
          {TABS.map((t) => (
            <button key={t.id} aria-pressed={p.tab === t.id} onClick={() => p.onTab(t.id)}>
              {t.label}
            </button>
          ))}
        </nav>
        <div className="top-right">
          <label className="searchbox">
            <Icon name="search" size={16} />
            <input
              type="search"
              placeholder="Buscar"
              aria-label="Buscar na biblioteca"
              value={p.query}
              onChange={(e) => p.onQuery(e.target.value)}
            />
          </label>
          {p.categories.length > 0 && (
            <select className="select" aria-label="Categoria" value={p.category} onChange={(e) => p.onCategory(e.target.value)}>
              <option value="">Todas as categorias</option>
              {p.categories.map((c) => (
                <option key={c} value={c}>{c}</option>
              ))}
            </select>
          )}
          <select className="select" aria-label="Status" value={p.statusFilter} onChange={(e) => p.onStatus(e.target.value)}>
            <option value="">Todos os status</option>
            {STATUSES.map((s) => (
              <option key={s.id} value={s.id}>{s.label}</option>
            ))}
          </select>
          <button className="btn primary" onClick={p.onAdd}><Icon name="plus" size={16} /> Adicionar</button>
          <button className="round" title="Resumo do ano" aria-label="Resumo do ano" onClick={p.onYear}><Icon name="chart" /></button>
          <button className="round" title="Ajustes" aria-label="Ajustes" onClick={p.onSettings}><Icon name="sliders" /></button>
          <button className="round" title="Tela cheia" aria-label="Tela cheia" onClick={p.onMode}><Icon name="expand" /></button>
        </div>
      </header>

      <main className="content">
        {p.progress && (
          <p className="progress">Baixando capas e informações… {p.progress.done} de {p.progress.total}</p>
        )}
        {p.continueGame && <ContinueBanner game={p.continueGame} onPlay={p.onPlay} onOpen={p.onOpen} />}
        {!p.hasAny ? (
          <div className="empty">
            <h2>Sua biblioteca está vazia</h2>
            <p>Adicione um jogo ou abra os ajustes para importar da Steam.</p>
            <button className="btn primary" onClick={p.onAdd}>Adicionar jogo</button>
          </div>
        ) : p.games.length === 0 ? (
          <div className="empty">
            <h2>Nenhum jogo encontrado</h2>
            <p>Tente outro nome ou outra seção.</p>
          </div>
        ) : (
          <ul className="grid">
            {p.games.map((g, i) => (
              <GameCard key={g.id} game={g} index={i} selected={g.id === p.selectedId} onSelect={p.onSelect} onOpen={p.onOpen} />
            ))}
          </ul>
        )}
      </main>
    </div>
  );
}
