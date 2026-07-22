import init from "./aedicule_web.js";

const status = document.getElementById("aedicule-startup-status");

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
		.map(([name, available]) => `${name}: ${available ? "yes" : "no"}`)
		.join("\n");
}

async function preflightBrowser(capabilities) {
	if (!capabilities.secureContext) {
		throw new Error("Aedicule requires a secure HTTPS browser context.");
	}
	if (!capabilities.crossOriginIsolated) {
		throw new Error("Aedicule requires cross-origin isolation for shared WebAssembly memory.");
	}
	if (!capabilities.sharedArrayBuffer) {
		throw new Error("This browser does not expose shared WebAssembly memory.");
	}
	if (!capabilities.webGpu) {
		throw new Error("This browser does not expose WebGPU. Safari requires version 26 or newer.");
	}
	const adapter = await navigator.gpu.requestAdapter();
	if (adapter === null) {
		throw new Error("WebGPU is present, but the browser did not provide a usable GPU adapter.");
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

async function loadApplication() {
	await globalThis.__AEDICULE_ISOLATION_READY;
	reportStartupDiagnostic("bootstrap");
	showStartupStatus("Checking browser capabilities…");
	const capabilities = browserCapabilities();
	reportStartupDiagnostic("capabilities", capabilities);
	await preflightBrowser(capabilities);
	showStartupStatus("Loading WAT application…");
	const response = await fetch("./code.wat", { cache: "no-store" });
	if (!response.ok) {
		throw new Error(`Aedicule could not load code.wat: HTTP ${response.status}`);
	}

	const bytes = await response.arrayBuffer();
	reportStartupDiagnostic("wat-fetched", { status: response.status, bytes: bytes.byteLength });
	const source = new TextDecoder("utf-8", { fatal: true }).decode(bytes);
	globalThis.__AEDICULE_WAT = source;
	showStartupStatus("Starting Aedicule…");
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
	status.textContent = `Aedicule could not start.\n\n${detail}\n\n${capabilitySummary(capabilities)}`;
}

loadApplication().catch(reportStartupFailure);
