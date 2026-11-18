import { useState } from "react";
import { Icon } from "../../components/Icon";
import { api, errorMessage } from "../../lib/ipc";
import { PASS, useActionHandler } from "../../input/bus";
import type { WishItem } from "../../lib/types";
import type { Wishlist } from "./useWishlist";

function gridCols(): number {
  const g = document.querySelector(".store-grid");
  return g ? getComputedStyle(g).gridTemplateColumns.split(" ").length : 1;
}

/** Lista de desejos: abrir a página do jogo ou tirar da lista. */
export function WishlistPanel({ wish, notify }: { wish: Wishlist; notify: (m: string) => void }) {
  const [sel, setSel] = useState(0);
  const items = wish.items;
  const idx = Math.min(sel, Math.max(0, items.length - 1));
  const cur = items[idx] ?? null;

  const open = (w: WishItem) => {
    if (w.url) api.openExternal(w.url).catch((e) => notify(errorMessage(e)));
  };

  useActionHandler((a) => {
    if (a === "lb" || a === "rb" || a === "lt" || a === "rt" || a === "menu" || a === "mode" || a === "back" || a === "y") return PASS;
    if (!items.length) return;
    const cols = gridCols();
    if (a === "left") setSel(Math.max(0, idx - 1));
    else if (a === "right") setSel(Math.min(items.length - 1, idx + 1));
    else if (a === "up") setSel(Math.max(0, idx - cols));
    else if (a === "down") setSel(Math.min(items.length - 1, idx + cols));
    else if (a === "ok" && cur) open(cur);
    else if (a === "x" && cur) void wish.remove(cur.id);
  });

  if (items.length === 0) {
    return (
      <div className="store">
        <div className="empty">
          <h2>Sua lista de desejos está vazia</h2>
          <p>Em Descobrir, toque na estrela de um jogo para guardá-lo aqui.</p>
        </div>
      </div>
    );
  }

  return (
    <div className="store">
      <ul className="store-grid">
        {items.map((w, i) => (
          <li key={w.id} data-i={i} className={`store-tile${i === idx ? " sel" : ""}`} onClick={() => setSel(i)}>
            <div className="cover">
              {w.cover ? <img src={w.cover} alt="" loading="lazy" decoding="async" draggable={false} /> : <span>{w.name}</span>}
            </div>
            <div className="meta">
              <span className="name">{w.name}</span>
              <span>{w.year ?? ""}</span>
            </div>
          </li>
        ))}
      </ul>
      {cur && (
        <section className="store-detail">
          <div className="store-detail-top">
            <h3>{cur.name}</h3>
            <div className="row">
              <button className="btn primary" disabled={!cur.url} onClick={() => open(cur)}><Icon name="external" size={16} /> Abrir página</button>
              <button className="round" title="Tirar da lista" aria-label="Tirar da lista" onClick={() => void wish.remove(cur.id)}><Icon name="trash" /></button>
            </div>
          </div>
        </section>
      )}
    </div>
  );
}
