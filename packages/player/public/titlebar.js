"use-strict";
{
	const titlebar = document.getElementById("system-titlebar");
	const closeBtn = document.getElementById("system-titlebar-close");
	const minimizeBtn = document.getElementById("system-titlebar-minimize");
	const resizeBtn = document.getElementById("system-titlebar-resize");
	closeBtn.addEventListener("click", () => {
		window.dispatchEvent(new Event("on-system-titlebar-click-close"));
	});
	minimizeBtn.addEventListener("click", () => {
		window.dispatchEvent(new Event("on-system-titlebar-click-minimize"));
	});
	resizeBtn.addEventListener("click", () => {
		window.dispatchEvent(new Event("on-system-titlebar-click-resize"));
	});
	window.SystemTitlebarAppearance = {
		Windows: "windows",
		MacOS: "macos",
		Hidden: "hidden",
	};
	window.SystemTitlebarResizeAppearance = {
		Restore: "restore",
		Maximize: "maximize",
	};
	let currentAppearance = localStorage.getItem("system-titlebar-appearance");

	const updateTitlebarVariable = () => {
		document.body.style.setProperty(
			"--system-titlebar-height",
			`${titlebar.clientHeight}px`,
		);
	};

	const titlebarObz = new ResizeObserver(updateTitlebarVariable);
	titlebarObz.observe(titlebar);

	window.setSystemTitlebarAppearance = function setSystemTitlebarAppearance(
		appearance,
	) {
		currentAppearance = appearance;
		localStorage.setItem("system-titlebar-appearance", appearance);
		titlebar.classList.remove("mac-os", "windows");
		if (appearance === SystemTitlebarAppearance.Windows) {
			titlebar.classList.add("windows");
		} else if (appearance === SystemTitlebarAppearance.MacOS) {
			titlebar.classList.add("mac-os");
		}
		updateTitlebarVariable();
	};

	window.setSystemTitlebarResizeAppearance =
		function setSystemTitlebarResizeAppearance(appearance) {
			const resizeButton = document.getElementById("system-titlebar-resize");
			resizeButton.classList.remove("maximize", "restore");
			resizeButton.classList.add(appearance);
		};

	window.setSystemTitlebarImmersiveMode =
		function setSystemTitlebarImmersiveMode(enabled) {
			if (enabled) {
				titlebar.classList.add("immersive");
			} else {
				titlebar.classList.remove("immersive");
			}
		};

	window.setSystemTitlebarFullscreen = function setSystemTitlebarFullscreen(
		enabled,
	) {
		if (enabled) {
			titlebar.classList.add("fullscreen");
		} else {
			titlebar.classList.remove("fullscreen");
		}
	};

	const initPlatform = () => {
		const userAgent = navigator.userAgent;
		if (userAgent.includes("Windows") || userAgent.includes("Linux")) {
			setSystemTitlebarAppearance(SystemTitlebarAppearance.Windows);
		} else if (userAgent.includes("Macintosh")) {
			setSystemTitlebarAppearance(SystemTitlebarAppearance.MacOS);
		} else {
			setSystemTitlebarAppearance(SystemTitlebarAppearance.Hidden);
		}
	};

	if (navigator.userAgent.includes("Android")) {
		setSystemTitlebarAppearance(SystemTitlebarAppearance.Hidden);
	} else if (currentAppearance) {
		setSystemTitlebarAppearance(currentAppearance);
	} else {
		initPlatform();
	}

	setSystemTitlebarResizeAppearance(SystemTitlebarResizeAppearance.Maximize);
}
