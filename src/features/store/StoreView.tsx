import { useEffect, useState } from "react";
import { Icon } from "../../components/Icon";
import { fmtBytes, fmtEta, fmtSpeed, pct } from "../../lib/format";
import { PASS, useActionHandler } from "../../input/bus";
import type { Game, InstallProgress, SourceGame } from "../../lib/types";
import { keyOf, type Store } from "./useStore";

const PHASE: Record<InstallProgress["phase"], string> = {
  start: "Preparando…",
  download: "Baixando",
  verify: "Verificando o arquivo",
  extract: "Instalando",
  done: "Concluindo",
  paused: "Pausado",
};

function gridCols(): number {
  const g = document.querySelector(".store-grid");
  return g ? getComputedStyle(g).gridTemplateColumns.split(" ").length : 1;
}

interface Props {
  store: Store;
  games: Game[];
  onPlay: (g: Game) => void;
  onManage: () => void;
}

/** Aba Loja: jogos das fontes cadastradas, com download, porcentagem, pausa e instalação. */
export function StoreView({ store, games, onPlay, onManage }: Props) {
  const items = store.games;
  const [sel, setSel] = useState(0);
  const [confirm, setConfirm] = useState<string | null>(null);
  const idx = Math.min(sel, Math.max(0, items.length - 1));
  const cur = items[idx] ?? null;
  const key = cur ? keyOf(cur) : "";
  const prog = key ? store.progress[key] : undefined;

  useEffect(() => {
    document.querySelector(`.store-tile[data-i="${idx}"]`)?.scrollIntoView({ block: "nearest" });
  }, [idx]);

  const primary = (g: SourceGame) => {
    const p = store.progress[keyOf(g)];
    if (p && p.phase !== "paused") return void store.pause(g);
    if (g.installed && !g.update) {
      const lg = games.find((x) => x.id === g.libraryId);
      if (lg) onPlay(lg);
      return;
    }
    void store.install(g);
  };

  const remove = (g: SourceGame) => {
    const k = keyOf(g);
    if (confirm === k) {
      setConfirm(null);
      void store.uninstall(g);
    } else {
      setConfirm(k);
    }
  };

  useActionHandler((a) => {
    if (a === "lb" || a === "rb" || a === "lt" || a === "rt" || a === "menu" || a === "mode") return PASS;
    if (a === "back") {
      if (confirm) setConfirm(null);
      return PASS;
    }
    if (a === "y") return onManage();
    if (!items.length) return;
    const cols = gridCols();
    if (a === "left") setSel(Math.max(0, idx - 1));
    else if (a === "right") setSel(Math.min(items.length - 1, idx + 1));
    else if (a === "up") setSel(Math.max(0, idx - cols));
    else if (a === "down") setSel(Math.min(items.length - 1, idx + cols));
    else if (a === "ok" && cur) primary(cur);
    else if (a === "x" && cur?.installed) remove(cur);
  });

  if (!store.loaded) return <div className="store"><p className="muted">Carregando…</p></div>;

  if (store.sources.length === 0) {
    return (
      <div className="store">
        <div className="empty">
          <h2>A Loja ainda não tem fontes</h2>
          <p>
            Uma fonte é o endereço de um catálogo de jogos que você cadastra. O ALEXANDRIA não traz nem indica fontes:
            use só catálogos de conteúdo que você tem o direito de baixar e usar (seus próprios jogos, código aberto, jogos gratuitos).
          </p>
          <button className="btn primary" onClick={onManage}><Icon name="plus" size={16} /> Adicionar fonte</button>
        </div>
      </div>
    );
  }

  const percent = prog ? pct(prog.done, prog.total) : null;
  const eta = prog && prog.phase === "download" && prog.speed > 0 && prog.total > prog.done ? fmtEta((prog.total - prog.done) / prog.speed) : "";
  const active = !!prog && prog.phase !== "paused";

  const primaryLabel = !cur
    ? ""
    : active
      ? "Pausar"
      : prog?.phase === "paused"
        ? "Continuar"
        : cur.installed && !cur.update
          ? "Jogar"
          : cur.update
            ? "Atualizar"
            : "Baixar";
  const primaryIcon = active ? "pause" : cur?.installed && !cur.update ? "play" : "download";

  return (
    <div className="store">
      <div className="store-head">
        <h2>Loja</h2>
        <div className="row">
          <button className="btn" onClick={onManage}><Icon name="store" size={16} /> Fontes</button>
        </div>
      </div>

      <ul className="store-grid">
        {items.map((g, i) => {
          const p = store.progress[keyOf(g)];
          const v = p ? pct(p.done, p.total) : null;
          return (
            <li key={keyOf(g)} data-i={i} className={`store-tile${i === idx ? " sel" : ""}`} onClick={() => setSel(i)}>
              <div className="cover">
                {g.cover ? <img src={g.cover} alt="" loading="lazy" decoding="async" draggable={false} /> : <span>{g.name}</span>}
                {g.installed && !p && (
                  <i className="badge" title="Instalado" aria-label="Instalado"><Icon name="check" size={12} /></i>
                )}
                {p && <div className="tile-progress"><span style={{ width: `${v ?? 100}%` }} /></div>}
                {p && v !== null && <b className="tile-pct">{v}%</b>}
              </div>
              <div className="meta">
                <span className="name">{g.name}</span>
                <span>{g.category ?? g.sourceName}</span>
              </div>
            </li>
          );
        })}
      </ul>

      {cur && (
        <section className="store-detail" aria-live="polite">
          <div className="store-detail-top">
            <div>
              <h3>{cur.name}</h3>
              <p className="muted">
                {[cur.sourceName, `v${cur.version}`, cur.license, cur.size ? fmtBytes(cur.size) : null, cur.host ? `de ${cur.host}` : null]
                  .filter(Boolean)
                  .join("  ·  ")}
              </p>
            </div>
            <div className="row">
              <button className="btn primary" onClick={() => primary(cur)}>
                <Icon name={primaryIcon} size={16} /> {primaryLabel}
              </button>
              {cur.installed && !active && (
                <button className="round" title={confirm === key ? "Confirmar remoção" : "Remover"} aria-label="Remover" onClick={() => remove(cur)}>
                  <Icon name={confirm === key ? "check" : "trash"} />
                </button>
              )}
            </div>
          </div>

          {prog ? (
            <div className="dl">
              <div className="dl-bar"><span className={percent === null ? "ind" : ""} style={percent === null ? undefined : { width: `${percent}%` }} /></div>
              <div className="dl-text">
                <b>{percent !== null ? `${percent}%` : "…"}</b>
                <span>{PHASE[prog.phase]}</span>
                {prog.phase === "download" && prog.total > 0 && <span>{fmtBytes(prog.done)} de {fmtBytes(prog.total)}</span>}
                {fmtSpeed(prog.speed) && <span>{fmtSpeed(prog.speed)}</span>}
                {eta && <span>faltam {eta}</span>}
              </div>
            </div>
          ) : (
            cur.description && <p className="store-desc">{cur.description}</p>
          )}
          {cur.update && !prog && <p className="muted">Nova versão disponível ({cur.installedVersion} → {cur.version}).</p>}
          {confirm === key && <p className="muted">Apertar de novo remove o jogo e apaga a pasta dele deste computador.</p>}
        </section>
      )}
    </div>
  );
}
