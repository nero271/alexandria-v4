import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useMemo, useState } from "react";
import { api, errorMessage } from "../../lib/ipc";
import { pct } from "../../lib/format";
import type { InstallProgress, SourceGame, SourceInfo } from "../../lib/types";

export interface DownloadSummary {
  count: number;
  percent: number | null;
}

export const keyOf = (g: SourceGame) => `${g.sourceId}:${g.id}`;

/** Catálogo das fontes + andamento dos downloads (sobrevive à troca de aba). */
export function useStore(notify: (m: string) => void, afterChange: () => Promise<void> | void) {
  const [games, setGames] = useState<SourceGame[]>([]);
  const [sources, setSources] = useState<SourceInfo[]>([]);
  const [progress, setProgress] = useState<Record<string, InstallProgress>>({});
  const [loaded, setLoaded] = useState(false);

  const load = useCallback(async () => {
    try {
      const [g, s] = await Promise.all([api.sourceGames(), api.sourceList()]);
      setGames(g);
      setSources(s);
    } catch (e) {
      notify(errorMessage(e));
    } finally {
      setLoaded(true);
    }
  }, [notify]);

  useEffect(() => {
    void load();
  }, [load]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let cancelled = false;
    void listen<InstallProgress>("install-progress", (e) => {
      setProgress((prev) => ({ ...prev, [e.payload.key]: e.payload }));
    }).then((f) => {
      if (cancelled) f();
      else unlisten = f;
    });
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  const drop = (key: string) =>
    setProgress((p) => {
      const n = { ...p };
      delete n[key];
      return n;
    });

  const install = useCallback(
    async (g: SourceGame) => {
      const key = keyOf(g);
      setProgress((p) => ({ ...p, [key]: { key, phase: "start", done: p[key]?.done ?? 0, total: p[key]?.total ?? 0, speed: 0 } }));
      try {
        await api.installGame(g.sourceId, g.id);
        drop(key);
        notify(`${g.name} instalado.`);
        await load();
        await afterChange();
      } catch (e) {
        const msg = errorMessage(e);
        if (msg.startsWith("Download pausado")) {
          setProgress((p) => ({ ...p, [key]: { ...(p[key] ?? { key, done: 0, total: 0, speed: 0 }), phase: "paused", speed: 0 } }));
        } else {
          drop(key);
          notify(msg);
        }
      }
    },
    [afterChange, load, notify],
  );

  const pause = useCallback(
    async (g: SourceGame) => {
      try {
        await api.pauseInstall(g.sourceId, g.id);
      } catch (e) {
        notify(errorMessage(e));
      }
    },
    [notify],
  );

  const uninstall = useCallback(
    async (g: SourceGame) => {
      try {
        await api.uninstallGame(g.sourceId, g.id);
        notify(`${g.name} foi removido.`);
        await load();
        await afterChange();
      } catch (e) {
        notify(errorMessage(e));
      }
    },
    [afterChange, load, notify],
  );

  const addSource = useCallback(
    async (url: string): Promise<boolean> => {
      try {
        const s = await api.sourceAdd(url);
        notify(`Fonte "${s.name}" adicionada com ${s.gameCount} jogo(s).`);
        await load();
        return true;
      } catch (e) {
        notify(errorMessage(e));
        return false;
      }
    },
    [load, notify],
  );

  const refreshSource = useCallback(
    async (id: number) => {
      try {
        const s = await api.sourceRefresh(id);
        notify(`"${s.name}" atualizada: ${s.gameCount} jogo(s).`);
        await load();
      } catch (e) {
        notify(errorMessage(e));
      }
    },
    [load, notify],
  );

  const removeSource = useCallback(
    async (id: number) => {
      try {
        await api.sourceRemove(id);
        await load();
      } catch (e) {
        notify(errorMessage(e));
      }
    },
    [load, notify],
  );

  // Resumo para o indicador no topo: só downloads em andamento (pausados não contam).
  const summary = useMemo<DownloadSummary | null>(() => {
    const active = Object.values(progress).filter((p) => p.phase !== "paused" && p.phase !== "done");
    if (active.length === 0) return null;
    const first = active[0];
    return { count: active.length, percent: pct(first.done, first.total) };
  }, [progress]);

  return { games, sources, progress, loaded, summary, install, pause, uninstall, addSource, refreshSource, removeSource };
}

export type Store = ReturnType<typeof useStore>;
