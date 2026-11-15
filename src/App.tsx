import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { asset } from "./lib/assets";
import { playIntro } from "./lib/sound";
import { FullscreenIntro } from "./components/FullscreenIntro";
import type { Game } from "./lib/types";
import { useActionHandler } from "./input/bus";
import { useInputSource } from "./input/useInputSource";
import { useBrowse } from "./features/browse/useBrowse";
import { DynamicBackground } from "./features/dynamic-bg/DynamicBackground";
import { DesktopShell } from "./features/library/DesktopShell";
import { DetailsPanel } from "./features/library/DetailsPanel";
import { EditGameModal } from "./features/library/EditGameModal";
import { FullscreenShell } from "./features/library/FullscreenShell";
import { LaunchOverlay } from "./features/library/LaunchOverlay";
import { useLibrary } from "./features/library/useLibrary";
import { SettingsModal } from "./features/settings/SettingsModal";
import { YearModal } from "./features/stats/YearModal";
import { useSettings } from "./features/settings/useSettings";

const TOAST_MS = 3800;

function gridColumns(): number {
  const g = document.querySelector(".grid");
  if (!g) return 1;
  return getComputedStyle(g).gridTemplateColumns.split(" ").length;
}

export default function App() {
  const [toast, setToast] = useState<string | null>(null);
  const notify = useCallback((m: string) => setToast(m), []);
  useEffect(() => {
    if (!toast) return;
    const t = window.setTimeout(() => setToast(null), TOAST_MS);
    return () => window.clearTimeout(t);
  }, [toast]);

  const cfg = useSettings(notify);
  const lib = useLibrary(notify);
  const browse = useBrowse(lib.games);

  // Último jogo jogado: vira o destaque "Continuar" e a seleção inicial.
  const lastPlayed = useMemo(
    () => lib.games.filter((g) => g.lastPlayedAt).sort((x, y) => (y.lastPlayedAt ?? 0) - (x.lastPlayedAt ?? 0))[0] ?? null,
    [lib.games],
  );
  const pickedInitial = useRef(false);
  useEffect(() => {
    if (pickedInitial.current || lib.loading || lib.games.length === 0) return;
    pickedInitial.current = true;
    if (lastPlayed) browse.select(lastPlayed.id);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [lib.loading, lib.games.length, lastPlayed]);
  const showContinue = browse.tab === "all" && !browse.query.trim() && !browse.category && !browse.status;
  const pad = useInputSource();
  // Tela cheia ao abrir é uma opção (desligada por padrão); sem a opção, sempre abre no desktop.
  const [mode, setMode] = useState<"desktop" | "fullscreen">("desktop");
  const started = useRef(false);
  useEffect(() => {
    if (!cfg.ready || started.current) return;
    started.current = true;
    if (cfg.s.start_fullscreen === "on") setMode("fullscreen");
  }, [cfg.ready, cfg.s.start_fullscreen]);

  const [detailsId, setDetailsId] = useState<number | null>(null);
  const [editing, setEditing] = useState<Game | "new" | null>(null);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [yearOpen, setYearOpen] = useState(false);
  const details = lib.games.find((g) => g.id === detailsId) ?? null;

  // Fullscreen de verdade: a janela ocupa a tela inteira, sem barras do Windows.
  useEffect(() => {
    document.body.dataset.mode = mode;
    getCurrentWindow().setFullscreen(mode === "fullscreen").catch(() => {});
  }, [mode]);

  // Ao entrar na tela cheia: animação (se as animações estiverem ligadas) e som de abertura.
  const [intro, setIntro] = useState(false);
  useEffect(() => {
    if (mode !== "fullscreen") {
      setIntro(false);
      return;
    }
    const reduce = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    if (cfg.s.animations !== "off" && !reduce) setIntro(true);
    if (cfg.s.intro_sound !== "off") playIntro();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [mode]);

  const toggleMode = () => setMode((m) => (m === "fullscreen" ? "desktop" : "fullscreen"));

  // --- Iniciar jogo: carregamento -> minimiza quando abrir -> volta quando fechar ---
  const [launching, setLaunching] = useState<Game | null>(null);
  const launchingRef = useRef<Game | null>(null);
  const trackedId = useRef<number | null>(null);
  const playingId = useRef<number | null>(null);
  const launchTimer = useRef<number | undefined>(undefined);
  const autoMin = useRef(true);
  autoMin.current = cfg.s.auto_minimize !== "off";

  const finishLaunch = useCallback(() => {
    window.clearTimeout(launchTimer.current);
    launchingRef.current = null;
    setLaunching(null);
  }, []);
  const minimizeApp = () => {
    if (autoMin.current) getCurrentWindow().minimize().catch(() => {});
  };
  const restoreApp = async () => {
    const w = getCurrentWindow();
    try {
      await w.unminimize();
      await w.show();
      await w.setFocus();
    } catch {
      /* sem janela para restaurar */
    }
  };

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let cancelled = false;
    void listen<{ id: number; state: string }>("game-state", (e) => {
      const { id, state } = e.payload;
      const cur = launchingRef.current;
      if (state === "tracked") trackedId.current = id;
      else if (state === "running" && cur?.id === id) {
        finishLaunch();
        playingId.current = id;
        minimizeApp();
      } else if (state === "exited") {
        const was = playingId.current === id || cur?.id === id || trackedId.current === id;
        if (was) {
          finishLaunch();
          playingId.current = null;
          trackedId.current = null;
          void restoreApp();
          void lib.reload();
        }
      } else if (state === "timeout" && cur?.id === id) {
        finishLaunch();
        notify("O jogo demorou para abrir. Ele pode ainda estar carregando.");
      }
    }).then((f) => {
      if (cancelled) f();
      else unlisten = f;
    });
    return () => {
      cancelled = true;
      unlisten?.();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Jogos fora da Steam: a janela do jogo tomando o foco indica que ele abriu.
  useEffect(() => {
    if (!launching || launching.platform === "steam") return;
    const onBlur = () => {
      const g = launchingRef.current;
      if (!g) return;
      finishLaunch();
      if (trackedId.current === g.id) {
        playingId.current = g.id;
        minimizeApp();
      }
    };
    window.addEventListener("blur", onBlur);
    return () => window.removeEventListener("blur", onBlur);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [launching, finishLaunch]);

  async function launch(g: Game) {
    trackedId.current = null;
    playingId.current = null;
    launchingRef.current = g;
    setLaunching(g);
    setDetailsId(null);
    window.clearTimeout(launchTimer.current);
    launchTimer.current = window.setTimeout(finishLaunch, 120_000);
    if (!(await lib.launchGame(g))) finishLaunch();
  }

  // Navegação da biblioteca (o componente aberto por último recebe os comandos antes deste).
  useActionHandler((a) => {
    const { selected, index } = browse;
    if (a === "menu") setSettingsOpen(true);
    else if (a === "mode") toggleMode();
    else if (a === "left") browse.selectIndex(index - 1);
    else if (a === "right") browse.selectIndex(index + 1);
    else if (a === "up" || a === "down") {
      const c = mode === "desktop" ? gridColumns() : 1;
      if (c > 1) browse.selectIndex(index + (a === "up" ? -c : c));
    } else if (a === "ok" && selected) {
      if (mode === "fullscreen") void launch(selected); // estilo console: confirmar já joga
      else setDetailsId(selected.id);
    } else if (a === "x" && selected) void lib.toggleFavorite(selected);
    else if (a === "y") {
      if (mode === "fullscreen") {
        if (selected) setDetailsId(selected.id);
      } else setEditing("new");
    } else if (a === "lt") browse.selectIndex(index - 6);
    else if (a === "rt") browse.selectIndex(index + 6);
    else if (a === "lb") browse.cycleTab(-1);
    else if (a === "rb") browse.cycleTab(1);
  });

  const neighbors = useMemo(
    () => browse.visible.slice(Math.max(0, browse.index - 2), browse.index + 3),
    [browse.visible, browse.index],
  );

  return (
    <>
      <DynamicBackground game={browse.selected} neighbors={neighbors} fallback={asset(cfg.s.app_bg)} />

      {mode === "fullscreen" ? (
        <FullscreenShell
          games={browse.visible}
          hasAny={lib.games.length > 0}
          selected={browse.selected}
          onSelect={browse.select}
          onOpen={(id) => {
            const g = lib.games.find((x) => x.id === id);
            if (g) void launch(g);
          }}
          onPlay={(g) => void launch(g)}
          tab={browse.tab}
          pad={pad}
        />
      ) : (
        <DesktopShell
          appName={cfg.appName}
          games={browse.visible}
          hasAny={lib.games.length > 0}
          selectedId={browse.selected?.id ?? null}
          onSelect={browse.select}
          onOpen={setDetailsId}
          tab={browse.tab}
          onTab={browse.setTab}
          category={browse.category}
          categories={browse.categories}
          onCategory={browse.setCategory}
          query={browse.query}
          onQuery={browse.setQuery}
          statusFilter={browse.status}
          onStatus={browse.setStatus}
          continueGame={showContinue ? lastPlayed : null}
          onPlay={(g) => void launch(g)}
          progress={lib.progress}
          onAdd={() => setEditing("new")}
          onSettings={() => setSettingsOpen(true)}
          onMode={toggleMode}
          onYear={() => setYearOpen(true)}
        />
      )}

      {details && (
        <DetailsPanel
          game={details}
          pad={pad}
          onClose={() => setDetailsId(null)}
          onLaunch={launch}
          onFavorite={(g) => void lib.toggleFavorite(g)}
          onEdit={(g) => {
            setDetailsId(null);
            setEditing(g);
          }}
        />
      )}

      {editing && (
        <EditGameModal
          target={editing}
          onClose={() => setEditing(null)}
          onCreate={async (n) => {
            const g = await lib.addGame(n);
            if (g) browse.select(g.id);
            return g;
          }}
          onUpdate={lib.updateGame}
          onStatus={lib.setStatus}
          onRating={lib.setRating}
          onRemove={lib.removeGame}
          onChanged={lib.replace}
          notify={notify}
        />
      )}

      {settingsOpen && (
        <SettingsModal
          s={cfg.s}
          set={cfg.set}
          replaceAll={cfg.replaceAll}
          platforms={lib.platforms}
          reload={lib.reload}
          progress={lib.progress}
          onSync={() => void lib.syncAll()}
          onEnrich={() => void lib.enrichAll()}
          onAdd={() => {
            setSettingsOpen(false);
            setEditing("new");
          }}
          onYear={() => {
            setSettingsOpen(false);
            setYearOpen(true);
          }}
          onClose={() => setSettingsOpen(false)}
          notify={notify}
        />
      )}

      {yearOpen && <YearModal games={lib.games} onClose={() => setYearOpen(false)} />}

      {launching && <LaunchOverlay game={launching} onDismiss={finishLaunch} />}

      {intro && <FullscreenIntro onDone={() => setIntro(false)} />}

      {toast && <div className="toast" role="status">{toast}</div>}
    </>
  );
}
