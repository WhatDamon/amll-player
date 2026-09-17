//! macOS 应用菜单。
//!
//! macOS 上 Tauri 会自动创建一份默认菜单，其中的「关于」项由 muda 直接调用
//! `orderFrontStandardAboutPanel` 弹出原生关于面板，不会产生菜单事件，前端无法接管。
//! 因此这里显式构建一份菜单，把需要接管的项（关于/设置/检查更新/播放控制/帮助）
//! 做成普通菜单项，由前端路由到应用内的对应界面或动作。
//!
//! 菜单文案由前端随应用语言同步过来（见 [`update_app_menu`] 命令），
//! 尚未同步前回退到英文。

use serde::Deserialize;
use tauri::{
    AppHandle, Emitter, Manager, Runtime,
    menu::{
        HELP_SUBMENU_ID, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu, WINDOW_SUBMENU_ID,
    },
};
use tracing::warn;

/// 自定义菜单项 id 的前缀，只有带此前缀的菜单项会被转发给前端。
const MENU_ID_PREFIX: &str = "amll.";

/// 菜单项被点击后广播给前端的应用内事件名，负载为菜单项 id。
///
/// 自定义菜单项的 id 需与前端 `MENU_ACTION_IDS` 保持一致。
pub const MENU_ACTION_EVENT: &str = "app-menu:action";

/// macOS 应用菜单的文案，字段缺省时回退到英文。
///
/// 文案中的 `{appName}` 会被替换为应用名称。
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

/// 取用文案模板，缺省时回退到英文文案，并填入应用名称。
fn label(template: Option<&str>, fallback: &str, app_name: &str) -> String {
    template.unwrap_or(fallback).replace("{appName}", app_name)
}

/// 构建 macOS 应用菜单。
///
/// 结构与 Tauri 的 `Menu::default()` 保持一致（App / 文件 / 编辑 / 显示 / 窗口 / 帮助），
/// 只有「关于」是自定义菜单项，其余继续使用系统预定义项以保留原生的快捷键与行为。
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

    // macOS 惯例：「设置」紧跟在「关于」下方，快捷键 ⌘,
    let settings = MenuItem::with_id(
        app,
        "amll.settings",
        l(labels.settings.as_deref(), "Settings…"),
        true,
        Some("Cmd+,"),
    )?;

    // Sparkle 系应用惯例：「检查更新…」放在退出上方
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
            &check_update,
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

    // 「播放」菜单：动作全部交给前端，这里刻意不给快捷键，
    // 避免与 ShotcutContext 已注册的系统级全局快捷键（⌥⌘P / ⌥⌘← / ⌥⌘→）抢同一个键
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

    // 「窗口」与「帮助」菜单必须使用 Tauri 约定的 id，
    // 否则 `init_app_menu` 不会把它们注册给 AppKit（窗口列表、帮助搜索栏会失效）
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

    // macOS 的「帮助」菜单内容为空，搜索栏由系统补上；这里放项目相关的外部链接
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

/// 处理菜单点击事件，把带 [`MENU_ID_PREFIX`] 前缀的菜单项转发给前端。
pub fn handle_menu_event<R: Runtime>(app: &AppHandle<R>, event: MenuEvent) {
    let id = event.id().as_ref();
    if !id.starts_with(MENU_ID_PREFIX) {
        return;
    }

    // 菜单有可能在窗口被最小化或隐藏时被点击，先把主窗口带回前台
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }

    if let Err(err) = app.emit_to("main", MENU_ACTION_EVENT, id) {
        warn!("转发应用菜单事件失败: {err}");
    }
}

/// 按前端传入的文案重建应用菜单，随应用语言变化调用。
#[tauri::command]
pub fn update_app_menu(app: AppHandle, labels: MenuLabels) -> Result<(), String> {
    let menu = create_menu(&app, &labels).map_err(|err| err.to_string())?;
    app.set_menu(menu).map_err(|err| err.to_string())?;
    Ok(())
}
