import { useEffect, useRef, type ReactNode } from "react";
import { useActionHandler, type Action } from "../input/bus";

const FOCUSABLE = "button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled])";

interface Props {
  title: ReactNode;
  label?: string;
  onClose: () => void;
  children: ReactNode;
  wide?: boolean;
}

/** Janela modal. Com controle: ↑↓ percorre os campos, confirmar aciona, voltar fecha. */
export function Modal({ title, label, onClose, children, wide }: Props) {
  const ref = useRef<HTMLDialogElement>(null);

  useEffect(() => {
    const d = ref.current;
    if (d && !d.open) d.showModal();
  }, []);

  useActionHandler((a: Action) => {
    const d = ref.current;
    if (!d) return;
    if (a === "back") return onClose();
    const items = Array.from(d.querySelectorAll<HTMLElement>(FOCUSABLE));
    const i = items.indexOf(document.activeElement as HTMLElement);
    const active = document.activeElement as HTMLInputElement | null;
    if ((a === "left" || a === "right") && active?.type === "range") {
      active.value = String(Number(active.value) + (a === "right" ? 5 : -5));
      active.dispatchEvent(new Event("input", { bubbles: true }));
    } else if (a === "down" || a === "right") items[Math.min(items.length - 1, i + 1)]?.focus();
    else if (a === "up" || a === "left") items[Math.max(0, i - 1)]?.focus();
    else if (a === "ok" && active && active.tagName !== "INPUT") active.click();
    else if (a === "ok" && active?.type === "checkbox") active.click();
  });

  return (
    <dialog ref={ref} aria-label={label ?? (typeof title === "string" ? title : undefined)} className={wide ? "wide" : ""} onClose={onClose} onCancel={onClose}>
      <h2>{title}</h2>
      {children}
    </dialog>
  );
}
