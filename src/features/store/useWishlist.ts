import { useCallback, useEffect, useState } from "react";
import { api, errorMessage } from "../../lib/ipc";
import type { DiscoverEntry, WishItem } from "../../lib/types";

/** Lista de desejos local (jogos do IGDB que você quer acompanhar). */
export function useWishlist(notify: (m: string) => void) {
  const [items, setItems] = useState<WishItem[]>([]);

  const load = useCallback(async () => {
    try {
      setItems(await api.wishlistList());
    } catch (e) {
      notify(errorMessage(e));
    }
  }, [notify]);

  useEffect(() => {
    void load();
  }, [load]);

  const has = useCallback((id: number) => items.some((w) => w.id === id), [items]);

  const remove = useCallback(
    async (id: number) => {
      try {
        await api.wishlistRemove(id);
        setItems((prev) => prev.filter((w) => w.id !== id));
      } catch (e) {
        notify(errorMessage(e));
      }
    },
    [notify],
  );

  const toggle = useCallback(
    async (e: DiscoverEntry) => {
      if (items.some((w) => w.id === e.id)) return remove(e.id);
      try {
        await api.wishlistAdd({ id: e.id, name: e.name, cover: e.cover, year: e.year, url: e.links[0] ?? e.page ?? null });
        await load();
        notify(`${e.name} entrou na lista de desejos.`);
      } catch (err) {
        notify(errorMessage(err));
      }
    },
    [items, load, notify, remove],
  );

  return { items, has, toggle, remove };
}

export type Wishlist = ReturnType<typeof useWishlist>;
