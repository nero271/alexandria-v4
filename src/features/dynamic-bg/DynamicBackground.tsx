import { useEffect, useRef, useState } from "react";
import { asset, preload } from "../../lib/assets";
import type { Game } from "../../lib/types";

interface Layer {
  id: number;
  src: string | null;
}

const FADE_MS = 450;

interface Props {
  game: Game | null;
  neighbors: Game[];
  fallback: string | null;
}

/** Fundo que troca com transição suave conforme o jogo selecionado. Só imagens estáticas. */
export function DynamicBackground({ game, neighbors, fallback }: Props) {
  const src = asset(game?.backgroundPath ?? game?.heroPath ?? game?.coverPath) ?? fallback;
  const [layers, setLayers] = useState<Layer[]>([{ id: 0, src }]);
  const counter = useRef(0);

  // Pré-carrega os vizinhos da seleção (cada imagem é buscada só uma vez).
  useEffect(() => {
    preload(neighbors.map((g) => asset(g.backgroundPath ?? g.heroPath ?? g.coverPath)));
  }, [neighbors]);

  useEffect(() => {
    const top = layers[layers.length - 1];
    if (top && top.src === src) return;
    let cancelled = false;
    const show = () => {
      if (cancelled) return;
      const id = ++counter.current;
      setLayers((prev) => [...prev.slice(-1), { id, src }]);
      // Mantém sempre a camada mais nova; nunca deixa a lista vazia.
      window.setTimeout(() => setLayers((prev) => (prev.length > 1 ? prev.slice(-1) : prev)), FADE_MS + 80);
    };
    if (!src) {
      show();
      return () => {
        cancelled = true;
      };
    }
    const img = new Image();
    img.src = src;
    img.decode().then(show, show); // só troca quando a nova imagem está pronta: sem piscadas
    return () => {
      cancelled = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [src]);

  return (
    <div className="bg-root" aria-hidden="true">
      {layers.map((l) => (
        <div key={l.id} className="bg-layer" style={l.src ? { backgroundImage: `url("${l.src}")` } : undefined} />
      ))}
      <div className="bg-scrim" />
    </div>
  );
}
