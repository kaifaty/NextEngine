import { StreamingLinearResampler } from "./pcmResampler";

const TARGET_SAMPLE_RATE = 16_000;

export interface AudioCaptureCallbacks {
  onChunk: (chunk: Uint8Array) => void;
  onLevel: (rms: number) => void;
}

export interface AudioCaptureInfo {
  contextSampleRate: number;
  trackSampleRate: number | null;
  echoCancellation: boolean | null;
  noiseSuppression: boolean | null;
  autoGainControl: boolean | null;
  startedAtMonotonicMs: number;
}

export interface NoiseCalibration {
  noiseFloorDbfs: number;
  durationMs: number;
  peakDbfs: number;
}

class PcmChunker {
  private readonly resampler: StreamingLinearResampler;
  private readonly output: number[] = [];

  constructor(
    inputSampleRate: number,
    private readonly chunkSamples: number,
    private readonly onChunk: (chunk: Uint8Array) => void,
  ) {
    this.resampler = new StreamingLinearResampler(inputSampleRate, TARGET_SAMPLE_RATE);
  }

  push(samples: Float32Array): void {
    this.output.push(...this.resampler.push(samples));
    this.emit(false);
  }

  flush(): void {
    this.output.push(...this.resampler.flush());
    this.emit(true);
  }

  private emit(flush: boolean): void {
    while (this.output.length >= this.chunkSamples || (flush && this.output.length > 0)) {
      const length = Math.min(this.chunkSamples, this.output.length);
      const samples = this.output.splice(0, length);
      const bytes = new Uint8Array(length * 2);
      const view = new DataView(bytes.buffer);
      samples.forEach((sample, index) => {
        const clamped = Math.max(-1, Math.min(1, sample));
        const value = clamped < 0 ? Math.round(clamped * 32_768) : Math.round(clamped * 32_767);
        view.setInt16(index * 2, value, true);
      });
      this.onChunk(bytes);
    }
  }
}

export class BrowserMicrophoneCapture {
  private context: AudioContext | null = null;
  private stream: MediaStream | null = null;
  private source: MediaStreamAudioSourceNode | null = null;
  private processor: AudioWorkletNode | null = null;
  private sink: GainNode | null = null;
  private chunker: PcmChunker | null = null;

  async start(
    deviceId: string,
    chunkMs: number,
    callbacks: AudioCaptureCallbacks,
  ): Promise<AudioCaptureInfo> {
    if (this.context !== null) {
      throw new Error("microphone capture is already active");
    }
    this.stream = await navigator.mediaDevices.getUserMedia(microphoneConstraints(deviceId));
    this.context = createAudioContext();
    await this.context.audioWorklet.addModule(
      new URL("../audio/pcm-worklet.js", import.meta.url),
    );
    this.chunker = new PcmChunker(
      this.context.sampleRate,
      Math.max(1, Math.round((TARGET_SAMPLE_RATE * chunkMs) / 1_000)),
      callbacks.onChunk,
    );
    this.source = this.context.createMediaStreamSource(this.stream);
    this.processor = new AudioWorkletNode(this.context, "nextengine-pcm-processor");
    this.sink = this.context.createGain();
    this.sink.gain.value = 0;
    this.processor.port.onmessage = (message: MessageEvent<Float32Array | { type: string }>) => {
      const samples = message.data;
      if (!(samples instanceof Float32Array)) return;
      let energy = 0;
      for (const sample of samples) {
        energy += sample * sample;
      }
      callbacks.onLevel(samples.length > 0 ? Math.sqrt(energy / samples.length) : 0);
      this.chunker?.push(samples);
    };
    this.source.connect(this.processor);
    this.processor.connect(this.sink);
    this.sink.connect(this.context.destination);
    await this.context.resume();
    const settings = this.stream.getAudioTracks()[0]?.getSettings();
    return {
      contextSampleRate: this.context.sampleRate,
      trackSampleRate: settings?.sampleRate ?? null,
      echoCancellation: settings?.echoCancellation ?? null,
      noiseSuppression: settings?.noiseSuppression ?? null,
      autoGainControl: settings?.autoGainControl ?? null,
      startedAtMonotonicMs: performance.now(),
    };
  }

  async stop(): Promise<void> {
    const chunker = this.chunker;
    this.source?.disconnect();
    this.source = null;
    if (this.processor !== null) {
      await this.flushProcessor(this.processor);
      this.processor.port.onmessage = null;
      this.processor.disconnect();
      this.processor = null;
    }
    this.chunker = null;
    this.sink?.disconnect();
    this.sink = null;
    for (const track of this.stream?.getTracks() ?? []) {
      track.stop();
    }
    this.stream = null;
    chunker?.flush();
    const context = this.context;
    this.context = null;
    if (context !== null && context.state !== "closed") {
      await context.close();
    }
  }

  private flushProcessor(processor: AudioWorkletNode): Promise<void> {
    return new Promise((resolve) => {
      const previous = processor.port.onmessage;
      let settled = false;
      const finish = () => {
        if (settled) return;
        settled = true;
        clearTimeout(timeout);
        processor.port.onmessage = previous;
        resolve();
      };
      const timeout = window.setTimeout(finish, 1_000);
      processor.port.onmessage = (message) => {
        if (
          typeof message.data === "object" &&
          message.data !== null &&
          !(message.data instanceof Float32Array) &&
          (message.data as { type?: unknown }).type === "flushed"
        ) {
          finish();
          return;
        }
        previous?.call(processor.port, message);
      };
      processor.port.postMessage({ type: "flush" });
    });
  }
}

/**
 * Measures ambient microphone energy without retaining or transmitting audio.
 * The user should remain quiet during this short calibration window.
 */
export async function calibrateMicrophoneNoise(
  deviceId: string,
  durationMs = 2_000,
): Promise<NoiseCalibration> {
  if (!Number.isInteger(durationMs) || durationMs < 500 || durationMs > 10_000) {
    throw new Error("длительность калибровки должна быть от 0.5 до 10 секунд");
  }
  const stream = await navigator.mediaDevices.getUserMedia(microphoneConstraints(deviceId));
  const context = createAudioContext();
  let source: MediaStreamAudioSourceNode | null = null;
  let processor: AudioWorkletNode | null = null;
  let sink: GainNode | null = null;
  const levels: number[] = [];
  try {
    await context.audioWorklet.addModule(
      new URL("../audio/pcm-worklet.js", import.meta.url),
    );
    source = context.createMediaStreamSource(stream);
    processor = new AudioWorkletNode(context, "nextengine-pcm-processor");
    sink = context.createGain();
    sink.gain.value = 0;
    processor.port.onmessage = (message: MessageEvent<Float32Array | { type: string }>) => {
      if (!(message.data instanceof Float32Array) || message.data.length === 0) return;
      let energy = 0;
      for (const sample of message.data) energy += sample * sample;
      levels.push(toDbfs(Math.sqrt(energy / message.data.length)));
    };
    source.connect(processor);
    processor.connect(sink);
    sink.connect(context.destination);
    await context.resume();
    await new Promise<void>((resolve) => window.setTimeout(resolve, durationMs));
  } finally {
    source?.disconnect();
    processor?.disconnect();
    processor?.port.close();
    sink?.disconnect();
    for (const track of stream.getTracks()) track.stop();
    if (context.state !== "closed") await context.close();
  }
  if (levels.length < 20) {
    throw new Error("микрофон не выдал достаточно аудио для калибровки");
  }
  const median = percentile(levels, 0.5);
  const noiseFloorDbfs = Math.max(-90, percentile(levels, 0.9));
  const peakDbfs = percentile(levels, 0.99);
  if (noiseFloorDbfs - median > 15) {
    throw new Error("во время калибровки слышна речь или переменный шум — повторите в тишине");
  }
  if (noiseFloorDbfs > -15) {
    throw new Error("уровень шума вне поддерживаемого диапазона; проверьте микрофон");
  }
  return { noiseFloorDbfs, durationMs, peakDbfs };
}

export async function listAudioInputs(): Promise<MediaDeviceInfo[]> {
  if (!navigator.mediaDevices?.enumerateDevices) {
    return [];
  }
  const devices = await navigator.mediaDevices.enumerateDevices();
  return devices.filter((device) => device.kind === "audioinput");
}

function microphoneConstraints(deviceId: string): MediaStreamConstraints {
  return {
    audio: {
      ...(deviceId ? { deviceId: { exact: deviceId } } : {}),
      channelCount: 1,
      echoCancellation: false,
      noiseSuppression: false,
      autoGainControl: false,
    },
    video: false,
  };
}

function createAudioContext(): AudioContext {
  try {
    return new AudioContext({ latencyHint: "interactive", sampleRate: TARGET_SAMPLE_RATE });
  } catch {
    return new AudioContext({ latencyHint: "interactive" });
  }
}

function toDbfs(rms: number): number {
  return 20 * Math.log10(Math.max(rms, 0.00000001));
}

function percentile(values: number[], quantile: number): number {
  const ordered = [...values].sort((left, right) => left - right);
  const index = Math.max(0, Math.min(ordered.length - 1, Math.ceil(quantile * ordered.length) - 1));
  return ordered[index] ?? -120;
}
