/** 点击 macOS 应用菜单里的「关于」时，Tauri 侧发给主窗口的事件名。 */
export const ABOUT_MENU_EVENT = "app-menu:about";

/**
 * macOS 应用菜单的文案，缺省字段由 Tauri 侧回退到英文。
 *
 * 文案中的 `{appName}` 为应用名称占位符。
 */
export interface MenuLabels {
	about?: string;
	closeWindow?: string;
	copy?: string;
	cut?: string;
	edit?: string;
	file?: string;
	fullscreen?: string;
	help?: string;
	hide?: string;
	hideOthers?: string;
	minimize?: string;
	paste?: string;
	quit?: string;
	redo?: string;
	selectAll?: string;
	services?: string;
	undo?: string;
	view?: string;
	window?: string;
	zoom?: string;
}
