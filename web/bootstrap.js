import init from "./aedicule_web.js";
import {
	directionFor,
	formatMessage,
	loadCatalog,
	webGpuFailureMessage,
} from "./launcher-i18n.mjs";

const status = document.getElementById("aedicule-startup-status");
const catalog = loadCatalog(navigator.languages);
const strings = catalog.strings;
document.documentElement.lang = catalog.locale.replaceAll("_", "-");
document.documentElement.dir = directionFor(catalog.locale);
const MAX_APPLICATION_ASSETS = 1024;
const MAX_APPLICATION_ASSET_BYTES = 16 * 1024 * 1024;
const MAX_APPLICATION_BYTES = 64 * 1024 * 1024;
let audioContext;
let guestAudioPaused = false;
globalThis.__AEDICULE_AUDIO_REQUEST_COUNT = 0;

function reportStartupDiagnostic(stage, detail = {}) {
	console.info("[Aedicule startup]", stage, JSON.stringify(detail));
}

function showStartupStatus(message) {
	status.textContent = message;
}

function browserCapabilities() {
	return {
		secureContext: globalThis.isSecureContext === true,
		crossOriginIsolated: globalThis.crossOriginIsolated === true,
		sharedArrayBuffer: typeof globalThis.SharedArrayBuffer === "function",
		atomicsWaitAsync: typeof globalThis.Atomics?.waitAsync === "function",
		webGpu: typeof navigator.gpu?.requestAdapter === "function",
	};
}

function capabilitySummary(capabilities) {
	return Object.entries(capabilities)
		.map(([name, available]) => `${name}: ${available ? strings.yes : strings.no}`)
		.join("\n");
}

async function preflightBrowser(capabilities) {
	if (!capabilities.secureContext) {
		throw new Error(strings.requiresSecureContext);
	}
	if (!capabilities.crossOriginIsolated) {
		throw new Error(strings.requiresIsolation);
	}
	if (!capabilities.sharedArrayBuffer) {
		throw new Error(strings.requiresSharedMemory);
	}
	if (!capabilities.webGpu) {
		throw new Error(webGpuFailureMessage("missing", strings));
	}
	const adapter = await navigator.gpu.requestAdapter();
	if (adapter === null) {
		throw new Error(webGpuFailureMessage("adapter", strings));
	}
	reportStartupDiagnostic("webgpu-adapter", {
		featureCount: adapter.features.size,
		maxTextureDimension2D: adapter.limits.maxTextureDimension2D,
	});
}

function animationFrame() {
	return new Promise(resolve => requestAnimationFrame(resolve));
}

function canvasSnapshot() {
	return Array.from(document.querySelectorAll("canvas"), canvas => {
		const bounds = canvas.getBoundingClientRect();
		return {
			bufferWidth: canvas.width,
			bufferHeight: canvas.height,
			cssWidth: bounds.width,
			cssHeight: bounds.height,
		};
	});
}

function validAssetName(name) {
	return typeof name === "string"
		&& name.startsWith("assets/")
		&& !name.includes("\\")
		&& !name.includes(":")
		&& !name.includes("\0")
		&& name.normalize("NFC") === name
		&& name.split("/").every(component => component !== "" && component !== "." && component !== "..");
}

function encodedAssetUrl(name) {
	return `./${name.split("/").map(encodeURIComponent).join("/")}`;
}

async function loadApplicationAssets() {
	const response = await fetch("./application-assets.json", { cache: "no-store" });
	if (response.status === 404) {
		globalThis.__AEDICULE_ASSETS = Object.create(null);
		reportStartupDiagnostic("assets-fetched", { count: 0, bytes: 0 });
		return;
	}
	if (!response.ok) {
		throw new Error(formatMessage(strings.assetCatalogHttp, { status: response.status }));
	}
	const names = await response.json();
	if (!Array.isArray(names) || names.length > MAX_APPLICATION_ASSETS) {
		throw new Error(strings.assetCatalogEntryLimit);
	}
	const assets = Object.create(null);
	let totalBytes = 0;
	for (const name of names) {
		if (!validAssetName(name) || Object.hasOwn(assets, name)) {
			throw new Error(strings.assetCatalogInvalidPath);
		}
		const assetResponse = await fetch(encodedAssetUrl(name), { cache: "no-store" });
		if (!assetResponse.ok) {
			throw new Error(formatMessage(strings.couldNotLoadAsset, {
				name,
				status: assetResponse.status,
			}));
		}
		const bytes = new Uint8Array(await assetResponse.arrayBuffer());
		if (bytes.byteLength > MAX_APPLICATION_ASSET_BYTES) {
			throw new Error(formatMessage(strings.assetLimit, { name }));
		}
		totalBytes += bytes.byteLength;
		if (totalBytes > MAX_APPLICATION_BYTES) {
			throw new Error(strings.assetTotalLimit);
		}
		assets[name] = bytes;
	}
	globalThis.__AEDICULE_ASSETS = assets;
	reportStartupDiagnostic("assets-fetched", { count: names.length, bytes: totalBytes });
}

function ensureAudioContext() {
	if (audioContext !== undefined) return audioContext;
	const AudioContext = globalThis.AudioContext ?? globalThis.webkitAudioContext;
	if (AudioContext === undefined) return undefined;
	audioContext = new AudioContext();
	return audioContext;
}

function unlockAudio() {
	if (guestAudioPaused) return;
	const context = ensureAudioContext();
	if (context?.state === "suspended") {
		context.resume().catch(error => console.warn(strings.audioUnlockFailed, error));
	}
}

globalThis.__AEDICULE_SET_AUDIO_PAUSED = paused => {
	guestAudioPaused = Boolean(paused);
	const context = ensureAudioContext();
	if (context === undefined) return;
	const transition = guestAudioPaused ? context.suspend() : context.resume();
	transition.catch(error => console.warn(strings.audioUnlockFailed, error));
};

for (const eventName of ["pointerdown", "keydown", "touchstart"]) {
	addEventListener(eventName, unlockAudio, { capture: true, passive: true });
}

globalThis.__AEDICULE_PLAY_PCM = (sampleRate, channels, samples, volume, pitch) => {
	const context = ensureAudioContext();
	if (context === undefined) {
		console.warn(strings.audioUnavailable);
		return;
	}
	if (!Number.isInteger(sampleRate) || sampleRate <= 0
		|| !Number.isInteger(channels) || channels <= 0
		|| !(samples instanceof Float32Array) || samples.length % channels !== 0) {
		throw new TypeError(strings.invalidPcmRequest);
	}
	globalThis.__AEDICULE_AUDIO_REQUEST_COUNT += 1;
	const frames = samples.length / channels;
	const buffer = context.createBuffer(channels, frames, sampleRate);
	for (let channel = 0; channel < channels; channel += 1) {
		const output = buffer.getChannelData(channel);
		for (let frame = 0; frame < frames; frame += 1) {
			output[frame] = samples[frame * channels + channel];
		}
	}
	const source = context.createBufferSource();
	const gain = context.createGain();
	source.buffer = buffer;
	source.playbackRate.value = pitch;
	gain.gain.value = volume;
	source.connect(gain).connect(context.destination);
	const start = () => source.start();
	if (context.state === "suspended") {
		context.resume().then(start).catch(error => console.warn(strings.audioPlaybackFailed, error));
	} else {
		start();
	}
};

async function loadApplication() {
	await globalThis.__AEDICULE_ISOLATION_READY;
	reportStartupDiagnostic("bootstrap");
	showStartupStatus(strings.checkingCapabilities);
	const capabilities = browserCapabilities();
	reportStartupDiagnostic("capabilities", capabilities);
	await preflightBrowser(capabilities);
	showStartupStatus(strings.loadingWat);
	const response = await fetch("./code.wat", { cache: "no-store" });
	if (!response.ok) {
		throw new Error(formatMessage(strings.couldNotLoadWat, { status: response.status }));
	}

	const bytes = await response.arrayBuffer();
	reportStartupDiagnostic("wat-fetched", { status: response.status, bytes: bytes.byteLength });
	const source = new TextDecoder("utf-8", { fatal: true }).decode(bytes);
	globalThis.__AEDICULE_WAT = source;
	showStartupStatus(strings.loadingApplicationAssets);
	await loadApplicationAssets();
	showStartupStatus(strings.starting);
	reportStartupDiagnostic("wasm-initializing");
	await init();
	reportStartupDiagnostic("wasm-initialized");
	await animationFrame();
	reportStartupDiagnostic("first-animation-frame", { canvases: canvasSnapshot() });
	status.remove();
	setTimeout(() => {
		reportStartupDiagnostic("settled", { canvases: canvasSnapshot() });
	}, 1000);
}

function reportStartupFailure(error) {
	const capabilities = browserCapabilities();
	console.error("[Aedicule startup] failed", error, JSON.stringify(capabilities));
	status.dataset.state = "error";
	const detail = error instanceof Error ? error.message : String(error);
	status.textContent = `${strings.couldNotStart}\n\n${detail}\n\n${capabilitySummary(capabilities)}`;
}

loadApplication().catch(reportStartupFailure);
