import { useActionHandler } from "../input/bus";

/** Mostra uma imagem grande; clicar ou apertar voltar fecha. */
export function Lightbox({ src, onClose }: { src: string; onClose: () => void }) {
  useActionHandler((a) => {
    if (a === "back" || a === "ok") onClose();
  });
  return (
    <div className="lightbox" onClick={onClose} role="dialog" aria-label="Captura de tela">
      <img src={src} alt="" />
    </div>
  );
}
