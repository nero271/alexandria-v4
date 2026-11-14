import { useEffect, useRef } from "react";

const DURATION_MS = 2600;

/** Animação de entrada no modo tela cheia (preto, brilho, linha de luz e nome do app). */
export function FullscreenIntro({ name, onDone }: { name: string; onDone: () => void }) {
  const done = useRef(onDone);
  done.current = onDone;
  useEffect(() => {
    const t = window.setTimeout(() => done.current(), DURATION_MS);
    return () => window.clearTimeout(t);
  }, []);
  return (
    <div className="intro" aria-hidden="true">
      <div className="intro-glow" />
      <div className="intro-line" />
      <div className="intro-name">{name}</div>
    </div>
  );
}
