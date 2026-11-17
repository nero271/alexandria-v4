export type InstallState = "installed" | "not_installed" | "unknown";
export type GameStatus = "playing" | "completed" | "backlog" | "dropped";
export type ImageKind = "cover" | "background" | "hero" | "logo";

export interface Game {
  id: number;
  name: string;
  platform: string;
  platformId: string | null;
  launchTarget: string | null;
  category: string | null;
  installState: InstallState;
  favorite: boolean;
  lastPlayedAt: number | null;
  description: string | null;
  coverPath: string | null;
  heroPath: string | null;
  backgroundPath: string | null;
  logoPath: string | null;
  createdAt: number;
  updatedAt: number;
  playtimeMinutes: number;
  status: GameStatus | null;
  rating: number | null;
}

export interface NewGame {
  name: string;
  platform?: string;
  platformId?: string;
  launchTarget?: string;
  category?: string;
  description?: string;
}

export interface GameUpdate {
  name?: string;
  platform?: string;
  launchTarget?: string;
  category?: string;
  description?: string;
}

export interface SteamStatus {
  found: boolean;
  path: string | null;
}

export interface SyncSummary {
  added: number;
  updated: number;
  uninstalled: number;
}

export interface SgdbGame {
  id: number;
  name: string;
}

export interface SgdbImage {
  url: string;
  thumb: string;
}

export interface IgdbGame {
  id: number;
  name: string;
  summary: string | null;
}

export interface PlatformStatus {
  steam: boolean;
  epic: boolean;
  gog: boolean;
  ubisoft: boolean;
  xbox: boolean;
}

export interface UpdateInfo {
  current: string;
  latest: string | null;
  newer: boolean;
  url: string | null;
}

export interface BackupSummary {
  added: number;
  skipped: number;
}

export interface Emulator {
  id: number;
  name: string;
  exePath: string;
  args: string;
}

export interface TopGame {
  id: number;
  name: string;
  minutes: number;
  sessions: number;
}

export interface YearStats {
  year: number;
  years: number[];
  sessions: number;
  minutes: number;
  days: number;
  longestStreak: number;
  currentStreak: number;
  top: TopGame[];
  lifetimeMinutes: number;
  gamesTotal: number;
}

export interface SourceInfo {
  id: number;
  url: string;
  name: string;
  gameCount: number;
  syncedAt: number;
}

export interface SourceGame {
  sourceId: number;
  sourceName: string;
  id: string;
  name: string;
  version: string;
  description: string | null;
  cover: string | null;
  category: string | null;
  license: string | null;
  size: number | null;
  host: string;
  installed: boolean;
  installedVersion: string | null;
  update: boolean;
  libraryId: number | null;
}

export interface InstallProgress {
  key: string;
  phase: "start" | "download" | "verify" | "extract" | "done" | "paused";
  done: number;
  total: number;
  speed: number;
}

export type Settings = Record<string, string>;
