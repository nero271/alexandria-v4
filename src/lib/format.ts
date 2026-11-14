/** 125 -> "2 h 5 min"; 0 -> "". */
export function fmtPlaytime(minutes: number): string {
  if (!minutes || minutes < 1) return "";
  const h = Math.floor(minutes / 60);
  const m = minutes % 60;
  return h > 0 ? `${h} h${m ? ` ${m} min` : ""}` : `${m} min`;
}
