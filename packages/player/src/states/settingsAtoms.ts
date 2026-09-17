import { atom } from "jotai";

/**
 * 设置页当前显示的标签，形如 `player.general` / `player.about` / `extension.<id>`。
 *
 * 放在这里而不是设置页内部，是为了让设置页之外的地方（例如 macOS 应用菜单、
 * 深度链接）也能在不静态引入懒加载的设置页的前提下切换标签。
 */
export const settingsPageAtom = atom("player.general");
