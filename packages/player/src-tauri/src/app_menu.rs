//! macOS 应用菜单。
//!
//! macOS 上 Tauri 会自动创建一份默认菜单，其中的「关于」项由 muda 直接调用
//! `orderFrontStandardAboutPanel` 弹出原生关于面板，不会产生菜单事件，前端无法接管。
//! 因此这里显式构建一份与默认菜单结构等价的菜单，把「关于」「设置」换成普通菜单项，
//! 由前端路由到应用内的对应界面。
//!
//! 菜单文案由前端随应用语言同步过来（见 [`update_app_menu`] 命令），
//! 尚未同步前回退到英文。

use serde::Deserialize;
use tauri::{
    AppHandle, Emitter, Manager, Runtime,
    menu::{
        HELP_SUBMENU_ID, IsMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu,
        WINDOW_SUBMENU_ID,
    },
};
use tracing::warn;

/// 自定义「关于」菜单项的 id。
pub const ABOUT_MENU_ID: &str = "amll.about";

/// 「关于」被点击后广播给前端的应用内事件名。
pub const ABOUT_MENU_EVENT: &str = "app-menu:about";

/// 自定义「设置」菜单项的 id。
pub const SETTINGS_MENU_ID: &str = "amll.settings";

/// 「设置」被点击后广播给前端的应用内事件名。
pub const SETTINGS_MENU_EVENT: &str = "app-menu:settings";

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
        ABOUT_MENU_ID,
        l(labels.about.as_deref(), "About {appName}"),
        true,
        None::<&str>,
    )?;

    // macOS 惯例：「设置」紧跟在「关于」下方，快捷键 ⌘,
    let settings = MenuItem::with_id(
        app,
        SETTINGS_MENU_ID,
        l(labels.settings.as_deref(), "Settings…"),
        true,
        Some("Cmd+,"),
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
        ],
    )?;

    // 与默认菜单一致，macOS 的「帮助」菜单内容为空，搜索栏由系统补上
    let help_submenu = Submenu::with_id_and_items(
        app,
        HELP_SUBMENU_ID,
        l(labels.help.as_deref(), "Help"),
        true,
        &[] as &[&dyn IsMenuItem<R>],
    )?;

    Menu::with_items(
        app,
        &[
            &app_submenu,
            &file_submenu,
            &edit_submenu,
            &view_submenu,
            &window_submenu,
            &help_submenu,
        ],
    )
}

/// 处理菜单点击事件，把「关于」「设置」转发给前端。
pub fn handle_menu_event<R: Runtime>(app: &AppHandle<R>, event: MenuEvent) {
    let event_name = match event.id().as_ref() {
        ABOUT_MENU_ID => ABOUT_MENU_EVENT,
        SETTINGS_MENU_ID => SETTINGS_MENU_EVENT,
        _ => return,
    };

    // 菜单有可能在窗口被最小化或隐藏时被点击，先把主窗口带回前台
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }

    if let Err(err) = app.emit_to("main", event_name, ()) {
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
