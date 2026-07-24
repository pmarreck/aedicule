function validatePcmRequest(request, invalidRequestMessage) {
	const {
		sampleRate,
		channels,
		samples,
		volume,
		pitch,
	} = request;
	if (!Number.isInteger(sampleRate) || sampleRate <= 0
		|| !Number.isInteger(channels) || channels <= 0
		|| !(samples instanceof Float32Array) || samples.length % channels !== 0
		|| !Number.isFinite(volume) || !Number.isFinite(pitch)) {
		throw new TypeError(invalidRequestMessage);
	}
}

/// Adapts interleaved host PCM to Web Audio while exposing measurable graph
/// lifecycle evidence through an injected BaseAudioContext and reporter.
export async function schedulePcmPlayback(
	context,
	request,
	{
		report = () => {},
		resume = candidate => candidate.resume(),
		invalidRequestMessage = "Invalid PCM playback request.",
	} = {},
) {
	validatePcmRequest(request, invalidRequestMessage);
	const {
		sampleRate,
		channels,
		samples,
		volume,
		pitch,
	} = request;
	const frames = samples.length / channels;
	const buffer = context.createBuffer(channels, frames, sampleRate);
	let nonzeroSamples = 0;
	let peak = 0;
	let energy = 0;
	for (let channel = 0; channel < channels; channel += 1) {
		const output = buffer.getChannelData(channel);
		for (let frame = 0; frame < frames; frame += 1) {
			const sample = samples[frame * channels + channel];
			output[frame] = sample;
			const magnitude = Math.abs(sample);
			if (magnitude > 0) nonzeroSamples += 1;
			peak = Math.max(peak, magnitude);
			energy += sample * sample;
		}
	}
	const rootMeanSquare = samples.length === 0
		? 0
		: Math.sqrt(energy / samples.length);
	report("pcm-admitted", {
		sampleRate,
		channels,
		sampleCount: samples.length,
		frames,
		nonzeroSamples,
		peak,
		rootMeanSquare,
	});

	const source = context.createBufferSource();
	const gain = context.createGain();
	source.buffer = buffer;
	source.playbackRate.value = pitch;
	gain.gain.value = volume;
	source.connect(gain);
	gain.connect(context.destination);
	let finishPlayback;
	const ended = new Promise(resolve => {
		finishPlayback = resolve;
	});
	source.onended = () => {
		report("source-ended", {
			frames,
			durationSeconds: buffer.duration,
		});
		finishPlayback();
	};

	if (context.state === "suspended") await resume(context);
	source.start();
	report("source-started", {
		frames,
		durationSeconds: buffer.duration,
		nonzeroSamples,
		peak,
		rootMeanSquare,
	});
	return {
		buffer,
		source,
		gain,
		ended,
		frames,
		nonzeroSamples,
		peak,
		rootMeanSquare,
	};
}
