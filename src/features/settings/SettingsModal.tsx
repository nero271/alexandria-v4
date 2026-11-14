import { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { Icon } from "../../components/Icon";
import { Modal } from "../../components/Modal";
import { api, errorMessage } from "../../lib/ipc";
import { APP_VERSION } from "../../lib/version";
import type { Settings, SteamStatus } from "../../lib/types";
import type { Progress } from "../library/useLibrary";

const SWATCHES = ["#cbb98d", "#9fb7c9", "#c9a0a0", "#a3b899", "#b7a3c9"];

interface Props {
  s: Settings;
  set: (key: string, value: string) => Promise<void>;
  replaceAll: (s: Settings) => void;
  steam: SteamStatus | null;
  progress: Progress | null;
  onSync: () => void;
  onEnrich: () => void;
  onAdd: () => void;
  onClose: () => void;
  notify: (m: string) => void;
}

/** Personalização do ALEXANDRIA, integrações e manutenção da biblioteca. */
export function SettingsModal({ s, set, replaceAll, steam, progress, onSync, onEnrich, onAdd, onClose, notify }: Props) {
  const [name, setName] = useState(s.app_name ?? "");
  const [key, setKey] = useState("");
  const [igdbId, setIgdbId] = useState(s.igdb_id ?? "");
  const [igdbSecret, setIgdbSecret] = useState("");
  const [scrim, setScrim] = useState(s.scrim !== undefined ? Number(s.scrim) : 60);

  const previewScrim = (v: number) => {
    setScrim(v);
    document.documentElement.style.setProperty("--scrim", String(v / 100));
  };

  async function chooseBackground() {
    const p = await open({ multiple: false, filters: [{ name: "Imagens", extensions: ["png", "jpg", "jpeg", "webp"] }] });
    if (typeof p !== "string") return;
    try {
      replaceAll(await api.setAppBackground(p));
    } catch (e) {
      notify(errorMessage(e));
    }
  }

  return (
    <Modal title={<Icon name="sliders" size={24} />} label="Ajustes" onClose={onClose} wide>
      <h3>Aparência</h3>
      <div className="f"><label htmlFor="s-name">Nome exibido do app</label>
        <div className="row">
          <input id="s-name" value={name} maxLength={40} placeholder="ALEXANDRIA" onChange={(e) => setName(e.target.value)} />
          <button className="btn" onClick={() => void set("app_name", name)}>Salvar</button>
        </div></div>
      <div className="f"><label>Cor de destaque</label>
        <div className="row">
          {SWATCHES.map((c) => (
            <button key={c} className="swatch" style={{ background: c }} aria-label={`Cor ${c}`} onClick={() => void set("accent", c)} />
          ))}
          <input key={s.accent || "padrao"} type="color" aria-label="Cor personalizada" defaultValue={s.accent || "#cbb98d"} onBlur={(e) => void set("accent", e.target.value)} />
        </div></div>
      <div className="f"><label htmlFor="s-scrim">Transparência do véu sobre o fundo ({scrim}%)</label>
        <input id="s-scrim" type="range" min={0} max={100} step={5} value={scrim}
          onChange={(e) => previewScrim(Number(e.target.value))}
          onPointerUp={() => void set("scrim", String(scrim))}
          onKeyUp={() => void set("scrim", String(scrim))}
          onBlur={() => void set("scrim", String(scrim))} /></div>
      <div className="f"><label className="check">
        <input type="checkbox" checked={s.animations !== "off"} onChange={(e) => void set("animations", e.target.checked ? "on" : "off")} />
        Animações e transições</label></div>
      <div className="f"><label>Imagem de fundo do app</label>
        <div className="row">
          <button className="btn" onClick={() => void chooseBackground()}>Escolher imagem…</button>
          {s.app_bg && <button className="btn" onClick={() => void set("app_bg", "")}>Remover</button>}
        </div></div>
      <p className="muted">O ícone do programa é definido na instalação e não muda com o app aberto.</p>

      <h3>SteamGridDB</h3>
      <p className="muted">Gere uma chave grátis em steamgriddb.com (Preferências, API). Ela fica guardada só neste computador.</p>
      <div className="f">
        <div className="row">
          <input type="password" aria-label="Chave do SteamGridDB" autoComplete="off"
            placeholder={s.sgdb_key_set ? "Chave salva (digite para trocar)" : "Cole a chave aqui"}
            value={key} onChange={(e) => setKey(e.target.value)} />
          <button className="btn" disabled={!key.trim()} onClick={async () => { await set("sgdb_key", key.trim()); setKey(""); }}>Salvar</button>
          {s.sgdb_key_set && <button className="btn" onClick={() => void set("sgdb_key", "")}>Remover</button>}
        </div></div>

      <h3>IGDB (descrições)</h3>
      <p className="muted">Crie um aplicativo grátis em dev.twitch.tv e cole abaixo o Client ID e o Client Secret. Ficam guardados só neste computador.</p>
      <div className="f"><label htmlFor="s-igid">Client ID</label>
        <input id="s-igid" value={igdbId} autoComplete="off" onChange={(e) => setIgdbId(e.target.value)} /></div>
      <div className="f"><label htmlFor="s-igsec">Client Secret</label>
        <div className="row">
          <input id="s-igsec" type="password" autoComplete="off" value={igdbSecret}
            placeholder={s.igdb_secret_set ? "Salvo (digite para trocar)" : "Cole o segredo aqui"}
            onChange={(e) => setIgdbSecret(e.target.value)} />
          <button className="btn" disabled={!igdbId.trim() && !igdbSecret.trim()}
            onClick={async () => { await set("igdb_id", igdbId.trim()); if (igdbSecret.trim()) await set("igdb_secret", igdbSecret.trim()); setIgdbSecret(""); notify("Dados do IGDB salvos."); }}>Salvar</button>
          {s.igdb_secret_set && <button className="btn" onClick={async () => { await set("igdb_id", ""); await set("igdb_secret", ""); setIgdbId(""); }}>Remover</button>}
        </div></div>

      <h3>Biblioteca</h3>
      <div className="row"><button className="btn" onClick={onAdd}>Adicionar jogo manualmente</button></div>

      <h3>Steam</h3>
      <p className="muted">
        {steam?.found ? `Steam encontrada em ${steam.path}.` : "A Steam não foi encontrada neste computador."}
      </p>
      <div className="row">
        <button className="btn" disabled={!steam?.found} onClick={onSync}>Sincronizar agora</button>
        <button className="btn" disabled={!steam?.found || !!progress} onClick={onEnrich}>
          {progress ? `Baixando… ${progress.done}/${progress.total}` : "Baixar capas e informações"}
        </button>
      </div>

      <p className="muted">ALEXANDRIA v{APP_VERSION}</p>
      <div className="acts"><button className="btn primary" onClick={onClose}>Concluir</button></div>
    </Modal>
  );
}
