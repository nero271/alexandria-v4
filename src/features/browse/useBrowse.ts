import { useMemo, useState } from "react";
import type { Game } from "../../lib/types";

export type Tab = "all" | "favorites" | "recent";
export const TABS: { id: Tab; label: string }[] = [
  { id: "all", label: "Todos" },
  { id: "favorites", label: "Favoritos" },
  { id: "recent", label: "Recentes" },
];

const RECENT_LIMIT = 30;

/** Filtros, ordenação e seleção da biblioteca (compartilhados pelos dois modos). */
export function useBrowse(games: Game[]) {
  const [tab, setTab] = useState<Tab>("all");
  const [category, setCategory] = useState("");
  const [query, setQuery] = useState("");
  const [selId, setSelId] = useState<number | null>(null);

  const categories = useMemo(
    () => Array.from(new Set(games.map((g) => g.category).filter((c): c is string => !!c))).sort((a, b) => a.localeCompare(b, "pt")),
    [games],
  );

  const visible = useMemo(() => {
    const q = query.trim().toLowerCase();
    const list = games.filter(
      (g) =>
        (!q || g.name.toLowerCase().includes(q)) &&
        (!category || g.category === category) &&
        (tab !== "favorites" || g.favorite) &&
        (tab !== "recent" || g.lastPlayedAt !== null),
    );
    return tab === "recent"
      ? list.sort((a, b) => (b.lastPlayedAt ?? 0) - (a.lastPlayedAt ?? 0)).slice(0, RECENT_LIMIT)
      : list.sort((a, b) => a.name.localeCompare(b.name, "pt"));
  }, [games, tab, category, query]);

  const found = visible.findIndex((g) => g.id === selId);
  const index = found >= 0 ? found : 0;
  const selected: Game | null = visible[index] ?? null;

  const selectIndex = (i: number) => {
    const g = visible[Math.max(0, Math.min(visible.length - 1, i))];
    if (g) setSelId(g.id);
  };
  const cycleTab = (d: 1 | -1) => {
    const i = TABS.findIndex((t) => t.id === tab);
    setTab(TABS[(i + d + TABS.length) % TABS.length].id);
    setSelId(null);
  };

  return {
    tab,
    setTab: (t: Tab) => {
      setTab(t);
      setSelId(null);
    },
    category,
    setCategory,
    query,
    setQuery,
    categories,
    visible,
    index,
    selected,
    select: setSelId,
    selectIndex,
    cycleTab,
  };
}
