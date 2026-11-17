import { invoke } from "@tauri-apps/api/core";
import type {
  BackupSummary, Emulator, Game, GameUpdate, IgdbGame, ImageKind, NewGame, PlatformStatus, SgdbGame, SgdbImage, Settings, SteamStatus, SyncSummary, UpdateInfo, YearStats, SourceGame, SourceInfo,
} from "./types";

/** Única porta de entrada para o Rust. Os nomes batem com os comandos em src-tauri/src/commands/. */
export const api = {
  listGames: () => invoke<Game[]>("list_games", { query: null, favoritesOnly: false }),
  addGame: (game: NewGame) => invoke<Game>("add_game", { game }),
  updateGame: (id: number, patch: GameUpdate) => invoke<Game>("update_game", { id, patch }),
  setFavorite: (id: number, favorite: boolean) => invoke<Game>("set_favorite", { id, favorite }),
  removeGame: (id: number) => invoke<void>("remove_game", { id }),
  setStatus: (id: number, status: string | null) => invoke<Game>("set_status", { id, status }),
  setRating: (id: number, rating: number | null) => invoke<Game>("set_rating", { id, rating }),
  clearImage: (id: number, kind: ImageKind) => invoke<Game>("clear_image", { id, kind }),
  launchGame: (id: number) => invoke<Game>("launch_game", { id }),

  steamStatus: () => invoke<SteamStatus>("steam_status"),
  steamSync: () => invoke<SyncSummary>("steam_sync"),
  platformStatus: () => invoke<PlatformStatus>("platform_status"),
  platformSync: (platform: "steam" | "epic" | "gog") => invoke<SyncSummary>("platform_sync", { platform }),
  enrichSteamGame: (id: number) => invoke<Game>("enrich_steam_game", { id }),

  sgdbSearch: (term: string) => invoke<SgdbGame[]>("sgdb_search", { term }),
  sgdbImages: (gameId: number, kind: ImageKind) => invoke<SgdbImage[]>("sgdb_images", { gameId, kind }),
  igdbSearch: (term: string) => invoke<IgdbGame[]>("igdb_search", { term }),
  igdbImages: (gameId: number, kind: ImageKind) => invoke<SgdbImage[]>("igdb_images", { gameId, kind }),
  applyImage: (id: number, kind: ImageKind, url: string) => invoke<Game>("apply_image", { id, kind, url }),
  applyLocalImage: (id: number, kind: ImageKind, path: string) =>
    invoke<Game>("apply_local_image", { id, kind, path }),

  exportBackup: (path: string) => invoke<number>("export_backup", { path }),
  importBackup: (path: string) => invoke<BackupSummary>("import_backup", { path }),
  checkUpdate: () => invoke<UpdateInfo>("check_update"),
  openExternal: (url: string) => invoke<void>("open_external", { url }),

  yearStats: (year: number, tzOffsetMinutes: number) => invoke<YearStats>("year_stats", { year, tzOffsetMinutes }),
  listScreenshots: (appId: string) => invoke<string[]>("list_screenshots", { appId }),
  listEmulators: () => invoke<Emulator[]>("list_emulators"),
  addEmulator: (name: string, exePath: string, args: string) => invoke<Emulator>("add_emulator", { name, exePath, args }),
  removeEmulator: (id: number) => invoke<void>("remove_emulator", { id }),

  sourceList: () => invoke<SourceInfo[]>("source_list"),
  sourceGames: () => invoke<SourceGame[]>("source_games"),
  sourceAdd: (url: string) => invoke<SourceInfo>("source_add", { url }),
  sourceRefresh: (id: number) => invoke<SourceInfo>("source_refresh", { id }),
  sourceRemove: (id: number) => invoke<void>("source_remove", { id }),
  installGame: (sourceId: number, gameId: string) => invoke<Game>("install_game", { sourceId, gameId }),
  pauseInstall: (sourceId: number, gameId: string) => invoke<void>("pause_install", { sourceId, gameId }),
  uninstallGame: (sourceId: number, gameId: string) => invoke<void>("uninstall_game", { sourceId, gameId }),

  getSettings: () => invoke<Settings>("get_settings"),
  setSetting: (key: string, value: string) => invoke<Settings>("set_setting", { key, value }),
  setAppBackground: (path: string) => invoke<Settings>("set_app_background", { path }),
};

/** Erros do Rust chegam como string; erros de JS como Error. */
export function errorMessage(e: unknown): string {
  return typeof e === "string" ? e : e instanceof Error ? e.message : "Erro inesperado.";
}
