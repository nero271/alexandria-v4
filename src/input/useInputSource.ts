import { useEffect, useState } from "react";
import { dispatch, type Action } from "./bus";
import type { PadKind } from "./glyphs";

const KEY_MAP: Record<string, Action> = {
  ArrowLeft: "left", ArrowRight: "right", ArrowUp: "up", ArrowDown: "down",
  Enter: "ok", Escape: "back", " ": "x", n: "y", N: "y",
  "[": "lb", "]": "rb", PageUp: "lt", PageDown: "rt", s: "menu", S: "menu", m: "mode", M: "mode",
};

/** Layout padrão (Xbox e PlayStation seguem o mesmo mapeamento da Gamepad API). */
const BUTTONS: [number, Action][] = [
  [0, "ok"], [1, "back"], [2, "x"], [3, "y"], [4, "lb"], [5, "rb"], [6, "lt"], [7, "rt"],
  [8, "mode"], [9, "menu"], [12, "up"], [13, "down"], [14, "left"], [15, "right"],
];
const REPEATABLE = new Set<Action>(["left", "right", "up", "down", "lt", "rt"]);
const AXIS = 0.55;

const padKind = (id: string): PadKind =>
  /054c|dualsense|dualshock|playstation|wireless controller/i.test(id) && !/xbox|045e/i.test(id)
    ? "ps"
    : "xbox";

const currentPad = (): Gamepad | null =>
  Array.from(navigator.getGamepads?.() ?? []).find((p): p is Gamepad => !!p && p.connected) ?? null;

/** Teclado + controles. O laço de leitura só roda com controle conectado e a janela em foco. */
export function useInputSource(onPadChange?: (connected: boolean, kind: PadKind | null) => void) {
  const [pad, setPad] = useState<PadKind | null>(null);

  useEffect(() => {
    let raf = 0;
    const held = new Map<Action, boolean>();
    const nextRepeat = new Map<Action, number>();

    const onKey = (e: KeyboardEvent) => {
      const el = e.target as HTMLElement | null;
      if (el?.closest("input, textarea, select")) return;
      if ((e.key === "Enter" || e.key === " ") && el?.closest("button, a")) return;
      if (e.ctrlKey || e.metaKey || e.altKey) return;
      const a = KEY_MAP[e.key];
      if (a) {
        e.preventDefault();
        dispatch(a);
      }
    };

    const onClick = (e: MouseEvent) => {
      (e.target as HTMLElement | null)?.closest("button")?.blur();
    };

    const loop = (t: number) => {
      const p = currentPad();
      if (p) {
        const kind = padKind(p.id);
        setPad((k) => (k === kind ? k : kind));
        const on = new Map<Action, boolean>();
        for (const [i, a] of BUTTONS) if (p.buttons[i]?.pressed) on.set(a, true);
        if ((p.axes[0] ?? 0) < -AXIS) on.set("left", true);
        if ((p.axes[0] ?? 0) > AXIS) on.set("right", true);
        if ((p.axes[1] ?? 0) < -AXIS) on.set("up", true);
        if ((p.axes[1] ?? 0) > AXIS) on.set("down", true);
        for (const a of ["left", "right", "up", "down", "ok", "back", "x", "y", "lb", "rb", "lt", "rt", "menu", "mode"] as Action[]) {
          if (on.get(a)) {
            if (!held.get(a)) {
              held.set(a, true);
              nextRepeat.set(a, t + 330);
              dispatch(a);
            } else if (REPEATABLE.has(a) && t > (nextRepeat.get(a) ?? 0)) {
              nextRepeat.set(a, t + 110);
              dispatch(a);
            }
          } else {
            held.set(a, false);
          }
        }
      }
      raf = requestAnimationFrame(loop);
    };

    const start = () => {
      if (!raf && currentPad() && document.hasFocus()) raf = requestAnimationFrame(loop);
    };
    const stop = () => {
      cancelAnimationFrame(raf);
      raf = 0;
    };
    const onConnect = () => {
      const p = currentPad();
      if (p) {
        setPad(padKind(p.id));
        onPadChange?.(true, padKind(p.id));
      }
      start();
    };
    const onDisconnect = () => {
      if (!currentPad()) {
        stop();
        setPad(null);
        onPadChange?.(false, null);
      }
    };

    window.addEventListener("keydown", onKey);
    document.addEventListener("click", onClick);
    window.addEventListener("gamepadconnected", onConnect);
    window.addEventListener("gamepaddisconnected", onDisconnect);
    window.addEventListener("focus", start);
    window.addEventListener("blur", stop); // com o jogo em primeiro plano, o launcher não lê nada
    start();
    return () => {
      stop();
      window.removeEventListener("keydown", onKey);
      document.removeEventListener("click", onClick);
      window.removeEventListener("gamepadconnected", onConnect);
      window.removeEventListener("gamepaddisconnected", onDisconnect);
      window.removeEventListener("focus", start);
      window.removeEventListener("blur", stop);
    };
  }, [onPadChange]);

  return pad;
}
