import assert from "node:assert/strict";
import { createMotionBridge } from "../../web/motion-input.mjs";

const listeners = new Map();
const eventTarget = {
	addEventListener(name, listener) {
		const registered = listeners.get(name) ?? [];
		registered.push(listener);
		listeners.set(name, registered);
	},
};
const dispatch = async (type, event = {}) => {
	for (const listener of listeners.get(type) ?? []) listener({ type, ...event });
	await Promise.resolve();
	await Promise.resolve();
};

let userActivationActive = false;
let permissionCalls = 0;
const permissionResults = [
	() => Promise.reject(new DOMException("activation arrived too early", "NotAllowedError")),
	() => Promise.resolve("granted"),
];
const globalObject = {
	__AEDICULE_MOTION_INTEREST: true,
	DeviceMotionEvent: {
		requestPermission() {
			permissionCalls += 1;
			return permissionResults.shift()();
		},
	},
};
createMotionBridge({
	globalObject,
	eventTarget,
	now: () => 321.4,
	userActivation: () => ({ isActive: userActivationActive }),
});

await dispatch("touchstart");
await dispatch("pointerdown");
assert.equal(permissionCalls, 0,
	"gesture-start edges without transient activation must not consume the permission attempt");

userActivationActive = true;
await dispatch("touchend");
assert.equal(permissionCalls, 1);
assert.equal(globalObject.__AEDICULE_MOTION_DIAGNOSTIC.permission, "failed");
assert.equal(globalObject.__AEDICULE_MOTION_DIAGNOSTIC.errorName, "NotAllowedError");

await dispatch("click");
assert.equal(permissionCalls, 2,
	"a rejected early completion must leave the following activated click retryable");
assert.equal(globalObject.__AEDICULE_MOTION_DIAGNOSTIC.permission, "granted");
assert.equal(globalObject.__AEDICULE_MOTION_DIAGNOSTIC.permissionAttempts, 2);

await dispatch("devicemotion", {
	acceleration: { x: 18, y: 2, z: 1 },
	accelerationIncludingGravity: { x: 18, y: 2, z: 10.8 },
	rotationRate: { alpha: 3, beta: 4, gamma: 5 },
});
assert.deepEqual(globalObject.__AEDICULE_MOTION_SAMPLES, [{
	elapsedMs: 321,
	ax: 18,
	ay: 2,
	az: 1,
	rx: 3,
	ry: 4,
	rz: 5,
}]);
assert.equal(globalObject.__AEDICULE_MOTION_DIAGNOSTIC.samplesCaptured, 1);
assert.equal(globalObject.__AEDICULE_MOTION_DIAGNOSTIC.accelerationSource, "user");

for (let index = 0; index < 40; index += 1) {
	await dispatch("devicemotion", {
		acceleration: null,
		accelerationIncludingGravity: { x: index, y: 0, z: 9.8 },
		rotationRate: null,
	});
}
assert.equal(globalObject.__AEDICULE_MOTION_SAMPLES.length, 32,
	"motion capture must remain bounded under a sensor event flood");
assert.equal(globalObject.__AEDICULE_MOTION_DIAGNOSTIC.samplesCaptured, 41);
assert.equal(globalObject.__AEDICULE_MOTION_DIAGNOSTIC.accelerationSource, "gravity-included");
const drained = globalObject.__AEDICULE_TAKE_MOTION_SAMPLES();
assert.equal(drained.length, 32);
assert.equal(globalObject.__AEDICULE_MOTION_SAMPLES.length, 0);
assert.deepEqual(globalObject.__AEDICULE_TAKE_MOTION_SAMPLES(), [],
	"an empty drain must not reinsert an undefined pseudo-sample");
globalObject.__AEDICULE_REPORT_MOTION_DRAIN({
	aboveThreshold: 2,
	belowThreshold: 39,
	gesturesEmitted: 1,
	lastMagnitude: 21,
	samplesDrained: 41,
	samplesQueued: 0,
});
assert.deepEqual(
	{
		aboveThreshold: globalObject.__AEDICULE_MOTION_DIAGNOSTIC.aboveThreshold,
		belowThreshold: globalObject.__AEDICULE_MOTION_DIAGNOSTIC.belowThreshold,
		gesturesEmitted: globalObject.__AEDICULE_MOTION_DIAGNOSTIC.gesturesEmitted,
		lastMagnitude: globalObject.__AEDICULE_MOTION_DIAGNOSTIC.lastMagnitude,
		samplesDrained: globalObject.__AEDICULE_MOTION_DIAGNOSTIC.samplesDrained,
		samplesQueued: globalObject.__AEDICULE_MOTION_DIAGNOSTIC.samplesQueued,
	},
	{
		aboveThreshold: 2,
		belowThreshold: 39,
		gesturesEmitted: 1,
		lastMagnitude: 21,
		samplesDrained: 41,
		samplesQueued: 0,
	},
);

const implicitListeners = new Map();
const implicitGlobal = { __AEDICULE_MOTION_INTEREST: true };
createMotionBridge({
	globalObject: implicitGlobal,
	eventTarget: {
		addEventListener(name, listener) { implicitListeners.set(name, listener); },
	},
	userActivation: () => ({ isActive: true }),
});
implicitListeners.get("click")({ type: "click" });
assert.equal(implicitGlobal.__AEDICULE_MOTION_DIAGNOSTIC.permission, "implicit");
