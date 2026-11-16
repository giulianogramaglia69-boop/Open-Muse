//! Open-Muse — a native desktop client for Muse (Meta's personal AI agent).
//!
//! Muse has no public agent API, so v1 is a thin native shell around the
//! official web app: the main window loads <https://muse.ai> and the user
//! signs in with their own Meta account inside the app.
//!
//! Native features implemented here:
//! - System tray icon with Show/Hide, Reload, "Launch at login", Quit.
//! - Global shortcut CmdOrCtrl+Shift+M toggles the main window.
//! - Single instance: a second launch focuses the existing window.
//! - Window size/position persistence (tauri-plugin-window-state).
//! - Native menu: About, Reload, Toggle DevTools, Zoom in/out/reset, Quit.
//! - Auto-start toggle (tauri-plugin-autostart).

use std::sync::Mutex;

use tauri::{
    menu::MenuBuilder,
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, WebviewWindow,
};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_global_shortcut::ShortcutState;

// ---------------------------------------------------------------------------
// App state
// ---------------------------------------------------------------------------

/// Current page zoom factor (1.0 = 100%). Applied with a CSS `zoom` on `<html>`.
struct ZoomState(Mutex<f64>);

// ---------------------------------------------------------------------------
// Window helpers
// ---------------------------------------------------------------------------

/// Show the main window (if hidden or minimised) and bring it to the front.
fn show_window(window: &WebviewWindow) {
    let _ = window.show();
    let _ = window.unminimize();
    let _ = window.set_focus();
}

/// Toggle the main window between visible and hidden.
fn toggle_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        match window.is_visible() {
            Ok(true) => {
                let _ = window.hide();
            }
            _ => show_window(&window),
        }
    }
}

/// Apply a zoom factor to the loaded page via a CSS `zoom` on `<html>`.
fn apply_zoom(window: &WebviewWindow, factor: f64) {
    let _ = window.eval(&format!(
        "document.documentElement.style.zoom = '{}'",
        factor
    ));
}

// ---------------------------------------------------------------------------
// Tray menu
// ---------------------------------------------------------------------------

/// (Re)build the tray menu, reflecting the current auto-start state.
fn tray_menu(app: &AppHandle) -> tauri::menu::Menu<tauri::Wry> {
    let autostart_on = app.autolaunch().is_enabled().unwrap_or(false);
    let autostart_label = if autostart_on {
        "✓ Launch at login"
    } else {
        "Launch at login"
    };

    MenuBuilder::new(app)
        .text("show-hide", "Show / Hide")
        .text("reload", "Reload")
        .text("autostart", autostart_label)
        .separator()
        .text("quit", "Quit")
        .build()
        .expect("failed to build tray menu")
}

/// Refresh the tray menu after the auto-start state changed.
fn refresh_tray_menu(app: &AppHandle) {
    if let Some(tray) = app.tray_by_id("main") {
        let _ = tray.set_menu(Some(tray_menu(app)));
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() {
    tauri::Builder::default()
        // Zoom factor starts at 100%.
        .manage(ZoomState(Mutex::new(1.0)))
        // A second launch focuses the already-running window.
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                show_window(&window);
            }
        }))
        // Persists window size and position across restarts.
        .plugin(tauri_plugin_window_state::Builder::default().build())
        // Powers the "Launch at login" tray toggle.
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_dialog::init())
        // Global shortcut: CmdOrCtrl+Shift+M toggles the main window.
        // ("commandorcontrol" resolves to Cmd on macOS, Ctrl elsewhere.)
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_shortcuts(["commandorcontrol+shift+m"])
                .expect("invalid global shortcut")
                .with_handler(|app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        toggle_window(app);
                    }
                })
                .build(),
        )
        .setup(|app| {
            // Native menu bar (macOS app menu / Linux window menu).
            let menu = MenuBuilder::new(app)
                .text("about", "About Open-Muse")
                .separator()
                .text("reload", "Reload")
                .text("devtools", "Toggle DevTools")
                .separator()
                .text("zoom-in", "Zoom In")
                .text("zoom-out", "Zoom Out")
                .text("zoom-reset", "Reset Zoom")
                .separator()
                .text("quit", "Quit Open-Muse")
                .build()?;
            app.set_menu(menu)?;

            // System tray icon with its menu.
            // Left-click toggles the window; right-click opens the menu.
            // (Tauri 2.12 removed Image::from_bytes, so the PNG is decoded
            // to raw RGBA with the `image` crate and wrapped in Image::new_owned.)
            let decoded = image::load_from_memory(include_bytes!("../icons/icon.png"))
                .expect("failed to decode tray icon");
            let rgba = decoded.to_rgba8();
            let (icon_width, icon_height) = rgba.dimensions();
            let icon = tauri::image::Image::new_owned(rgba.into_raw(), icon_width, icon_height);
            let _tray = TrayIconBuilder::with_id("main")
                .icon(icon)
                .menu(&tray_menu(app.handle()))
                .tooltip("Open-Muse")
                .show_menu_on_left_click(true)
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        ..
                    } = event
                    {
                        toggle_window(tray.app_handle());
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_menu_event(|app, event| {
            let id = event.id().as_ref();
            let window = app.get_webview_window("main");
            match id {
                "show-hide" => toggle_window(app),
                "reload" => {
                    if let Some(w) = &window {
                        let _ = w.reload();
                    }
                }
                "devtools" => {
                    if let Some(w) = &window {
                        if w.is_devtools_open() {
                            w.close_devtools();
                        } else {
                            w.open_devtools();
                        }
                    }
                }
                "zoom-in" | "zoom-out" | "zoom-reset" => {
                    if let Some(w) = &window {
                        let state = app.state::<ZoomState>();
                        let mut factor = state.0.lock().unwrap();
                        *factor = match id {
                            "zoom-in" => (*factor + 0.1).min(2.0),
                            "zoom-out" => (*factor - 0.1).max(0.5),
                            _ => 1.0,
                        };
                        apply_zoom(w, *factor);
                    }
                }
                "autostart" => {
                    let autolaunch = app.autolaunch();
                    let enabled = autolaunch.is_enabled().unwrap_or(false);
                    let _ = if enabled {
                        autolaunch.disable()
                    } else {
                        autolaunch.enable()
                    };
                    refresh_tray_menu(app);
                }
                "about" => {
                    app.dialog()
                        .message(
                            "Open-Muse 0.1.0\n\nA native desktop client for Muse (Meta's personal AI agent).\n\nMuse has no public agent API yet, so Open-Muse wraps the official web app at muse.ai — sign in with your own Meta account inside the app.",
                        )
                        .title("About Open-Muse")
                        .show(|_| {});
                }
                "quit" => app.exit(0),
                _ => {}
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running Open-Muse");
}
