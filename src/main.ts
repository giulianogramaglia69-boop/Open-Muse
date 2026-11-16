// Open-Muse frontend entry point.
//
// Intentionally empty: the app has NO frontend logic.
// The packaged app loads https://muse.ai directly as the main window URL
// (see src-tauri/tauri.conf.json), and every native feature — tray, global
// shortcut, single instance, menu, auto-start, window state — lives in
// Rust in src-tauri/src/main.rs. This file exists only because Vite needs
// an entry point for the placeholder page shown in `tauri dev`.
