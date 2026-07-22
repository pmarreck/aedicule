export const supportedLocales = Object.freeze([
	"am", "ar", "az", "bg", "bn", "bs", "da", "de", "el", "en",
	"es", "fa", "fi", "fil", "fr", "ha", "he", "hi", "hr", "hu",
	"id", "ig", "is", "it", "ja", "km", "ko", "mk", "nb", "nl",
	"pa", "pl", "ps", "pt_br", "ro", "ru", "sl", "sq", "sr", "sv",
	"sw", "ta", "th", "tr", "uk", "ur", "vi", "yo", "zh_hans", "zh_hant",
]);

export const english = Object.freeze({
	title: "Aedicule demos",
	intro: "Choose a portable WebAssembly Text application.",
	ulamName: "Ulam Flower",
	ulamDescription: "An exact-arithmetic recursive complex walk with guest-owned interactive controls.",
	vibesteroidsName: "Vibesteroids",
	vibesteroidsDescription: "A deterministic vector-space arcade application with generated audio.",
	open: "Open demo",
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
