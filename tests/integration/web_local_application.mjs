import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

import {
	applicationRecordFromFile,
	parseAedArchive,
} from "../../packaging/web/local-application.mjs";
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

const archiveBytes = await readFile(new URL("../../demos/vibesteroids.aed", import.meta.url));
const packaged = parseAedArchive(archiveBytes);
assert.match(decoder.decode(packaged.wat), /\(module/);
assert.deepEqual(
	packaged.assets.map(asset => asset.name),
	["assets/audio/satellite-destroyed.flac"],
);
assert.ok(packaged.assets[0].bytes.byteLength > 0);

const unsupportedCompression = Uint8Array.from(archiveBytes);
new DataView(
	unsupportedCompression.buffer,
	unsupportedCompression.byteOffset,
	unsupportedCompression.byteLength,
).setUint16(8, 8, true);
assert.throws(
	() => parseAedArchive(unsupportedCompression),
	/unsupported compression/,
);

await assert.rejects(
	() => applicationRecordFromFile({
		name: "not-an-application.zip",
		arrayBuffer: async () => archiveBytes.buffer,
	}),
	/choose a \.wat or \.aed file/,
);
