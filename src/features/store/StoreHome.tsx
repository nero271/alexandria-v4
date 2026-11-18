import { useState, type CSSProperties } from "react";
import { PASS, useActionHandler } from "../../input/bus";
import type { Game } from "../../lib/types";
import { DiscoverPanel } from "./DiscoverPanel";
import { StoreView } from "./StoreView";
import type { Store } from "./useStore";
import { useWishlist } from "./useWishlist";
import { WishlistPanel } from "./WishlistPanel";

type Panel = "discover" | "wishlist" | "sources";
const PANELS: { id: Panel; label: string }[] = [
  { id: "discover", label: "Descobrir" },
  { id: "wishlist", label: "Desejos" },
  { id: "sources", label: "Fontes" },
];

interface Props {
  store: Store;
  games: Game[];
  onPlay: (g: Game) => void;
  onManage: () => void;
  notify: (m: string) => void;
}

/** Aba Loja: Descobrir (IGDB), Desejos (lista local) e Fontes (catálogos para baixar). */
export function StoreHome({ store, games, onPlay, onManage, notify }: Props) {
  const [panel, setPanel] = useState<Panel>("discover");
  const wish = useWishlist(notify);
  const idx = PANELS.findIndex((p) => p.id === panel);

  // Com o controle: L2/R2 trocam de painel. O resto passa para o painel aberto.
  useActionHandler((a) => {
    if (a === "lt") setPanel(PANELS[(idx + PANELS.length - 1) % PANELS.length].id);
    else if (a === "rt") setPanel(PANELS[(idx + 1) % PANELS.length].id);
    else return PASS;
  });

  return (
    <div className="store-home">
      <nav className="seg" style={{ "--i": idx, "--n": PANELS.length } as CSSProperties}>
        {PANELS.map((p) => (
          <button key={p.id} aria-pressed={panel === p.id} onClick={() => setPanel(p.id)}>{p.label}</button>
        ))}
      </nav>
      {panel === "discover" && <DiscoverPanel wish={wish} games={games} notify={notify} />}
      {panel === "wishlist" && <WishlistPanel wish={wish} notify={notify} />}
      {panel === "sources" && <StoreView store={store} games={games} onPlay={onPlay} onManage={onManage} />}
    </div>
  );
}
