import assert from "node:assert/strict";
import vm from "node:vm";

import {
	browserProcessEnvironment,
	inputObserverSource,
	requestBrowserClose,
	removeBrowserProfile,
	waitForFile,
	waitForDebuggablePage,
} from "./web_browser_startup";

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
const browser = {
	addEventListener(type, listener) { listeners.set(type, listener); },
};
vm.runInNewContext(inputObserverSource(false), browser);
listeners.get("mousedown")({ button: 0, buttons: 1, clientX: 4, clientY: 8 });
assert.equal(browser.__AEDICULE_INPUT_PROBE.counts.mousedown, 1);
assert.equal(browser.__AEDICULE_INPUT_PROBE.events.length, 1);
