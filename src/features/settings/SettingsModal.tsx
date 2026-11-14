import { useEffect, useState } from "react";
import { open, save } from "@tauri-apps/plugin-dialog";
import { disable, enable, isEnabled } from "@tauri-apps/plugin-autostart";
import { Icon } from "../../components/Icon";
import { Modal } from "../../components/Modal";
import { api, errorMessage } from "../../lib/ipc";
import { APP_VERSION } from "../../lib/version";
import type { PlatformStatus, Settings } from "../../lib/types";
import type { Progress } from "../library/useLibrary";

const SWATCHES = ["#cbb98d", "#9fb7c9", "#c9a0a0", "#a3b899", "#b7a3c9"];
const JSON_FILTER = [{ name: "Backup do ALEXANDRIA", extensions: ["json"] }];

interface Props {
  s: Settings;
  set: (key: string, value: string) => Promise<void>;
  replaceAll: (s: Settings) => void;
  platforms: PlatformStatus | null;
  progress: Progress | null;
  onSync: () => void;
  onEnrich: () => void;
  onAdd: () => void;
  reload: () => Promise<void>;
  onClose: () => void;
  notify: (m: string) => void;
}

function Toggle({ label, checked, onChange }: { label: string; checked: boolean; onChange: (v: boolean) => void }) {
  return (
    <div className="f">
      <label className="check">
        <input type="checkbox" checked={checked} onChange={(e) => onChange(e.target.checked)} />
        {label}
      </label>
    </div>
  );
}

/** Personalização, comportamento, integrações, lojas, backup, desempenho e atualização. */
export function SettingsModal({ s, set, replaceAll, platforms, progress, onSync, onEnrich, onAdd, reload, onClose, notify }: Props) {
  const [name, setName] = useState(s.app_name ?? "");
  const [key, setKey] = useState("");
  const [igdbId, setIgdbId] = useState(s.igdb_id ?? "");
  const [igdbSecret, setIgdbSecret] = useState("");
  const [scrim, setScrim] = useState(s.scrim !== undefined ? Number(s.scrim) : 60);
  const [auto, setAuto] = useState(false);
  const [diag, setDiag] = useState<string | null>(null);
  const [upd, setUpd] = useState<{ msg: string; url: string | null } | null>(null);

  useEffect(() => {
    isEnabled().then(setAuto).catch(() => setAuto(false));
  }, []);

  const previewScrim = (v: number) => {
    setScrim(v);
    document.documentElement.style.setProperty("--scrim", String(v / 100));
  };

  const run = async (fn: () => Promise<void>) => {
    try {
      await fn();
    } catch (e) {
      notify(errorMessage(e));
    }
  };

  async function toggleAutostart(on: boolean) {
    await run(async () => {
      if (on) await enable();
      else await disable();
      setAuto(await isEnabled());
    });
  }

  async function chooseBackground() {
    const p = await open({ multiple: false, filters: [{ name: "Imagens", extensions: ["png", "jpg", "jpeg", "webp"] }] });
    if (typeof p === "string") await run(async () => replaceAll(await api.setAppBackground(p)));
  }

  async function doExport() {
    const p = await save({ defaultPath: "alexandria-backup.json", filters: JSON_FILTER });
    if (p) await run(async () => notify(`Backup salvo com ${await api.exportBackup(p)} jogo(s).`));
  }

  async function doImport() {
    const p = await open({ multiple: false, filters: JSON_FILTER });
    if (typeof p !== "string") return;
    await run(async () => {
      const r = await api.importBackup(p);
      replaceAll(await api.getSettings());
      await reload();
      notify(`Importado: ${r.added} novo(s); ${r.skipped} já existiam.`);
    });
  }

  async function measure() {
    setDiag("Medindo…");
    const perf = performance as Performance & { memory?: { usedJSHeapSize: number } };
    const heap = perf.memory ? `${Math.round(perf.memory.usedJSHeapSize / 1048576)} MB` : "indisponível";
    const nodes = document.getElementsByTagName("*").length;
    let frames = 0;
    const t0 = performance.now();
    await new Promise<void>((res) => {
      const tick = (t: number) => {
        frames++;
        if (t - t0 >= 2000) res();
        else requestAnimationFrame(tick);
      };
      requestAnimationFrame(tick);
    });
    const fps = Math.round(frames / ((performance.now() - t0) / 1000));
    setDiag(`Memória da interface: ${heap} · Elementos na tela: ${nodes} · ${fps} quadros por segundo`);
  }

  async function checkUpdate() {
    setUpd({ msg: "Verificando…", url: null });
    try {
      const u = await api.checkUpdate();
      setUpd(
        u.newer
          ? { msg: `Nova versão disponível: ${u.latest} (você tem ${u.current}).`, url: u.url }
          : { msg: `Você está na versão mais recente (${u.current}).`, url: null },
      );
    } catch (e) {
      setUpd({ msg: errorMessage(e), url: null });
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
      <Toggle label="Animações e transições" checked={s.animations !== "off"} onChange={(v) => void set("animations", v ? "on" : "off")} />
      <Toggle label="Sons de navegação" checked={s.sounds === "on"} onChange={(v) => void set("sounds", v ? "on" : "off")} />
      <div className="f"><label>Imagem de fundo do app</label>
        <div className="row">
          <button className="btn" onClick={() => void chooseBackground()}>Escolher imagem…</button>
          {s.app_bg && <button className="btn" onClick={() => void set("app_bg", "")}>Remover</button>}
        </div></div>

      <h3>Início</h3>
      <Toggle label="Abrir já em tela cheia" checked={s.start_fullscreen === "on"} onChange={(v) => void set("start_fullscreen", v ? "on" : "off")} />
      <Toggle label="Iniciar com o Windows" checked={auto} onChange={(v) => void toggleAutostart(v)} />
      <p className="muted">As duas opções começam desligadas. Juntas, o ALEXANDRIA liga em tela cheia, como um console.</p>

      <h3>SteamGridDB (imagens)</h3>
      <p className="muted">Gere uma chave grátis em steamgriddb.com (Preferências, API). Ela fica guardada só neste computador.</p>
      <div className="f">
        <div className="row">
          <input type="password" aria-label="Chave do SteamGridDB" autoComplete="off"
            placeholder={s.sgdb_key_set ? "Chave salva (digite para trocar)" : "Cole a chave aqui"}
            value={key} onChange={(e) => setKey(e.target.value)} />
          <button className="btn" disabled={!key.trim()} onClick={async () => { await set("sgdb_key", key.trim()); setKey(""); }}>Salvar</button>
          {s.sgdb_key_set && <button className="btn" onClick={() => void set("sgdb_key", "")}>Remover</button>}
        </div></div>

      <h3>IGDB (descrições e imagens)</h3>
      <p className="muted">Crie um aplicativo grátis em dev.twitch.tv e cole o Client ID e o Client Secret. Ficam só neste computador.</p>
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

      <h3>Lojas</h3>
      <p className="muted">
        {platforms
          ? `Steam: ${platforms.steam ? "encontrada" : "não encontrada"} · Epic: ${platforms.epic ? "encontrada" : "não encontrada"} · GOG: ${platforms.gog ? "encontrado" : "não encontrado"}`
          : "Verificando lojas…"}
      </p>
      <div className="row">
        <button className="btn" onClick={onSync}>Sincronizar agora</button>
        <button className="btn" disabled={!platforms?.steam || !!progress} onClick={onEnrich}>
          {progress ? `Baixando… ${progress.done}/${progress.total}` : "Baixar capas da Steam"}
        </button>
        <button className="btn" onClick={onAdd}><Icon name="plus" size={16} /> Adicionar jogo</button>
      </div>

      <h3>Backup</h3>
      <div className="row">
        <button className="btn" onClick={() => void doExport()}>Exportar biblioteca</button>
        <button className="btn" onClick={() => void doImport()}>Importar backup</button>
      </div>
      <p className="muted">O backup não inclui as chaves de API. Importar nunca apaga nem substitui jogos que já existem.</p>

      <h3>Desempenho</h3>
      <div className="row"><button className="btn" onClick={() => void measure()}>Medir agora</button></div>
      {diag && <p className="muted">{diag}</p>}

      <h3>Atualização</h3>
      <div className="row">
        <button className="btn" onClick={() => void checkUpdate()}>Verificar atualização</button>
        {upd?.url && <button className="btn primary" onClick={() => void run(() => api.openExternal(upd.url as string))}>Abrir página</button>}
      </div>
      {upd && <p className="muted">{upd.msg}</p>}

      <p className="muted">ALEXANDRIA v{APP_VERSION}</p>
      <div className="acts"><button className="btn primary" onClick={onClose}>Concluir</button></div>
    </Modal>
  );
}
