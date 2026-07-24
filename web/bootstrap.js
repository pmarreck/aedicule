import init from "./aedicule_web.js";
import {
	directionFor,
	formatMessage,
	loadCatalog,
	webGpuFailureMessage,
} from "./launcher-i18n.mjs";
import { schedulePcmPlayback } from "./audio.mjs";
import { loadLocalApplication } from "./local-application.mjs";
import { withExclusiveStartupLock } from "./startup-lock.mjs";

const status = document.getElementById("aedicule-startup-status");
const catalog = loadCatalog(navigator.languages);
const strings = catalog.strings;
document.documentElement.lang = catalog.locale.replaceAll("_", "-");
document.documentElement.dir = directionFor(catalog.locale);
const MAX_APPLICATION_ASSETS = 1024;
const MAX_APPLICATION_ASSET_BYTES = 16 * 1024 * 1024;
const MAX_APPLICATION_BYTES = 64 * 1024 * 1024;
const MAX_STARTUP_DIAGNOSTICS = 128;
const MAX_AUDIO_DIAGNOSTICS = 128;
const earlyDiagnostics = Array.isArray(globalThis.__AEDICULE_EARLY_DIAGNOSTICS)
	? globalThis.__AEDICULE_EARLY_DIAGNOSTICS
	: [];
globalThis.__AEDICULE_STARTUP_TIMELINE = earlyDiagnostics.slice(-MAX_STARTUP_DIAGNOSTICS);
globalThis.__AEDICULE_RUNTIME_DIAGNOSTIC = {
	framesStarted: 0,
	framesCompleted: 0,
	inProgress: false,
	lastStartedMs: null,
	lastCompletedMs: null,
	lastDurationMs: null,
	lastGuestElapsedMs: null,
	lastBackground: null,
};
globalThis.__AEDICULE_AUDIO_DIAGNOSTIC = {
	requestCount: 0,
	timeline: [],
};
let audioContext;
let guestAudioPaused = false;
let startupStatusMessage = status.textContent;
globalThis.__AEDICULE_AUDIO_REQUEST_COUNT = 0;

function reportStartupDiagnostic(stage, detail = {}) {
	detail = {
		...detail,
		elapsedMs: Math.round(performance.now()),
	};
	globalThis.__AEDICULE_STARTUP_TIMELINE.push({ stage, ...detail });
	if (globalThis.__AEDICULE_STARTUP_TIMELINE.length > MAX_STARTUP_DIAGNOSTICS) {
		globalThis.__AEDICULE_STARTUP_TIMELINE.shift();
	}
	console.info("[Aedicule startup]", stage, JSON.stringify(detail));
	if (status.isConnected) {
		status.textContent = `${startupStatusMessage}\n\n${stage} · ${detail.elapsedMs} ms`;
	}
}

function showStartupStatus(message) {
	startupStatusMessage = message;
	if (!status.isConnected) return;
	const latest = globalThis.__AEDICULE_STARTUP_TIMELINE.at(-1);
	status.textContent = latest === undefined
		? message
		: `${message}\n\n${latest.stage} · ${latest.elapsedMs} ms`;
}

function browserEnvironment() {
	return {
		userAgent: navigator.userAgent,
		platform: navigator.userAgentData?.platform ?? navigator.platform,
		visibilityState: document.visibilityState,
		documentFocused: document.hasFocus(),
		serviceWorkerControlled: navigator.serviceWorker?.controller != null,
		navigationType: performance.getEntriesByType("navigation")[0]?.type ?? "unknown",
	};
}

function reportAudioDiagnostic(stage, detail = {}) {
	const diagnostic = globalThis.__AEDICULE_AUDIO_DIAGNOSTIC;
	const entry = {
		stage,
		elapsedMs: Math.round(performance.now()),
		contextState: audioContext?.state ?? "unavailable",
		userActivationActive: navigator.userActivation?.isActive ?? null,
		userActivationSeen: navigator.userActivation?.hasBeenActive ?? null,
		...detail,
	};
	diagnostic.timeline.push(entry);
	if (diagnostic.timeline.length > MAX_AUDIO_DIAGNOSTICS) diagnostic.timeline.shift();
	diagnostic.last = entry;
	console.info("[Aedicule audio]", stage, JSON.stringify(entry));
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
	reportStartupDiagnostic("webgpu-adapter-request", browserEnvironment());
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

globalThis.__AEDICULE_REPORT_FRAME_STARTED = guestElapsedMs => {
	const diagnostic = globalThis.__AEDICULE_RUNTIME_DIAGNOSTIC;
	diagnostic.framesStarted += 1;
	diagnostic.inProgress = true;
	diagnostic.lastStartedMs = performance.now();
	diagnostic.lastGuestElapsedMs = guestElapsedMs;
	if (diagnostic.framesStarted === 1) {
		reportStartupDiagnostic("runtime-frame-started", {
			frame: diagnostic.framesStarted,
			guestElapsedMs,
		});
	}
};

globalThis.__AEDICULE_REPORT_FRAME_COMPLETED = (guestElapsedMs, background) => {
	const diagnostic = globalThis.__AEDICULE_RUNTIME_DIAGNOSTIC;
	const completedAt = performance.now();
	diagnostic.framesCompleted += 1;
	diagnostic.inProgress = false;
	diagnostic.lastCompletedMs = completedAt;
	diagnostic.lastDurationMs = diagnostic.lastStartedMs === null
		? null
		: completedAt - diagnostic.lastStartedMs;
	diagnostic.lastGuestElapsedMs = guestElapsedMs;
	diagnostic.lastBackground = background;
	if (diagnostic.framesCompleted === 1 || diagnostic.framesCompleted % 300 === 0) {
		reportStartupDiagnostic("runtime-frame-completed", {
			frame: diagnostic.framesCompleted,
			durationMs: diagnostic.lastDurationMs,
			guestElapsedMs,
			background,
		});
	}
};

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

function admitLocalApplicationAssets(entries) {
	if (!Array.isArray(entries) || entries.length > MAX_APPLICATION_ASSETS) {
		throw new Error(strings.assetCatalogEntryLimit);
	}
	const assets = Object.create(null);
	let totalBytes = 0;
	for (const { name, bytes } of entries) {
		if (!validAssetName(name) || Object.hasOwn(assets, name)) {
			throw new Error(strings.assetCatalogInvalidPath);
		}
		if (!(bytes instanceof Uint8Array) || bytes.byteLength > MAX_APPLICATION_ASSET_BYTES) {
			throw new Error(formatMessage(strings.assetLimit, { name }));
		}
		totalBytes += bytes.byteLength;
		if (totalBytes > MAX_APPLICATION_BYTES) throw new Error(strings.assetTotalLimit);
		assets[name] = bytes;
	}
	globalThis.__AEDICULE_ASSETS = assets;
	reportStartupDiagnostic("assets-fetched", {
		count: entries.length,
		bytes: totalBytes,
		source: "local-browser-storage",
	});
}

function ensureAudioContext() {
	if (audioContext !== undefined) return audioContext;
	const AudioContext = globalThis.AudioContext ?? globalThis.webkitAudioContext;
	if (AudioContext === undefined) {
		reportAudioDiagnostic("context-unavailable");
		return undefined;
	}
	try {
		audioContext = new AudioContext();
		reportAudioDiagnostic("context-created", {
			sampleRate: audioContext.sampleRate,
		});
		audioContext.addEventListener("statechange", () => {
			reportAudioDiagnostic("context-state-changed");
		});
	} catch (error) {
		reportAudioDiagnostic("context-failed", {
			errorName: error?.name ?? typeof error,
			errorMessage: error?.message ?? String(error),
		});
		throw error;
	}
	return audioContext;
}

async function resumeAudioContext(context, reason) {
	reportAudioDiagnostic("context-resume-requested", { reason });
	try {
		await context.resume();
		reportAudioDiagnostic("context-resumed", { reason });
	} catch (error) {
		reportAudioDiagnostic("context-resume-failed", {
			reason,
			errorName: error?.name ?? typeof error,
			errorMessage: error?.message ?? String(error),
		});
		throw error;
	}
}

function unlockAudio(event) {
	reportAudioDiagnostic("unlock-event", { eventType: event.type });
	if (guestAudioPaused) return;
	const context = ensureAudioContext();
	if (context?.state === "suspended") {
		resumeAudioContext(context, "user-activation")
			.catch(error => console.warn(strings.audioUnlockFailed, error));
	}
}

globalThis.__AEDICULE_SET_AUDIO_PAUSED = paused => {
	guestAudioPaused = Boolean(paused);
	const context = ensureAudioContext();
	if (context === undefined) return;
	const transition = guestAudioPaused
		? context.suspend()
		: resumeAudioContext(context, "guest-resumed");
	transition.catch(error => console.warn(strings.audioUnlockFailed, error));
};

for (const eventName of ["pointerdown", "keydown", "touchstart"]) {
	addEventListener(eventName, unlockAudio, { capture: true, passive: true });
}

globalThis.__AEDICULE_PLAY_PCM = (sampleRate, channels, samples, volume, pitch) => {
	reportAudioDiagnostic("pcm-requested", {
		sampleRate,
		channels,
		sampleCount: samples.length,
		volume,
		pitch,
	});
	const context = ensureAudioContext();
	if (context === undefined) {
		reportAudioDiagnostic("source-failed", { reason: "audio-context-unavailable" });
		console.warn(strings.audioUnavailable);
		return;
	}
	globalThis.__AEDICULE_AUDIO_REQUEST_COUNT += 1;
	globalThis.__AEDICULE_AUDIO_DIAGNOSTIC.requestCount
		= globalThis.__AEDICULE_AUDIO_REQUEST_COUNT;
	schedulePcmPlayback(
		context,
		{ sampleRate, channels, samples, volume, pitch },
		{
			report: reportAudioDiagnostic,
			resume: candidate => resumeAudioContext(candidate, "pcm-playback"),
			invalidRequestMessage: strings.invalidPcmRequest,
		},
	).catch(error => {
		reportAudioDiagnostic("source-failed", {
			errorName: error?.name ?? typeof error,
			errorMessage: error?.message ?? String(error),
		});
		console.warn(strings.audioPlaybackFailed, error);
	});
};

async function initializeApplication() {
	reportStartupDiagnostic("bootstrap", browserEnvironment());
	showStartupStatus(strings.checkingCapabilities);
	const capabilities = browserCapabilities();
	reportStartupDiagnostic("capabilities", { ...capabilities, ...browserEnvironment() });
	await preflightBrowser(capabilities);
	showStartupStatus(strings.loadingWat);
	const localToken = new URL(location.href).searchParams.get("local");
	if (localToken === null) {
		const response = await fetch("./code.wat", { cache: "no-store" });
		if (!response.ok) {
			throw new Error(formatMessage(strings.couldNotLoadWat, { status: response.status }));
		}
		const bytes = await response.arrayBuffer();
		reportStartupDiagnostic("wat-fetched", { status: response.status, bytes: bytes.byteLength });
		globalThis.__AEDICULE_WAT = new TextDecoder("utf-8", { fatal: true }).decode(bytes);
		showStartupStatus(strings.loadingApplicationAssets);
		await loadApplicationAssets();
	} else {
		const application = await loadLocalApplication(localToken);
		reportStartupDiagnostic("wat-fetched", {
			status: "local",
			bytes: application.wat.byteLength,
		});
		globalThis.__AEDICULE_WAT = new TextDecoder("utf-8", { fatal: true })
			.decode(application.wat);
		showStartupStatus(strings.loadingApplicationAssets);
		admitLocalApplicationAssets(application.assets);
	}
	showStartupStatus(strings.starting);
	reportStartupDiagnostic("wasm-initializing");
	await init();
	reportStartupDiagnostic("wasm-initialized");
	while (globalThis.__AEDICULE_RUNTIME_DIAGNOSTIC.framesCompleted === 0) {
		await animationFrame();
	}
	reportStartupDiagnostic("first-animation-frame", {
		canvases: canvasSnapshot(),
		runtime: { ...globalThis.__AEDICULE_RUNTIME_DIAGNOSTIC },
	});
	status.remove();
	setTimeout(() => {
		reportStartupDiagnostic("settled", { canvases: canvasSnapshot() });
	}, 1000);
}

async function loadApplication() {
	await globalThis.__AEDICULE_ISOLATION_READY;
	return withExclusiveStartupLock(
		navigator.locks,
		"aedicule-startup-v1",
		(stage, detail) => {
			if (stage === "startup-lock-waiting") {
				showStartupStatus(strings.waitingForStartupLock);
			}
			reportStartupDiagnostic(stage, detail);
		},
		initializeApplication,
	);
}

function reportStartupFailure(error) {
	const capabilities = browserCapabilities();
	const failedStage = globalThis.__AEDICULE_STARTUP_TIMELINE.at(-1)?.stage ?? "unknown";
	const errorName = error?.name ?? typeof error;
	const detail = error instanceof Error ? error.message : String(error);
	const failure = {
		failedStage,
		errorName,
		errorMessage: detail,
		errorCause: error?.cause === undefined ? null : String(error.cause),
		errorStack: error?.stack ?? null,
		...browserEnvironment(),
		...capabilities,
	};
	reportStartupDiagnostic("startup-failed", failure);
	console.error("[Aedicule startup] failed", error, JSON.stringify(failure));
	status.dataset.state = "error";
	const adapterRecovery = failedStage === "webgpu-adapter-request"
		? `\n\n${strings.webGpuAdapterRecovery}`
		: "";
	status.textContent = `${strings.couldNotStart}\n\n${errorName}: ${detail}${adapterRecovery}\n\n`
		+ `${capabilitySummary(capabilities)}\n`
		+ `stage: ${failedStage}\n`
		+ `elapsedMs: ${Math.round(performance.now())}`;
}

loadApplication().catch(reportStartupFailure);
