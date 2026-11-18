import { useEffect, useRef } from "react";
import { Logo } from "./Logo";

const DURATION_MS = 3150;

/** Animação de entrada no modo tela cheia: brilho, linha de luz e a logo (III) se formando. */
export function FullscreenIntro({ onDone }: { onDone: () => void }) {
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
      <Logo className="intro-logo" />
    </div>
  );
}
