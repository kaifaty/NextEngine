# Live ASR + vocal-emotion timeline research

| Field | Value |
| --- | --- |
| Date | 2026-08-18 |
| Status | Bounded research; no production model or public contract selected |
| Scope | Local Russian microphone stream → provisional/final ASR words + emotion intervals → one aligned timeline and derived tagged text |
| Hardware examined | Linux x86_64, NVIDIA GeForce RTX 3080 10 GiB |

> **Selection update (2026-08-18):** the generic timeline and emotion-window
> findings in this report still apply, but the SimulStreaming/Whisper ASR
> selection and implementation order are superseded for the current experiment
> by
> `docs/development/voxtral-emotion2vec-facade-research-2026-08-18.md`.
> The selected pair is Voxtral Mini 4B Realtime 2602 plus emotion2vec+.

## Executive conclusion

The smallest viable design is one persistent optional `ai-host` with a single
16 kHz audio sample clock and four private stages:

```text
PCM chunks → streaming VAD ────────────────┐
         ├→ streaming ASR → timed words ───┼→ timeline joiner → Live revisions
         └→ sliding emotion2vec+ windows ──┘                    → final turn
```

ASR words and emotion observations must remain separate timestamped tracks.
Inline tagged text is a derived view, not the source of truth. Partial words
can be revised, emotion windows overlap, and the two tracks become immutable at
different times. Joining by character offsets would therefore corrupt tags.

For the first benchmark, use:

- Silero VAD on CPU;
- SimulStreaming's current Whisper path as the reference streaming-ASR policy,
  initially with multilingual Whisper `turbo` and Russian fixed as the source
  language;
- the already pinned `emotion2vec_plus_base` on trailing windows;
- one serialized GPU work queue so ASR and emotion inference do not create
  uncontrolled concurrent VRAM peaks;
- a local WebSocket developer adapter, with versioned JSON control/events and
  binary PCM input;
- structured timeline JSON as authority and escaped tagged text only as an
  export/prompt view.

This is a bounded evaluation path under Deferred Proposed SPEC-16/ADR-017.
Accepted ADR-005 still requires optional process isolation and a deterministic
text fallback.

## What the official implementations establish

### Streaming ASR

[SimulStreaming](https://github.com/ufal/SimulStreaming) is the current
successor to WhisperStreaming. Its official interface accepts incremental
audio, emits JSONL with word timestamps, confirmed and unconfirmed text,
`is_final`, and emission time, and includes a microphone/TCP server. Russian is
listed among its Whisper source languages. It also explicitly warms the model
because the first decode is slower. This is the closest reference behavior to
the required Live revision protocol.

The older
[Whisper-Streaming](https://github.com/ufal/whisper_streaming) demonstrates the
important LocalAgreement rule: commit a prefix only after consecutive decodes
agree. Its published long-form experiment reported 3.3 s latency, so subsecond
stable Russian words must not be assumed without a local benchmark. Its own
README now recommends SimulStreaming for new work.

[faster-whisper](https://github.com/SYSTRAN/faster-whisper) remains a useful
fallback backend. It supports multilingual Whisper, word timestamps and Silero
VAD, and CTranslate2 quantization can reduce memory. It is not by itself a
complete streaming policy: a controller still has to manage overlapping audio,
stable-prefix agreement, buffer trimming and finalization.

OpenAI documents multilingual Whisper `turbo` as an 809M-parameter model with
approximately 6 GiB VRAM use. That plus the current 1.12 GB emotion2vec+
checkpoint appears plausible on the 10 GiB RTX 3080, but this is only an
inference from declared sizes. Peak residency and concurrent inference must be
measured before selecting the pair. See the
[official Whisper repository](https://github.com/openai/whisper).

### VAD and endpointing

[Silero VAD](https://github.com/snakers4/silero-vad) is designed for streaming,
supports 8/16 kHz audio, and has a small ONNX/PyTorch runtime. It should own
speech onset/end observations, but not dialogue truth. Initial endpoint values
such as 300–500 ms of silence are hypotheses to benchmark, not fixed contract
defaults.

### Emotion output

The official
[emotion2vec repository](https://github.com/ddlBoJack/emotion2vec) exposes
utterance- and frame-granularity representations. The current emotion2vec+
classifier, however, is utterance-level: the
[FunASR implementation](https://github.com/modelscope/FunASR/blob/main/funasr/models/emotion2vec/model.py)
loads the supplied waveform, extracts all frames, averages them over time and
then applies the classification projection. It has no causal cache.

Therefore, a timeline must be estimated by repeated overlapping waveform
windows. Applying the trained utterance projection independently to every
internal frame would be an unsupported semantic change and should not be used
without a labeled evaluation.

The emotion2vec paper combines frame- and utterance-level representation
learning and reports multilingual transfer, but this does not make the
released nine-class head a calibrated continuous-emotion detector. See
[emotion2vec: Self-Supervised Pre-Training for Speech Emotion Representation](https://arxiv.org/abs/2312.15185).

The Hugging Face checkpoint currently declares the non-specific value
`model-license`, not a classified SPDX license. The installed artifact remains
valid for a local research PoC but cannot be distributed or promoted as a
shipping pack until its exact weight/data terms are classified. See the
[checkpoint model card](https://huggingface.co/emotion2vec/emotion2vec_plus_base).

### Optional final alignment

[WhisperX](https://github.com/m-bain/whisperX) can refine finalized word
timestamps using phoneme forced alignment and lists a Russian wav2vec2 aligner.
It is an optional quality experiment, not the Live baseline: it adds another
model, latency, provenance/license work and a second revision after end of
speech. Use it only if measured Whisper word-boundary error prevents acceptable
emotion attachment.

### Representation precedent

[W3C EmotionML](https://www.w3.org/TR/emotionml/) establishes useful concepts:
category or dimensional emotion, confidence, expressed modality, and half-open
media time intervals. The engine should reuse those concepts, not the XML wire
format. A compact versioned JSON protocol is easier to bound and validate.

## Recommended process topology

```text
client/game/tool
  │ WebSocket: start + PCM16 chunks + stop/cancel
  ▼
optional persistent ai-host
  ├─ capture ingress: sequence validation and sample-clock assignment
  ├─ CPU VAD: speech regions and endpoint candidate
  ├─ GPU scheduler
  │    ├─ streaming Whisper decode
  │    └─ emotion2vec sliding-window inference
  ├─ timeline joiner: revisions, watermarks, interval/word alignment
  └─ WebSocket: ready, timeline_update, utterance_final, error
```

The server loads and warms both models once before emitting `ready`. A client
connection creates only bounded per-stream buffers and revision state; it does
not reload weights. Initial concurrency should be one active microphone stream.

The developer WebSocket is an adapter, not a proposed engine public contract.
Future engine integration still needs an engine-owned IPC handshake, bounded
messages, deadlines, cancellation, model/content hashes and provenance under
ADR-005/SPEC-16.

## One clock and one revision model

All temporal data should use integer samples relative to stream start:

- `sample_rate_hz = 16000`;
- every interval is half-open `[start_sample, end_sample)`;
- client chunks have monotonic `sequence`, exact `start_sample` and bounded
  PCM byte length;
- milliseconds are derived only for display/export;
- wall clock and model completion order never define audio position.

Each update carries:

- `stream_id` and monotonically increasing `revision`;
- `replace_from_sample`: the earliest provisional timeline position replaced
  by this update;
- `committed_before_sample`: no later update may alter content before this
  watermark;
- current word and emotion interval revisions from `replace_from_sample`;
- optional derived `tagged_text` for debugging.

The watermark is the minimum of the stable ASR prefix, finalized emotion
intervals and VAD-finalized speech region. A final dialogue utterance is emitted
once. Late or duplicate chunks cannot create a second final result.

## Proposed research wire shape

```json
{
  "schema_version": 1,
  "type": "timeline_update",
  "stream_id": "opaque-id",
  "revision": 17,
  "sample_rate_hz": 16000,
  "replace_from_sample": 24000,
  "committed_before_sample": 16000,
  "words": [
    {
      "id": 4,
      "start_sample": 26240,
      "end_sample": 31840,
      "text": "обманул",
      "asr_confidence": 0.88,
      "state": "provisional"
    }
  ],
  "emotion_segments": [
    {
      "id": 2,
      "start_sample": 24000,
      "end_sample": 40000,
      "label": "angry",
      "model_score": 0.91,
      "stability": 0.78,
      "state": "provisional",
      "modality": "observed_vocal_expression"
    }
  ],
  "canonical_text": "Но ты меня обманул.",
  "tagged_text": "<emotion label=\"angry\">Но ты меня обманул.</emotion>"
}
```

`model_score` is the upstream uncalibrated score. `stability` is a separate
engine/tool-derived measure based on agreement across overlapping observations;
neither is a claim about the speaker's internal state.

## Emotion timeline construction

### Raw observations

Start with two modes using the same resident model:

1. **Fast provisional:** trailing 1.0 s window, 250 ms hop, only while VAD
   reports sufficient speech occupancy.
2. **Stable provisional/final:** trailing 2.0 s window, 250 ms hop; at VAD end,
   recompute the exact utterance boundaries and finalize the interval track.

These values are benchmark seeds. Existing exploratory measurements on this
host show steady-state model inference around 8.2 ms for 1 s and 11.5 ms for
2 s windows, but no emotion-transition quality has yet been measured.

Each raw observation stores the full normalized nine-label vector, window
interval, inference revision, model hash and VAD speech occupancy. Do not store
only the winner because later smoothing and threshold experiments need the
distribution.

### Smoothing and segment formation

For the first implementation, use a bounded causal filter rather than training
another model:

1. normalize upstream labels to the stable English enum;
2. smooth score vectors across recent overlapping windows;
3. require a candidate label to exceed a configurable score and current-label
   margin for multiple consecutive hops;
4. enforce a minimum dwell time before a switch;
5. use `unknown`/no annotation when evidence is insufficient rather than
   forcing a class;
6. merge adjacent equal labels and preserve the original raw observations for
   diagnostics.

Threshold, margin and dwell values must be fitted on a small labeled Russian
transition corpus. Hard-coding values from one microphone sample would turn
model overconfidence into flickering tags.

A later final-only experiment may compare a nine-state Viterbi path with an
explicit transition penalty. Its time cost is trivial (`O(T * 9²)`), but it is
not justified until the simple hysteresis baseline is measured.

## Joining emotion to words

Use temporal overlap, never text offsets:

1. For each finalized word interval, compute overlap with finalized emotion
   intervals.
2. Select the label with the greatest overlap-weighted stable score.
3. If coverage or stability is below threshold, attach no emotion annotation.
4. Move an emotion transition to the nearest word boundary for tagged-text
   rendering while retaining the exact acoustic boundary in structured data.
5. Coalesce consecutive words with the same selected label into one text span.

Conceptually:

```text
emotion_weight(word, label) =
    Σ overlap(word, emotion_interval) × stable_score(label)
    ─────────────────────────────────────────────────────────
                 Σ overlap(word, emotion_interval)
```

Short words do not contain enough acoustic context for independent emotion
classification. A word receives context from the overlapping multi-second
emotion track; the output must not imply phoneme-level psychological certainty.

## Canonical and tagged outputs

Keep all three views:

1. `canonical_text`: finalized ASR text without injected tags;
2. structured `words` + `emotion_segments`: timeline source of truth;
3. derived tagged text or grouped spans for presentation/LLM context.

Preferred LLM context is structured and keeps user text separate from metadata:

```json
{
  "text": "Я думал, всё получится. Но ты меня обманул.",
  "observed_vocal_expression": [
    {"word_range": [0, 3], "label": "neutral", "stability": 0.72},
    {"word_range": [4, 7], "label": "angry", "stability": 0.86}
  ]
}
```

A readable export may be:

```text
<emotion label="neutral">Я думал, всё получится.</emotion>
<emotion label="angry">Но ты меня обманул.</emotion>
```

The inline form must escape text and remain data, never prompt/control syntax or
the authoritative `CanonicalUtterance` content.

## ASR option comparison

| Option | Strength | Limitation | Decision |
| --- | --- | --- | --- |
| SimulStreaming + Whisper `turbo` | Current streaming policy, Russian, word/revision/final JSONL, warm-up path | Torch VRAM peak and Russian latency/quality unmeasured on 3080 | First reference benchmark |
| faster-whisper + LocalAgreement controller | CTranslate2 quantization, word timestamps, integrated Silero VAD | Requires more streaming/revision glue; old WhisperStreaming controller is superseded | Memory/throughput fallback |
| WhisperX final alignment | Potentially tighter finalized Russian word boundaries | Extra model, latency and artifact/license closure; not streaming | Add only if timestamp error fails criterion |
| SenseVoiceSmall joint ASR/SER | Fast joint ASR/emotion/event output | Released checkpoint documents Chinese/Cantonese/English/Japanese/Korean, not Russian; emotion still does not solve our interval semantics | Reject for Russian baseline |

The last point follows the current
[official SenseVoice repository](https://github.com/FunAudioLLM/SenseVoice),
which distinguishes broad research claims from the five-language released
checkpoint.

## Performance and quality evidence required

### Latency and resource metrics

- server cold start and ready/warm time;
- model CPU RAM and GPU residency separately and together;
- capture-to-first provisional emotion p50/p95;
- capture-to-confirmed ASR word p50/p95;
- end-of-speech-to-`utterance_final` p50/p95;
- emotion and ASR GPU queue wait versus inference time;
- real-time factor, dropped/late chunks and maximum bounded queue depth;
- revision churn: words and emotion intervals replaced per minute.

### Quality metrics

- Russian ASR WER on clean and microphone/noisy speech;
- word-boundary median/p95 absolute error on a small manually aligned set;
- emotion macro-F1/UAR, confusion matrix and calibration error;
- transition boundary F1 with a declared tolerance such as ±500 ms;
- label fragmentation and final-versus-provisional revision rate;
- tagged-span precision: whether a word is attached to the intended acoustic
  emotion region;
- explicit speaker/microphone/noise slices to expose bias and brittleness.

The first corpus should contain natural neutral speech, acted primary emotions,
within-utterance transitions and deliberately ambiguous/mixed delivery in
Russian. Recordings and datasets remain outside the repository with hashes and
provenance.

## Smallest implementation sequence

1. **Persistent emotion server:** `serve`, model warm-up, WebSocket PCM input,
   1/2 s observations, timeline revisions and benchmark counters.
2. **ASR reference adapter:** add SimulStreaming/Whisper behind the same GPU
   scheduler; preserve confirmed/unconfirmed words and emission timestamps.
3. **Fusion:** common sample clock, watermark, overlap join, grouped spans and
   final tagged export.
4. **Evidence:** record the bounded Russian transition corpus, measure latency,
   word boundaries, emotion transitions and joint VRAM.
5. **Decision point:** keep the pair, choose faster-whisper fallback, add final
   forced alignment, or reject the model based on measured criteria.

No engine public contract, roadmap status or shipping model should change at
steps 1–3. The deterministic authored/text fallback remains mandatory.

## Rejected shortcuts

- Reload models per chunk or per connection: measured checkpoint startup
  dominates warm inference.
- Attach emotion tags to partial string character ranges: ASR revisions make
  ranges stale.
- Treat `granularity="frame"` as frame-level nine-class output: the released
  classifier head is trained/evaluated with time pooling.
- Classify every word independently: most words are too short and emotion is a
  contextual acoustic property.
- Use upstream scores as calibrated probabilities or internal mental state:
  they are model scores for observed vocal expression.
- Adopt SenseVoiceSmall solely to obtain joint tags: the released checkpoint is
  not a Russian ASR baseline and joint output does not eliminate timeline and
  revision requirements.
