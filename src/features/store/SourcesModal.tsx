import { useState } from "react";
import { Icon } from "../../components/Icon";
import { Modal } from "../../components/Modal";
import type { Store } from "./useStore";

function hostOf(url: string): string | null {
  try {
    const u = new URL(url.trim());
    return u.protocol === "https:" ? u.host : null;
  } catch {
    return null;
  }
}

/** Gerenciar as fontes (catálogos) da Loja. Sempre pede confirmação antes de adicionar. */
export function SourcesModal({ store, notify, onClose }: { store: Store; notify: (m: string) => void; onClose: () => void }) {
  const [url, setUrl] = useState("");
  const [confirming, setConfirming] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  function ask() {
    const host = hostOf(url);
    if (!host) return notify("Use o endereço completo do catálogo, começando com https://");
    setConfirming(host);
  }

  async function add() {
    setBusy(true);
    const ok = await store.addSource(url.trim());
    setBusy(false);
    if (ok) {
      setUrl("");
      setConfirming(null);
    }
  }

  return (
    <Modal title={<Icon name="store" size={24} />} label="Fontes da Loja" onClose={onClose} wide>
      <p className="muted">
        Uma fonte é o endereço de um catálogo (arquivo JSON). O ALEXANDRIA não traz nem indica fontes. Cada jogo só é instalado
        se o arquivo baixado bater com o SHA-256 do catálogo. Use apenas fontes de conteúdo que você tem o direito de baixar e usar.
      </p>

      {store.sources.length > 0 && (
        <ul className="emu-list">
          {store.sources.map((s) => (
            <li key={s.id}>
              <span className="name">{s.name} <span className="muted">· {s.gameCount} jogo(s)</span></span>
              <span className="row">
                <button className="round" title="Atualizar catálogo" aria-label="Atualizar catálogo" onClick={() => void store.refreshSource(s.id)}>
                  <Icon name="refresh" size={14} />
                </button>
                <button className="round" title="Remover fonte" aria-label="Remover fonte" onClick={() => void store.removeSource(s.id)}>
                  <Icon name="close" size={14} />
                </button>
              </span>
            </li>
          ))}
        </ul>
      )}

      {confirming ? (
        <>
          <p>
            Adicionar a fonte de <b>{confirming}</b>? O ALEXANDRIA vai baixar e executar programas deste endereço. Só continue se
            você confia nele e tem direito de usar o conteúdo.
          </p>
          <div className="acts">
            <button className="btn" onClick={() => setConfirming(null)}>Cancelar</button>
            <button className="btn primary" disabled={busy} onClick={() => void add()}>{busy ? "Adicionando…" : "Adicionar mesmo assim"}</button>
          </div>
        </>
      ) : (
        <>
          <div className="f"><label htmlFor="src-url">Endereço do catálogo</label>
            <div className="row">
              <input id="src-url" value={url} inputMode="url" autoComplete="off" placeholder="https://exemplo.com/catalogo.json" onChange={(e) => setUrl(e.target.value)} />
              <button className="btn" disabled={!url.trim()} onClick={ask}>Adicionar</button>
            </div></div>
          <div className="acts"><button className="btn primary" onClick={onClose}>Concluir</button></div>
        </>
      )}
    </Modal>
  );
}
