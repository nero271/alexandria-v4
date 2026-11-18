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

/** Abertura da tela cheia: um acorde etéreo que "abre" (filtro subindo) e termina com cintilações.
 *  Composição original sintetizada na hora; não usa nenhum áudio de terceiros. */
export function playIntro() {
  try {
    ctx ??= new AudioContext();
    if (ctx.state === "suspended") void ctx.resume();
    const c = ctx;
    const t = c.currentTime;
    const master = c.createGain();
    master.gain.setValueAtTime(0.0001, t);
    master.gain.exponentialRampToValueAtTime(0.16, t + 0.9);
    master.gain.setValueAtTime(0.16, t + 1.5);
    master.gain.exponentialRampToValueAtTime(0.0001, t + 3.2);
    master.connect(c.destination);

    const lp = c.createBiquadFilter();
    lp.type = "lowpass";
    lp.Q.value = 0.7;
    lp.frequency.setValueAtTime(260, t);
    lp.frequency.exponentialRampToValueAtTime(4200, t + 1.4);
    lp.connect(master);

    // Ré maior com sétima maior; cada nota com dois osciladores levemente desafinados.
    for (const f of [73.42, 146.83, 220, 293.66, 369.99, 554.37]) {
      for (const d of [-6, 6]) {
        const o = c.createOscillator();
        const g = c.createGain();
        o.type = "sawtooth";
        o.frequency.value = f;
        o.detune.value = d;
        g.gain.value = f < 150 ? 0.5 : 0.22;
        o.connect(g).connect(lp);
        o.start(t);
        o.stop(t + 3.3);
      }
    }
    // Cintilações no final.
    for (const [f, at] of [[1318.5, 1.35], [1760, 1.55], [2217.5, 1.75]] as const) {
      const o = c.createOscillator();
      const g = c.createGain();
      o.type = "sine";
      o.frequency.value = f;
      g.gain.setValueAtTime(0.0001, t + at);
      g.gain.exponentialRampToValueAtTime(0.05, t + at + 0.02);
      g.gain.exponentialRampToValueAtTime(0.0001, t + at + 1.1);
      o.connect(g).connect(master);
      o.start(t + at);
      o.stop(t + at + 1.2);
    }
  } catch {
    /* sem áudio disponível: a animação segue em silêncio */
  }
}
