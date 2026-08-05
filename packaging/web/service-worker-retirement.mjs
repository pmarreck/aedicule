const LEGACY_ISOLATION_SCRIPT = "/coi-serviceworker.js";

/**
 * Identifies only Aedicule's retired isolation worker across every worker
 * lifecycle slot, leaving unrelated registrations under the origin intact.
 */
export function isLegacyIsolationRegistration(registration) {
	for (const worker of [
		registration?.active,
		registration?.waiting,
		registration?.installing,
	]) {
		try {
			if (new URL(worker?.scriptURL).pathname.endsWith(LEGACY_ISOLATION_SCRIPT)) {
				return true;
			}
		} catch {
			// An absent or malformed script URL is not Aedicule's retired worker.
		}
	}
	return false;
}

/**
 * Unregisters stale isolation workers without blocking the current page load;
 * callers choose whether to await this operation or launch it in the background.
 */
export async function retireLegacyIsolationServiceWorkers(serviceWorkerContainer) {
	if (typeof serviceWorkerContainer?.getRegistrations !== "function") {
		return { matched: 0, unregistered: 0 };
	}
	const registrations = await serviceWorkerContainer.getRegistrations();
	const legacy = registrations.filter(isLegacyIsolationRegistration);
	const results = await Promise.all(legacy.map(registration => registration.unregister()));
	return {
		matched: legacy.length,
		unregistered: results.filter(Boolean).length,
	};
}
