/** 125 -> "2 h 5 min"; 0 -> "". */
export function fmtPlaytime(minutes: number): string {
  if (!minutes || minutes < 1) return "";
  const h = Math.floor(minutes / 60);
  const m = minutes % 60;
  return h > 0 ? `${h} h${m ? ` ${m} min` : ""}` : `${m} min`;
}

/** 1536000000 -> "1,4 GB". */
export function fmtBytes(n: number): string {
  const units = ["B", "KB", "MB", "GB", "TB"];
  let v = n;
  let i = 0;
  while (v >= 1024 && i < units.length - 1) {
    v /= 1024;
    i++;
  }
  return `${v.toLocaleString("pt-BR", { maximumFractionDigits: i >= 3 ? 1 : i === 0 ? 0 : 1 })} ${units[i]}`;
}

export const fmtSpeed = (bytesPerSec: number): string => (bytesPerSec > 0 ? `${fmtBytes(bytesPerSec)}/s` : "");

/** Segundos -> "42 s", "5 min", "1 h 10 min". */
export function fmtEta(sec: number): string {
  if (!Number.isFinite(sec) || sec <= 0) return "";
  if (sec < 60) return `${Math.ceil(sec)} s`;
  const m = Math.ceil(sec / 60);
  if (m < 60) return `${m} min`;
  return `${Math.floor(m / 60)} h ${m % 60} min`;
}

/** Porcentagem inteira, ou null quando o tamanho total ainda não é conhecido. */
export const pct = (done: number, total: number): number | null =>
  total > 0 ? Math.min(100, Math.floor((done * 100) / total)) : null;
