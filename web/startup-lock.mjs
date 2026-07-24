/**
 * Serializes only the expensive WebGPU/Wasm startup critical section across
 * same-origin tabs, then releases before ordinary multi-tab frame execution.
 */
export async function withExclusiveStartupLock(locks, name, report, work) {
	if (typeof locks?.request !== "function") {
		report("startup-lock-unavailable", {});
		return work();
	}

	const detail = { name };
	report("startup-lock-requested", detail);
	report("startup-lock-waiting", detail);
	return locks.request(name, { mode: "exclusive" }, async () => {
		report("startup-lock-acquired", detail);
		try {
			return await work();
		} finally {
			report("startup-lock-released", detail);
		}
	});
}
