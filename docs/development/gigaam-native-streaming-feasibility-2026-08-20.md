# GigaAM-v3 native streaming feasibility

| Field | Value |
| --- | --- |
| Date | `2026-08-20` |
| Status | Bounded research; local `gigastt` baseline implemented; native training is not authorized |
| Scope | Preserve GigaAM-v3 Russian quality while producing bounded stateful partial ASR for dialogue prewarm |
| Installed control | `ai-sage/GigaAM-v3`, revision `7655ad717f8122257385bb4b2f373db3697e8680`, `e2e_rnnt` |
| Upstream inspected | `salute-developers/GigaAM` commit `7447938d791c4f3e643386ee22c33777004293a5`; all public branches/history and open PRs checked `2026-08-20` |

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

The missing part is substantial: the current official repository does not ship
chunk/causal training controls, cache-aware encoder inference, or a stateful
streaming decode API. Upstream issue 18 requesting streaming inference remains
open. A maintainer also confirmed in issue 70 that the released checkpoints
were fine-tuned in full-context mode and that enabling local attention only at
inference can noticeably degrade quality. A useful derivative therefore needs
streaming-aware fine-tuning, not only an inference mask or wrapper.

For the game prototype, the lowest-risk near-term design is two-pass:

```text
native streaming ASR -> replaceable partial text -> cancelable intent/LLM prewarm
                 endpoint -> GigaAM-v3 final pass -> canonical admitted text
```

This hides dialogue latency while retaining GigaAM as the final Russian quality
authority. A trained GigaAM streaming student can replace the first pass later
without changing the facade.

## Full upstream audit: what is new

The official `main` at `7447938` and its seven published branches contain no
native streaming encoder or cache-aware inference surface. Searching the full
public history found only offline long-form/chunking work and the open MLX pull
request described below. The repository has no published Git tags or GitHub
releases; model releases remain on Hugging Face.

The following recent upstream work is useful but does not change that result:

- issue 70 provides the missing authoritative clarification: the published
  checkpoints were trained full-context; the paper's local-attention graph used
  a matching training regime; `transcribe_longform` plus VAD is the maintainer's
  recommendation for long files;
- PR 62 adds Apple MLX live-microphone demos, but its source calls the path
  `pseudo-streaming` and re-runs an offline model over a sliding/growing buffer;
- PR 73 fixes ONNX RNN-T decoder-state view retention, aligns the symbol limit
  with the PyTorch path and exposes token frames. These are valuable runtime and
  timing fixes inside complete decodes, not persistent encoder caches;
- PR 85 proposes an optional Silero VAD backend for `transcribe_longform`;
- PR 86 proposes utterance/segment/word confidence and raw token log-probability
  output. It is explicitly uncalibrated, but would still help revision gating,
  final-pass routing and pseudo-label filtering.

The most relevant new community implementation is
[`ekhodzitsky/gigastt`](https://github.com/ekhodzitsky/gigastt), linked from
official issue 76 but not maintained or endorsed by Salute. Version `2.18.0` at
commit `7bc17f438ebd4daaf8956aa2373cf77635bff41c` is an MIT Rust/ONNX Runtime
server/crate with WebSocket partials, C ABI and CPU/CUDA/CoreML providers. It
implements bounded buffered emulation over the offline GigaAM-v3 RNN-T:

```text
100 ms transport frames
  -> accumulate 0.8 s of new audio
  -> re-encode a retained window capped at 2.5 s
  -> fresh RNN-T decode for the overlapping window
  -> suppress the 1.5 s already-emitted left context
  -> replace live tail / commit stable prefix
  -> VAD or blank-run endpoint
```

This avoids the duration-slope failure of growing-buffer re-decode, but it is
not native streaming: the offline encoder and fresh decoder recompute the
overlapping window. The project's own 100-clip paired measurements report
`WER_batch -> WER_stream` of `4.97 -> 19.46` on Golos crowd and
`4.82 -> 15.42` on Golos far-field, a regression of `+14.49` and `+10.60`
percentage points respectively. Author-reported warm Apple M1 CPU latency is
TTFP p50 `1.653 s` on crowd and `0.820 s` on far-field, with per-partial compute
and queue lag p50 `51/41 ms`. These numbers are not verified on our RTX 3080,
microphone or whisper corpus.

`gigastt` is therefore a high-value immediate baseline behind the existing
facade, advertised honestly as `buffered_emulation`. It may be sufficient for
cancelable early-intent prewarm while final GigaAM remains authoritative. Its
published streaming quality gap prevents promoting it as the canonical ASR
without a local paired evaluation.

## Local GigaSTT integration result

The Phase 1 service now exposes `gigastt-v3-rnnt-buffered` beside Voxtral and
final-only GigaAM through the same per-turn selector. The pinned external
artifact set is `gigastt 2.18.0` plus the release's GigaAM-v3 RNNT INT8 ONNX
encoder/decoder/joint and vocabulary. Startup validates every digest and the
runtime version. One CPU sidecar remains resident; a turn uses GigaSTT's local
WebSocket with 16 kHz PCM, manual endpointing, VAD/punctuation/ITN disabled.
Capabilities explicitly report `streaming_mode=buffered_emulation`, 800 ms
partial cadence and the 2.5 s/1.5 s rolling-window policy in the dashboard.

One paced diagnostic take (`7.216 s`, raw route, one run per ASR) produced:

| Route | First transcript | Finish to final | Busy RTF | Observed final text |
| --- | ---: | ---: | ---: | --- |
| Voxtral Realtime Q4 | `7975.609 ms` | `739.321 ms` | `0.519678` | empty |
| GigaAM-v3 e2e RNNT final-only | `7768.445 ms` | `530.612 ms` | `0.073032` | `Знает он, что у него хвостик есть один всего.` |
| GigaSTT / GigaAM-v3 RNNT INT8 | `1766.664 ms` | `105.998 ms` | `0.014690` | `знает год у него хвостик есть один` |

The GigaSTT sidecar added about `283400 KiB` RSS and no CUDA allocation; the
main joint process reported about `2600964 KiB` RSS and `6344 MiB` VRAM after
the run. The single take confirms the desired latency shape but also shows the
known quality trade-off: GigaSTT produced text early, while its final diverged
from the stronger offline GigaAM result. This clip has no frozen reference
transcript, so these strings are qualitative evidence, not WER. The three
content-free timing reports remain external under `/tmp/nextengine-*-comparison.json`.

No promotion follows from one take. The next discriminator remains a frozen,
reference-transcribed normal/whisper/noise set with paired WER/CER, empty-rate,
first useful partial and revision-churn measurements.

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
2. Benchmark pinned `gigastt` as a no-training `buffered_emulation` control.
   Measure first useful partial, revision churn, WER/CER, empty-rate, duration
   slope and final agreement against current Voxtral and offline GigaAM through
   the same front-end routes. Do not relabel it native streaming.
3. Implement the paper's fixed chunk attention and chunkwise-causal convolution
   geometry behind a new experimental model/runtime identifier. First prove
   that its full-context mode reproduces the pinned checkpoint.
4. Add state objects for feature framing, subsampling tails, per-layer
   attention/convolution caches and RNN-T predictor state. Prove that feeding
   the same WAV in different transport frame sizes produces the same final
   hypothesis and that per-chunk work has no duration slope.
5. Evaluate untrained 1 s and 2 s chunk modes only as negative controls. The
   maintainer's full-context clarification means an inference-only switch is
   expected to regress, but the measurement still bounds the starting point.
6. Only if the architecture control is correct, fine-tune one fixed-latency
   checkpoint. Start with the ordinary character RNN-T output for provisional
   text; punctuation and normalization are not required for intent prewarm and
   can remain in the final `e2e_rnnt` pass.
7. Compare the streaming student, `gigastt`, current Voxtral and offline GigaAM through
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
- [Maintainer clarification on chunkwise inference](https://github.com/salute-developers/GigaAM/issues/70#issuecomment-4372753044) — released checkpoints are full-context and inference-only local attention can degrade quality.
- [MLX pseudo-streaming PR](https://github.com/salute-developers/GigaAM/pull/62) — community sliding-buffer microphone implementation for Apple Silicon.
- [ONNX RNN-T decoder-state fix PR](https://github.com/salute-developers/GigaAM/pull/73) — state-view memory fix and token-frame output within offline decode.
- [Token-confidence PR](https://github.com/salute-developers/GigaAM/pull/86) — proposed uncalibrated confidence surfaces.
- [`gigastt` implementation](https://github.com/ekhodzitsky/gigastt) and [streaming protocol/results](https://github.com/ekhodzitsky/gigastt/blob/main/docs/benchmarks.md#streaming-measurement-protocol) — bounded Rust/ONNX buffered-emulation baseline and paired WER/latency evidence.
- [Stateful Conformer with cache-based inference](https://arxiv.org/abs/2312.17279) — bounded context and activation-cache design for true streaming Conformers.
