import { useEffect, useRef, useState } from "react";
import { Icon } from "../../components/Icon";
import { Modal } from "../../components/Modal";
import { api, errorMessage } from "../../lib/ipc";
import { PASS, useActionHandler } from "../../input/bus";
import type { DiscoverEntry, Game } from "../../lib/types";
import type { Wishlist } from "./useWishlist";

type Kind = "popular" | "recent" | "upcoming" | "search";
const KINDS: { id: Exclude<Kind, "search">; label: string }[] = [
  { id: "popular", label: "Populares" },
  { id: "recent", label: "Lançamentos" },
  { id: "upcoming", label: "Em breve" },
];

function gridCols(): number {
  const g = document.querySelector(".store-grid");
  return g ? getComputedStyle(g).gridTemplateColumns.split(" ").length : 1;
}

interface Props {
  wish: Wishlist;
  games: Game[];
  notify: (m: string) => void;
}

/** Descobrir jogos pelo IGDB: populares, lançamentos, em breve e busca por nome. */
export function DiscoverPanel({ wish, games, notify }: Props) {
  const [kind, setKind] = useState<Kind>("popular");
  const [term, setTerm] = useState("");
  const [items, setItems] = useState<DiscoverEntry[]>([]);
  const [status, setStatus] = useState<"loading" | "ok" | "error">("loading");
  const [error, setError] = useState<string | null>(null);
  const [sel, setSel] = useState(0);
  const [searchOpen, setSearchOpen] = useState(false);
  const [draft, setDraft] = useState("");
  const cache = useRef(new Map<string, DiscoverEntry[]>());

  useEffect(() => {
    let alive = true;
    const key = kind === "search" ? `s:${term.toLowerCase()}` : kind;
    const hit = cache.current.get(key);
    setSel(0);
    if (hit) {
      setItems(hit);
      setStatus("ok");
      return;
    }
    setStatus("loading");
    api
      .igdbDiscover(kind, term)
      .then((l) => {
        if (!alive) return;
        cache.current.set(key, l);
        setItems(l);
        setStatus("ok");
      })
      .catch((e) => {
        if (!alive) return;
        setError(errorMessage(e));
        setStatus("error");
      });
    return () => {
      alive = false;
    };
  }, [kind, term]);

  const idx = Math.min(sel, Math.max(0, items.length - 1));
  const cur = items[idx] ?? null;
  const owned = (name: string) => games.some((g) => g.name.toLowerCase() === name.toLowerCase());

  useEffect(() => {
    document.querySelector(`.store-tile[data-i="${idx}"]`)?.scrollIntoView({ block: "nearest" });
  }, [idx]);

  const openPage = (e: DiscoverEntry) => {
    const url = e.links[0] ?? e.page;
    if (url) api.openExternal(url).catch((err) => notify(errorMessage(err)));
  };

  const doSearch = () => {
    const t = draft.trim();
    if (!t) return;
    setTerm(t);
    setKind("search");
    setSearchOpen(false);
  };

  useActionHandler((a) => {
    if (a === "lb" || a === "rb" || a === "lt" || a === "rt" || a === "menu" || a === "mode" || a === "back") return PASS;
    if (a === "y") return void setSearchOpen(true);
    if (!items.length) return;
    const cols = gridCols();
    if (a === "left") setSel(Math.max(0, idx - 1));
    else if (a === "right") setSel(Math.min(items.length - 1, idx + 1));
    else if (a === "up") setSel(Math.max(0, idx - cols));
    else if (a === "down") setSel(Math.min(items.length - 1, idx + cols));
    else if (a === "ok" && cur) openPage(cur);
    else if (a === "x" && cur) void wish.toggle(cur);
  });

  return (
    <div className="store">
      <div className="store-head">
        <div className="chips">
          {KINDS.map((k) => (
            <button key={k.id} className="btn" aria-pressed={kind === k.id} onClick={() => setKind(k.id)}>{k.label}</button>
          ))}
          {kind === "search" && <button className="btn" aria-pressed>“{term}”</button>}
        </div>
        <button className="btn" onClick={() => { setDraft(term); setSearchOpen(true); }}><Icon name="search" size={16} /> Buscar jogo</button>
      </div>

      {status === "loading" && <p className="muted">Buscando no IGDB…</p>}
      {status === "error" && (
        <div className="empty">
          <h2>Não foi possível carregar</h2>
          <p>{error}</p>
          <p>As chaves do IGDB ficam em Ajustes (ícone de controles deslizantes).</p>
        </div>
      )}
      {status === "ok" && items.length === 0 && <p className="muted">Nenhum jogo encontrado.</p>}

      <ul className="store-grid">
        {items.map((g, i) => (
          <li key={g.id} data-i={i} className={`store-tile${i === idx ? " sel" : ""}`} onClick={() => setSel(i)}>
            <div className="cover">
              {g.cover ? <img src={g.cover} alt="" loading="lazy" decoding="async" draggable={false} /> : <span>{g.name}</span>}
              {wish.has(g.id) && <i className="star" aria-label="Na lista de desejos">★</i>}
              {owned(g.name) && <i className="badge" title="Já está na sua biblioteca" aria-label="Na biblioteca"><Icon name="check" size={12} /></i>}
            </div>
            <div className="meta">
              <span className="name">{g.name}</span>
              <span>{[g.year, g.genres[0]].filter(Boolean).join("  ·  ")}</span>
            </div>
          </li>
        ))}
      </ul>

      {cur && (
        <section className="store-detail">
          <div className="store-detail-top">
            <div>
              <h3>{cur.name}</h3>
              <p className="muted">
                {[cur.year, cur.rating ? `Nota ${cur.rating}` : null, cur.genres.join(", "), cur.platforms.slice(0, 5).join(", ")]
                  .filter(Boolean)
                  .join("  ·  ")}
              </p>
            </div>
            <div className="row">
              <button className="btn primary" disabled={!cur.links[0] && !cur.page} onClick={() => openPage(cur)}>
                <Icon name="external" size={16} /> {cur.links[0] ? "Ver na loja" : "Ver no IGDB"}
              </button>
              <button className="round" aria-pressed={wish.has(cur.id)} title={wish.has(cur.id) ? "Tirar da lista de desejos" : "Lista de desejos"} aria-label="Lista de desejos" onClick={() => void wish.toggle(cur)}>
                <Icon name="star" filled={wish.has(cur.id)} />
              </button>
            </div>
          </div>
          {cur.summary && <p className="store-desc">{cur.summary}</p>}
        </section>
      )}

      {searchOpen && (
        <Modal title={<Icon name="search" size={24} />} label="Buscar jogo" onClose={() => setSearchOpen(false)}>
          <div className="f"><label htmlFor="disc-q">Nome do jogo</label>
            <input id="disc-q" value={draft} autoFocus autoComplete="off" onChange={(e) => setDraft(e.target.value)} onKeyDown={(e) => e.key === "Enter" && doSearch()} /></div>
          <div className="acts">
            <button className="btn" onClick={() => setSearchOpen(false)}>Cancelar</button>
            <button className="btn primary" disabled={!draft.trim()} onClick={doSearch}>Buscar</button>
          </div>
        </Modal>
      )}
    </div>
  );
}
