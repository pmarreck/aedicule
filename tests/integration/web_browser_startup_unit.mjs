import assert from "node:assert/strict";
import vm from "node:vm";

import {
	audioPlaybackFailures,
	browserProcessEnvironment,
	chooseCanvasInputPoint,
	inputObserverSource,
	requestBrowserClose,
	removeBrowserProfile,
	waitForFile,
	waitForDebuggablePage,
	waitForSemanticInputQuiescence,
} from "./web_browser_startup";

assert.deepEqual(
	chooseCanvasInputPoint(
		{ cssWidth: 780, cssHeight: 437 },
		{ panels: [{ x: 12, y: 213, width: 756, height: 168 }] },
	),
	{ x: 390, y: 109 },
	"browser input probes must avoid guest-owned control panels",
);
assert.deepEqual(
	chooseCanvasInputPoint({ cssWidth: 780, cssHeight: 437 }, null),
	{ x: 390, y: 109 },
	"a guest without native controls must leave the deterministic probe unobstructed",
);

const semanticCountSequence = [1, 2, 3, 3, 3];
let semanticCountReads = 0;
assert.equal(await waitForSemanticInputQuiescence({
	send: async () => ({
		result: {
			value: { pointer: semanticCountSequence[semanticCountReads++] },
		},
	}),
}), true);
assert.equal(semanticCountReads, semanticCountSequence.length);

const completeAudioTimeline = [
	{ stage: "unlock-event", userActivationSeen: true },
	{ stage: "context-resumed", contextState: "running" },
	{ stage: "pcm-admitted", nonzeroSamples: 24, peak: 0.75, rootMeanSquare: 0.25 },
	{ stage: "source-started", contextState: "running" },
	{ stage: "source-ended", contextState: "running" },
];
assert.deepEqual(audioPlaybackFailures(completeAudioTimeline), []);
assert.deepEqual(audioPlaybackFailures(
	completeAudioTimeline.map(entry => (
		entry.stage === "context-resumed"
			? { ...entry, stage: "context-created" }
			: entry
	)),
), [], "a context created running needs no redundant resume transition");
for (const [name, mutate, expected] of [
	["activation", timeline => timeline.map(entry => (
		entry.stage === "unlock-event" ? { ...entry, userActivationSeen: false } : entry
	)), "user activation"],
	["resume", timeline => timeline.filter(entry => entry.stage !== "context-resumed"), "running context"],
	["nonzero", timeline => timeline.map(entry => (
		entry.stage === "pcm-admitted"
			? { ...entry, nonzeroSamples: 0, peak: 0, rootMeanSquare: 0 }
			: entry
	)), "non-zero PCM"],
	["start", timeline => timeline.filter(entry => entry.stage !== "source-started"), "source start"],
	["end", timeline => timeline.filter(entry => entry.stage !== "source-ended"), "source completion"],
]) {
	const failures = audioPlaybackFailures(mutate(structuredClone(completeAudioTimeline)));
	assert.equal(failures.length, 1, `${name} mutation must produce one focused failure`);
	assert.match(failures[0], new RegExp(expected));
}

const browserCloseCommands = [];
await requestBrowserClose({
	send: async command => { browserCloseCommands.push(command); },
}, {
	grace: () => new Promise(() => {}),
});
assert.deepEqual(browserCloseCommands, ["Browser.close"]);

await requestBrowserClose({
	send: async () => { throw new Error("browser already closed its protocol socket"); },
}, {
	grace: async () => {},
});

assert.deepEqual(
	browserProcessEnvironment("/temporary/chrome-profile", {
		HOME: "/homeless-shelter",
		PATH: "/programs",
		XDG_CACHE_HOME: "/existing/cache",
		XDG_CONFIG_HOME: "/existing/config",
	}),
	{
		HOME: "/temporary/chrome-profile",
		PATH: "/programs",
		XDG_CACHE_HOME: "/existing/cache",
		XDG_CONFIG_HOME: "/existing/config",
	},
);
assert.deepEqual(
	browserProcessEnvironment("/temporary/chrome-profile", { PATH: "/programs" }),
	{
		HOME: "/temporary/chrome-profile",
		PATH: "/programs",
		XDG_CACHE_HOME: "/temporary/chrome-profile/cache",
		XDG_CONFIG_HOME: "/temporary/chrome-profile/config",
	},
);

for (const [address, expected] of [
	["disabled:", undefined],
	["", undefined],
	["unix:path=/run/user/1000/bus", "unix:path=/run/user/1000/bus"],
	["unix:abstract=/tmp/dbus-test", "unix:abstract=/tmp/dbus-test"],
	["tcp:host=127.0.0.1,port=1234", "tcp:host=127.0.0.1,port=1234"],
]) {
	const environment = browserProcessEnvironment("/temporary/chrome-profile", {
		DBUS_SESSION_BUS_ADDRESS: address,
		PATH: "/programs",
	});
	assert.equal(environment.DBUS_SESSION_BUS_ADDRESS, expected);
}

let timeoutCallback;
let watcherClosed = 0;
let timerCancelled = 0;
const watcher = {
	close() { watcherClosed += 1; },
	on() {},
};
const missingPortFile = waitForFile("/temporary/chrome-profile/DevToolsActivePort", 17, {
	cancel: timer => {
		assert.equal(timer, 23);
		timerCancelled += 1;
	},
	exists: () => false,
	schedule: (callback, milliseconds) => {
		assert.equal(milliseconds, 17);
		timeoutCallback = callback;
		return 23;
	},
	watch: (directory, callback) => {
		assert.equal(directory, "/temporary/chrome-profile");
		assert.equal(typeof callback, "function");
		return watcher;
	},
});
timeoutCallback();
await assert.rejects(missingPortFile, /timed out waiting for Chrome's DevTools endpoint/);
assert.equal(watcherClosed, 1);
assert.equal(timerCancelled, 1);

const expectedPage = {
	type: "page",
	webSocketDebuggerUrl: "ws://127.0.0.1:9222/devtools/page/1",
};
let listCalls = 0;
let retries = 0;
const page = await waitForDebuggablePage(9222, 1_000, {
	listPages: async url => {
		assert.equal(url, "http://127.0.0.1:9222/json/list");
		listCalls += 1;
		return listCalls < 3 ? [] : [expectedPage];
	},
	retry: async () => { retries += 1; },
});
assert.equal(page, expectedPage);
assert.equal(listCalls, 3);
assert.equal(retries, 2);

let removedProfile;
let removeOptions;
let removeAttempts = 0;
let removeRetries = 0;
await removeBrowserProfile("/temporary/chrome-profile", {
	remove: (profile, options) => {
		removedProfile = profile;
		removeOptions = options;
		removeAttempts += 1;
		if (removeAttempts < 3) {
			const error = new Error("late Chrome helper recreated the profile");
			error.code = "ENOTEMPTY";
			throw error;
		}
	},
	retry: async () => { removeRetries += 1; },
});
assert.equal(removedProfile, "/temporary/chrome-profile");
assert.deepEqual(removeOptions, {
	force: true,
	maxRetries: 8,
	recursive: true,
	retryDelay: 25,
});
assert.equal(removeAttempts, 3);
assert.equal(removeRetries, 2);

const listeners = new Map();
class HTMLInputElement {
	focus() {}
}
const browser = {
	addEventListener(type, listener) { listeners.set(type, listener); },
	HTMLInputElement,
};
vm.runInNewContext(inputObserverSource(false), browser);
new browser.HTMLInputElement().focus();
assert.equal(browser.__AEDICULE_INPUT_PROBE.hiddenInputFocusRequests, 1);
listeners.get("mousedown")({ button: 0, buttons: 1, clientX: 4, clientY: 8 });
assert.equal(browser.__AEDICULE_INPUT_PROBE.counts.mousedown, 1);
assert.equal(browser.__AEDICULE_INPUT_PROBE.events.length, 1);
