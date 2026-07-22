const isolationHeaders = Object.freeze({
	"Cross-Origin-Embedder-Policy": "require-corp",
	"Cross-Origin-Opener-Policy": "same-origin",
	"Cross-Origin-Resource-Policy": "same-origin",
});

/**
 * Re-wraps same-origin static assets with the isolation policy required by
 * shared WebAssembly memory on hosts, such as GitHub Pages, without headers.
 */
async function isolatedResponse(request) {
	const response = await fetch(request);
	if (response.type === "opaque" || response.status === 0) return response;
	const headers = new Headers(response.headers);
	for (const [name, value] of Object.entries(isolationHeaders)) headers.set(name, value);
	return new Response(response.body, {
		status: response.status,
		statusText: response.statusText,
		headers,
	});
}

if (typeof document === "undefined") {
	addEventListener("install", event => event.waitUntil(skipWaiting()));
	addEventListener("activate", event => event.waitUntil(clients.claim()));
	addEventListener("fetch", event => {
		if (event.request.cache === "only-if-cached" && event.request.mode !== "same-origin") return;
		const protocol = new URL(event.request.url).protocol;
		if (protocol === "http:" || protocol === "https:") {
			event.respondWith(isolatedResponse(event.request));
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
