const NAMES: Record<string, string> = {
  steam: "Steam",
  epic: "Epic",
  gog: "GOG",
  ubisoft: "Ubisoft",
  xbox: "Xbox",
  manual: "Manual",
  fonte: "Loja",
};

/** "emulator:3" -> "Emulador"; "steam" -> "Steam". */
export function platformLabel(platform: string): string {
  if (platform.startsWith("emulator:")) return "Emulador";
  return NAMES[platform] ?? platform;
}
