# Speech timeline: bounded optimization research (2026-08-18)

## Scope

This note records a bounded follow-up after the persistent-audio-buffer fix in
`e42e0f3`. The subject is the current resident `Voxtral Mini 4B Realtime` plus
`emotion2vec_plus_base` service on the Linux RTX 3080 development host. It is
not a design for future stateless ASR models and it does not change an engine
public contract.

The falsifiable questions were:

1. Can larger PCM messages reduce scheduler/FFI overhead without worsening
   first-result latency or emotion delivery?
2. Is PCM conversion or emotion metadata work a material part of the hot path?
3. Is the remaining delay caused by a growing audio buffer, or by serialized
   model work and the finalization path?

## Current evidence

The resident process loads each model once. On the same 30-second paced WAV,
the 80 ms transport baseline measured approximately 31.15 s wall time,
real-time factor 1.038, worker busy RTF 0.523, first affect 1.234 s, first
transcript 2.274 s and finish-to-final 0.924 s. It submitted 375 Voxtral push
jobs, 113 stable emotion jobs, had maximum scheduler depth 2 and no overloads.

The ASR push total was 13.18 s, while emotion stable inference was only 1.50 s;
emotion stable queue wait totalled 5.13 s. This points to scheduling and
serialization—not an unbounded turn buffer—as the next wrapper-level target.

### Chunk-size A/B

The benchmark used the same resident process, profile, audio and paced input;
only `benchmark --chunk-ms` changed. Results are single-run diagnostics, so
they establish direction rather than a quality claim.

| transport chunk | ASR pushes (10 s / 30 s) | ASR inference pattern | first affect (10 s) | stable-emotion queue (10 s) | conclusion |
| ---: | ---: | --- | ---: | ---: | --- |
| 80 ms | 125 / 375 | many zero-cost feeds, decode roughly every 240 ms | 1.094 s | p95 118 ms | best current interleaving |
| 160 ms | 63 / not run | fewer calls, but decode work moves into each call | 1.134 s | p95 157 ms | no measured wall-time win |
| 240 ms | 42 / 125 | p50 about 118 ms on every feed | 1.330 s | p95 123 ms; 3.78 s total at 10 s | worsens emotion scheduling |
| 480 ms | 21 / not run | p50 about 222 ms on every feed | 1.672 s | 18 stable jobs coalesced/stale at 10 s | unacceptable for responsive UI |

At 30 s, 240 ms reduced pushes from 375 to 125, but wall time only moved
31.155 s to 31.059 s; first affect moved 1.234 s to 1.327 s and worker busy
RTF worsened 0.523 to 0.551. The native stream therefore benefits from
frequent small feeds because its configured 240 ms decode cadence can yield
between feeds. A larger network message is not equivalent to a larger native
feed for this scheduler.

### Conversion and metadata microbenchmarks

The current Python PCM16-to-float32 path uses `array.array` plus a Python
generator. A local 10,000-iteration microbenchmark on 16 kHz buffers measured
the following conversion-only times:

| audio in one call | current path | NumPy vectorized path | direction |
| ---: | ---: | ---: | ---: |
| 80 ms | 0.105 ms | 0.0026 ms | about 40x faster |
| 240 ms | 0.304 ms | 0.0031 ms | about 98x faster |
| 480 ms | 0.614 ms | 0.0042 ms | about 146x faster |

This is a real local CPU saving, but even the current 80 ms path is only about
40 ms over 30 seconds of ASR feeds. It is worth applying because it is low
risk, but it is not expected to solve the multi-second tail by itself.

The in-memory emotion probe also hashes every waveform window to preserve the
standalone probe schema. Hashing a 2.4 s float32 window measured about 0.076 ms
locally, or under 9 ms across 117 windows. Runtime emotion calls do not consume
that metadata, so a metadata-free internal fast path is sensible, but again is
not the primary bottleneck.

## External constraints and prior art

- The [official Voxtral realtime model card](https://huggingface.co/mistralai/Voxtral-Mini-4B-Realtime-2602)
  describes causal streaming, 80 ms token slots and configurable delays up to
  2.4 s; its published FLEURS table shows the expected latency/quality tradeoff
  (the 480 ms setting is materially better than 240 ms for Russian).
- The [Mistral realtime transcription documentation](https://docs.mistral.ai/studio/audio/speech_to_text/realtime_transcription)
  separates microphone chunk duration from `target_streaming_delay_ms` and
  demonstrates fast provisional plus slower final delay policies. This supports
  measuring the two controls independently; it does not justify running two
  local GPU streams without a VRAM/throughput budget.
- The [FunASR streaming tutorial](https://modelscope.github.io/FunASR/tutorial.html)
  keeps streaming ASR state in a persistent cache and flushes with `is_final`.
  Its emotion2vec example is utterance-granularity, not a stateful frame-level
  emotion API; therefore overlapping bounded windows remain the correct current
  adapter strategy.
- The [websockets server reference](https://websockets.readthedocs.io/en/13.1/reference/asyncio/server.html)
  documents bounded `max_queue` and `write_limit` defaults. Current event
  payloads are at most about 3.3 KiB and send latency rounds to zero, so the
  WebSocket transport is not presently the measured bottleneck.

## Recommended next experiments, in order

### 1. Preserve 80 ms ingress; instrument scheduler slices

Keep the current 80 ms transport default. Add per-job `queued_at`, `started_at`
and `finished_at` diagnostics (without audio or prompt contents), plus separate
`pcm_convert_ms`, native `feed_ms`, emotion inference and event-send timings.
The acceptance question is whether queue wait, not model inference, dominates
the p95 first-affect and finish-to-final paths under a 30/60-second corpus.

### 2. Reduce emotion pressure without changing the ASR stream

The current cadence produces 117 emotion windows in 30 seconds (1 s fast and
2 s stable windows with 250 ms hops). Test a 500 ms stable hop and a
voice-activity/change-triggered policy. Keep the 250 ms timeline output by
holding or smoothing the last observation; do not fabricate word-level emotion
spans. Compare first-affect latency, label churn, transition-boundary F1 and
ASR queue wait. A CPU VAD gate is an optional experiment, with explicit finish
remaining the fallback endpoint.

### 3. Vectorize PCM conversion

Prototype a little-endian NumPy `frombuffer(..., dtype="<i2")` conversion and
verify that the local transcribe.cpp binding accepts the contiguous buffer
without changing samples. Add byte-order and equivalence tests, then compare
worker RTF and p95 queue wait. This is a safe micro-optimization, not a reason
to increase transport chunk size.

### 4. Remove unused runtime emotion metadata

Add an internal `include_metadata=False` path to `EmotionProbe.analyze_waveform`;
keep the standalone file/CLI output hash and audio metadata unchanged. Measure
allocation count and CPU time before/after. Do not remove the public schema.

### 5. Measure, then consider lane separation

The current single worker serializes ASR and emotion. First run emotion2vec on a
bounded CPU worker and compare inference time, GPU residency and ASR p95. Only
if CPU emotion is acceptably fast should a separate lane be considered. A second
CUDA worker/stream is a later experiment: it can reduce queue wait but may
increase VRAM pressure and completion-order races. Every result must remain
revision/generation checked.

### 6. Tune delay as a quality/latency policy

Do not lower Voxtral from 480 ms by default. Run a Russian, clean/noisy and
emotion-transition corpus at 240/480/960 ms and report WER, revision churn and
first-confirmed-word latency. A dual provisional/final policy is attractive,
but two resident local streams should be rejected unless the measured GPU
budget and scheduler isolation pass.

## Explicitly deferred

- WebSocket compression/write-limit tuning: no current send or browser-buffer
  evidence warrants it.
- `torch.compile`, ONNX conversion or model quantization: these are separate
  model/runtime experiments with warm-up and quality risk, not wrapper fixes.
- Ring-buffer trimming for long turns: the current stateful stream and 30 s
  bounded session already avoid re-analyzing a growing file. Do not apply the
  stateless-model architecture to the current model.
- Aggressive noise suppression or denoising: it may remove prosodic cues needed
  by emotion2vec; evaluate only as a paired ASR-plus-emotion quality experiment.

## Decision

The next implementation slice should be instrumentation plus emotion cadence
control, not larger input chunks. PCM vectorization and runtime metadata
elision can follow as low-risk cleanups. The decision to split workers or alter
the 480 ms model delay requires a corpus-based quality/latency report; the
current smoke benchmarks are insufficient for that product claim.
