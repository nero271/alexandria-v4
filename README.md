# ALEXANDRIA

A game launcher for Windows. It has a normal desktop mode and a fullscreen mode meant for a controller and a TV, where everything can be done without touching the keyboard or mouse (apart from typing text, which has an on-screen keyboard).

It reads the games you already have installed from Steam, Epic Games Launcher and GOG Galaxy, and lets you add anything else by pointing it at an executable. The library lives in a local SQLite database. Nothing is uploaded anywhere.

Built with Tauri 2, Rust and React.

## What it does

- Imports installed games from Steam, Epic and GOG, and notices when they are installed or removed
- Downloads cover art, backgrounds, logos and descriptions (Steam's own assets, SteamGridDB and IGDB)
- Changes the background as you move between games
- Works with Xbox, DualShock 4 and DualSense controllers, and switches the button prompts to match
- Tracks play time (read from Steam, or counted by the launcher for games it starts directly)
- Lets you rename games and replace any image or description; your edits are never overwritten by a later import
- Exports and imports the library as a JSON file

## Setting up

You need Node.js 20, Rust (MSVC toolchain) and the Visual Studio C++ build tools.

```
npm install
npx tauri icon app-icon.png
npm run tauri dev
```

`npm run tauri build` produces an installer. The GitHub workflow in `.github/workflows` does the same on a Windows runner and attaches the installer to a release.

Cover search needs your own keys, entered in the settings. Both are free:

- SteamGridDB: an API key from your profile page
- IGDB: a Client ID and Secret from an app registered at dev.twitch.tv

They are stored on your machine only. Steam artwork and descriptions work without any key.

## Notes

- The only network requests are to the Steam CDN and store, SteamGridDB, IGDB, and the GitHub API when you check for updates.
- The launcher does not inject anything into games.
- Status: early. It is changing quickly and has seen limited testing, so expect rough edges.
- Not affiliated with or endorsed by Valve, Epic Games, GOG, Microsoft or Sony.

Changes are listed in [CHANGELOG.md](CHANGELOG.md).
