import assert from "node:assert/strict";
import { pathToFileURL } from "node:url";

const root = process.argv[2];
const {
	isLegacyIsolationRegistration,
	retireLegacyIsolationServiceWorkers,
} = await import(pathToFileURL(`${root}/packaging/web/service-worker-retirement.mjs`));

function registration(slot, scriptURL, result = true) {
	let unregisterCalls = 0;
	return {
		[slot]: { scriptURL },
		async unregister() {
			unregisterCalls += 1;
			return result;
		},
		get unregisterCalls() {
			return unregisterCalls;
		},
	};
}

const cases = [
	[registration("active", "https://example.test/coi-serviceworker.js"), true],
	[registration("waiting", "https://example.test/app/coi-serviceworker.js"), true],
	[registration("installing", "https://example.test/coi-serviceworker.js?v=old"), true],
	[registration("active", "https://example.test/application-worker.js"), false],
	[registration("active", "not a URL"), false],
	[{}, false],
];
assert.deepEqual(
	cases.map(([candidate]) => isLegacyIsolationRegistration(candidate)),
	cases.map(([, expected]) => expected),
	"the complete registration set must classify only the retired isolation script",
);

const legacyActive = registration("active", "https://example.test/coi-serviceworker.js");
const unrelated = registration("active", "https://example.test/other-worker.js");
const legacyWaiting = registration("waiting", "https://example.test/app/coi-serviceworker.js", false);
assert.deepEqual(
	await retireLegacyIsolationServiceWorkers({
		async getRegistrations() {
			return [legacyActive, unrelated, legacyWaiting];
		},
	}),
	{ matched: 2, unregistered: 1 },
);
assert.equal(legacyActive.unregisterCalls, 1);
assert.equal(legacyWaiting.unregisterCalls, 1);
assert.equal(unrelated.unregisterCalls, 0);
assert.deepEqual(
	await retireLegacyIsolationServiceWorkers(undefined),
	{ matched: 0, unregistered: 0 },
);
