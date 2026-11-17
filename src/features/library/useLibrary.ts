import { useCallback, useEffect, useRef, useState } from "react";
import { api, errorMessage } from "../../lib/ipc";
import type { Game, GameUpdate, NewGame, PlatformStatus } from "../../lib/types";

const FOCUS_SYNC_MIN_INTERVAL_MS = 30_000;
const PLATFORMS = ["steam", "epic", "gog", "ubisoft", "xbox"] as const;

export interface Progress {
  done: number;
  total: number;
}

export function useLibrary(notify: (msg: string) => void) {
  const [games, setGames] = useState<Game[]>([]);
  const [loading, setLoading] = useState(true);
  const [platforms, setPlatforms] = useState<PlatformStatus | null>(null);
  const [progress, setProgress] = useState<Progress | null>(null);
  const platformsRef = useRef<PlatformStatus | null>(null);
  const busy = useRef(false);
  const lastSync = useRef(0);

  const fail = useCallback((e: unknown) => notify(errorMessage(e)), [notify]);
  const replace = useCallback(
    (g: Game) => setGames((prev) => prev.map((x) => (x.id === g.id ? g : x))),
    [],
  );
  const reload = useCallback(async () => {
    try {
      setGames(await api.listGames());
    } catch (e) {
      fail(e);
    } finally {
      setLoading(false);
    }
  }, [fail]);

  /** Baixa capas e descrições que faltam nos jogos da Steam (um por vez, sem sobrecarregar). */
  const enrichAll = useCallback(async () => {
    try {
      const todo = (await api.listGames()).filter(
        (g) => g.platform === "steam" && (!g.coverPath || !g.description),
      );
      if (!todo.length) return notify("Todas as capas e informações já estão baixadas.");
      let done = 0;
      setProgress({ done, total: todo.length });
      for (const g of todo) {
        try {
          replace(await api.enrichSteamGame(g.id));
        } catch {
          /* um jogo sem arte não impede os demais */
        }
        setProgress({ done: ++done, total: todo.length });
      }
    } catch (e) {
      fail(e);
    } finally {
      setProgress(null);
    }
  }, [fail, notify, replace]);

  /** Sincroniza Steam, Epic e GOG (as que existirem no computador). */
  const syncAll = useCallback(
    async (quiet = false) => {
      if (busy.current) return;
      busy.current = true;
      try {
        const st = quiet && platformsRef.current ? platformsRef.current : await api.platformStatus();
        platformsRef.current = st;
        setPlatforms(st);
        const total = { added: 0, updated: 0, uninstalled: 0 };
        let steamAdded = 0;
        let any = false;
        for (const p of PLATFORMS) {
          if (!st[p]) continue;
          any = true;
          try {
            const s = await api.platformSync(p);
            total.added += s.added;
            total.updated += s.updated;
            total.uninstalled += s.uninstalled;
            if (p === "steam") steamAdded = s.added;
          } catch (e) {
            if (!quiet) fail(e);
          }
        }
        lastSync.current = Date.now();
        await reload();
        const changed = total.added + total.updated + total.uninstalled > 0;
        if (!any) {
          if (!quiet) notify("Nenhuma loja (Steam, Epic, GOG, Ubisoft ou Xbox) foi encontrada neste computador.");
        } else if (!quiet || changed) {
          notify(
            changed
              ? `Lojas: ${total.added} novo(s), ${total.updated} reinstalado(s), ${total.uninstalled} desinstalado(s).`
              : "Lojas: biblioteca já está atualizada.",
          );
        }
        if (steamAdded > 0) void enrichAll();
      } catch (e) {
        if (!quiet) fail(e);
      } finally {
        busy.current = false;
      }
    },
    [enrichAll, fail, notify, reload],
  );

  // Início: carrega a biblioteca e sincroniza com as lojas.
  useEffect(() => {
    void (async () => {
      await reload();
      await syncAll(true);
    })();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Ao voltar para o launcher (por exemplo, depois de fechar um jogo): atualiza recentes e
  // tempo de jogo; sincroniza as lojas se já passou um tempo. Sem processo em segundo plano.
  useEffect(() => {
    const onFocus = () => {
      void reload();
      if (Date.now() - lastSync.current > FOCUS_SYNC_MIN_INTERVAL_MS) void syncAll(true);
    };
    window.addEventListener("focus", onFocus);
    return () => window.removeEventListener("focus", onFocus);
  }, [reload, syncAll]);

  const guard = useCallback(
    async <T,>(fn: () => Promise<T>): Promise<T | null> => {
      try {
        return await fn();
      } catch (e) {
        fail(e);
        return null;
      }
    },
    [fail],
  );

  return {
    games,
    loading,
    platforms,
    progress,
    replace,
    reload,
    syncAll,
    enrichAll,
    addGame: async (g: NewGame) => {
      const created = await guard(() => api.addGame(g));
      if (created) setGames((p) => [...p, created]);
      return created;
    },
    updateGame: async (id: number, patch: GameUpdate) => {
      const g = await guard(() => api.updateGame(id, patch));
      if (g) replace(g);
      return g;
    },
    setStatus: async (g: Game, status: string | null) => {
      const r = await guard(() => api.setStatus(g.id, status));
      if (r) replace(r);
      return r;
    },
    setRating: async (g: Game, rating: number | null) => {
      const r = await guard(() => api.setRating(g.id, rating));
      if (r) replace(r);
      return r;
    },
    toggleFavorite: async (g: Game) => {
      const r = await guard(() => api.setFavorite(g.id, !g.favorite));
      if (r) replace(r);
    },
    removeGame: async (g: Game) => {
      const ok = await guard(async () => (await api.removeGame(g.id), true));
      if (ok) setGames((p) => p.filter((x) => x.id !== g.id));
      return !!ok;
    },
    launchGame: async (g: Game) => {
      const r = await guard(() => api.launchGame(g.id));
      if (r) replace(r);
      return !!r;
    },
  };
}
