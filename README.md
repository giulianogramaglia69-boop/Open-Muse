# Open-Muse

A native desktop client for **Muse** — Meta's personal AI agent — for Linux,
Windows and macOS. One codebase, built with [Tauri v2](https://tauri.app)
(Rust + a tiny web frontend).

> **Honest note:** Muse has no public API for the agent itself, so v1 of
> Open-Muse is a native wrapper around the official web app at
> <https://muse.ai>. You sign in with your own Meta account inside the app;
> Open-Muse never sees your credentials and stores nothing of yours. If Meta
> ever ships a public agent API, v2 will use it (see Roadmap).

## Features

- Main window loads the Muse web app (`https://muse.ai`)
- **System tray icon** with Show / Hide, Reload, Launch at login, Quit
- **Global shortcut** `CmdOrCtrl+Shift+M` toggles the window from anywhere
- **Single instance** — a second launch focuses the existing window
- **Window state persistence** — size and position are remembered
- **Native menu** — About, Reload, Toggle DevTools, Zoom in / out / reset
- **Launch at login** toggle (tray menu)

## Architecture

- **Backend: 100% Rust.** All native behaviour lives in
  `src-tauri/src/main.rs` (Tauri v2): system tray, global shortcut
  (`CmdOrCtrl+Shift+M`), single-instance enforcement, native menu, auto-start
  toggle and window-state persistence. No app logic runs in JavaScript.
- **Frontend: as thin as possible.** The main window is just a webview that
  loads `https://muse.ai`. The local files (`index.html`, `src/main.ts`) are
  only a static placeholder shown in `tauri dev` — they contain zero
  application logic.

## Download

Grab the installer for your OS from the
[Releases](https://github.com/giulianogramaglia69-boop/Open-Muse/releases)
page (`.deb` / `.AppImage` for Linux, `.msi` for Windows, `.dmg` for macOS).
Releases are built automatically by GitHub Actions on every `v*` tag.

## Build from source

Requirements: [Node.js](https://nodejs.org) 18+ and
[Rust](https://rustup.rs) (stable).

```sh
git clone https://github.com/giulianogramaglia69-boop/Open-Muse.git
cd Open-Muse
npm install
npm run tauri dev      # run in development mode
npm run tauri build    # production build -> src-tauri/target/release/bundle/
```

### Linux

Tauri needs the system WebKit and a few dev libraries. On Debian/Ubuntu:

```sh
sudo apt update
sudo apt install -y libwebkit2gtk-4.1-dev build-essential curl wget file \
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

Notes:

- Ubuntu 24.04 uses `libwebkit2gtk-4.1-dev`; on Ubuntu 22.04 use
  `libwebkit2gtk-4.0-dev` instead.
- On Wayland, the global shortcut depends on the desktop's GlobalShortcuts
  portal (GNOME/KDE); if it doesn't register, the tray menu still works.

### macOS

Install the Xcode Command Line Tools once:

```sh
xcode-select --install
```

Then `npm install && npm run tauri build` as above. The output is a
`.app` and a `.dmg` in `src-tauri/target/release/bundle/`.

### Windows

Install Rust via [rustup](https://rustup.rs) and the Visual Studio
Build Tools (C++ workload). WebView2 is preinstalled on Windows 11 and
available on Windows 10 via Windows Update. Then
`npm install && npm run tauri build`.

## Roadmap

- [ ] **Muse Spark Model API chat mode** — an optional in-app chat that talks
      to the Muse Spark model API with the user's own API key (from
      [dev.meta.ai](https://dev.meta.ai)), for a native chat experience
      without the web app. (Depends on Meta's API availability.)
- [ ] Keyboard shortcut to create a new chat
- [ ] Notifications for Muse replies
- [ ] Custom themes / compact mode

## Contributing

Issues and pull requests are welcome. Keep changes small and explain the
"why" in the PR description.

## License

MIT — see [LICENSE](LICENSE). Open-Muse is not affiliated with Meta.
