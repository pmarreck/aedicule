import assert from "node:assert/strict";
import { createTouchBridge } from "../../web/touch-input.mjs";

const listeners = new Map();
const globalObject = { __AEDICULE_TOUCH_MAX_CONTACTS: 2 };
const captured = [];
const canvas = {
	getBoundingClientRect: () => ({ left: 10, top: 20, width: 320, height: 240 }),
	setPointerCapture: id => {
		if (id === 45) throw new DOMException("capture unavailable", "NotFoundError");
		captured.push(id);
	},
};
const eventTarget = {
	addEventListener(name, listener) {
		listeners.set(name, listener);
	},
};
createTouchBridge({
	globalObject,
	eventTarget,
	canvas: () => canvas,
});

assert.deepEqual(globalObject.__AEDICULE_TOUCH_DIAGNOSTIC, {
	enabled: true,
	maxContacts: 2,
	queued: 0,
	rawActive: 0,
	occludedActive: 0,
	enqueued: 0,
	ignoredDisabled: 0,
	occludedStarts: 0,
	captureFailures: 0,
	lastPhase: "-",
});

const dispatch = (type, overrides = {}) => {
	let prevented = 0;
	let stopped = 0;
	listeners.get(type)({
		pointerType: "touch",
		pointerId: 41,
		clientX: 42,
		clientY: 84,
		preventDefault: () => prevented += 1,
		stopImmediatePropagation: () => stopped += 1,
		...overrides,
	});
	return { prevented, stopped };
};

assert.deepEqual(dispatch("pointerdown"), { prevented: 1, stopped: 1 });
assert.deepEqual(captured, [41]);
assert.deepEqual(globalObject.__AEDICULE_TOUCH_EVENTS, [
	{ phase: "start", id: 41, x: 32, y: 64 },
]);
assert.equal(globalObject.__AEDICULE_TOUCH_DIAGNOSTIC.rawActive, 1);
assert.equal(globalObject.__AEDICULE_TOUCH_DIAGNOSTIC.enqueued, 1);
assert.equal(globalObject.__AEDICULE_TOUCH_DIAGNOSTIC.lastPhase, "start");

dispatch("pointermove", { clientY: 100 });
dispatch("pointerup", { clientX: 999, clientY: 999 });
assert.deepEqual(globalObject.__AEDICULE_TOUCH_EVENTS.slice(1), [
	{ phase: "move", id: 41, x: 32, y: 80 },
	{ phase: "end", id: 41, x: 989, y: 979 },
]);
assert.equal(globalObject.__AEDICULE_TOUCH_DIAGNOSTIC.rawActive, 0);
assert.equal(globalObject.__AEDICULE_TOUCH_DIAGNOSTIC.enqueued, 3);

globalObject.__AEDICULE_UI_SNAPSHOT = {
	panels: [{ x: 0, y: 0, width: 100, height: 100 }],
	sliders: [], buttons: [], externalLinks: [], textFields: [],
};
assert.deepEqual(
	dispatch("pointerdown", { pointerId: 42, clientX: 30, clientY: 50 }),
	{ prevented: 0, stopped: 0 },
	"an AVP-owned touch must continue to GPUI",
);
assert.deepEqual(
	dispatch("pointermove", { pointerId: 42, clientX: 200, clientY: 200 }),
	{ prevented: 0, stopped: 0 },
);
assert.deepEqual(
	dispatch("pointerup", { pointerId: 42, clientX: 200, clientY: 200 }),
	{ prevented: 0, stopped: 0 },
);
assert.equal(
	globalObject.__AEDICULE_TOUCH_EVENTS.some(event => event.id === 42),
	false,
	"a contact starting on an AVP control stays occluded for its lifetime",
);
assert.equal(globalObject.__AEDICULE_TOUCH_DIAGNOSTIC.occludedStarts, 1);
assert.equal(globalObject.__AEDICULE_TOUCH_DIAGNOSTIC.occludedActive, 0);
dispatch("pointerdown", { pointerId: 43, clientX: 200, clientY: 200 });
dispatch("pointermove", { pointerId: 43, clientX: 210, clientY: 210 });
listeners.get("blur")();
assert.deepEqual(globalObject.__AEDICULE_TOUCH_EVENTS.at(-1), { phase: "cancel-all" });

assert.deepEqual(
	dispatch("pointerdown", { pointerId: 45, clientX: 200, clientY: 200 }),
	{ prevented: 1, stopped: 1 },
	"pointer-capture failure must not abort raw touch delivery",
);
assert.equal(globalObject.__AEDICULE_TOUCH_DIAGNOSTIC.captureFailures, 1);
assert.deepEqual(globalObject.__AEDICULE_TOUCH_EVENTS.at(-1),
	{ phase: "start", id: 45, x: 190, y: 180 });
dispatch("pointerup", { pointerId: 45, clientX: 200, clientY: 200 });

assert.deepEqual(
	dispatch("pointerdown", { pointerType: "mouse", pointerId: 1 }),
	{ prevented: 0, stopped: 0 },
	"real mouse input is never intercepted by the touch bridge",
);

globalObject.__AEDICULE_TOUCH_MAX_CONTACTS = undefined;
const before = globalObject.__AEDICULE_TOUCH_EVENTS.length;
dispatch("pointerdown", { pointerId: 44 });
assert.equal(globalObject.__AEDICULE_TOUCH_EVENTS.length, before);
assert.equal(globalObject.__AEDICULE_TOUCH_DIAGNOSTIC.enabled, false);
assert.equal(globalObject.__AEDICULE_TOUCH_DIAGNOSTIC.ignoredDisabled, 1);
