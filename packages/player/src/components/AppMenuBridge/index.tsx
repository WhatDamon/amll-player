import { isLyricPageOpenedAtom } from "@applemusic-like-lyrics/react-full";
import { getName } from "@tauri-apps/api/app";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { platform } from "@tauri-apps/plugin-os";
import { useSetAtom } from "jotai";
import { type FC, useEffect } from "react";
import i18n from "../../i18n.ts";
import { router } from "../../router.tsx";
import { settingsPageAtom } from "../../states/settingsAtoms.ts";
import {
	ABOUT_MENU_EVENT,
	type MenuLabels,
	SETTINGS_MENU_EVENT,
} from "./types.ts";

/** 应用菜单只由 macOS 上的 Tauri 侧创建。 */
const isMacos = () => platform() === "macos";

/** 应用名称在运行期不会变化，缓存一次即可。 */
let appNamePromise: Promise<string> | undefined;
const getAppName = () => (appNamePromise ??= getName());

/** 按当前语言收集菜单文案并同步给 Tauri 侧，未同步前菜单显示英文。 */
const syncMenuLabels = async () => {
	try {
		const appName = await getAppName();
		const labels: MenuLabels = {
			about: i18n.t("menu.about", { appName }),
			closeWindow: i18n.t("menu.closeWindow"),
			copy: i18n.t("menu.copy"),
			cut: i18n.t("menu.cut"),
			edit: i18n.t("menu.edit"),
			file: i18n.t("menu.file"),
			fullscreen: i18n.t("menu.fullscreen"),
			help: i18n.t("menu.help"),
			hide: i18n.t("menu.hide", { appName }),
			hideOthers: i18n.t("menu.hideOthers"),
			minimize: i18n.t("menu.minimize"),
			paste: i18n.t("menu.paste"),
			quit: i18n.t("menu.quit", { appName }),
			redo: i18n.t("menu.redo"),
			selectAll: i18n.t("menu.selectAll"),
			services: i18n.t("menu.services"),
			settings: i18n.t("menu.settings"),
			undo: i18n.t("menu.undo"),
			view: i18n.t("menu.view"),
			window: i18n.t("menu.window"),
			zoom: i18n.t("menu.zoom"),
		};
		await invoke("update_app_menu", { labels });
	} catch (err) {
		console.error("同步 macOS 应用菜单失败:", err);
	}
};

/**
 * 打通 macOS 应用菜单与前端界面：
 * 菜单文案跟随应用语言，菜单里的「关于」「设置」改为进入应用内的对应界面而不是原生功能。
 */
export const AppMenuBridge: FC = () => {
	const setSettingsPage = useSetAtom(settingsPageAtom);
	const setLyricPageOpened = useSetAtom(isLyricPageOpenedAtom);

	useEffect(() => {
		if (!isMacos()) return;

		/** 跳到应用内设置页；`page` 缺省时保留上次停留的标签，与侧边栏入口行为一致。 */
		const openSettings = (page?: string) => {
			if (page) setSettingsPage(page);
			// 歌词页是全屏遮罩，不先关掉就会盖住要跳转的设置页
			setLyricPageOpened(false);
			// 已经在设置页时不再压入历史记录，否则关闭设置页要按很多次返回键
			if (router.state.location.pathname !== "/settings") {
				router.navigate("/settings").catch(console.error);
			}
		};

		const unlistens = [
			listen(ABOUT_MENU_EVENT, () => openSettings("player.about")),
			listen(SETTINGS_MENU_EVENT, () => openSettings()),
		];

		return () => {
			for (const unlisten of unlistens) {
				unlisten
					.then((off) => off())
					.catch((err) => console.error("取消监听应用菜单事件失败:", err));
			}
		};
	}, [setLyricPageOpened, setSettingsPage]);

	useEffect(() => {
		if (!isMacos()) return;

		syncMenuLabels();
		i18n.on("languageChanged", syncMenuLabels);

		return () => {
			i18n.off("languageChanged", syncMenuLabels);
		};
	}, []);

	return null;
};
