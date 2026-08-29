# Voxtral Realtime + emotion2vec timeline facade research

| Field | Value |
| --- | --- |
| Date | 2026-08-18 |
| Status | Bounded research; implementation and product adoption remain unapproved |
| Selected experiment | Voxtral Mini 4B Realtime 2602 Q4_K_M + emotion2vec/emotion2vec_plus_base |
| Target | Resident local microphone service producing revisioned text, vocal-affect intervals and an LLM-ready fused turn |
| Host envelope | Linux x86_64, NVIDIA GeForce RTX 3080 10 GiB |

## Decision

Use one resident optional speech-timeline service with replaceable, role-based
adapters:

```text
16 kHz mono PCM + integer sample clock
        │
        ├── VoxtralTranscriberAdapter ── text revisions / optional timed units ─┐
        ├── Emotion2VecAffectAdapter ─── overlapping affect observations ───────┤
        └── optional VAD adapter ──────── speech regions / endpoint ─────────────┤
                                                                                ▼
                                                                  TimelineFusion
                                                                                │
                                  structured timeline + final canonical text + derived LLM context
```

The facade must not be named after either model. Use a service role such as
`SpeechTimelineService`, with `StreamingTranscriber`, `VocalAffectAnalyzer`
and optional `SpeechEndpointDetector` adapters. Model selection is startup
configuration and a capability handshake, not a type in the consumer API.

The existing microphone wrapper at Git object
`e839a38:lab/scripts/voxtral_microphone.py` is the correct Voxtral adapter seed.
It already captures the required PCM shape, loads the model once per invocation,
keeps one streaming session, feeds 250 ms chunks by default, and renders current
text. It is a foreground single-client CLI, not a background service: a new CLI
invocation reloads the weights.

The immediate implementation should therefore extract its capture-independent
model/session logic into an adapter and put it behind a resident daemon. Do not
create a second Voxtral loading path.

## The critical timing limitation

Voxtral Realtime is natively streaming. The official report describes aligned
audio and text streams at 12.5 Hz: one model step represents 80 ms of audio, and
the configured transcription delay is expressed in those steps. Mistral
recommends 480 ms as the quality/latency operating point. This makes the model a
good candidate for low-latency text, but it does **not** mean the currently
available serving APIs return word timestamps.

The official vLLM Realtime WebSocket protocol returns only:

- `transcription.delta` with incremental text;
- `transcription.done` with final text and usage.

It contains no token position, word interval or timestamp field. The current
local `transcribe.cpp` runtime is equally explicit:

- Voxtral advertises `TRANSCRIBE_TIMESTAMPS_NONE`;
- its published segment is the entire current transcript over `[0, audio_ms]`;
- `transcribe_stream_update.audio_committed_ms` is documented as a family
  progress/drain hint, not a text boundary;
- in `transcribe.cpp` commit `9315160`, the Voxtral family never advances
  `stream_audio_committed_us` during `feed()`, so the wrapper observes `0` until
  `finalize()` sets it to the total audio duration;
- `committed_text` is an append-only display convenience derived from repeated
  prefix agreement. The runtime documents it as best-effort; `full_text` remains
  the authoritative current hypothesis.

Consequently, using `audio_committed_ms` to attach a newly committed phrase to
an emotion interval would fabricate precision. The facade must declare timing
quality and degrade deliberately.

## Three supported alignment grades

| Grade | ASR evidence | Allowed fusion | Current Voxtral state |
| --- | --- | --- | --- |
| `utterance` | Provisional/final text without timed lexical units | Whole-turn or VAD-region vocal-affect context; no word-level claim | Works with the current wrapper/runtime |
| `model_slot` | Lexical/control token plus its synchronized 80 ms output slot | Approximate word/clause grouping after model-specific calibration | Requires a small `transcribe.cpp` adapter extension and validation |
| `word` | Explicit acoustic word `[start_sample,end_sample)` intervals | Direct overlap-weighted word/span tags | Requires an ASR that exposes word timestamps or an optional final aligner |

The stable facade capability should be `timing_precision`, not a boolean
`supports_timestamps`. Suggested values are `none`, `utterance`, `model_slot`,
`segment`, and `word`, together with `resolution_samples` and an evidence label.

### Why `model_slot` is worth testing

Inside the evaluated `transcribe.cpp` implementation, Voxtral preserves every
generated token, including control tokens, in decoder order. Decoder output is
produced against synchronized 80 ms audio embeddings. A private runtime adapter
can expose a bounded event such as:

```json
{
  "token_id": 1234,
  "text": "обманул",
  "output_slot": 87,
  "lexical_kind": "text"
}
```

The Voxtral adapter can then group lexical tokens using its word-boundary
control tokens and map output slots back to approximate acoustic time using the
configured delay. That mapping must remain adapter-private: another ASR may use
CTC frames, RNN-T timestamps or true words.

This is a research hypothesis, not yet a timestamp guarantee. The exact slot
offset, multi-token-word grouping and boundary error must be measured against a
small manually aligned Russian corpus. Until that passes, the public result is
`utterance`, not `model_slot`.

### Optional exact final path

If model-slot error is too large for acceptable emotion attachment, add an
optional `TranscriptAligner` role after end of speech. It accepts finalized
audio plus finalized text and returns timed words. This adds another model,
latency and provenance burden, so it should not block the Live baseline. It may
produce a second, final-only fusion revision while the Live path continues to
use utterance/model-slot context.

## Facade boundary

The consumer should depend on behavioral roles and declared capabilities:

```text
SpeechTimelineService
  start_session(SessionConfig) -> SessionReady
  push_pcm(PcmChunk)            -> zero or more TimelineEvent
  finish_input()                -> UtteranceFinal
  cancel()

StreamingTranscriber
  capabilities() -> TranscriberCapabilities
  start()/push_pcm()/finish()/reset()

VocalAffectAnalyzer
  capabilities() -> AffectCapabilities
  observe(AudioWindow) -> AffectObservation
```

Minimum transcriber capabilities:

- native sample rate and accepted PCM encoding;
- languages and language-hint support;
- native streaming versus repeated-window emulation;
- provisional-revision model (`replace_all`, `stable_prefix`, timed units);
- `timing_precision` and resolution;
- supported latency/delay range;
- maximum practical session duration;
- model/runtime identity, revision, artifact hash and license/provenance status;
- declared resident CPU/GPU resource envelope.

Minimum affect capabilities:

- native sample rate;
- streaming state or required window/hop ranges;
- label vocabulary and stable normalized mapping;
- score semantics (`uncalibrated_observed_expression` for the current model);
- whether embeddings, frame features or utterance labels are available;
- model/runtime identity, revision, artifact hash and license/provenance status.

Configuration selects adapters by role:

```toml
[speech_timeline]
transcriber = "voxtral_realtime_transcribe_cpp"
affect = "emotion2vec_plus_base"
endpoint = "silero_vad"

[speech_timeline.transcriber_options]
delay_ms = 480
language = "ru"

[speech_timeline.affect_options]
fast_window_ms = 1000
stable_window_ms = 2000
hop_ms = 250
```

Swapping to a future ASR or emotion model changes configuration plus its adapter.
The facade rejects incompatible PCM/timing requirements at `start_session`, or
reports the explicit reduced alignment grade. It never silently invents fields.

## Stable timeline model

All acoustic data uses one integer sample clock, initially 16 kHz, with
half-open intervals `[start_sample,end_sample)`. Wall time is recorded only for
latency measurement and never determines content alignment.

Keep three independently revisioned tracks because untimed ASR text cannot
safely share one sample replacement watermark with emotion intervals:

1. `transcript`: authoritative current `full_text`, optional best-effort stable
   prefix, finality, and optional timed lexical units;
2. `vocal_affect`: raw overlapping score vectors plus smoothed intervals on the
   audio clock;
3. `fusion`: derived text spans/LLM context with an explicit alignment grade.

Suggested update shape:

```json
{
  "schema_version": 1,
  "type": "speech_timeline.update",
  "session_id": "opaque",
  "revision": 19,
  "audio_received_samples": 56000,
  "transcript": {
    "revision": 8,
    "text": "Но ты меня обманул",
    "stable_prefix_utf8_bytes": 13,
    "is_final": false,
    "timing_precision": "utterance",
    "units": []
  },
  "vocal_affect": {
    "revision": 11,
    "replace_from_sample": 32000,
    "segments": [
      {
        "start_sample": 32000,
        "end_sample": 52000,
        "label": "angry",
        "model_score": 0.91,
        "stability": 0.78,
        "state": "provisional",
        "semantics": "observed_vocal_expression"
      }
    ]
  },
  "fusion": {
    "revision": 6,
    "alignment_grade": "utterance",
    "spans": [],
    "turn_expression": "angry"
  }
}
```

When timed units become available, `fusion.spans` contains lexical unit IDs or
word ranges plus the overlap-weighted affect evidence. Untimed text continues
to work unchanged and receives only turn/region-level metadata.

At `finish_input`, keep:

- finalized plain `canonical_text`;
- structured timed affect evidence;
- optional timed lexical units and fused spans;
- a derived LLM context view.

Inline tags are not authority. They are omitted when timing grade is
insufficient and regenerated from the structured final result when it is
sufficient.

## How emotion2vec runs beside Voxtral

The current emotion2vec+ classifier pools an entire supplied waveform before
classification and has no causal cache. It therefore remains a sliding-window
analyzer:

- fast provisional window: trailing 1.0 s;
- stable window: trailing 2.0 s;
- hop: 250 ms;
- end-of-turn pass over the exact VAD region;
- full nine-label score vector retained for smoothing and diagnostics;
- normalized English labels, with upstream Chinese/English display strings
  hidden inside the adapter;
- `unknown` when stability/coverage thresholds are not met.

Existing exploratory measurements on this host put warm emotion inference at
about 8.2 ms for 1 s and 11.5 ms for 2 s audio. Thus the intrinsic first affect
observation is approximately one second of captured context plus queue and
inference time; after that, new observations can arrive every 250 ms. The
several-second delay seen in one-shot use is dominated by checkpoint loading,
not warm inference.

Voxtral Q4_K_M was measured at about 4080 MiB process VRAM, approximately 3x
realtime offline throughput and 1.2–1.4 s model load on this host. Its configured
480 ms model delay is not the same as end-to-end capture-to-delta latency. Both
models must be loaded and warmed before the service emits `ready`.

Use one bounded GPU scheduler for the first implementation:

1. Voxtral feed/decode has highest priority because missed streaming deadlines
   accumulate user-visible lag.
2. At most one pending provisional emotion job is retained; replace an obsolete
   pending window with the newest one instead of building backlog.
3. Final emotion and optional alignment run after Voxtral finalization.
4. Start with one active microphone session.
5. Measure combined peak VRAM before allowing overlap or multiple sessions.

The simplest laboratory topology is one process with both adapters and one GPU
queue. If Python/CUDA allocator interaction or fault isolation becomes a real
problem, move adapters to resident worker processes without changing the
facade/event model.

## LLM-facing result

Prefer structured context and keep user text separate from observations:

```json
{
  "text": "Я думал, всё получится. Но ты меня обманул.",
  "speech_context": {
    "alignment_grade": "model_slot",
    "observed_vocal_expression": [
      {
        "text_unit_range": [0, 3],
        "label": "neutral",
        "stability": 0.72
      },
      {
        "text_unit_range": [4, 7],
        "label": "angry",
        "stability": 0.86
      }
    ]
  }
}
```

At `utterance` grade, send a turn-level distribution/summary instead of fake
word ranges. The description remains "observed vocal expression", never a claim
about the speaker's internal psychological state. Upstream scores remain
uncalibrated until a representative evaluation says otherwise.

## Resident service and transport

For the laboratory service, use a local WebSocket or Unix-domain-socket adapter:

- control messages are bounded, versioned JSON;
- audio is binary PCM16, not base64, on the local custom transport;
- chunks carry monotonic sequence and exact start sample;
- server answers `session.ready` only after models are loaded and warmed;
- disconnect/cancel frees bounded per-session audio/revision buffers but keeps
  weights resident;
- server startup, health, capability and model identity are observable;
- no audio is persisted unless an explicit external debug path is requested.

This transport is a lab adapter, not an engine public contract. Under Accepted
ADR-005, any later engine integration remains optional, process-isolated,
bounded, cancellable and backed by a deterministic text-only fallback. Model
output is untrusted proposal data and never mutates authoritative gameplay
state directly. SPEC-16/ADR-017 remain Deferred Proposed.

## Measurements and acceptance gates

Instrument every event with both `audio_end_sample` and monotonic completion
time. Report distributions, not one stopwatch result:

- cold start to `ready`, with model load and warm-up split by adapter;
- combined resident CPU RAM and peak/process GPU memory;
- PCM capture to first Voxtral delta and to final transcript, p50/p95;
- configured delay versus measured lexical emission lag once slot metadata is
  available;
- emotion window end to provisional event, p50/p95;
- speech endpoint to finalized fused turn, p50/p95;
- GPU queue wait and inference duration per adapter;
- dropped/coalesced emotion windows and maximum queue depth;
- transcript and emotion revision churn;
- Russian WER/CER and timestamp boundary error by alignment grade;
- emotion macro-F1/UAR, transition boundary error and tagged-span precision.

Initial falsifiable gates for the resident prototype:

1. A second client session causes no model reload.
2. The service exposes the first 1 s emotion observation without a growing
   queue and continues at a 250 ms cadence.
3. Voxtral continues faster than realtime under the joint serialized workload.
4. Disconnect/cancel releases session buffers while model residency remains
   stable.
5. The API reports `utterance` until model-slot timing passes a labeled boundary
   test; no word-tag example is emitted under `none`/`utterance` capability.
6. Absence or failure of the service cannot affect deterministic gameplay.

Do not set a product latency promise from the current smoke tests. In
particular, 480 ms is a trained transcription-delay knob, 1 s is the selected
emotion context window, and VAD endpointing adds its own finalization delay.

## Smallest implementation sequence

1. **Extract adapters:** reuse model/session functions from
   `e839a38:lab/scripts/voxtral_microphone.py`; wrap the existing emotion probe
   loader/normalizer. Keep microphone capture as a separate client.
2. **Resident facade:** start both models once, warm them, add one session and a
   bounded PCM fan-out/GPU scheduler, then expose capabilities and raw track
   revisions.
3. **Untimed fusion:** ship honest `utterance`-grade final context plus the full
   affect timeline. This already preserves emotional information for the LLM.
4. **Model-slot experiment:** expose Voxtral generated token IDs/control kinds
   and decoder output slots through the private runtime binding. Measure Russian
   word-boundary error before enabling clause/word span tags.
5. **Decision:** keep model-slot fusion, add a final aligner, or remain at
   utterance-level based on tagged-span precision and latency.

Implementation should occur on the branch containing `e839a38`, or after that
change is coherently brought into the target branch. The current worktree does
not contain `lab/scripts/voxtral_microphone.py`; recreating it here would fork
the implementation and its tests.

## Primary sources and exact local evidence

- [Mistral official model card](https://huggingface.co/mistralai/Voxtral-Mini-4B-Realtime-2602): architecture, 80 ms slots, supported delays, recommended 480 ms, Apache-2.0 and official runtime envelope.
- [Voxtral Realtime technical report](https://arxiv.org/abs/2602.11298): natively streaming aligned audio/text streams, causal encoder and multilingual evaluation.
- [vLLM Realtime speech-to-text protocol](https://docs.vllm.ai/en/stable/serving/online_serving/speech_to_text/#realtime-api): PCM format and text-only delta/done event surface.
- [vLLM Realtime protocol types](https://docs.vllm.ai/en/stable/api/vllm/entrypoints/speech_to_text/realtime/protocol/): confirms timestamp-free `TranscriptionDelta` and `TranscriptionDone` fields.
- [emotion2vec official repository](https://github.com/ddlBoJack/emotion2vec) and [FunASR implementation](https://github.com/modelscope/FunASR/blob/main/funasr/models/emotion2vec/model.py): representation/classifier behavior behind the sliding-window decision.
- NextEngine Git object `e839a38:lab/scripts/voxtral_microphone.py`: current microphone/session wrapper.
- NextEngine Git object `e839a38:docs/development/voxtral-mini-4b-realtime-2602-research-2026-08-17.md`: local RTX 3080 runtime, quality and resource evidence.
- External `transcribe.cpp` commit `9315160`: capability declaration, stream text policy and Voxtral feed/finalize cursor behavior inspected for this report.
