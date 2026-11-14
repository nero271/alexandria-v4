import { useEffect, useState } from "react";
import { Modal } from "../../components/Modal";
import { api, errorMessage } from "../../lib/ipc";
import type { Game, ImageKind, SgdbGame, SgdbImage } from "../../lib/types";

const LABELS: Record<ImageKind, string> = {
  cover: "capa vertical",
  background: "fundo",
  hero: "arte horizontal",
  logo: "logo",
};

interface Props {
  game: Game;
  kind: ImageKind;
  onClose: () => void;
  onPicked: (g: Game) => void;
}

/** Busca automática no SteamGridDB pelo nome do jogo; o usuário escolhe a imagem. */
export function ImagePicker({ game, kind, onClose, onPicked }: Props) {
  const [matches, setMatches] = useState<SgdbGame[]>([]);
  const [matchId, setMatchId] = useState<number | null>(null);
  const [images, setImages] = useState<SgdbImage[]>([]);
  const [busy, setBusy] = useState(true);
  const [error, setError] = useState<string | null>(null);

  async function loadImages(id: number) {
    setMatchId(id);
    setBusy(true);
    setError(null);
    try {
      setImages(await api.sgdbImages(id, kind));
    } catch (e) {
      setError(errorMessage(e));
    } finally {
      setBusy(false);
    }
  }

  useEffect(() => {
    void (async () => {
      try {
        const found = await api.sgdbSearch(game.name);
        setMatches(found);
        if (found.length) await loadImages(found[0].id);
        else {
          setError("Nenhum jogo com esse nome no SteamGridDB.");
          setBusy(false);
        }
      } catch (e) {
        setError(errorMessage(e));
        setBusy(false);
      }
    })();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

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
      {matches.length > 1 && (
        <div className="chips">
          {matches.map((m) => (
            <button key={m.id} className="btn" aria-pressed={m.id === matchId} onClick={() => void loadImages(m.id)}>
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
