const MAX_TOUCH_QUEUE = 256;
const CONTROL_COLLECTIONS = Object.freeze([
	"panels", "sliders", "buttons", "externalLinks", "textFields",
]);

function pointInRect(x, y, rect) {
	return x >= rect.x && y >= rect.y
		&& x < rect.x + rect.width && y < rect.y + rect.height;
}

function controlOccludes(snapshot, x, y) {
	return CONTROL_COLLECTIONS.some(name =>
		Array.isArray(snapshot?.[name])
		&& snapshot[name].some(rect => pointInRect(x, y, rect))
	);
}

/**
 * Captures identity-bearing touch PointerEvents before GPUI lowers them to a
 * primary mouse button. The Rust adapter remains authoritative for bounds,
 * identity transitions, terminal coordinates, and guest delivery.
 */
export function createTouchBridge({
	globalObject = globalThis,
	eventTarget = globalThis,
	canvas = () => document.querySelector("canvas"),
} = {}) {
	const queue = [];
	const occluded = new Set();
	const raw = new Set();
	globalObject.__AEDICULE_TOUCH_EVENTS = queue;

	const enabled = () => Number.isInteger(globalObject.__AEDICULE_TOUCH_MAX_CONTACTS)
		&& globalObject.__AEDICULE_TOUCH_MAX_CONTACTS > 0;
	const enqueue = event => {
		if (queue.length >= MAX_TOUCH_QUEUE) {
			queue.splice(0, queue.length, { phase: "cancel-all" });
			occluded.clear();
			raw.clear();
			return;
		}
		queue.push(event);
	};
	const cancelAll = () => {
		if (!enabled()) return;
		occluded.clear();
		raw.clear();
		if (queue.at(-1)?.phase !== "cancel-all") enqueue({ phase: "cancel-all" });
	};
	const pointer = phase => event => {
		if (!enabled() || event.pointerType !== "touch") return;
		const targetCanvas = canvas();
		if (targetCanvas === null || targetCanvas === undefined) return;
		const bounds = targetCanvas.getBoundingClientRect();
		const x = event.clientX - bounds.left;
		const y = event.clientY - bounds.top;
		if (!Number.isInteger(event.pointerId) || !Number.isFinite(x) || !Number.isFinite(y)) return;

		if (phase === "start") {
			if (controlOccludes(globalObject.__AEDICULE_UI_SNAPSHOT, x, y)) {
				occluded.add(event.pointerId);
				return;
			}
			raw.add(event.pointerId);
			targetCanvas.setPointerCapture?.(event.pointerId);
		} else if (occluded.has(event.pointerId)) {
			if (phase === "end" || phase === "cancel") occluded.delete(event.pointerId);
			return;
		} else if (!raw.has(event.pointerId)) {
			return;
		}
		event.preventDefault();
		event.stopImmediatePropagation();
		enqueue({ phase, id: event.pointerId, x, y });
		if (phase === "end" || phase === "cancel") raw.delete(event.pointerId);
	};

	eventTarget.addEventListener("pointerdown", pointer("start"), {
		capture: true, passive: false,
	});
	eventTarget.addEventListener("pointermove", pointer("move"), {
		capture: true, passive: false,
	});
	eventTarget.addEventListener("pointerup", pointer("end"), {
		capture: true, passive: false,
	});
	eventTarget.addEventListener("pointercancel", pointer("cancel"), {
		capture: true, passive: false,
	});
	eventTarget.addEventListener("lostpointercapture", pointer("cancel"), {
		capture: true, passive: false,
	});
	eventTarget.addEventListener("blur", cancelAll, { capture: true });
	return { cancelAll };
}
