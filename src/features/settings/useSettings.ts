import { useCallback, useEffect, useState } from "react";
import { api, errorMessage } from "../../lib/ipc";
import { setSoundsEnabled } from "../../lib/sound";
import type { Settings } from "../../lib/types";

export function useSettings(notify: (m: string) => void) {
  const [s, setS] = useState<Settings>({});
  const [ready, setReady] = useState(false);

  useEffect(() => {
    api
      .getSettings()
      .then(setS)
      .catch((e) => notify(errorMessage(e)))
      .finally(() => setReady(true));
  }, [notify]);

  // Aplica a personalização em tempo real.
  useEffect(() => {
    const root = document.documentElement;
    root.style.setProperty("--accent", s.accent || "#cbb98d");
    root.style.setProperty("--scrim", String((s.scrim !== undefined ? Number(s.scrim) : 60) / 100));
    root.style.setProperty("--fs-scale", String((s.tv_scale !== undefined ? Number(s.tv_scale) : 100) / 100));
    root.style.setProperty("--safe", `${s.tv_margin !== undefined ? Number(s.tv_margin) : 0}vmin`);
    root.dataset.anim = s.animations === "off" ? "off" : "on";
    document.title = s.app_name || "ALEXANDRIA";
    setSoundsEnabled(s.sounds === "on");
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

  return { s, ready, set, replaceAll: setS, appName: s.app_name || "ALEXANDRIA" };
}
