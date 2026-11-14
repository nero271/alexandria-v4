import { useCallback, useEffect, useState } from "react";
import { api, errorMessage } from "../../lib/ipc";
import type { Settings } from "../../lib/types";

export function useSettings(notify: (m: string) => void) {
  const [s, setS] = useState<Settings>({});

  useEffect(() => {
    api.getSettings().then(setS).catch((e) => notify(errorMessage(e)));
  }, [notify]);

  // Aplica a personalização visual em tempo real.
  useEffect(() => {
    const root = document.documentElement;
    root.style.setProperty("--accent", s.accent || "#cbb98d");
    root.style.setProperty("--scrim", String((s.scrim !== undefined ? Number(s.scrim) : 60) / 100));
    root.dataset.anim = s.animations === "off" ? "off" : "on";
    document.title = s.app_name || "ALEXANDRIA";
  }, [s]);

  const set = useCallback(
    async (key: string, value: string) => {
      try {
        setS(await api.setSetting(key, value));
      } catch (e) {
        notify(errorMessage(e));
      }
    },
    [notify],
  );

  return {
    s,
    set,
    replaceAll: setS,
    appName: s.app_name || "ALEXANDRIA",
    mode: (s.mode === "fullscreen" ? "fullscreen" : "desktop") as "desktop" | "fullscreen",
  };
}
