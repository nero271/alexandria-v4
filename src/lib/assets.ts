import { convertFileSrc } from "@tauri-apps/api/core";

const preloaded = new Set<string>();

/** Caminho local do cache -> endereço que a janela consegue exibir. */
export const asset = (path?: string | null): string | null => (path ? convertFileSrc(path) : null);

/** Pré-carrega imagens próximas da seleção, uma única vez cada. */
export function preload(urls: (string | null | undefined)[]) {
  for (const u of urls) {
    if (!u || preloaded.has(u)) continue;
    preloaded.add(u);
    const img = new Image();
    img.decoding = "async";
    img.src = u;
  }
}
