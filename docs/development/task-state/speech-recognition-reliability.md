# Speech recognition reliability — current task state

| Field | Value |
| --- | --- |
| Status | `ACTIVE` |
| Updated | `2026-08-21` |
| Task key | `speech-recognition-reliability` |
| Scope | Public-data, utterance-level Russian ASR reliability in `tools/speech-timeline` |
| Definition of done | R0–R3 satisfy the gates and boundaries in `DEV-SPEECH-RELIABILITY-001`, or the first failed gate leaves the candidate explicitly report-only |
| Authority | Working context only; Accepted SPEC/ADR, `AGENTS.md` and the development specification outrank this file |

## Resume in 60 seconds

- **Current conclusion:** R0 is implemented and verified as a fail-closed external corpus runner; no real corpus or accuracy result exists yet.
- **Why:** The runner now closes source rights/hashes, speech and augmentation partitions, Russian scoring normalization and deterministic WER/CER before any replay/model claim.
- **Next action:** Implement bounded transcript-revision feature capture as R1, then replay one exact Voxtral/audio route.
- **Current blocker:** Public corpora have not been operator-acquired or hash-closed; the committed recipe is intentionally planned.
- **Do not retry:** Treat microphone diagnostics as training data or invent hashes for unavailable corpus/model artifacts.
- **Reconsider when:** An exact external source snapshot and resident Voxtral replay identity are available for closure.

## Current evidence

| Evidence | Result | Consequence |
| --- | --- | --- |
| `docs/development/speech-recognition-reliability-public-data-spec-2026-08-20.md` | `PASS` as approved development scope | Names the claim boundary, R0–R3 increments and evaluation gates |
| `tools/speech-timeline/examples/speech-reliability-public-safe-v0.recipe.json` | `REPORT_ONLY` | Exact planned recipe exists but honestly reports missing source/replay closure |
| `tools/speech-timeline/src/nextengine_speech_timeline/reliability.py` | `PASS` in 113-test speech-timeline suite | Normalizer, WER/CER, strict manifests, external WAV/SHA verification and deterministic partitions are executable |
| `cargo run -p xtask -- host-check` | `PASS` on pinned Rust 1.97.1 | Localized tooling change did not regress the workspace host gate |
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

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1: transcript revision stability plus signal/VAD features can calibrate exact-match risk cheaply | Required signals already exist in the resident path and need no second ASR | No frozen public replay/evaluation exists yet | R1 trace capture followed by source-disjoint R2 logistic baseline |
| H2: public-only calibration has useful target coverage | Common Voice/FLEURS cover Russian speakers and read/conversational domains | Whisper and real player microphone domain shift remain weakly represented | Held-out coverage/OOD report against the fixed gates |

## Required context

Read these sources in precedence order before acting:

1. `docs/architecture/agent-routing.md`; SPEC-11, SPEC-15, SPEC-16, SPEC-34; ADR-005, ADR-017, ADR-030, ADR-046 and ADR-053.
2. `docs/development/speech-recognition-reliability-public-data-spec-2026-08-20.md`.
3. `docs/development/task-state/audio-emotion-asr-timeline.md` for the resident service lineage and active Voxtral constraints.
4. `tools/speech-timeline/README.md` and `tools/speech-timeline/src/nextengine_speech_timeline/` for the implemented experiment boundary.

## Next action

1. Start R1 with bounded revision statistics and feature completeness checks on one exact ASR/audio route.
2. Replay only after an external recipe has exact source/index and resident model/runtime hashes.
3. Preserve current speech service behavior when no reliability artifact is configured.

## Do not retry

- Player microphone recordings as an implicit training corpus — diagnostic retention is not training consent; reconsider only after a separate explicit opt-in program exists.
- Random filesystem-order splits — they leak derivatives/speakers and are not reproducible; retain fixed source/speaker hashing.
- Calling attenuated voiced speech “whisper” — it does not reproduce whisper acoustics; keep it labeled attenuation.

## Handoff

- **Workspace state:** R0 implementation, example recipe, tests and documentation are ready as one localized Python tooling commit.
- **Checks:** 113 speech-timeline tests, CLI dry-run/score smoke, compileall, recipe JSON parse, `uv lock --check`, `git diff --check` and `host-check` pass.
- **Remaining risk:** No real corpus was downloaded, closed, replayed or evaluated; R0 enables that work but makes no accuracy claim.
- **Promotion needed:** None for R0; any public engine contract or shipped model-pack promotion still requires the normal Accepted ADR/SPEC workflow.
