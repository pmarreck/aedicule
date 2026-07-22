import assert from "node:assert/strict";
import vm from "node:vm";

import {
	inputObserverSource,
	removeBrowserProfile,
	waitForDebuggablePage,
} from "./web_browser_startup";

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
removeBrowserProfile("/temporary/chrome-profile", (profile, options) => {
	removedProfile = profile;
	removeOptions = options;
});
assert.equal(removedProfile, "/temporary/chrome-profile");
assert.deepEqual(removeOptions, {
	force: true,
	maxRetries: 8,
	recursive: true,
	retryDelay: 25,
});

const listeners = new Map();
const browser = {
	addEventListener(type, listener) { listeners.set(type, listener); },
};
vm.runInNewContext(inputObserverSource(false), browser);
listeners.get("mousedown")({ button: 0, buttons: 1, clientX: 4, clientY: 8 });
assert.equal(browser.__AEDICULE_INPUT_PROBE.counts.mousedown, 1);
assert.equal(browser.__AEDICULE_INPUT_PROBE.events.length, 1);
