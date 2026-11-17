import { useEffect, useRef } from "react";
import { playFor } from "../lib/sound";

export type Action =
  | "left" | "right" | "up" | "down"
  | "ok" | "back" | "x" | "y" | "lb" | "rb" | "lt" | "rt" | "menu" | "mode";

/** Devolvido por um handler para deixar o comando passar para a tela de baixo. */
export const PASS = Symbol("pass");
type Handler = (a: Action) => void | typeof PASS;
const stack: Handler[] = [];

/** Só o componente aberto mais recentemente recebe os comandos: evita conflitos de entrada. */
export function dispatch(a: Action) {
  playFor(a);
  for (let i = stack.length - 1; i >= 0; i--) {
    if (stack[i](a) !== PASS) return;
  }
}

export function useActionHandler(handler: Handler, active = true) {
  const ref = useRef(handler);
  ref.current = handler;
  useEffect(() => {
    if (!active) return;
    const h: Handler = (a) => ref.current(a);
    stack.push(h);
    return () => {
      const i = stack.lastIndexOf(h);
      if (i >= 0) stack.splice(i, 1);
    };
  }, [active]);
}
