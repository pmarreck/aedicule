const isolationHeaders = Object.freeze({
	"Cross-Origin-Embedder-Policy": "require-corp",
	"Cross-Origin-Opener-Policy": "same-origin",
	"Cross-Origin-Resource-Policy": "same-origin",
});
const runtimeCacheName = "aedicule-runtime-v1";
const immutableRuntimePattern = /\/aedicule_web_bg\.[0-9a-f]{64}\.wasm$/;

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
	globalThis.__AEDICULE_ISOLATION_READY = navigator.serviceWorker.register("./coi-serviceworker.js").then(async () => {
		await navigator.serviceWorker.ready;
		if (!globalThis.crossOriginIsolated && !controlledAtLoad) {
			location.reload();
			return new Promise(() => {});
		}
	}).catch(error => console.error("[Aedicule isolation] failed", error));
}
