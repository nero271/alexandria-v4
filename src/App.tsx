import { useCallback, useEffect, useMemo, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { asset } from "./lib/assets";
import type { Game } from "./lib/types";
import { useActionHandler } from "./input/bus";
import { useInputSource } from "./input/useInputSource";
import { useBrowse } from "./features/browse/useBrowse";
import { DynamicBackground } from "./features/dynamic-bg/DynamicBackground";
import { DesktopShell } from "./features/library/DesktopShell";
import { DetailsPanel } from "./features/library/DetailsPanel";
import { EditGameModal } from "./features/library/EditGameModal";
import { FullscreenShell } from "./features/library/FullscreenShell";
import { useLibrary } from "./features/library/useLibrary";
import { SettingsModal } from "./features/settings/SettingsModal";
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
  const pad = useInputSource();
  const mode = cfg.mode;

  const [detailsId, setDetailsId] = useState<number | null>(null);
  const [editing, setEditing] = useState<Game | "new" | null>(null);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const details = lib.games.find((g) => g.id === detailsId) ?? null;

  // Fullscreen de verdade: a janela ocupa a tela inteira, sem barras do Windows.
  useEffect(() => {
    document.body.dataset.mode = mode;
    getCurrentWindow().setFullscreen(mode === "fullscreen").catch(() => {});
  }, [mode]);

  const toggleMode = () => void cfg.set("mode", mode === "fullscreen" ? "desktop" : "fullscreen");

  async function launch(g: Game) {
    if (await lib.launchGame(g)) {
      notify(`Iniciando ${g.name}…`);
      setDetailsId(null);
      getCurrentWindow().minimize().catch(() => {}); // libera recursos para o jogo
    }
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
          progress={lib.progress}
          onAdd={() => setEditing("new")}
          onSettings={() => setSettingsOpen(true)}
          onMode={toggleMode}
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
          steam={lib.steam}
          progress={lib.progress}
          onSync={() => void lib.syncSteam()}
          onEnrich={() => void lib.enrichAll()}
          onAdd={() => {
            setSettingsOpen(false);
            setEditing("new");
          }}
          onClose={() => setSettingsOpen(false)}
          notify={notify}
        />
      )}

      {toast && <div className="toast" role="status">{toast}</div>}
    </>
  );
}
