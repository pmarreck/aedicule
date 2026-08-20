const MAX_MOTION_SAMPLES = 32;
const PERMISSION_EVENTS = Object.freeze(["keydown", "pointerup", "touchend", "click"]);

/**
 * Owns the browser-privileged half of Aedicule motion input. Permission is
 * attempted only during transient activation and remains retryable after an
 * exception, while capture stays bounded for the Rust adapter's next frame.
 */
export function createMotionBridge({
	globalObject = globalThis,
	eventTarget = globalThis,
	now = () => performance.now(),
	userActivation = () => globalObject.navigator?.userActivation,
} = {}) {
	const samples = [];
	const diagnostic = {
		interest: Boolean(globalObject.__AEDICULE_MOTION_INTEREST),
		permission: "unrequested",
		permissionAttempts: 0,
		skippedNoInterest: 0,
		skippedNoActivation: 0,
		samplesCaptured: 0,
		samplesQueued: 0,
		samplesDrained: 0,
		belowThreshold: 0,
		aboveThreshold: 0,
		gesturesEmitted: 0,
		lastMagnitude: null,
		accelerationSource: "none",
		lastActivationEvent: "none",
		errorName: null,
	};
	let permissionInFlight = false;

	globalObject.__AEDICULE_MOTION_SAMPLES = samples;
	globalObject.__AEDICULE_MOTION_DIAGNOSTIC = diagnostic;
	globalObject.__AEDICULE_TAKE_MOTION_SAMPLES = () => {
		const drained = samples.splice(0);
		diagnostic.samplesQueued = samples.length;
		return drained;
	};
	globalObject.__AEDICULE_REPORT_MOTION_DRAIN = detail => {
		for (const name of [
			"samplesQueued", "samplesDrained", "belowThreshold", "aboveThreshold",
			"gesturesEmitted", "lastMagnitude",
		]) {
			if (Object.hasOwn(detail, name)) diagnostic[name] = detail[name];
		}
	};

	const requestPermission = event => {
		diagnostic.interest = Boolean(globalObject.__AEDICULE_MOTION_INTEREST);
		diagnostic.lastActivationEvent = event?.type ?? "unknown";
		if (!diagnostic.interest) {
			diagnostic.skippedNoInterest += 1;
			return;
		}
		if (["granted", "denied", "implicit"].includes(diagnostic.permission)
			|| permissionInFlight) return;
		if (userActivation()?.isActive === false) {
			diagnostic.skippedNoActivation += 1;
			return;
		}

		const deviceMotionEvent = globalObject.DeviceMotionEvent;
		if (typeof deviceMotionEvent?.requestPermission !== "function") {
			diagnostic.permission = "implicit";
			return;
		}

		diagnostic.permissionAttempts += 1;
		diagnostic.permission = "requesting";
		diagnostic.errorName = null;
		permissionInFlight = true;
		let request;
		try {
			request = deviceMotionEvent.requestPermission();
		} catch (error) {
			permissionInFlight = false;
			diagnostic.permission = "failed";
			diagnostic.errorName = error?.name ?? typeof error;
			return;
		}
		Promise.resolve(request).then(state => {
			permissionInFlight = false;
			if (state === "granted" || state === "denied") {
				diagnostic.permission = state;
				return;
			}
			diagnostic.permission = "failed";
			diagnostic.errorName = `unexpected-${String(state)}`;
		}).catch(error => {
			permissionInFlight = false;
			diagnostic.permission = "failed";
			diagnostic.errorName = error?.name ?? typeof error;
		});
	};

	for (const eventName of PERMISSION_EVENTS) {
		eventTarget.addEventListener(eventName, requestPermission,
			{ capture: true, passive: true });
	}
	eventTarget.addEventListener("devicemotion", event => {
		const hasUserAcceleration = event.acceleration?.x != null;
		const acceleration = hasUserAcceleration
			? event.acceleration
			: event.accelerationIncludingGravity;
		const rotation = event.rotationRate;
		samples.push({
			elapsedMs: Math.round(now()),
			ax: acceleration?.x ?? 0,
			ay: acceleration?.y ?? 0,
			az: acceleration?.z ?? 0,
			rx: rotation?.alpha ?? 0,
			ry: rotation?.beta ?? 0,
			rz: rotation?.gamma ?? 0,
		});
		if (samples.length > MAX_MOTION_SAMPLES) {
			samples.splice(0, samples.length - MAX_MOTION_SAMPLES);
		}
		diagnostic.samplesCaptured += 1;
		diagnostic.samplesQueued = samples.length;
		diagnostic.accelerationSource = hasUserAcceleration ? "user" : "gravity-included";
	}, { passive: true });

	return { diagnostic, requestPermission, samples };
}
