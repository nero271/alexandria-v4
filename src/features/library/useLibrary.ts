import { useCallback, useEffect, useRef, useState } from "react";
import { api, errorMessage } from "../../lib/ipc";
import type { Game, GameUpdate, NewGame, SteamStatus, SyncSummary } from "../../lib/types";

const FOCUS_SYNC_MIN_INTERVAL_MS = 30_000;

export interface Progress {
  done: number;
  total: number;
}

export function useLibrary(notify: (msg: string) => void) {
  const [games, setGames] = useState<Game[]>([]);
  const [loading, setLoading] = useState(true);
  const [steam, setSteam] = useState<SteamStatus | null>(null);
  const [progress, setProgress] = useState<Progress | null>(null);
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

  const syncSteam = useCallback(
    async (quiet = false): Promise<SyncSummary | null> => {
      if (busy.current) return null;
      busy.current = true;
      try {
        const s = await api.steamSync();
        lastSync.current = Date.now();
        await reload();
        const changed = s.added + s.updated + s.uninstalled > 0;
        if (!quiet || changed) {
          notify(
            changed
              ? `Steam: ${s.added} novo(s), ${s.updated} reinstalado(s), ${s.uninstalled} desinstalado(s).`
              : "Steam: biblioteca já está atualizada.",
          );
        }
        if (s.added > 0) void enrichAll();
        return s;
      } catch (e) {
        if (!quiet) fail(e);
        return null;
      } finally {
        busy.current = false;
      }
    },
    [enrichAll, fail, notify, reload],
  );

  // Início: carrega a biblioteca e sincroniza com a Steam, se ela existir.
  useEffect(() => {
    void (async () => {
      await reload();
      try {
        const st = await api.steamStatus();
        setSteam(st);
        if (st.found) await syncSteam(true);
      } catch (e) {
        fail(e);
      }
    })();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Detecta mudanças na Steam ao voltar para o launcher (sem processo em segundo plano).
  useEffect(() => {
    const onFocus = () => {
      if (steam?.found && Date.now() - lastSync.current > FOCUS_SYNC_MIN_INTERVAL_MS) void syncSteam(true);
    };
    window.addEventListener("focus", onFocus);
    return () => window.removeEventListener("focus", onFocus);
  }, [steam, syncSteam]);

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
    steam,
    progress,
    replace,
    syncSteam,
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
