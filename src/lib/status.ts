import type { IconName } from "../components/Icon";
import type { GameStatus } from "./types";

export const STATUSES: { id: GameStatus; label: string; icon: IconName }[] = [
  { id: "playing", label: "Jogando", icon: "play" },
  { id: "completed", label: "Terminado", icon: "check" },
  { id: "backlog", label: "Na fila", icon: "list" },
  { id: "dropped", label: "Abandonado", icon: "ban" },
];

export const statusMeta = (s: GameStatus | null) => STATUSES.find((x) => x.id === s) ?? null;

export const stars = (n: number | null) => (n ? "★".repeat(n) + "☆".repeat(5 - n) : "");

/** "há 3 dias", "há 2 h"... a partir de um instante em segundos Unix. */
export function timeAgo(seconds: number): string {
  const diff = Math.max(0, Date.now() / 1000 - seconds);
  const min = Math.floor(diff / 60);
  if (min < 1) return "agora há pouco";
  if (min < 60) return `há ${min} min`;
  const h = Math.floor(min / 60);
  if (h < 24) return `há ${h} h`;
  const d = Math.floor(h / 24);
  return d === 1 ? "ontem" : `há ${d} dias`;
}
