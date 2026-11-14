export type InstallState = "installed" | "not_installed" | "unknown";
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

export type Settings = Record<string, string>;
