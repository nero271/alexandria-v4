import { useEffect, useRef } from "react";

export type Action =
  | "left" | "right" | "up" | "down"
  | "ok" | "back" | "x" | "y" | "lb" | "rb" | "lt" | "rt" | "menu" | "mode";

type Handler = (a: Action) => void;
const stack: Handler[] = [];

/** Só o componente aberto mais recentemente recebe os comandos: evita conflitos de entrada. */
export function dispatch(a: Action) {
  stack[stack.length - 1]?.(a);
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
