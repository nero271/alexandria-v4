import { useEffect, useRef, useState } from "react";
import { useActionHandler, type Action } from "../input/bus";

const ROWS = ["1234567890", "qwertyuiop", "asdfghjkl", "zxcvbnm.-_", "áàâãéêíóôõúç"];
const BOTTOM = ["Maiúsc", "Espaço", "⌫", "OK"];
const GRID: string[][] = [...ROWS.map((r) => Array.from(r)), BOTTOM];

interface Props {
  target: HTMLInputElement | HTMLTextAreaElement;
  onClose: () => void;
}

/** Teclado na tela para digitar com o controle (direcional move, confirmar digita). */
export function OnScreenKeyboard({ target, onClose }: Props) {
  const ref = useRef<HTMLDialogElement>(null);
  const [text, setText] = useState(target.value);
  const [shift, setShift] = useState(false);
  const [pos, setPos] = useState({ r: 1, c: 0 });

  useEffect(() => {
    const d = ref.current;
    if (d && !d.open) d.showModal();
  }, []);

  // Escreve no campo original de um jeito que o React percebe (dispara o evento "input").
  const commit = (v: string) => {
    const max = target.maxLength;
    if (max > 0 && v.length > max) return;
    setText(v);
    const setter = Object.getOwnPropertyDescriptor(Object.getPrototypeOf(target), "value")?.set;
    setter?.call(target, v);
    target.dispatchEvent(new Event("input", { bubbles: true }));
  };

  const press = (k: string) => {
    if (k === "Maiúsc") setShift((s) => !s);
    else if (k === "Espaço") commit(text + " ");
    else if (k === "⌫") commit(text.slice(0, -1));
    else if (k === "OK") onClose();
    else commit(text + (shift ? k.toUpperCase() : k));
  };

  useActionHandler((a: Action) => {
    if (a === "back") return onClose();
    if (a === "x") return commit(text.slice(0, -1));
    if (a === "y") return commit(text + " ");
    if (a === "ok") return press(GRID[pos.r][pos.c]);
    setPos((p) => {
      let { r, c } = p;
      if (a === "up") r = Math.max(0, r - 1);
      else if (a === "down") r = Math.min(GRID.length - 1, r + 1);
      else if (a === "left") c = Math.max(0, c - 1);
      else if (a === "right") c += 1;
      else return p;
      return { r, c: Math.min(c, GRID[r].length - 1) };
    });
  });

  return (
    <dialog
      ref={ref}
      className="osk"
      aria-label="Teclado na tela"
      onCancel={(e) => {
        e.preventDefault();
        onClose();
      }}
    >
      <div className="osk-text">{target.type === "password" ? "•".repeat(text.length) : text || " "}</div>
      {GRID.map((row, r) => (
        <div className="osk-row" key={r}>
          {row.map((k, c) => (
            <button
              key={k + c}
              className={`osk-key${pos.r === r && pos.c === c ? " focus" : ""}${k === "Maiúsc" && shift ? " on" : ""}${k.length > 1 ? " wide" : ""}`}
              onMouseEnter={() => setPos({ r, c })}
              onClick={() => press(k)}
            >
              {k.length === 1 && shift ? k.toUpperCase() : k}
            </button>
          ))}
        </div>
      ))}
    </dialog>
  );
}
