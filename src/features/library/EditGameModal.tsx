import { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { Modal } from "../../components/Modal";
import { asset } from "../../lib/assets";
import { api, errorMessage } from "../../lib/ipc";
import type { Game, GameUpdate, IgdbGame, ImageKind, NewGame } from "../../lib/types";
import { ImagePicker } from "./ImagePicker";

const KINDS: [ImageKind, string][] = [
  ["cover", "Capa vertical"],
  ["background", "Imagem de fundo"],
  ["hero", "Arte horizontal"],
  ["logo", "Logo"],
];
const IMG_FILTER = [{ name: "Imagens", extensions: ["png", "jpg", "jpeg", "webp", "gif"] }];
const pathOf = (g: Game, k: ImageKind) =>
  ({ cover: g.coverPath, background: g.backgroundPath, hero: g.heroPath, logo: g.logoPath })[k];

interface Props {
  target: Game | "new";
  onClose: () => void;
  onCreate: (n: NewGame) => Promise<Game | null>;
  onUpdate: (id: number, p: GameUpdate) => Promise<Game | null>;
  onRemove: (g: Game) => Promise<boolean>;
  onChanged: (g: Game) => void;
  notify: (m: string) => void;
}

/** Cadastro e personalização individual: nome, categoria, executável, descrição e imagens. */
export function EditGameModal({ target, onClose, onCreate, onUpdate, onRemove, onChanged, notify }: Props) {
  const [game, setGame] = useState<Game | null>(target === "new" ? null : target);
  const [name, setName] = useState(game?.name ?? "");
  const [platform, setPlatform] = useState(game?.platform ?? "manual");
  const [category, setCategory] = useState(game?.category ?? "");
  const [exe, setExe] = useState(game?.launchTarget ?? "");
  const [desc, setDesc] = useState(game?.description ?? "");
  const [igdb, setIgdb] = useState<IgdbGame[] | null>(null);
  const [picker, setPicker] = useState<ImageKind | null>(null);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const isSteam = game?.platform === "steam";

  const apply = (g: Game) => {
    setGame(g);
    onChanged(g);
  };

  async function save() {
    if (!name.trim()) return notify("Informe o nome do jogo.");
    if (!game) {
      const created = await onCreate({
        name,
        platform: platform || undefined,
        category: category || undefined,
        launchTarget: exe || undefined,
        description: desc || undefined,
      });
      if (created) {
        setGame(created);
        notify("Jogo adicionado. Agora você pode escolher as imagens.");
      }
    } else {
      const g = await onUpdate(game.id, { name, platform, category, launchTarget: isSteam ? undefined : exe, description: desc });
      if (g) {
        setGame(g);
        notify("Alterações salvas.");
      }
    }
  }

  async function searchIgdb() {
    try {
      setIgdb(await api.igdbSearch(name.trim() || game?.name || ""));
    } catch (e) {
      notify(errorMessage(e));
    }
  }

  async function pickExe() {
    const p = await open({ multiple: false, filters: [{ name: "Programas", extensions: ["exe", "bat"] }] });
    if (typeof p === "string") setExe(p);
  }

  async function pickLocal(kind: ImageKind) {
    if (!game) return;
    const p = await open({ multiple: false, filters: IMG_FILTER });
    if (typeof p !== "string") return;
    try {
      apply(await api.applyLocalImage(game.id, kind, p));
    } catch (e) {
      notify(errorMessage(e));
    }
  }

  async function clear(kind: ImageKind) {
    if (!game) return;
    try {
      apply(await api.clearImage(game.id, kind));
    } catch (e) {
      notify(errorMessage(e));
    }
  }

  async function steamArt() {
    if (!game) return;
    try {
      apply(await api.enrichSteamGame(game.id));
      notify("Imagens e descrição da Steam baixadas (só o que faltava).");
    } catch (e) {
      notify(errorMessage(e));
    }
  }

  return (
    <Modal title={game ? "Editar jogo" : "Adicionar jogo"} onClose={onClose} wide>
      <div className="f"><label htmlFor="e-name">Nome exibido</label>
        <input id="e-name" value={name} maxLength={200} onChange={(e) => setName(e.target.value)} autoFocus /></div>
      <div className="f"><label htmlFor="e-cat">Categoria</label>
        <input id="e-cat" value={category} maxLength={60} onChange={(e) => setCategory(e.target.value)} /></div>
      {isSteam ? (
        <p className="muted">Plataforma: Steam (AppID {game?.platformId}). O jogo é iniciado pela própria Steam.</p>
      ) : (
        <>
          <div className="f"><label htmlFor="e-plat">Plataforma de origem</label>
            <input id="e-plat" value={platform} maxLength={40} onChange={(e) => setPlatform(e.target.value)} /></div>
          <div className="f"><label htmlFor="e-exe">Executável</label>
            <div className="row">
              <input id="e-exe" value={exe} placeholder="C:\Jogos\Celeste\Celeste.exe" onChange={(e) => setExe(e.target.value)} />
              <button className="btn" onClick={() => void pickExe()}>Escolher…</button>
            </div></div>
        </>
      )}
      <div className="f"><label htmlFor="e-desc">Descrição</label>
        <textarea id="e-desc" rows={3} value={desc} onChange={(e) => setDesc(e.target.value)} />
        <div className="row"><button className="btn" onClick={() => void searchIgdb()}>Buscar descrição no IGDB</button></div>
        {igdb && (igdb.length === 0 ? <p className="muted">Nada encontrado no IGDB.</p> : (
          <ul className="igdb">
            {igdb.map((g) => (
              <li key={g.id}>
                <button className="btn" disabled={!g.summary} onClick={() => { setDesc(g.summary ?? ""); setIgdb(null); }}>
                  <b>{g.name}</b>{g.summary ? ` — ${g.summary.slice(0, 90)}…` : " (sem descrição)"}
                </button>
              </li>
            ))}
          </ul>
        ))}</div>

      {game && (
        <>
          <h3>Imagens</h3>
          {KINDS.map(([kind, label]) => {
            const p = pathOf(game, kind);
            return (
              <div className="img-row" key={kind}>
                {p ? <img className="thumb" src={asset(p) ?? ""} alt="" /> : <div className="thumb" />}
                <span>{label}</span>
                <span className="row">
                  <button className="btn" onClick={() => setPicker(kind)}>Buscar online</button>
                  <button className="btn" onClick={() => void pickLocal(kind)}>Arquivo…</button>
                  {p && <button className="btn" onClick={() => void clear(kind)}>Limpar</button>}
                </span>
              </div>
            );
          })}
          {isSteam && <button className="btn" onClick={() => void steamArt()}>Baixar da Steam o que falta</button>}
        </>
      )}

      <div className="acts">
        {game && (
          <button
            className="btn danger"
            onClick={async () => {
              if (!confirmDelete) return setConfirmDelete(true);
              if (await onRemove(game)) onClose();
            }}
          >
            {confirmDelete ? "Confirmar remoção" : "Remover da biblioteca"}
          </button>
        )}
        <button className="btn" onClick={onClose}>Fechar</button>
        <button className="btn primary" onClick={() => void save()}>{game ? "Salvar" : "Adicionar"}</button>
      </div>

      {picker && game && <ImagePicker game={game} kind={picker} onClose={() => setPicker(null)} onPicked={apply} />}
    </Modal>
  );
}
