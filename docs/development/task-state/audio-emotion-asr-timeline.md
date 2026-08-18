# Audio emotion + ASR timeline — current task state

| Field | Value |
| --- | --- |
| Status | `ACTIVE` |
| Updated | `2026-08-18` |
| Task key | `audio-emotion-asr-timeline` |
| Scope | Research and prototype an isolated live pipeline that aligns Voxtral text with bounded emotion2vec intervals behind replaceable model adapters. |
| Definition of done | A resident prototype exposes versioned transcript/affect/fusion revisions, declares timing precision, avoids model reload between clients and reports joint latency/resource evidence. |
| Authority | Working context only; Accepted ADR-005 and repository architecture outrank this file. SPEC-16/ADR-017 remain Deferred Proposed. |

## Resume in 60 seconds

- **Current conclusion:** Use a resident optional `SpeechTimelineService` with `VoxtralTranscriberAdapter`, `Emotion2VecAffectAdapter`, optional VAD and a deterministic timeline fuser. Keep three independently revisioned tracks and derive LLM context from them.
- **Why:** The existing Voxtral wrapper already performs real streaming inside one invocation, while warm emotion2vec inference is fast. Reloading either model per connection/window is the avoidable delay.
- **Critical limit:** Current Voxtral public APIs return streaming text but no lexical timestamps. `transcribe.cpp` reports timestamp kind `NONE`; its Voxtral `audio_committed_ms` remains zero during feed and is not a text boundary.
- **Implementation plan:** `docs/plans/2026-08-18-speech-timeline-service-implementation.md` defines the recommended standalone localhost WebSocket vertical, commit boundaries, faults, measurements and optional VAD/model-slot increments.
- **Next action:** Confirm the plan's default scope (`standalone / explicit finish first / utterance MVP / WebSocket`), create `codex/speech-timeline-service`, converge the exact Voxtral commits, and execute Commit 1 without recreating the wrapper.
- **Current blocker:** The Voxtral wrapper exists on `codex/architecture-foundation-promotion`, not in this worktree. Research is unblocked; implementation should use/merge that source rather than recreate it.
- **Do not retry:** Per-window process launch/checkpoint reload; ASR attachment through `audio_committed_ms`; fabricated word timestamps from text arrival time.
- **Reconsider when:** A model/runtime exposes better timed lexical units, or measured model-slot boundary error is too high and justifies a final aligner.

## Current evidence

| Evidence | Result | Consequence |
| --- | --- | --- |
| `tools/emotion-probe/` at commit `365a2d8` | `PASS` microphone/file PoC | Reuse its loader and score normalization; keep weights resident. |
| RTX 3080 emotion run, 2026-08-18 | model load 5.2–5.6 s; warm inference p50 8.2/11.5 ms for 1/2 s windows | First tag is dominated by the 1 s context window; later cadence can be 250 ms. |
| `e839a38:lab/scripts/voxtral_microphone.py` | 16 kHz streaming wrapper, one model/session per invocation, default 250 ms feed and 480 ms model delay | Extract adapter logic; CLI is not a persistent service. |
| `e839a38:docs/development/voxtral-mini-4b-realtime-2602-research-2026-08-17.md` | Q4_K_M ~4080 MiB process VRAM, ~3x realtime, 1.2–1.4 s model load | Plausible resident ASR on the 10 GiB host; joint peak remains mandatory evidence. |
| `transcribe.cpp` commit `9315160` | Voxtral capability `TIMESTAMPS_NONE`; whole-text segment; feed cursor is not lexical time | Baseline fusion grade is `utterance`, not word. |
| Official Voxtral/vLLM protocol | 80 ms aligned model slots but public Realtime events carry only text delta/final | Test a private token-slot adapter; keep timing capability explicit. |
| `docs/development/voxtral-emotion2vec-facade-research-2026-08-18.md` | bounded pair/facade research complete | Supersedes the prior SimulStreaming ASR selection and defines staged implementation/evidence. |
| `docs/plans/2026-08-18-speech-timeline-service-implementation.md` | proposed implementation sequence | Defines the smallest resident vertical, commit/test boundaries and four explicit scope choices; it does not promote SPEC-16. |
| ADR-005; SPEC-16/ADR-017 | Accepted isolation/fallback boundary; multimodal track remains Deferred Proposed | No direct gameplay mutation or product-shipped claim. |

## Decisions that constrain the work

### D-001 — One acoustic clock; structured tracks before tagged text

- **Observation:** ASR text and emotion describe different granularities and revise independently.
- **Decision:** Preserve audio intervals on one integer sample clock; canonical text, timed affect and optional timed lexical units remain separate. Inline tagged text is a derived view only.
- **Consequence:** Untimed text receives turn/region context, not invented word tags.
- **Reconsider when:** A verified joint model emits calibrated word-level affect with acoustic timestamps.

### D-002 — SimulStreaming/Whisper selection superseded

- **Observation:** The user selected Voxtral Mini 4B Realtime 2602 and identified the existing wrapper after the generic ASR research.
- **Evidence:** `e839a38:lab/scripts/voxtral_microphone.py` plus local Voxtral research and runtime measurements.
- **Decision:** Do not implement the prior SimulStreaming-first path for this experiment. Retain that report only as generic timeline/alternative-ASR evidence.
- **Consequence:** Voxtral-specific timestamp limitations now determine the first fusion grade.
- **Reconsider when:** Voxtral fails joint residency, Russian quality or latency gates.

### D-003 — Model-agnostic facade with capability negotiation

- **Observation:** Models differ in streaming state, revision semantics, timestamps, vocabularies and resource envelopes.
- **Decision:** Consumers depend on `SpeechTimelineService`; adapters fill `StreamingTranscriber`, `VocalAffectAnalyzer` and optional endpoint/alignment roles. Startup returns exact capabilities and identities.
- **Rejected:** A `VoxtralEmotion2VecService` public API or hard-coded model fields.
- **Consequence:** Model swap is configuration plus adapter; incompatible timing degrades explicitly or fails preflight.

### D-004 — Voxtral starts at `utterance`; `model_slot` is an experiment

- **Observation:** The model is trained over synchronized 80 ms audio/text streams, but current local and official serving APIs omit token/word timestamps.
- **Evidence:** Official report/model card; vLLM Realtime protocol; `transcribe.cpp` capability and source at `9315160`.
- **Decision:** Current adapter declares `utterance`. A private extension may expose token/control IDs with decoder output slots and graduate to `model_slot` only after Russian boundary validation.
- **Rejected:** Mapping `committed_text` changes or `audio_committed_ms` directly to word time.
- **Consequence:** Accurate word-level final tags may still require an optional `TranscriptAligner`.

### D-005 — Resident bounded scheduler

- **Observation:** Cold model loads dominate warm emotion inference and add 1.2–1.4 s for Voxtral alone.
- **Decision:** Load and warm both adapters once; use one bounded GPU queue, prioritize Voxtral, and coalesce obsolete emotion jobs. Begin with one active session.
- **Consequence:** Second-session no-reload and combined peak VRAM are prototype gates.
- **Reconsider when:** One-process CUDA/runtime interaction fails a reproducible check; then isolate resident workers behind the same facade.

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1: both models fit and remain faster than realtime under serialized joint load | Voxtral uses ~4080 MiB and emotion windows are short. | Combined allocator peaks and queue interaction are unmeasured. | Load/warm both, record peak, then replay paced PCM with per-adapter queue metrics. |
| H2: Voxtral token slots are accurate enough for clause-level affect attachment | Training uses explicitly aligned 80 ms audio/text streams. | Public APIs omit timing; slot offset/grouping error is unknown. | Expose token/control slots and compare to manually aligned Russian words. |
| H3: utterance-grade context already improves downstream LLM responses | It preserves vocal evidence honestly without timestamp invention. | Mixed emotion inside a turn may be smeared. | Blind downstream response evaluation: text-only versus turn affect versus timed spans. |
| H4: 1 s/250 ms emotion windows give useful Live transitions | Warm inference is ~8 ms and windows overlap densely. | Utterance-pooled classifier may smear or flicker. | Labeled Russian within-turn transition corpus with boundary/F1 and churn metrics. |

## Required context

Read in precedence order:

1. `docs/architecture/agent-routing.md`
2. `docs/architecture/adr/005-offline-first-ai-process-boundary.md`
3. `docs/architecture/16-text-canonical-multimodal-dialogue-and-model-packs.md`
4. `docs/architecture/adr/017-text-canonical-multimodal-dialogue-and-replaceable-model-packs.md`
5. `docs/architecture/09-tooling-sdk-and-observability.md`
6. `docs/development/voxtral-emotion2vec-facade-research-2026-08-18.md`
7. `e839a38:lab/scripts/voxtral_microphone.py` and its tests
8. `tools/emotion-probe/README.md` and implementation

## Smallest next action

1. Work from the branch/source containing commit `e839a38`.
2. Separate microphone capture from Voxtral model/session code and wrap the latter as `StreamingTranscriber` without changing its inference behavior.
3. Wrap resident emotion loading and 1 s/2 s window inference as `VocalAffectAnalyzer`.
4. Add one-session facade, capability handshake, binary PCM input and transcript/affect revisions at `utterance` grade.
5. Measure cold/warm readiness, second session reload count, combined VRAM, capture-to-event latency, queue wait and cancellation cleanup.
6. Only then extend `transcribe.cpp` to expose token slots for a bounded alignment experiment.

## Do not retry

- One process per chunk/window or one model load per client.
- Treating 480 ms configured delay as measured end-to-end latency.
- Treating text arrival/commit time as acoustic word time.
- Emitting inline word emotion tags while capability is `none`/`utterance`.
- Recreating the Voxtral wrapper in this branch instead of using the tested `e839a38` source.

## Handoff

- **Workspace state:** Pair/facade research and task-state update are documentation-only; the current worktree intentionally lacks the Voxtral wrapper present on `codex/architecture-foundation-promotion`.
- **Checks:** Run documentation diff/path/link validation before handoff.
- **Remaining risk:** Joint VRAM and sustained latency, Russian ASR and slot alignment accuracy, VAD endpoint delay, emotion transition quality, and emotion2vec+ redistribution terms.
- **Promotion needed:** None; no Accepted architecture or roadmap change is authorized by this research.
