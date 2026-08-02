const AED_MIME_TYPE = "application/vnd.aedicule.app+zip";
const MAX_APPLICATION_BYTES = 64 * 1024 * 1024;
const MAX_ENTRY_BYTES = 16 * 1024 * 1024;
const LOCAL_DATABASE = "aedicule-local-applications-v1";
const LOCAL_STORE = "applications";
const RETAINED_APPLICATIONS = 4;

function bytesOf(value) {
	return value instanceof Uint8Array
		? value
		: new Uint8Array(value);
}

/**
 * Normalizes a browser File into a bounded local application record. A `.aed`
 * crosses this boundary as raw bytes: the Wasm runtime's shared Rust archive
 * reader is the only `.aed` implementation, replacing the JavaScript zip
 * parser that silently diverged from it (stored entries only) the moment
 * packages became compressed. JavaScript checks nothing here but the size
 * bound and the four-byte zip signature that makes error messages immediate.
 */
export async function applicationRecordFromFile(file) {
	const extension = file.name.toLowerCase().match(/\.[^.]+$/)?.[0];
	if (extension !== ".wat" && extension !== ".aed") {
		throw new Error("choose a .wat or .aed file");
	}
	const bytes = new Uint8Array(await file.arrayBuffer());
	if (extension === ".aed") {
		if (bytes.byteLength > MAX_APPLICATION_BYTES) {
			throw new Error(`.aed exceeds ${MAX_APPLICATION_BYTES} bytes`);
		}
		if (bytes.byteLength < 4
			|| bytes[0] !== 0x50 || bytes[1] !== 0x4b || bytes[2] !== 0x03 || bytes[3] !== 0x04) {
			throw new Error("invalid .aed: not a ZIP archive");
		}
		return { package: bytes };
	}
	if (bytes.byteLength === 0) throw new Error("selected .wat file is empty");
	if (bytes.byteLength > MAX_ENTRY_BYTES) {
		throw new Error(`.wat exceeds ${MAX_ENTRY_BYTES} bytes`);
	}
	new TextDecoder("utf-8", { fatal: true }).decode(bytes);
	return { wat: bytes, assets: [] };
}

function requestValue(request) {
	return new Promise((resolve, reject) => {
		request.addEventListener("success", () => resolve(request.result), { once: true });
		request.addEventListener("error", () => reject(request.error), { once: true });
	});
}

function transactionCompletion(transaction) {
	return new Promise((resolve, reject) => {
		transaction.addEventListener("complete", resolve, { once: true });
		transaction.addEventListener("abort", () => reject(transaction.error), { once: true });
		transaction.addEventListener("error", () => reject(transaction.error), { once: true });
	});
}

async function openLocalDatabase(indexedDb) {
	if (indexedDb === undefined) throw new Error("this browser does not provide local app storage");
	const request = indexedDb.open(LOCAL_DATABASE, 1);
	request.addEventListener("upgradeneeded", () => {
		const store = request.result.createObjectStore(LOCAL_STORE, { keyPath: "token" });
		store.createIndex("createdAt", "createdAt");
	}, { once: true });
	return requestValue(request);
}

async function pruneLocalApplications(store) {
	const count = await requestValue(store.count());
	let remaining = Math.max(0, count - RETAINED_APPLICATIONS);
	if (remaining === 0) return;
	await new Promise((resolve, reject) => {
		const request = store.index("createdAt").openKeyCursor();
		request.addEventListener("error", () => reject(request.error), { once: true });
		request.addEventListener("success", () => {
			const cursor = request.result;
			if (cursor === null || remaining === 0) {
				resolve();
				return;
			}
			store.delete(cursor.primaryKey);
			remaining -= 1;
			cursor.continue();
		});
	});
}

/**
 * Stores selected bytes only in origin-local IndexedDB and returns an opaque
 * token suitable for a same-origin runner URL; no upload or network API exists.
 */
export async function storeLocalApplication(
	application,
	{
		indexedDb = globalThis.indexedDB,
		randomUuid = () => globalThis.crypto.randomUUID(),
		now = () => Date.now(),
	} = {},
) {
	const database = await openLocalDatabase(indexedDb);
	try {
		const transaction = database.transaction(LOCAL_STORE, "readwrite");
		const completed = transactionCompletion(transaction);
		const store = transaction.objectStore(LOCAL_STORE);
		const token = randomUuid();
		store.put({ token, createdAt: now(), ...application });
		await pruneLocalApplications(store);
		await completed;
		return token;
	} finally {
		database.close();
	}
}

/**
 * Retrieves a previously selected local application without granting the WAT
 * guest access to IndexedDB or any other browser storage namespace.
 */
export async function loadLocalApplication(
	token,
	{ indexedDb = globalThis.indexedDB } = {},
) {
	if (!/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i.test(token)) {
		throw new Error("invalid local application token");
	}
	const database = await openLocalDatabase(indexedDb);
	try {
		const transaction = database.transaction(LOCAL_STORE, "readonly");
		const completed = transactionCompletion(transaction);
		const record = await requestValue(transaction.objectStore(LOCAL_STORE).get(token));
		await completed;
		if (record === undefined) throw new Error("local application is no longer available");
		if (record.package !== undefined) return { package: bytesOf(record.package) };
		return { wat: bytesOf(record.wat), assets: record.assets.map(asset => ({
			name: asset.name,
			bytes: bytesOf(asset.bytes),
		})) };
	} finally {
		database.close();
	}
}
