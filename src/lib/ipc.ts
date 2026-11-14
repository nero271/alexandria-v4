import { invoke } from "@tauri-apps/api/core";
import type {
  Game, GameUpdate, IgdbGame, ImageKind, NewGame, SgdbGame, SgdbImage, Settings, SteamStatus, SyncSummary,
} from "./types";

/** Única porta de entrada para o Rust. Os nomes batem com os comandos em src-tauri/src/commands/. */
export const api = {
  listGames: () => invoke<Game[]>("list_games", { query: null, favoritesOnly: false }),
  addGame: (game: NewGame) => invoke<Game>("add_game", { game }),
  updateGame: (id: number, patch: GameUpdate) => invoke<Game>("update_game", { id, patch }),
  setFavorite: (id: number, favorite: boolean) => invoke<Game>("set_favorite", { id, favorite }),
  removeGame: (id: number) => invoke<void>("remove_game", { id }),
  clearImage: (id: number, kind: ImageKind) => invoke<Game>("clear_image", { id, kind }),
  launchGame: (id: number) => invoke<Game>("launch_game", { id }),

  steamStatus: () => invoke<SteamStatus>("steam_status"),
  steamSync: () => invoke<SyncSummary>("steam_sync"),
  enrichSteamGame: (id: number) => invoke<Game>("enrich_steam_game", { id }),

  sgdbSearch: (term: string) => invoke<SgdbGame[]>("sgdb_search", { term }),
  sgdbImages: (gameId: number, kind: ImageKind) => invoke<SgdbImage[]>("sgdb_images", { gameId, kind }),
  igdbSearch: (term: string) => invoke<IgdbGame[]>("igdb_search", { term }),
  applyImage: (id: number, kind: ImageKind, url: string) => invoke<Game>("apply_image", { id, kind, url }),
  applyLocalImage: (id: number, kind: ImageKind, path: string) =>
    invoke<Game>("apply_local_image", { id, kind, path }),

  getSettings: () => invoke<Settings>("get_settings"),
  setSetting: (key: string, value: string) => invoke<Settings>("set_setting", { key, value }),
  setAppBackground: (path: string) => invoke<Settings>("set_app_background", { path }),
};

/** Erros do Rust chegam como string; erros de JS como Error. */
export function errorMessage(e: unknown): string {
  return typeof e === "string" ? e : e instanceof Error ? e.message : "Erro inesperado.";
}
