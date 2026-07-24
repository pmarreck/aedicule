import assert from "node:assert/strict";
import { schedulePcmPlayback } from "../../web/audio.mjs";

class FakeAudioBuffer {
	constructor(channels, frames, sampleRate) {
		this.duration = frames / sampleRate;
		this.channelData = Array.from(
			{ length: channels },
			() => new Float32Array(frames),
		);
	}

	getChannelData(channel) {
		return this.channelData[channel];
	}
}

class FakeAudioNode {
	constructor(name, calls) {
		this.name = name;
		this.calls = calls;
	}

	connect(destination) {
		this.calls.push(`connect:${this.name}->${destination.name}`);
		return destination;
	}
}

class FakeBufferSource extends FakeAudioNode {
	constructor(calls) {
		super("source", calls);
		this.playbackRate = { value: 1 };
		this.started = false;
		this.onended = null;
	}

	start() {
		this.started = true;
		this.calls.push("start");
	}

	finish() {
		this.calls.push("finish");
		this.onended?.();
	}
}

class FakeAudioContext {
	constructor() {
		this.state = "suspended";
		this.calls = [];
		this.destination = new FakeAudioNode("destination", this.calls);
		this.buffers = [];
		this.sources = [];
	}

	createBuffer(channels, frames, sampleRate) {
		this.calls.push(`buffer:${channels}x${frames}@${sampleRate}`);
		const buffer = new FakeAudioBuffer(channels, frames, sampleRate);
		this.buffers.push(buffer);
		return buffer;
	}

	createBufferSource() {
		this.calls.push("source");
		const source = new FakeBufferSource(this.calls);
		this.sources.push(source);
		return source;
	}

	createGain() {
		this.calls.push("gain");
		const gain = new FakeAudioNode("gain", this.calls);
		gain.gain = { value: 1 };
		return gain;
	}
}

async function schedulesNonSilentPcmThroughTheCompleteGraph() {
	const context = new FakeAudioContext();
	const reports = [];
	const playback = await schedulePcmPlayback(
		context,
		{
			sampleRate: 48_000,
			channels: 2,
			samples: new Float32Array([0, 0.25, -0.5, 0.75]),
			volume: 0.4,
			pitch: 1.25,
		},
		{
			report: (stage, detail) => reports.push({ stage, ...detail }),
			resume: async candidate => {
				candidate.calls.push("resume");
				candidate.state = "running";
			},
		},
	);

	assert.deepEqual(
		Array.from(context.buffers[0].channelData[0]),
		[0, -0.5],
		"interleaved PCM must fill the left channel",
	);
	assert.deepEqual(
		Array.from(context.buffers[0].channelData[1]),
		[0.25, 0.75],
		"interleaved PCM must fill the right channel",
	);
	assert.deepEqual(context.calls, [
		"buffer:2x2@48000",
		"source",
		"gain",
		"connect:source->gain",
		"connect:gain->destination",
		"resume",
		"start",
	]);
	assert.equal(playback.nonzeroSamples, 3);
	assert.equal(playback.peak, 0.75);
	assert.equal(playback.rootMeanSquare, Math.sqrt(0.875 / 4));
	assert.equal(playback.frames, 2);
	assert.equal(playback.source.playbackRate.value, 1.25);
	assert.equal(playback.gain.gain.value, 0.4);
	assert.equal(reports.at(-1).stage, "source-started");

	playback.source.finish();
	await playback.ended;
	assert.equal(reports.at(-1).stage, "source-ended");
}

async function refusesToStartWhenAContextCannotResume() {
	const context = new FakeAudioContext();
	await assert.rejects(
		schedulePcmPlayback(
			context,
			{
				sampleRate: 48_000,
				channels: 1,
				samples: new Float32Array([0.5]),
				volume: 1,
				pitch: 1,
			},
			{
				resume: async candidate => {
					candidate.calls.push("resume-failed");
					throw new Error("autoplay denied");
				},
			},
		),
		/autoplay denied/,
	);
	assert.equal(context.sources[0].started, false);
	assert.equal(context.calls.includes("start"), false);
}

await schedulesNonSilentPcmThroughTheCompleteGraph();
await refusesToStartWhenAContextCannotResume();
