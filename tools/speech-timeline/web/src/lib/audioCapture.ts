const TARGET_SAMPLE_RATE = 16_000;

export interface AudioCaptureCallbacks {
  onChunk: (chunk: Uint8Array) => void;
  onLevel: (rms: number) => void;
}

class PcmChunker {
  private readonly ratio: number;
  private readonly output: number[] = [];
  private source: number[] = [];
  private sourcePosition = 0;

  constructor(
    inputSampleRate: number,
    private readonly chunkSamples: number,
    private readonly onChunk: (chunk: Uint8Array) => void,
  ) {
    this.ratio = inputSampleRate / TARGET_SAMPLE_RATE;
  }

  push(samples: Float32Array): void {
    for (const sample of samples) {
      this.source.push(Number.isFinite(sample) ? sample : 0);
    }
    this.resample(false);
  }

  flush(): void {
    if (this.source.length > 0) {
      this.source.push(this.source[this.source.length - 1]);
      this.resample(true);
    }
    this.emit(true);
  }

  private resample(final: boolean): void {
    while (this.sourcePosition + 1 < this.source.length) {
      const index = Math.floor(this.sourcePosition);
      const fraction = this.sourcePosition - index;
      const first = this.source[index] ?? 0;
      const second = this.source[index + 1] ?? first;
      this.output.push(first + (second - first) * fraction);
      this.sourcePosition += this.ratio;
      this.emit(false);
    }
    const consumed = Math.floor(this.sourcePosition);
    if (consumed > 0) {
      this.source.splice(0, consumed);
      this.sourcePosition -= consumed;
    }
    if (final) {
      this.source = [];
      this.sourcePosition = 0;
    }
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
  ): Promise<void> {
    if (this.context !== null) {
      throw new Error("microphone capture is already active");
    }
    this.stream = await navigator.mediaDevices.getUserMedia({
      audio: {
        ...(deviceId ? { deviceId: { exact: deviceId } } : {}),
        channelCount: 1,
        echoCancellation: true,
        noiseSuppression: true,
        autoGainControl: true,
      },
      video: false,
    });
    this.context = new AudioContext({ latencyHint: "interactive" });
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
    this.processor.port.onmessage = (message: MessageEvent<Float32Array>) => {
      const samples = message.data;
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
  }

  async stop(): Promise<void> {
    const chunker = this.chunker;
    this.chunker = null;
    if (this.processor !== null) {
      this.processor.port.onmessage = null;
      this.processor.disconnect();
      this.processor = null;
    }
    this.source?.disconnect();
    this.source = null;
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
}

export async function listAudioInputs(): Promise<MediaDeviceInfo[]> {
  if (!navigator.mediaDevices?.enumerateDevices) {
    return [];
  }
  const devices = await navigator.mediaDevices.enumerateDevices();
  return devices.filter((device) => device.kind === "audioinput");
}
