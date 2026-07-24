const isolationHeaders = Object.freeze({
	"Cross-Origin-Embedder-Policy": "require-corp",
	"Cross-Origin-Opener-Policy": "same-origin",
	"Cross-Origin-Resource-Policy": "same-origin",
});
const runtimeCacheName = "aedicule-runtime-v1";
const immutableRuntimePattern = /\/aedicule_web_bg\.[0-9a-f]{64}\.wasm$/;
const earlyDiagnosticLimit = 32;
globalThis.__AEDICULE_EARLY_DIAGNOSTICS = [];

/**
 * Makes service-worker registration and the isolation reload observable before
 * the localized module bootstrap can run. Entries remain bounded because a
 * failed registration may retry across several document loads.
 */
function reportEarlyIsolationDiagnostic(stage, detail = {}) {
	const entry = {
		stage,
		elapsedMs: Math.round(performance.now()),
		...detail,
	};
	globalThis.__AEDICULE_EARLY_DIAGNOSTICS.push(entry);
	if (globalThis.__AEDICULE_EARLY_DIAGNOSTICS.length > earlyDiagnosticLimit) {
		globalThis.__AEDICULE_EARLY_DIAGNOSTICS.shift();
	}
	console.info("[Aedicule isolation]", stage, JSON.stringify(entry));
	const status = globalThis.document?.getElementById("aedicule-startup-status");
	if (status !== null && status !== undefined) {
		status.textContent = `Loading Aedicule…\n\n${stage} · ${entry.elapsedMs} ms`;
	}
}

function isImmutableRuntimeRequest(request) {
	return request.method === "GET"
		&& immutableRuntimePattern.test(new URL(request.url).pathname);
}

/**
 * Re-wraps same-origin static assets with the isolation policy required by
 * shared WebAssembly memory on hosts, such as GitHub Pages, without headers.
 */
async function isolatedResponse(request) {
	const response = await fetch(request);
	if (response.type === "opaque" || response.status === 0) return response;
	const headers = new Headers(response.headers);
	for (const [name, value] of Object.entries(isolationHeaders)) headers.set(name, value);
	if (isImmutableRuntimeRequest(request)) {
		headers.set("Cache-Control", "public, max-age=31536000, immutable");
	}
	return new Response(response.body, {
		status: response.status,
		statusText: response.statusText,
		headers,
	});
}

async function cacheFirstImmutableRuntime(request) {
	const cache = await caches.open(runtimeCacheName);
	const retained = await cache.match(request);
	if (retained !== undefined) return retained;
	const response = await isolatedResponse(request);
	if (response.ok) await cache.put(request, response.clone());
	return response;
}

if (typeof document === "undefined") {
	addEventListener("install", event => event.waitUntil(skipWaiting()));
	addEventListener("activate", event => event.waitUntil(clients.claim()));
	addEventListener("fetch", event => {
		if (event.request.cache === "only-if-cached" && event.request.mode !== "same-origin") return;
		const protocol = new URL(event.request.url).protocol;
		if (protocol === "http:" || protocol === "https:") {
			event.respondWith(
				isImmutableRuntimeRequest(event.request)
					? cacheFirstImmutableRuntime(event.request)
					: isolatedResponse(event.request),
			);
		}
	});
} else if (globalThis.isSecureContext && "serviceWorker" in navigator) {
	const controlledAtLoad = navigator.serviceWorker.controller !== null;
	reportEarlyIsolationDiagnostic("isolation-registering", {
		controlledAtLoad,
		crossOriginIsolated: globalThis.crossOriginIsolated === true,
	});
	globalThis.__AEDICULE_ISOLATION_READY = navigator.serviceWorker
		.register("./coi-serviceworker.js")
		.then(async registration => {
			reportEarlyIsolationDiagnostic("isolation-registered", {
				scope: registration.scope,
			});
			await navigator.serviceWorker.ready;
			reportEarlyIsolationDiagnostic("isolation-ready", {
				serviceWorkerControlled: navigator.serviceWorker.controller !== null,
				crossOriginIsolated: globalThis.crossOriginIsolated === true,
			});
			if (!globalThis.crossOriginIsolated && !controlledAtLoad) {
				reportEarlyIsolationDiagnostic("isolation-reloading");
				location.reload();
				return new Promise(() => {});
			}
		})
		.catch(error => {
			reportEarlyIsolationDiagnostic("isolation-failed", {
				errorName: error?.name ?? typeof error,
				errorMessage: error?.message ?? String(error),
			});
			throw error;
		});
}
