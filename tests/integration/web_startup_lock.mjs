import assert from "node:assert/strict";
import { pathToFileURL } from "node:url";

const root = process.argv[2];
const { withExclusiveStartupLock } = await import(
	pathToFileURL(`${root}/web/startup-lock.mjs`)
);

{
	const events = [];
	const result = await withExclusiveStartupLock(
		undefined,
		"aedicule-startup-v1",
		(stage, detail) => events.push([stage, detail]),
		async () => {
			events.push(["work"]);
			return 42;
		},
	);
	assert.equal(result, 42);
	assert.deepEqual(events, [
		["startup-lock-unavailable", {}],
		["work"],
	]);
}

{
	const events = [];
	const locks = {
		async request(name, options, work) {
			events.push(["manager-request", name, options]);
			return work({ name, mode: options.mode });
		},
	};
	const result = await withExclusiveStartupLock(
		locks,
		"aedicule-startup-v1",
		(stage, detail) => events.push([stage, detail]),
		async () => {
			events.push(["work"]);
			return "ready";
		},
	);
	assert.equal(result, "ready");
	assert.deepEqual(events, [
		["startup-lock-requested", { name: "aedicule-startup-v1" }],
		["startup-lock-waiting", { name: "aedicule-startup-v1" }],
		["manager-request", "aedicule-startup-v1", { mode: "exclusive" }],
		["startup-lock-acquired", { name: "aedicule-startup-v1" }],
		["work"],
		["startup-lock-released", { name: "aedicule-startup-v1" }],
	]);
}

{
	const events = [];
	const locks = {
		request(_name, _options, work) {
			return work({});
		},
	};
	await assert.rejects(
		withExclusiveStartupLock(
			locks,
			"aedicule-startup-v1",
			stage => events.push(stage),
			async () => {
				throw new Error("guest failed");
			},
		),
		/guest failed/,
	);
	assert.deepEqual(events, [
		"startup-lock-requested",
		"startup-lock-waiting",
		"startup-lock-acquired",
		"startup-lock-released",
	]);
}
