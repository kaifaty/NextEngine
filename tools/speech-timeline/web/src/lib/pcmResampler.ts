export class StreamingLinearResampler {
  private readonly ratio: number;
  private source: number[] = [];
  private sourceStart = 0;
  private nextSourcePosition = 0;
  private inputSamples = 0;

  constructor(inputSampleRate: number, outputSampleRate: number) {
    if (inputSampleRate <= 0 || outputSampleRate <= 0) {
      throw new Error("sample rates must be positive");
    }
    this.ratio = inputSampleRate / outputSampleRate;
  }

  push(samples: Float32Array): number[] {
    for (const sample of samples) {
      this.source.push(Number.isFinite(sample) ? sample : 0);
    }
    this.inputSamples += samples.length;
    return this.drain(false);
  }

  flush(): number[] {
    if (this.inputSamples === 0 || this.source.length === 0) {
      this.reset();
      return [];
    }
    this.source.push(this.source[this.source.length - 1] ?? 0);
    const output = this.drain(true);
    this.reset();
    return output;
  }

  private drain(final: boolean): number[] {
    const output: number[] = [];
    const lastSourcePosition = this.sourceStart + this.source.length - 1;
    while (
      this.nextSourcePosition + 1 <= lastSourcePosition &&
      (!final || this.nextSourcePosition < this.inputSamples)
    ) {
      const localPosition = this.nextSourcePosition - this.sourceStart;
      const index = Math.floor(localPosition);
      const fraction = localPosition - index;
      const first = this.source[index] ?? 0;
      const second = this.source[index + 1] ?? first;
      output.push(first + (second - first) * fraction);
      this.nextSourcePosition += this.ratio;
    }

    const desiredStart = Math.floor(this.nextSourcePosition);
    const removable = Math.max(0, desiredStart - this.sourceStart);
    const removeCount = Math.min(removable, Math.max(0, this.source.length - 1));
    if (removeCount > 0) {
      this.source.splice(0, removeCount);
      this.sourceStart += removeCount;
    }
    return output;
  }

  private reset(): void {
    this.source = [];
    this.sourceStart = 0;
    this.nextSourcePosition = 0;
    this.inputSamples = 0;
  }
}
