import type { Action } from "../input/bus";

/** Sons curtos gerados na hora (sem arquivos). Desligados por padrão. */
let enabled = false;
let ctx: AudioContext | null = null;

export const setSoundsEnabled = (v: boolean) => {
  enabled = v;
};

function tone(freq: number, dur: number, gain: number) {
  if (!enabled) return;
  try {
    ctx ??= new AudioContext();
    if (ctx.state === "suspended") void ctx.resume();
    const osc = ctx.createOscillator();
    const g = ctx.createGain();
    const t = ctx.currentTime;
    osc.type = "sine";
    osc.frequency.value = freq;
    g.gain.setValueAtTime(0.0001, t);
    g.gain.exponentialRampToValueAtTime(gain, t + 0.008);
    g.gain.exponentialRampToValueAtTime(0.0001, t + dur);
    osc.connect(g).connect(ctx.destination);
    osc.start(t);
    osc.stop(t + dur + 0.02);
  } catch {
    /* sem áudio disponível: segue em silêncio */
  }
}

export function playFor(a: Action) {
  if (!enabled) return;
  if (a === "ok") tone(990, 0.12, 0.045);
  else if (a === "back") tone(440, 0.1, 0.04);
  else if (a === "x" || a === "y" || a === "menu" || a === "mode") tone(740, 0.08, 0.04);
  else tone(880, 0.045, 0.028);
}
