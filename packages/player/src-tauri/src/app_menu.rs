use serde::Deserialize;
use tauri::{
    AppHandle, Emitter, Manager, Runtime,
    menu::{
        HELP_SUBMENU_ID, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu, WINDOW_SUBMENU_ID,
    },
};
use tracing::warn;

/// Prefix of the menu item ids that are forwarded to the frontend.
const MENU_ID_PREFIX: &str = "amll.";

/// Emitted to the frontend when one of those items is clicked; the payload is the
/// menu item id, which has to match `MENU_ACTION_IDS` on the frontend side.
pub const MENU_ACTION_EVENT: &str = "app-menu:action";

/// Labels of the macOS application menu; missing fields fall back to English.
///
/// `{appName}` inside a label is replaced with the application name.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct MenuLabels {
    pub about: Option<String>,
    pub settings: Option<String>,
    pub services: Option<String>,
    pub hide: Option<String>,
    pub hide_others: Option<String>,
    pub quit: Option<String>,
    pub file: Option<String>,
    pub close_window: Option<String>,
    pub edit: Option<String>,
    pub undo: Option<String>,
    pub redo: Option<String>,
    pub cut: Option<String>,
    pub copy: Option<String>,
    pub paste: Option<String>,
    pub select_all: Option<String>,
    pub view: Option<String>,
    pub fullscreen: Option<String>,
    pub window: Option<String>,
    pub minimize: Option<String>,
    pub zoom: Option<String>,
    pub help: Option<String>,
    pub show_all: Option<String>,
    pub check_update: Option<String>,
    pub playback: Option<String>,
    pub play_pause: Option<String>,
    pub prev_song: Option<String>,
    pub next_song: Option<String>,
    pub cycle_repeat: Option<String>,
    pub toggle_shuffle: Option<String>,
    pub github_repo: Option<String>,
    pub report_issue: Option<String>,
}

/// Fills the application name into a label template, falling back to English.
fn label(template: Option<&str>, fallback: &str, app_name: &str) -> String {
    template.unwrap_or(fallback).replace("{appName}", app_name)
}

/// Builds the macOS application menu.
///
/// The structure mirrors Tauri's `Menu::default()` (App / File / Edit / View / Window /
/// Help), but every item that needs to reach the app is a plain menu item that emits
/// `MENU_ACTION_EVENT` instead of running a native selector. About has to be one of them:
/// muda's predefined About calls `orderFrontStandardAboutPanel` and never reports a menu
/// event, so it cannot be intercepted. All other items stay predefined to keep the native
/// accelerators and behaviour.
///
/// Labels are synced from the frontend (see `update_app_menu`) and fall back to English
/// until the first sync.
pub fn create_menu<R: Runtime>(app: &AppHandle<R>, labels: &MenuLabels) -> tauri::Result<Menu<R>> {
    let app_name = app.package_info().name.clone();
    let l = |template: Option<&str>, fallback: &str| label(template, fallback, &app_name);

    let about = MenuItem::with_id(
        app,
        "amll.about",
        l(labels.about.as_deref(), "About {appName}"),
        true,
        None::<&str>,
    )?;

    // macOS convention: Settings… sits right below About, with ⌘,
    let settings = MenuItem::with_id(
        app,
        "amll.settings",
        l(labels.settings.as_deref(), "Settings…"),
        true,
        Some("Cmd+,"),
    )?;

    // Application-level item, grouped with About and Settings
    let check_update = MenuItem::with_id(
        app,
        "amll.check-update",
        l(labels.check_update.as_deref(), "Check for Updates…"),
        true,
        None::<&str>,
    )?;

    let app_submenu = Submenu::with_items(
        app,
        &app_name,
        true,
        &[
            &about,
            &PredefinedMenuItem::separator(app)?,
            &settings,
            &check_update,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::services(app, Some(&l(labels.services.as_deref(), "Services")))?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::hide(app, Some(&l(labels.hide.as_deref(), "Hide {appName}")))?,
            &PredefinedMenuItem::hide_others(
                app,
                Some(&l(labels.hide_others.as_deref(), "Hide Others")),
            )?,
            &PredefinedMenuItem::show_all(app, Some(&l(labels.show_all.as_deref(), "Show All")))?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::quit(app, Some(&l(labels.quit.as_deref(), "Quit {appName}")))?,
        ],
    )?;

    let file_submenu = Submenu::with_items(
        app,
        l(labels.file.as_deref(), "File"),
        true,
        &[&PredefinedMenuItem::close_window(
            app,
            Some(&l(labels.close_window.as_deref(), "Close Window")),
        )?],
    )?;

    let edit_submenu = Submenu::with_items(
        app,
        l(labels.edit.as_deref(), "Edit"),
        true,
        &[
            &PredefinedMenuItem::undo(app, Some(&l(labels.undo.as_deref(), "Undo")))?,
            &PredefinedMenuItem::redo(app, Some(&l(labels.redo.as_deref(), "Redo")))?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::cut(app, Some(&l(labels.cut.as_deref(), "Cut")))?,
            &PredefinedMenuItem::copy(app, Some(&l(labels.copy.as_deref(), "Copy")))?,
            &PredefinedMenuItem::paste(app, Some(&l(labels.paste.as_deref(), "Paste")))?,
            &PredefinedMenuItem::select_all(
                app,
                Some(&l(labels.select_all.as_deref(), "Select All")),
            )?,
        ],
    )?;

    let view_submenu = Submenu::with_items(
        app,
        l(labels.view.as_deref(), "View"),
        true,
        &[&PredefinedMenuItem::fullscreen(
            app,
            Some(&l(labels.fullscreen.as_deref(), "Toggle Full Screen")),
        )?],
    )?;

    // Playback actions are dispatched to the frontend, so no accelerators here on purpose:
    // ShotcutContext already registers system-wide global shortcuts (⌥⌘P / ⌥⌘← / ⌥⌘→)
    // and AppKit key equivalents would fight over the same keys.
    let playback_submenu = Submenu::with_items(
        app,
        l(labels.playback.as_deref(), "Playback"),
        true,
        &[
            &MenuItem::with_id(
                app,
                "amll.play-pause",
                l(labels.play_pause.as_deref(), "Play/Pause"),
                true,
                None::<&str>,
            )?,
            &MenuItem::with_id(
                app,
                "amll.prev-song",
                l(labels.prev_song.as_deref(), "Previous Song"),
                true,
                None::<&str>,
            )?,
            &MenuItem::with_id(
                app,
                "amll.next-song",
                l(labels.next_song.as_deref(), "Next Song"),
                true,
                None::<&str>,
            )?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(
                app,
                "amll.cycle-repeat",
                l(labels.cycle_repeat.as_deref(), "Repeat"),
                true,
                None::<&str>,
            )?,
            &MenuItem::with_id(
                app,
                "amll.toggle-shuffle",
                l(labels.toggle_shuffle.as_deref(), "Shuffle"),
                true,
                None::<&str>,
            )?,
        ],
    )?;

    // These two submenus have to keep Tauri's reserved ids, otherwise `init_app_menu` does
    // not hand them to AppKit and the window list / Help search field disappear.
    let window_submenu = Submenu::with_id_and_items(
        app,
        WINDOW_SUBMENU_ID,
        l(labels.window.as_deref(), "Window"),
        true,
        &[
            &PredefinedMenuItem::minimize(app, Some(&l(labels.minimize.as_deref(), "Minimize")))?,
            &PredefinedMenuItem::maximize(app, Some(&l(labels.zoom.as_deref(), "Zoom")))?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::close_window(
                app,
                Some(&l(labels.close_window.as_deref(), "Close Window")),
            )?,
            &PredefinedMenuItem::bring_all_to_front(app, None::<&str>)?,
        ],
    )?;

    // On macOS the Help menu starts out empty and the system adds the search field,
    // so it is used for the project links.
    let help_submenu = Submenu::with_id_and_items(
        app,
        HELP_SUBMENU_ID,
        l(labels.help.as_deref(), "Help"),
        true,
        &[
            &MenuItem::with_id(
                app,
                "amll.github-repo",
                l(labels.github_repo.as_deref(), "GitHub Repository"),
                true,
                None::<&str>,
            )?,
            &MenuItem::with_id(
                app,
                "amll.report-issue",
                l(labels.report_issue.as_deref(), "Report an Issue"),
                true,
                None::<&str>,
            )?,
        ],
    )?;

    Menu::with_items(
        app,
        &[
            &app_submenu,
            &file_submenu,
            &edit_submenu,
            &view_submenu,
            &playback_submenu,
            &window_submenu,
            &help_submenu,
        ],
    )
}

/// Handles menu clicks and forwards the custom items to the frontend.
pub fn handle_menu_event<R: Runtime>(app: &AppHandle<R>, event: MenuEvent) {
    let id = event.id().as_ref();
    if !id.starts_with(MENU_ID_PREFIX) {
        return;
    }

    // The menu can be clicked while the main window is hidden or minimised, so bring it
    // back to the front first.
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }

    if let Err(err) = app.emit_to("main", MENU_ACTION_EVENT, id) {
        warn!("转发应用菜单事件失败: {err}");
    }
}

/// Rebuilds the application menu with the labels supplied by the frontend, called whenever
/// the application language changes.
#[tauri::command]
pub fn update_app_menu(app: AppHandle, labels: MenuLabels) -> Result<(), String> {
    let menu = create_menu(&app, &labels).map_err(|err| err.to_string())?;
    app.set_menu(menu).map_err(|err| err.to_string())?;
    Ok(())
}
