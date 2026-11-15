import { useEffect, useState } from "react";
import { Modal } from "../../components/Modal";
import { api, errorMessage } from "../../lib/ipc";
import type { Game, ImageKind, SgdbImage } from "../../lib/types";

const LABELS: Record<ImageKind, string> = {
  cover: "capa vertical",
  background: "fundo",
  hero: "arte horizontal",
  logo: "logo",
};

type Source = "sgdb" | "igdb";
interface Match {
  id: number;
  name: string;
}

interface Props {
  game: Game;
  kind: ImageKind;
  onClose: () => void;
  onPicked: (g: Game) => void;
}

/** Busca automática pelo nome do jogo no SteamGridDB ou no IGDB; o usuário escolhe a imagem. */
export function ImagePicker({ game, kind, onClose, onPicked }: Props) {
  const [source, setSource] = useState<Source>("sgdb");
  const [matches, setMatches] = useState<Match[]>([]);
  const [matchId, setMatchId] = useState<number | null>(null);
  const [images, setImages] = useState<SgdbImage[]>([]);
  const [busy, setBusy] = useState(true);
  const [error, setError] = useState<string | null>(null);

  async function load(src: Source, id: number) {
    setMatchId(id);
    setBusy(true);
    setError(null);
    try {
      setImages(src === "sgdb" ? await api.sgdbImages(id, kind) : await api.igdbImages(id, kind));
    } catch (e) {
      setImages([]);
      setError(errorMessage(e));
    } finally {
      setBusy(false);
    }
  }

  async function search(src: Source) {
    setBusy(true);
    setError(null);
    setMatches([]);
    setImages([]);
    setMatchId(null);
    try {
      const found: Match[] =
        src === "sgdb"
          ? await api.sgdbSearch(game.name)
          : (await api.igdbSearch(game.name)).map((g) => ({ id: g.id, name: g.name }));
      setMatches(found);
      if (!found.length) {
        setError("Nada encontrado com esse nome.");
        setBusy(false);
        return;
      }
      await load(src, found[0].id);
    } catch (e) {
      setError(errorMessage(e));
      setBusy(false);
    }
  }

  useEffect(() => {
    void search(source);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [source]);

  async function pick(img: SgdbImage) {
    setBusy(true);
    try {
      onPicked(await api.applyImage(game.id, kind, img.url));
      onClose();
    } catch (e) {
      setError(errorMessage(e));
      setBusy(false);
    }
  }

  return (
    <Modal title={`Escolher ${LABELS[kind]}: ${game.name}`} onClose={onClose} wide>
      <div className="chips">
        <button className="btn" aria-pressed={source === "sgdb"} onClick={() => setSource("sgdb")}>SteamGridDB</button>
        <button className="btn" aria-pressed={source === "igdb"} onClick={() => setSource("igdb")}>IGDB</button>
      </div>
      {matches.length > 1 && (
        <div className="chips">
          {matches.map((m) => (
            <button key={m.id} className="btn" aria-pressed={m.id === matchId} onClick={() => void load(source, m.id)}>
              {m.name}
            </button>
          ))}
        </div>
      )}
      {error && <p className="form-error">{error}</p>}
      {busy && <p className="muted">Buscando…</p>}
      <div className="pick-grid">
        {images.map((img) => (
          <button key={img.url} className="pick" onClick={() => void pick(img)} aria-label="Usar esta imagem">
            <img src={img.thumb} alt="" loading="lazy" />
          </button>
        ))}
      </div>
      <div className="acts"><button className="btn" onClick={onClose}>Cancelar</button></div>
    </Modal>
  );
}
