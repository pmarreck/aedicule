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

/**
 * Derives the stage a startup failure actually occurred in. The lock's
 * `finally` above appends `startup-lock-released` after failing work has
 * already recorded its own last stage, so the raw timeline tail mislabels
 * every in-lock failure; popping trailing release entries uncovers the truth.
 * Every other lock stage is a genuine failure position and is preserved.
 */
export function failedStartupStage(timeline) {
	let index = timeline.length - 1;
	while (index >= 0 && timeline[index].stage === "startup-lock-released") {
		index -= 1;
	}
	return timeline[index]?.stage ?? "unknown";
}
