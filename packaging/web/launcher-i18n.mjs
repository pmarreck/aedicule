export const supportedLocales = Object.freeze([
	"am", "ar", "az", "bg", "bn", "bs", "da", "de", "el", "en",
	"es", "fa", "fi", "fil", "fr", "ha", "he", "hi", "hr", "hu",
	"id", "ig", "is", "it", "ja", "km", "ko", "mk", "nb", "nl",
	"pa", "pl", "ps", "pt_br", "ro", "ru", "sl", "sq", "sr", "sv",
	"sw", "ta", "th", "tr", "uk", "ur", "vi", "yo", "zh_hans", "zh_hant",
]);

export const english = Object.freeze({
	title: "Aedicule",
	intro: "Two tiny WAT worlds. One portable frontplane.",
	ulamName: "Ulam Flower",
	ulamDescription: "Exact arithmetic, recursive motion.",
	vibesteroidsName: "Vibesteroids",
	vibesteroidsDescription: "Vector arcade action with generated audio.",
	open: "Launch",
	vibesteroidsPackage: "Download Vibesteroids .aed",
	downloads: "Download apps",
	source: "Source",
	assetCatalogEntryLimit: "Aedicule asset catalog exceeds its entry limit.",
	assetCatalogHttp: "Aedicule could not load the asset catalog: HTTP {status}",
	assetCatalogInvalidPath: "Aedicule asset catalog contains an invalid or duplicate path.",
	assetLimit: "Aedicule package asset exceeds its byte limit: {name}",
	assetTotalLimit: "Aedicule package assets exceed their total byte limit.",
	audioPlaybackFailed: "[Aedicule audio] playback failed",
	audioUnavailable: "[Aedicule audio] Web Audio is unavailable",
	audioUnlockFailed: "[Aedicule audio] unlock failed",
	checkingCapabilities: "Checking browser capabilities…",
	couldNotLoadAsset: "Aedicule could not load package asset {name}: HTTP {status}",
	couldNotLoadWat: "Aedicule could not load code.wat: HTTP {status}",
	couldNotStart: "Aedicule could not start.",
	invalidPcmRequest: "invalid Aedicule PCM playback request",
	loadingApplicationAssets: "Loading application assets…",
	loadingWat: "Loading WAT application…",
	no: "no",
	requiresIsolation: "Aedicule requires cross-origin isolation for shared WebAssembly memory.",
	requiresSecureContext: "Aedicule requires a secure HTTPS browser context.",
	requiresSharedMemory: "This browser does not expose shared WebAssembly memory.",
	requiresWebGpu: "This browser does not expose WebGPU.",
	starting: "Starting Aedicule…",
	unusableWebGpu: "WebGPU is present, but the browser did not provide a usable GPU adapter.",
	webGpuFirefoxHelp: "Firefox: open about:config, set dom.webgpu.enabled to true, then reload. Details: https://developer.mozilla.org/en-US/docs/Mozilla/Firefox/Experimental_features#webgpu_api",
	webGpuChromeHelp: "Chrome/Chromium: update first, enable “Use graphics acceleration when available” at chrome://settings/system, then inspect chrome://gpu. Advanced unsupported/blocklisted systems can try chrome://flags/#enable-unsafe-webgpu (plus #enable-vulkan on Linux). Details: https://developer.chrome.com/docs/web-platform/webgpu/troubleshooting-tips",
	webGpuSafariHelp: "Safari: update to Safari 26 or newer. In Safari Technology Preview, enable WebGPU, GPU Process: DOM Rendering, and GPU Process: Canvas Rendering under Settings → Feature Flags. Details: https://webkit.org/blog/17333/webkit-features-in-safari-26-0/#webgpu",
	webGpuRetryHelp: "Reload this page first. Restart the browser only if the change does not take effect; if WebGPU is still unusable, update the browser, OS, and GPU driver.",
	yes: "yes",
});

const rtlLocales = new Set(["ar", "he", "fa", "ps", "ur"]);
const catalogs = Object.freeze({ en: english });

/**
 * Resolves BCP-47-like browser preferences with longest locale matches and
 * deterministic Chinese region-to-script folding.
 */
export function resolveLocale(languages) {
	for (const language of languages) {
		const normalized = language.toLowerCase().replaceAll("-", "_");
		const parts = normalized.split("_");
		if (parts[0] === "zh") {
			if (parts.includes("hant")) return "zh_hant";
			if (parts.includes("hans")) return "zh_hans";
			if (parts.some(part => ["tw", "hk", "mo"].includes(part))) return "zh_hant";
			return "zh_hans";
		}
		for (const locale of [...supportedLocales].sort((left, right) => right.length - left.length)) {
			if (normalized === locale || normalized.startsWith(`${locale}_`)) return locale;
		}
	}
	return "en";
}

export function directionFor(locale) {
	return rtlLocales.has(locale) ? "rtl" : "ltr";
}

/**
 * Formats named message fields while rejecting incomplete catalogs instead
 * of leaking implementation placeholders into a user-visible diagnostic.
 */
export function formatMessage(message, values) {
	return message.replaceAll(/\{([^{}]+)\}/g, (_placeholder, name) => {
		if (!Object.hasOwn(values, name)) throw new Error(`unknown message value: ${name}`);
		return String(values[name]);
	});
}

/**
 * Keeps browser-owned WebGPU recovery steps beside the capability failure so
 * a blank/black page never forces visitors to search for hidden preferences.
 */
export function webGpuFailureMessage(kind, strings) {
	const heading = {
		adapter: strings.unusableWebGpu,
		missing: strings.requiresWebGpu,
	}[kind];
	if (heading === undefined) throw new Error(`unknown WebGPU failure kind: ${kind}`);
	return [
		heading,
		"",
		strings.webGpuFirefoxHelp,
		"",
		strings.webGpuChromeHelp,
		"",
		strings.webGpuSafariHelp,
		"",
		strings.webGpuRetryHelp,
	].join("\n");
}

/**
 * Keeps prepare-phase omissions visible to developers while returning the
 * complete English catalog until independently reviewed translations exist.
 */
export function loadCatalog(languages, warn = warning => console.warn(warning)) {
	const requested = resolveLocale(languages);
	if (catalogs[requested]) return { locale: requested, strings: catalogs[requested] };
	warn(`WARN: i18n missing-locale ${requested}`);
	return { locale: "en", strings: english };
}

export function localizeDocument(document, languages) {
	const catalog = loadCatalog(languages);
	document.documentElement.lang = catalog.locale.replaceAll("_", "-");
	document.documentElement.dir = directionFor(catalog.locale);
	for (const element of document.querySelectorAll("[data-i18n]")) {
		element.textContent = catalog.strings[element.dataset.i18n];
	}
	document.title = catalog.strings.title;
}
