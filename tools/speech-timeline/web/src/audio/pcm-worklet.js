class NextEnginePcmProcessor extends AudioWorkletProcessor {
  constructor() {
    super();
    this.pending = [];
    this.targetSamples = Math.max(128, Math.round(sampleRate * 0.02));
    this.port.onmessage = (message) => {
      if (message.data?.type === "flush") {
        this.emitPending(true);
        this.port.postMessage({ type: "flushed" });
      }
    };
  }

  process(inputs, outputs) {
    const input = inputs[0]?.[0];
    if (input && input.length > 0) {
      for (const sample of input) this.pending.push(sample);
      this.emitPending(false);
    }
    for (const output of outputs) {
      for (const channel of output) {
        channel.fill(0);
      }
    }
    return true;
  }

  emitPending(flush) {
    while (this.pending.length >= this.targetSamples || (flush && this.pending.length > 0)) {
      const length = Math.min(this.targetSamples, this.pending.length);
      const block = new Float32Array(this.pending.splice(0, length));
      this.port.postMessage(block, [block.buffer]);
    }
  }
}

registerProcessor("nextengine-pcm-processor", NextEnginePcmProcessor);
