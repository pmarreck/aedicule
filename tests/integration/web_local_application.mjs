import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

import { applicationRecordFromFile } from "../../packaging/web/local-application.mjs";
import { choosePreviewRotation } from "../../packaging/web/launcher.mjs";

const encoder = new TextEncoder();
const decoder = new TextDecoder("utf-8", { fatal: true });

assert.equal(choosePreviewRotation(() => 0), "counterclockwise");
assert.equal(choosePreviewRotation(() => 0.499_999), "counterclockwise");
assert.equal(choosePreviewRotation(() => 0.5), "clockwise");
assert.equal(choosePreviewRotation(() => 0.999_999), "clockwise");

const bareWat = encoder.encode("(module)");
const bareRecord = await applicationRecordFromFile({
	name: "small example.wat",
	arrayBuffer: async () => bareWat.buffer,
});
assert.equal(decoder.decode(bareRecord.wat), "(module)");
assert.deepEqual(bareRecord.assets, []);

// A `.aed` crosses into storage as raw bytes: the Wasm runtime's Rust archive
// reader is the only `.aed` implementation, so JavaScript must not expand,
// filter, or otherwise interpret the archive. Byte identity is the contract.
const archiveBytes = new Uint8Array(
	await readFile(new URL("../../demos/vibesteroids.aed", import.meta.url)),
);
const packaged = await applicationRecordFromFile({
	name: "vibesteroids.aed",
	arrayBuffer: async () => archiveBytes.buffer,
});
assert.equal(packaged.wat, undefined);
assert.deepEqual(packaged.package, archiveBytes);

await assert.rejects(
	() => applicationRecordFromFile({
		name: "not-a-zip.aed",
		arrayBuffer: async () => encoder.encode("plain text").buffer,
	}),
	/not a ZIP archive/,
);

await assert.rejects(
	() => applicationRecordFromFile({
		name: "not-an-application.zip",
		arrayBuffer: async () => archiveBytes.buffer,
	}),
	/choose a \.wat or \.aed file/,
);
