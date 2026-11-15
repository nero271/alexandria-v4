import type { Action } from "./bus";

export type PadKind = "xbox" | "ps";

const XBOX: Partial<Record<Action, string>> = {
  ok: "A", back: "B", x: "X", y: "Y", lb: "LB", rb: "RB", lt: "LT", rt: "RT", menu: "Menu", mode: "View",
};
const PS: Partial<Record<Action, string>> = {
  ok: "✕", back: "○", x: "□", y: "△", lb: "L1", rb: "R1", lt: "L2", rt: "R2", menu: "Options", mode: "Share",
};
const KEYBOARD: Partial<Record<Action, string>> = {
  ok: "Enter", back: "Esc", x: "Espaço", y: "N", lb: "[", rb: "]", lt: "PgUp", rt: "PgDn", menu: "S", mode: "M",
};

export const glyph = (a: Action, pad: PadKind | null) =>
  ((pad === "ps" ? PS : pad === "xbox" ? XBOX : KEYBOARD)[a] ?? "");
