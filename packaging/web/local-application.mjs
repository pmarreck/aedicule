const AED_MIME_TYPE = "application/vnd.aedicule.app+zip";
const MAX_APPLICATION_BYTES = 64 * 1024 * 1024;
const MAX_ENTRY_BYTES = 16 * 1024 * 1024;
const MAX_ENTRIES = 1024;
const LOCAL_DATABASE = "aedicule-local-applications-v1";
const LOCAL_STORE = "applications";
const RETAINED_APPLICATIONS = 4;
const ZIP_LOCAL_ENTRY = 0x04034b50;
const ZIP_CENTRAL_ENTRY = 0x02014b50;
const ZIP_END = 0x06054b50;

function bytesOf(value) {
	return value instanceof Uint8Array
		? value
		: new Uint8Array(value);
}

function boundedSlice(bytes, start, length, description) {
	const end = start + length;
	if (!Number.isSafeInteger(end) || start < 0 || end > bytes.byteLength) {
		throw new Error(`truncated .aed ${description}`);
	}
	return bytes.slice(start, end);
}

function validEntryName(name) {
	return name.length > 0
		&& !name.includes("\\")
		&& !name.includes(":")
		&& !name.includes("\0")
		&& name.normalize("NFC") === name
		&& name.split("/").every(component => (
			component !== "" && component !== "." && component !== ".."
		));
}

function crc32(bytes) {
	let crc = 0xffff_ffff;
	for (const byte of bytes) {
		crc ^= byte;
		for (let bit = 0; bit < 8; bit += 1) {
			crc = (crc >>> 1) ^ (0xedb8_8320 & -(crc & 1));
		}
	}
	return (crc ^ 0xffff_ffff) >>> 0;
}

function findZipEnd(bytes, view) {
	const earliest = Math.max(0, bytes.byteLength - 65_557);
	for (let offset = bytes.byteLength - 22; offset >= earliest; offset -= 1) {
		if (view.getUint32(offset, true) === ZIP_END) return offset;
	}
	throw new Error("invalid .aed: missing ZIP end record");
}

/**
 * Validates Aedicule's bounded stored-ZIP package profile in memory, exposing
 * only the guest WAT and capability-scoped asset subtree to the browser host.
 */
export function parseAedArchive(value) {
	const bytes = bytesOf(value);
	if (bytes.byteLength > MAX_APPLICATION_BYTES) {
		throw new Error(`.aed exceeds ${MAX_APPLICATION_BYTES} bytes`);
	}
	if (bytes.byteLength < 22) throw new Error("invalid .aed: truncated ZIP");
	const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
	const decoder = new TextDecoder("utf-8", { fatal: true });
	const endOffset = findZipEnd(bytes, view);
	const disk = view.getUint16(endOffset + 4, true);
	const centralDisk = view.getUint16(endOffset + 6, true);
	const entriesOnDisk = view.getUint16(endOffset + 8, true);
	const entryCount = view.getUint16(endOffset + 10, true);
	const centralOffset = view.getUint32(endOffset + 16, true);
	if (disk !== 0 || centralDisk !== 0 || entriesOnDisk !== entryCount) {
		throw new Error("unsupported multi-disk .aed archive");
	}
	if (entryCount > MAX_ENTRIES + 1) {
		throw new Error(`.aed contains more than ${MAX_ENTRIES} application entries`);
	}

	let offset = 0;
	let entries = 0;
	let totalBytes = 0;
	const names = new Set();
	let wat;
	const assets = [];
	while (offset < centralOffset) {
		if (offset + 30 > bytes.byteLength || view.getUint32(offset, true) !== ZIP_LOCAL_ENTRY) {
			throw new Error("invalid .aed local entry");
		}
		const flags = view.getUint16(offset + 6, true);
		const compression = view.getUint16(offset + 8, true);
		const expectedCrc = view.getUint32(offset + 14, true);
		const compressedSize = view.getUint32(offset + 18, true);
		const uncompressedSize = view.getUint32(offset + 22, true);
		const nameLength = view.getUint16(offset + 26, true);
		const extraLength = view.getUint16(offset + 28, true);
		if ((flags & 1) !== 0) throw new Error("encrypted .aed entries are unsupported");
		if ((flags & 8) !== 0) throw new Error(".aed data descriptors are unsupported");
		if (compression !== 0) throw new Error("unsupported compression in .aed entry");
		if (compressedSize !== uncompressedSize) {
			throw new Error("invalid stored .aed entry size");
		}
		if (uncompressedSize > MAX_ENTRY_BYTES) {
			throw new Error(`.aed entry exceeds ${MAX_ENTRY_BYTES} bytes`);
		}
		const nameStart = offset + 30;
		const dataStart = nameStart + nameLength + extraLength;
		const name = decoder.decode(boundedSlice(bytes, nameStart, nameLength, "entry name"));
		const data = boundedSlice(bytes, dataStart, compressedSize, `entry ${name}`);
		if (!validEntryName(name)) throw new Error(`invalid .aed entry path: ${name}`);
		if (!names.add(name)) throw new Error(`duplicate .aed entry: ${name}`);
		if (crc32(data) !== expectedCrc) throw new Error(`invalid .aed checksum: ${name}`);
		entries += 1;
		if (entries > MAX_ENTRIES + 1) {
			throw new Error(`.aed contains more than ${MAX_ENTRIES} application entries`);
		}
		totalBytes += data.byteLength;
		if (totalBytes > MAX_APPLICATION_BYTES) {
			throw new Error(`.aed contents exceed ${MAX_APPLICATION_BYTES} bytes`);
		}
		if (entries === 1) {
			if (name !== "mimetype" || decoder.decode(data) !== AED_MIME_TYPE) {
				throw new Error(`.aed must begin with ${AED_MIME_TYPE} mimetype`);
			}
		} else if (name === "code.wat") {
			wat = data;
		} else if (name.startsWith("assets/")) {
			assets.push({ name, bytes: data });
		}
		offset = dataStart + compressedSize;
	}
	if (offset !== centralOffset || view.getUint32(centralOffset, true) !== ZIP_CENTRAL_ENTRY) {
		throw new Error("invalid .aed central directory");
	}
	if (entries !== entryCount) throw new Error("invalid .aed entry count");
	if (wat === undefined) throw new Error(".aed package lacks code.wat");
	return { wat, assets };
}

/**
 * Normalizes a browser File into the same bounded virtual application record
 * whether the visitor selected a bare WAT or an asset-bearing `.aed` package.
 */
export async function applicationRecordFromFile(file) {
	const extension = file.name.toLowerCase().match(/\.[^.]+$/)?.[0];
	if (extension !== ".wat" && extension !== ".aed") {
		throw new Error("choose a .wat or .aed file");
	}
	const bytes = new Uint8Array(await file.arrayBuffer());
	if (extension === ".aed") return parseAedArchive(bytes);
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
		return { wat: bytesOf(record.wat), assets: record.assets.map(asset => ({
			name: asset.name,
			bytes: bytesOf(asset.bytes),
		})) };
	} finally {
		database.close();
	}
}
