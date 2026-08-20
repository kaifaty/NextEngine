# GigaAM-v3 native streaming feasibility

| Field | Value |
| --- | --- |
| Date | `2026-08-20` |
| Status | Bounded research; implementation and training are not authorized |
| Scope | Preserve GigaAM-v3 Russian quality while producing bounded stateful partial ASR for dialogue prewarm |
| Installed control | `ai-sage/GigaAM-v3`, revision `7655ad717f8122257385bb4b2f373db3697e8680`, `e2e_rnnt` |
| Upstream inspected | `salute-developers/GigaAM` commit `7447938d791c4f3e643386ee22c33777004293a5` |

## Conclusion

The installed GigaAM-v3 checkpoint is not a native streaming model even though
its decoder is RNN-T. Its public encoder attends to the complete utterance, its
Conformer convolutions are symmetric, and its public decode path recreates the
RNN-T predictor state for every `transcribe()` call. Re-decoding a growing or
overlapping waveform would be buffered emulation, not bounded stateful
streaming.

A genuine streaming derivative is nevertheless technically plausible. The
GigaAM authors pre-trained the v3 SSL encoder with dynamic chunkwise attention
specifically to support later full-context or streaming fine-tuning. Their
paper reports fixed-chunk streaming experiments and uses a small convolution
kernel compatible with bounded future context. This makes adaptation much more
promising than converting an arbitrary full-context Conformer from scratch.

The missing part is substantial: the current public repository does not ship
chunk/causal training controls, cache-aware encoder inference, or a stateful
streaming decode API. Upstream issue 18 requesting streaming inference remains
open. We would have to implement and validate those pieces ourselves before
fine-tuning can be evaluated honestly.

For the game prototype, the lowest-risk near-term design is two-pass:

```text
native streaming ASR -> replaceable partial text -> cancelable intent/LLM prewarm
                 endpoint -> GigaAM-v3 final pass -> canonical admitted text
```

This hides dialogue latency while retaining GigaAM as the final Russian quality
authority. A trained GigaAM streaming student can replace the first pass later
without changing the facade.

## What the current checkpoint actually does

The pinned `e2e_rnnt` configuration is a 16-layer, 768-wide rotary Conformer
with 4x convolutional subsampling, kernel size 5, and a 1025-class RNN-T head.
Local inspection of the exact model code found:

- for a single utterance the encoder passes no causal or chunk attention mask;
- every attention layer forms query/key/value over the complete encoded time
  axis and exposes no K/V cache;
- subsampling and Conformer convolution use symmetric padding and expose no
  persistent convolution state;
- the RNN-T LSTM has a usable hidden state, but `_greedy_decode()` owns it only
  for one complete call and discards it afterward;
- `transcribe()` computes the whole feature sequence and encoder output before
  decoding, while `transcribe_longform()` performs VAD segmentation and separate
  complete-utterance calls.

The current upstream source and fine-tuning CLI expose CTC/RNN-T training,
mixed precision, gradient accumulation, activation checkpointing and RNN-T
loss sub-batching, but no public chunk-size, causal-convolution, left-context,
lookahead, cache or streaming-inference option.

## How native streaming ASR is built

A true streaming Conformer-RNN-T retains bounded state at every layer:

```text
PCM frames
  -> stateful log-mel framing
  -> subsampling convolution tails
  -> [bounded left cache | current chunk | fixed lookahead]
  -> per-layer attention K/V + convolution caches
  -> new encoder frames only
  -> persistent RNN-T predictor hidden state + last token
  -> replaceable partial transcript revision
```

Training must match this inference geometry. Attention is restricted to chunks
and bounded history, future context is limited to the configured lookahead, and
Conformer convolutions are causal or chunkwise-causal. The model learns token
emission without relying on audio that will not yet exist at runtime. RNN-T is
useful because its predictor state is naturally incremental, but it does not
make a bidirectional encoder streamable by itself.

The runtime then caches intermediate activations so each frame is encoded once.
Without those caches, a fixed rolling window can cap work but repeatedly
recomputes overlap and requires transcript stitching; a growing window makes
per-update cost increase with utterance duration.

## Why GigaAM is a credible adaptation base

The GigaAM paper reports dynamic chunk-size sampling during SSL pre-training
over 1, 2, 4 and 8 second chunks. The authors state that fixed chunk size is
preferable during fine-tuning because dynamic fine-tuning degraded quality.
For the dynamically pre-trained encoder, their reported streaming WER was
approximately `5.49` at a 1 s chunk, `4.56` at 2 s, `3.77` at 4 s and `3.44`
at 8 s; the 200 ms configuration degraded to `10.37`. These are paper-level
controls, not expected scores for the released e2e checkpoint or our microphone.

The paper's main experiment used 100,000 hours for SSL pre-training and about
2,000 labelled hours / 100,000 steps for supervised fine-tuning. We should not
repeat SSL pre-training. We can initialize from the released v3 weights, but a
production-quality streaming derivative still needs representative labelled
Russian speech and paired normal/whisper/noise evaluation.

## Proposed bounded feasibility sequence

Do not start with a large training run.

1. Freeze a small reference-transcribed Russian microphone set with normal,
   whisper, noise, pauses and short/long utterances. Record offline GigaAM WER,
   empty-rate and endpoint-to-final latency as the quality ceiling.
2. Implement the paper's fixed chunk attention and chunkwise-causal convolution
   geometry behind a new experimental model/runtime identifier. First prove
   that its full-context mode reproduces the pinned checkpoint.
3. Add state objects for feature framing, subsampling tails, per-layer
   attention/convolution caches and RNN-T predictor state. Prove that feeding
   the same WAV in different transport frame sizes produces the same final
   hypothesis and that per-chunk work has no duration slope.
4. Evaluate untrained 1 s and 2 s chunk modes. This tests how much of the
   dynamically chunked SSL capability survived full-context ASR fine-tuning.
5. Only if the architecture control is correct, fine-tune one fixed-latency
   checkpoint. Start with the ordinary character RNN-T output for provisional
   text; punctuation and normalization are not required for intent prewarm and
   can remain in the final `e2e_rnnt` pass.
6. Compare the streaming student, current Voxtral and offline GigaAM through
   the same facade and frozen corpus. Promote nothing based on a microphone
   impression or borrowed benchmark.

Suggested first operating points are 1 s and 2 s. One second is the more useful
latency target for early intent; two seconds is the quality fallback indicated
by the authors' ablation. A 200 ms target is not justified by their results.

## Cost and difficulty

This is a medium-to-large ASR research task, not a wrapper change.

- A rolling-window demonstration without training is small, but it is not an
  admissible native-streaming route and risks unstable duplicated text.
- A cache-correct fixed-chunk inference prototype is likely weeks of focused
  model/runtime work because every encoder state boundary needs parity tests.
- A useful fine-tuned prototype adds data preparation, RNN-T training and
  repeated WER/latency experiments. The repository supports memory-saving
  training controls, but the current 10-GiB RTX 3080 is best treated as an
  inference and small-experiment device. Full 240M-parameter RNN-T adaptation
  would be substantially faster and less fragile on rented 24–80 GiB GPUs.
- Production quality adds real whisper/noise/domain data, partial-stability and
  endpointing evaluation, export/runtime optimization and failure handling.

An engineering estimate should therefore be split into a short architectural
feasibility spike, then a go/no-go training phase. It would be misleading to
promise that a small LoRA or decoder-only fine-tune can solve streaming: the
encoder's attention and convolution behavior is the primary boundary.

## Admission gates

- first useful partial and partial-gap p95 reported from speech onset;
- WER/CER and empty-rate separated by normal/whisper/noise;
- fixed per-chunk compute and bounded cache memory across utterance duration;
- transport-frame invariance and offline/chunk parity controls;
- revision churn and conservative stable-prefix measurements;
- endpoint-to-final p95 and final agreement with offline GigaAM;
- resident RAM/VRAM with at least the configured physical safety margin;
- no world action, TTS or irreversible LLM result admitted from provisional
  text.

## Rejected shortcuts

- Treating the presence of an RNN-T head as proof of streaming support.
- Re-decoding the complete accumulated waveform on every update.
- Calling overlapping fixed-window transcription native streaming without
  declaring recomputation, stitching and revision semantics.
- Fine-tuning only the predictor/head while leaving the full-context encoder
  unchanged.
- Training at several chunk sizes before one fixed geometry has cache/parity
  evidence; the GigaAM paper found dynamic fine-tuning degraded quality.
- Repeating the authors' full SSL pre-training before testing the released
  dynamically chunk-pretrained initialization.

## Primary sources

- [GigaAM-v3 model card](https://huggingface.co/ai-sage/GigaAM-v3) — released variants, Russian evaluation and pinned architecture surface.
- [GigaAM paper](https://arxiv.org/abs/2506.01192) — dynamic chunk pre-training, fixed-chunk/streaming ablations, datasets and training scale.
- [Official GigaAM repository](https://github.com/salute-developers/GigaAM) — current inference and fine-tuning implementation.
- [Official fine-tuning guide](https://github.com/salute-developers/GigaAM/blob/main/train_utils/README.md) — available RNN-T training and memory controls.
- [Open upstream streaming request](https://github.com/salute-developers/GigaAM/issues/18) — streaming inference is not yet a shipped public API.
- [Stateful Conformer with cache-based inference](https://arxiv.org/abs/2312.17279) — bounded context and activation-cache design for true streaming Conformers.

