# Speech recognition reliability — current task state

| Field | Value |
| --- | --- |
| Status | `ACTIVE` |
| Updated | `2026-08-22` |
| Task key | `speech-recognition-reliability` |
| Scope | Public-data, utterance-level Russian ASR reliability in `tools/speech-timeline` |
| Definition of done | R0–R3 satisfy the gates and boundaries in `DEV-SPEECH-RELIABILITY-001`, or the first failed gate leaves the candidate explicitly report-only |
| Authority | Working context only; Accepted SPEC/ADR, `AGENTS.md` and the development specification outrank this file |

## Resume in 60 seconds

- **Current conclusion:** R0 and R1 are implemented and verified as fail-closed tooling; no real corpus, replay or accuracy result exists yet.
- **Why:** R1 adds O(1) transcript-revision feature accumulation in the live service path plus a typed fail-closed payload, and a single-attempt prepared-corpus replay runner over the production WebSocket path; both are proven on synthetic fixtures only.
- **Next action:** Operator acquires public corpora externally, closes the recipe (`prepare`), then replays one exact route with `reliability-corpus replay`; R2 starts only after split-before-augmentation and train/calibration/heldout partitions exist.
- **Current blocker:** Public corpora are not acquired or hash-closed; no resident Voxtral replay identity has been bound to a closed recipe.
- **Do not retry:** Treating microphone diagnostics as training data; inventing hashes for unavailable corpus/model artifacts; retrying failed clips to green inside the replay runner.
- **Reconsider when:** An exact external source snapshot and a closed resident replay identity are available for closure.

## Current evidence

| Evidence | Result | Consequence |
| --- | --- | --- |
| `docs/development/speech-recognition-reliability-public-data-spec-2026-08-20.md` | `PASS` as approved development scope | Names the claim boundary, R0–R3 increments and evaluation gates |
| `tools/speech-timeline/src/nextengine_speech_timeline/reliability_features.py` | `PASS` in 153-test suite (40 new) | Golden churn vectors, empty/vanish/oscillation cases, clipping levels, non-finite/identity fail-closed paths, bounded memory (≈1 KiB growth over 3 700 revisions) are executable |
| `tools/speech-timeline/src/nextengine_speech_timeline/service.py` | `PASS` websocket + unit coverage | Terminal metrics carry `recognition_reliability_features`; a 30-s-like turn with 600 revisions and 479 jobs encodes to ~12.6 KB of the 64 KB event limit; existing microphone/UI flows unchanged |
| `tools/speech-timeline/src/nextengine_speech_timeline/corpus_replay.py` | `PASS` fixture replay suite | Planned/unclosed recipes, tampered audio and identity mismatch abort before the first clip; overload/timeout/no-speech/speech-but-empty/model-failure stay typed outcomes with one attempt each; outputs atomic `0600`, transcripts confined to external files |
| `cargo run -p xtask -- host-check` (R0 handoff) | `PASS` on pinned Rust 1.97.1 | Localized Python-only R1 change re-ran focused package checks instead; rerun before any cross-cutting claim |
| External public corpus snapshot | `NOT_RUN` | No dataset-quality, coverage or calibrated reliability claim is currently allowed |

## Decisions that still constrain the work

### D-001 — Reliability means pipeline transcript exactness risk

- **Observation:** ASR quality failures can arise from signal, VAD, preprocessing, scheduling or decoding; loudness and linguistic plausibility alone are not ground truth.
- **Evidence:** `DEV-SPEECH-RELIABILITY-001` sections 1–4 and 11.
- **Decision:** Name the result `recognition_reliability`; predict normalized final-transcript exactness and retain typed `no_speech`, `unheard` and technical outcomes.
- **Rejected alternatives:** A diction, accent or medical-quality score; an LLM plausibility judgement; RMS as the sole oracle.
- **Consequences:** Runtime/UI wording must describe pipeline confidence, not a property of the speaker.
- **Uncertainty:** Generic public Russian audio may not calibrate player microphones well enough for useful clear coverage.
- **Reconsider when:** Held-out results fail the declared selective-risk/coverage gates or target-domain evidence becomes admissible.

### D-002 — Corpus data and derived artifacts remain external and explicit

- **Observation:** Public speech sources have distinct authentication, attribution, redistribution and privacy constraints.
- **Evidence:** SPEC-11, SPEC-34, ADR-030, ADR-046 and `DEV-SPEECH-RELIABILITY-001` sections 5–7.
- **Decision:** Require an explicit external store, closed source index hashes and exact rights status; never implicitly download or ingest microphone turns.
- **Rejected alternatives:** Committing datasets, silent telemetry reuse, re-hosting Common Voice clips or accepting unknown terms as warnings.
- **Consequences:** The committed public-safe recipe remains planned until an operator acquires and verifies exact snapshots.
- **Uncertainty:** Final redistribution status of a trained artifact still needs source-by-source review.
- **Reconsider when:** Repository governance admits a source and derived artifact with exact provenance and notices.

### D-003 — R1 features fold history into bounded numbers, never into events

- **Observation:** The service already had a TURN_TOO_LARGE terminal failure from unbounded job serialization; appending revision history would reintroduce it.
- **Evidence:** Long-turn soak evidence in `audio-emotion-asr-timeline.md`; measured 30-s-like event at 12.6 KB with the new payload.
- **Decision:** Accumulate revisions incrementally with one retained previous-text buffer; emit only the numeric summary inside `utterance.final` metrics; any capture fault degrades to a typed incomplete payload without failing the turn.
- **Rejected alternatives:** Serializing `TranscriptRevisionTraceV0` rows into WebSocket events; client-side-only feature computation (would bypass the production path); failing sessions on feature bugs.
- **Consequences:** The client-side replay collector keeps the bounded trace externally only; wall-clock fields make the vector sequence-deterministic, not PCM-deterministic alone.
- **Reconsider when:** A consumer needs full revision traces at runtime rather than during offline corpus replay.

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1: transcript revision stability plus signal/VAD features can calibrate exact-match risk cheaply | Required signals now flow from the live path as a bounded vector with no second ASR | No frozen public replay/evaluation exists yet | Closed-corpus replay followed by source-disjoint R2 logistic baseline |
| H2: public-only calibration has useful target coverage | Common Voice/FLEURS cover Russian speakers and read/conversational domains | Whisper and real player microphone domain shift remain weakly represented | Held-out coverage/OOD report against the fixed gates |

## Required context

Read these sources in precedence order before acting:

1. `docs/architecture/agent-routing.md`; SPEC-11, SPEC-15, SPEC-16, SPEC-34; ADR-005, ADR-017, ADR-030, ADR-046 and ADR-053.
2. `docs/development/speech-recognition-reliability-public-data-spec-2026-08-20.md`.
3. `docs/development/task-state/audio-emotion-asr-timeline.md` for the resident service lineage and active Voxtral constraints.
4. `tools/speech-timeline/README.md` and `tools/speech-timeline/src/nextengine_speech_timeline/` for the implemented experiment boundary.

## Next action

1. Operator step: acquire Common Voice/FLEURS/MUSAN/RIRS snapshots outside Git, fill the recipe with verified hashes, run `reliability-corpus prepare`.
2. Bind the exact resident identity into a copy of the recipe (`identity_status: closed`) and run `reliability-corpus replay --asr-model <route> --mode paced` once per route.
3. Only with frozen replay outputs and proven splits, start R2 baselines (constant, stability rule, L2 logistic, optional isotonic) under the section-16 gates.

## Do not retry

- Player microphone recordings as an implicit training corpus — diagnostic retention is not training consent; reconsider only after a separate explicit opt-in program exists.
- Random filesystem-order splits — they leak derivatives/speakers and are not reproducible; retain fixed source/speaker hashing.
- Calling attenuated voiced speech “whisper” — it does not reproduce whisper acoustics; keep it labeled attenuation.
- Hidden retry loops around failed replay clips — one attempt per clip, failures stay typed rows.

## Handoff

- **Workspace state:** R1 feature capture, service-path integration and the prepared-corpus replay runner are committed as localized `tools/speech-timeline` changes; datasets, ready files, profiles and reports remain outside Git.
- **Checks:** 153 speech-timeline tests, compileall, CLI smoke (`score`, `dry-run`, `replay` refusal paths), `git diff --check` pass; `host-check` not run for this Python-only increment (no Rust/workspace surface touched).
- **Remaining risk:** No real corpus replayed or scored; determinism is proven for identical event sequences on fixtures, not across two live model runs whose wall timings differ by nature.
- **Promotion needed:** None; the feature payload stays an internal experiment schema until an Accepted ADR promotes a runtime contract.
