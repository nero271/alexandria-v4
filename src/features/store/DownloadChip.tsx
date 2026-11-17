import { Icon } from "../../components/Icon";
import type { DownloadSummary } from "./useStore";

/** Indicador de download em andamento (aparece em qualquer aba). */
export function DownloadChip({ d, onClick }: { d: DownloadSummary; onClick?: () => void }) {
  return (
    <button className="chip" onClick={onClick} title="Downloads" aria-label="Downloads em andamento">
      <Icon name="download" size={14} /> {d.percent !== null ? `${d.percent}%` : "…"}
      {d.count > 1 ? ` · ${d.count}` : ""}
    </button>
  );
}
