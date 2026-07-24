import {
	formatMessage,
	loadCatalog,
	localizeDocument,
} from "./launcher-i18n.mjs";
import {
	applicationRecordFromFile,
	storeLocalApplication,
} from "./local-application.mjs";

export function choosePreviewRotation(random = Math.random) {
	return random() < 0.5 ? "counterclockwise" : "clockwise";
}

function installPreviewMotion(document, random = Math.random) {
	const preview = document.querySelector("[data-vibesteroids-preview]");
	if (preview === null) return;
	const chooseDirection = () => {
		preview.dataset.rotation = choosePreviewRotation(random);
	};
	preview.addEventListener("pointerenter", chooseDirection);
	preview.addEventListener("focusin", chooseDirection);
}

function installLocalApplicationLauncher(
	document,
	{
		location = globalThis.location,
		navigator = globalThis.navigator,
	} = {},
) {
	const dropzone = document.getElementById("local-application-dropzone");
	const picker = document.getElementById("local-application-picker");
	const status = document.getElementById("local-application-status");
	if (dropzone === null || picker === null || status === null) return;
	const strings = loadCatalog(navigator.languages).strings;

	const launch = async file => {
		status.dataset.state = "loading";
		status.textContent = strings.loadingLocalApplication;
		console.info("[Aedicule gallery]", "local-application-selected", JSON.stringify({
			name: file.name,
			bytes: file.size,
		}));
		try {
			const application = await applicationRecordFromFile(file);
			const token = await storeLocalApplication(application);
			console.info("[Aedicule gallery]", "local-application-stored", JSON.stringify({
				assets: application.assets.length,
				watBytes: application.wat.byteLength,
			}));
			const destination = new URL("./run/", location.href);
			destination.searchParams.set("local", token);
			location.assign(destination);
		} catch (error) {
			console.error("[Aedicule gallery]", "local-application-failed", error);
			status.dataset.state = "error";
			status.textContent = formatMessage(strings.localApplicationFailed, {
				message: error instanceof Error ? error.message : String(error),
			});
		}
	};

	picker.addEventListener("change", () => {
		const file = picker.files?.[0];
		if (file !== undefined) launch(file);
	});
	for (const eventName of ["dragenter", "dragover"]) {
		dropzone.addEventListener(eventName, event => {
			event.preventDefault();
			if (event.dataTransfer !== null) event.dataTransfer.dropEffect = "copy";
			dropzone.dataset.drag = "active";
		});
	}
	for (const eventName of ["dragleave", "drop"]) {
		dropzone.addEventListener(eventName, event => {
			event.preventDefault();
			delete dropzone.dataset.drag;
		});
	}
	dropzone.addEventListener("drop", event => {
		const file = event.dataTransfer?.files?.[0];
		if (file !== undefined) launch(file);
	});
}

if (typeof document !== "undefined") {
	localizeDocument(document, navigator.languages);
	installPreviewMotion(document);
	installLocalApplicationLauncher(document);
}
