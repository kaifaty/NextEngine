# Audio emotion + ASR timeline — current task state

| Field | Value |
| --- | --- |
| Status | `ACTIVE` |
| Updated | `2026-08-18` |
| Task key | `audio-emotion-asr-timeline` |
| Scope | Research and prototype an isolated live pipeline that aligns ASR text timestamps with bounded emotion intervals. |
| Definition of done | A reviewable design selects the smallest viable VAD/ASR/emotion/alignment pipeline, versioned provisional/final timeline shapes, measured latency targets and the next falsifiable prototype. |
| Authority | Working context only; Accepted ADR-005 and the repository architecture outrank this file. SPEC-16/ADR-017 remain Deferred Proposed. |

## Resume in 60 seconds

- **Current conclusion:** Prototype one persistent optional `ai-host` with Silero VAD, a SimulStreaming/Whisper `turbo` reference ASR, the existing emotion2vec+ sliding windows, one serialized GPU queue and a WebSocket developer adapter. Keep structured timed tracks authoritative and derive tagged text.
- **Why:** Official implementations expose the needed VAD and confirmed/unconfirmed word timestamps, while the current emotion classifier is utterance-pooled and must be sampled over overlapping windows. A common integer sample clock plus revision watermark handles both forms without corrupting partial text.
- **Next action:** Implement the persistent emotion-only server and measure warm Live event latency/resource use before adding the ASR model.
- **Current blocker:** None.
- **Do not retry:** Per-window process launch and checkpoint reload; measured startup dominates inference and cannot meet Live latency.
- **Reconsider when:** A target runtime cannot keep the model resident or a native streaming emotion model demonstrably improves end-to-end quality/resource use.

## Current evidence

| Evidence | Result | Consequence |
| --- | --- | --- |
| `tools/emotion-probe/` at commit `365a2d8` | `PASS` microphone/file PoC | Existing wrapper is reusable for model loading and score normalization but is not a server. |
| RTX 3080 exploratory run, 2026-08-18 | model load 5.2–5.6 s; warm inference p50 8.2/11.5/12.7/14.7 ms for 1/2/3/4 s windows | Persistent model residency is the primary latency fix; values are exploratory, not product evidence. |
| `docs/architecture/16-text-canonical-multimodal-dialogue-and-model-packs.md` | `Deferred Proposed` | Partial ASR/emotion is presentation-only; only finalized text may become `CanonicalUtterance`. |
| `docs/architecture/adr/005-offline-first-ai-process-boundary.md` | `Accepted` | Speech and audio-understanding models remain optional and process-isolated with deterministic fallback. |
| `docs/development/audio-emotion-asr-timeline-research-2026-08-18.md` | bounded research complete | Selects the first reference pipeline, revision/timeline shape, fusion algorithm, metrics and staged prototype; it does not promote SPEC-16. |

## Decisions that still constrain the work

### D-001 — One audio-clock timeline before tagged text

- **Observation:** ASR words and emotion scores describe different temporal granularities and may revise at different times.
- **Evidence:** Current emotion2vec+ yields utterance scores for a supplied waveform window; the desired consumer needs time-aligned text.
- **Decision:** Preserve structured word and emotion intervals on one monotonic sample clock; tagged text is a derived serialization, never the source of truth.
- **Rejected alternatives:** Inject emotion labels directly into partial ASR strings; revisions would make offsets ambiguous and tags unstable.
- **Consequences:** Every provisional item needs identity/revision/finality, and alignment occurs only from timestamps rather than string offsets.
- **Uncertainty:** Best emotion window/hop and tag-transition thresholds for Russian gameplay speech.
- **Reconsider when:** A selected joint speech model emits calibrated word-level emotion with independently verified timestamps.

### D-002 — Benchmark SimulStreaming/Whisper first, retain faster-whisper fallback

- **Observation:** SimulStreaming is the current successor to WhisperStreaming and already exposes Russian incremental words, confirmed/unconfirmed text, finality and emission time. The current released SenseVoiceSmall checkpoint does not document Russian among its five ASR languages.
- **Evidence:** Official SimulStreaming, faster-whisper, SenseVoice and OpenAI Whisper repositories linked from the dated research report.
- **Decision:** Use SimulStreaming with multilingual Whisper `turbo` as the first ASR reference; serialize its GPU work with emotion2vec+. Evaluate faster-whisper plus a stable-prefix controller only if peak VRAM or latency fails.
- **Rejected alternatives:** SenseVoiceSmall as the Russian baseline; its joint emotion tags do not remove temporal alignment needs and its released checkpoint language scope does not include Russian.
- **Consequences:** Joint GPU residency and Russian stable-word latency are mandatory early measurements, not assumed properties.
- **Uncertainty:** Whether Torch Whisper `turbo` plus emotion2vec+ fits the 10 GiB RTX 3080 with safe peak headroom.
- **Reconsider when:** A measured Russian-capable streaming checkpoint provides better stable-word latency/quality and classified distribution terms within the resource envelope.

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1: separate streaming ASR plus sliding-window emotion2vec+ is sufficient | Warm emotion inference is inexpensive and components remain replaceable. | emotion2vec+ is not causal and abrupt transitions may be smeared. | Labeled transition corpus with word timestamps and end-to-end lag/error metrics. |
| H2: VAD-defined utterance finalization plus provisional windows gives stable UX | Separates fast Live updates from higher-quality final analysis. | Endpoint latency can dominate short utterances. | Measure VAD endpoint p50/p95 and final-tag revision rate. |
| H3: Whisper `turbo` and emotion2vec+ can reside together on the RTX 3080 | Declared Whisper VRAM is about 6 GiB and emotion checkpoint bytes are about 1.12 GB. | Runtime peaks, allocator reserve and simultaneous decode are unmeasured. | Load/warm both, record process/GPU peak, then run a serialized and a controlled-overlap workload. |

## Required context

Read these sources in precedence order before acting:

1. `docs/architecture/agent-routing.md`
2. `docs/architecture/adr/005-offline-first-ai-process-boundary.md`
3. `docs/architecture/16-text-canonical-multimodal-dialogue-and-model-packs.md`
4. `docs/architecture/adr/017-text-canonical-multimodal-dialogue-and-replaceable-model-packs.md`
5. `docs/architecture/09-tooling-sdk-and-observability.md`
6. `tools/emotion-probe/README.md` and `tools/emotion-probe/src/nextengine_emotion_probe/`
7. `docs/development/audio-emotion-asr-timeline-research-2026-08-18.md` once created

## Next action

1. Add an emotion-only persistent server with model warm-up, bounded WebSocket PCM ingress and revisioned interval events.
2. Success criterion: repeated client sessions reload no weights, warm 1/2 s inference and capture-to-event latency are reported separately, and disconnect/cancel releases bounded session buffers.
3. Non-regression: existing file/record CLI remains usable and `ai-host` absence cannot affect gameplay; remove the developer server adapter if it cannot meet these boundaries.

## Do not retry

- One process per audio window — checkpoint load is orders of magnitude slower than warm inference; reconsider only if the model/runtime changes enough to make cold start fit the Live budget.
- Raw tagged text as timeline authority — partial transcript revisions invalidate character offsets; reconsider only if all upstream output is immutable and final.

## Handoff

- **Workspace state:** Task-state and dated research report are the only current uncommitted research changes; existing emotion probe is committed at `365a2d8`.
- **Checks:** Direct source/path/link validation and documentation diff check remain before handoff.
- **Remaining risk:** Joint VRAM, Russian ASR quality/stable-word latency, timestamp accuracy, VAD endpoint latency, emotion transition quality and license classification of emotion2vec+ weights remain open.
- **Promotion needed:** None now; no Accepted architecture or roadmap change is authorized by this bounded research.
