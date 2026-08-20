# Audio emotion + ASR timeline — current task state

| Field | Value |
| --- | --- |
| Status | `PHASE_1_IMPLEMENTED`; RAW remains the protocol fallback while the dashboard exposes measured gain/full-DPDF/whisper-preserving A/B routes |
| Updated | `2026-08-20` |
| Task key | `audio-emotion-asr-timeline` |
| Scope | Phase 1 prototypes Voxtral/emotion2vec; Phase 2 adds replaceable LLM/TTS; Phase 3A proves them in a one-character simple-dialogue scene before 3B-3D boundary/internal-model work; prerequisite-gated Phase 4 fine-tunes FunctionGemma. |
| Definition of done | A resident prototype exposes versioned transcript/affect/fusion revisions, declares timing precision, avoids model reload between clients and reports joint latency/resource evidence. |
| Authority | Working context only; Accepted ADR-005 and repository architecture outrank this file. SPEC-16/ADR-017 remain Deferred Proposed. |

## Resume in 60 seconds

- **Current conclusion:** Phase 1 implements the resident model-neutral timeline service. A new explicit `whisper` route uses calibrated one-stage gain, DPDFNet, an aligned 12 dB dry safety floor and a limiter; `gain_only` and the previous full `enhanced` route remain controls. On the exact 6.944-s user whisper WAV, all four routes preserved 111,104 samples and preprocessing stayed within 16 ms p95, but Voxtral returned empty text even when gain-only raised RMS from −46.47 to −26.75 dBFS. The wrapper can no longer erase the signal completely, but level/denoise alone does not solve this model's whisper recognition.
- **Why:** The existing Voxtral wrapper already performs real streaming inside one invocation, while warm emotion2vec inference is fast. Reloading either model per connection/window is the avoidable delay.
- **Critical limit:** Current Voxtral public APIs return streaming text but no lexical timestamps. `transcribe.cpp` reports timestamp kind `NONE`; its Voxtral `audio_committed_ms` remains zero during feed and is not a text boundary.
- **Implementation plan:** `docs/plans/2026-08-18-speech-timeline-service-implementation.md` has approved scope `A/A/A/A`: standalone authenticated localhost WebSocket, explicit finish first and honest `utterance` alignment; VAD/model-slot remain later increments.
- **Phase 2 plan:** `docs/plans/2026-08-18-conversation-service-phase-2.md` adds a `ConversationService` facade for large-LLM dialogue and TTS only; two Phase 2 scope choices remain pending.
- **Phase 3 plan:** `docs/plans/2026-08-18-engine-neural-capability-integration-phase-3.md` now starts with 3A: one existing character, push-to-talk, session-local persona/history, subtitles/TTS and no world/internal-model/tool context. 3B-3D then harden proven boundaries, shadow-integrate the strategic model and freeze catalogs.
- **Phase 4 plan:** `docs/plans/2026-08-18-functiongemma-strategic-integration-phase-4.md` fine-tunes FunctionGemma only after Phase 3 freezes a consumer-backed strategic catalog and corpus seed.
- **Next action:** Let the user compare the route-labelled WAVs live, then freeze a small transcribed Russian normal/whisper corpus and compare the accepted front-end against a whisper-capable ASR/model adaptation. Evaluate neural VAD and render-reference AEC only as independent increments; see `docs/development/speech-input-front-end-research-2026-08-20.md`.
- **Current blocker:** The PCM front-end increment is testable; whispered recognition is model/evidence-blocked. One external user clip falsifies amplitude as the sole cause but cannot estimate WER or select a replacement ASR without fixed reference transcripts.
- **Do not retry:** Per-window process launch/checkpoint reload; ASR attachment through `audio_committed_ms`; fabricated word timestamps; RMS/retained energy as the quality oracle; more gain or stronger denoising on the same failed clip.
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
| `tools/speech-timeline` ASR route contract, 2026-08-20 | protocol defaults to `raw`; dashboard/CLI expose raw, gain-only, full-DPDF and whisper-preserving A/B; focused protocol/WebSocket tests prove raw bypasses `reset/process/flush` while processed routes preserve the sample clock | DPDFNet presence no longer silently changes ASR input; identical external WAVs can be replayed through each advertised route with lineage, signal levels and timing metrics. |
| `docs/development/voxtral-emotion2vec-facade-research-2026-08-18.md` | bounded pair/facade research complete | Supersedes the prior SimulStreaming ASR selection and defines staged implementation/evidence. |
| User confirmation `A/A/A/A`, 2026-08-18 | first implementation scope approved | Standalone / explicit finish / utterance MVP / authenticated localhost WebSocket; no engine-facing IPC, mandatory VAD or model-slot gate. |
| `docs/plans/2026-08-18-speech-timeline-service-implementation.md` | approved implementation sequence | Defines the smallest resident vertical and commit/test boundaries; it does not promote SPEC-16. |
| User Phase 2 request, 2026-08-18 | add TTS, LLM and FunctionGemma connection design | Preserve Phase 1 boundary and add a separate conversation facade rather than coupling models into the speech timeline implementation. |
| Official Google FunctionGemma docs/model card, checked 2026-08-18 | 270M function-calling specialization; fine-tuning intended; single/parallel baseline, multi-step/multi-turn not trained out of the box; not a direct dialogue model | Do not add a generic dialogue role. A later trained adapter may serve only as a bounded post-LLM strategic-call compiler behind validation. |
| User FunctionGemma ordering clarification, 2026-08-18 | FunctionGemma follows the large LLM and mediates LLM tool calls to the engine strategic neural model | Use `LLM plan → FunctionGemma compile → StrategicAgentGateway → LLM response → TTS`, not FunctionGemma-first routing. |
| User FunctionGemma staging clarification, 2026-08-18 | FunctionGemma needs engine/strategic-network fine-tuning and tight integration; there is no value in connecting it now | Keep Phase 2 limited to LLM/TTS and defer FunctionGemma behind explicit prerequisites. |
| User intermediate-stage clarification, 2026-08-18 | Existing neural-network capabilities must first be brought into engine boundaries between Phase 2 and FunctionGemma | Add a consumer-driven Phase 3 capability integration/shadow stage and renumber FunctionGemma as Phase 4. |
| User Phase 3A clarification, 2026-08-18 | Phase 3 is large and begins with a demo scene containing one character for simple dialogue, without world context or internal-model integration | Make the playable dialogue vertical the first independently closable gate; move catalog hardening, strategic shadow work and FunctionGemma preparation to 3B-3D. |
| `docs/plans/2026-08-18-conversation-service-phase-2.md` | proposed Phase 2 sequence | Defines LLM/TTS worker topology, context seam, evidence and two pending scope choices without tool calling or strategic integration. |
| `docs/plans/2026-08-18-engine-neural-capability-integration-phase-3.md` | proposed Phase 3 sequence | Maps actual models to engine-owned roles, validators, owners and fallbacks; shadow-integrates the strategic model and freezes consumer-backed catalogs. |
| `docs/plans/2026-08-18-functiongemma-strategic-integration-phase-4.md` | deferred Phase 4 sequence | Reopens exact Phase 3 catalogs, then builds corpus, measures/fine-tunes FunctionGemma and integrates it shadow-first. |
| `codex/speech-timeline-service`, Phase 1 fake suite | 86 Python service tests + 14 preserved Voxtral probe tests pass | Auth/version/size/state/fault/dashboard boundaries, four-way ASR routing, second-session residency, cadence, no-word-span fusion, bounded terminal metrics and client/benchmark paths are executable without weights. |
| Joint RTX 3080 run, exact artifacts, 2026-08-18 | 2 × 4.5 s paced turns; load counts Voxtral/emotion `1/1`; worker-busy RTF p95 `0.690`; first chunk→update p95 `1.363 s`; finish→final p95 `0.970 s`; process peak `5870 MiB` VRAM | Both CUDA models remain resident with >1 GiB headroom; explicit-finish vertical passes. Report: `/tmp/nextengine-speech-timeline-phase1-benchmark.json` (external, content-free). |
| 30 s paced soak, 2026-08-18 | worker-busy RTF `0.530`; first update `1.136 s`; finish→final `1.352 s`; no reload | Paced streaming remains faster than realtime without unbounded ASR backlog. Direct internal scheduler saturation still fails closed. |
| Vue dashboard + turn-bound regression, 2026-08-18 | Browser source builds with pinned Vue/Vite/TypeScript; same-origin HTTP/bootstrap and WebSocket-origin tests pass; CLI/browser auto-finalize at the advertised 30 s/960,000-byte bound; overflow emits one terminal `TURN_TOO_LARGE` | The diagnostic view can expose transcript/raw affect/smoothed fusion without token copy-paste or repeated overflow spam. |
| Real 30 s unpaced ingress after browser overload report, 2026-08-18 | completed in `15.478 s`; worker-busy RTF `0.515`; first update `0.597 s`; scheduler max useful depth `4`, overloads `0`, model load counts `1/1` | A bounded two-ASR-job admission semaphore absorbs transient transport bursts without expanding the model scheduler or failing a live turn. External content-free report: `/tmp/nextengine-speech-backpressure-30s.json`. |
| Optimized 80 ms paced ingress, partial decode 240 ms, 2026-08-18 | 3 × 10.44 s; first affect p95 `1.094 s`, first ASR revision p95 `1.806 s`, finish→final p95 `0.810 s`, worker-busy RTF p95 `0.594`, max queue depth `2`, overloads `0`; 44.1/48 kHz 12 s resampler checks have zero sample drift | Keep model delay 480 ms for quality, separate partial cadence at 240 ms, and expose ASR/affect latency separately. Report: `/tmp/nextengine-speech-optimized-final.json` (external, content-free). |
| Long-turn terminal-event soak, 2026-08-18 | Before fix, a 29–30 s paced turn completed model work but `utterance.final` JSON reached `68,660` bytes because all per-job metrics were serialized; the sender then failed with `EVENT_TOO_LARGE` and the client waited. After fix, exact 30 s completes in `31.204 s`, finish→final `0.987 s`, worker-busy RTF `0.554`; all `479` jobs remain represented by aggregates plus a 64-job tail. | Bound diagnostic metrics on the wire and fail closed on any future oversized event; do not treat this protocol failure as model inference degradation. Reports: `/tmp/nextengine-speech-diag-29s-fixed-full.json`, `/tmp/nextengine-speech-diag-30s-fixed-metrics.json` (external, content-free). |
| User screenshot + emotion quality review, 2026-08-18 | Final raw `other`/one-hop transition could make the current fusion `unknown` and incorrectly overwrite the utterance result. | Final expression now aggregates only admitted primary speech segments; raw `other`/`unknown` stay diagnostic. Explicit browser quiet-room calibration adjusts only VAD thresholds; final diagnostics expose transition/abstention evidence. Research: `docs/development/emotion-timeline-quality-research-2026-08-18.md`. |
| External RESD-70 v2 held-out screen, 2026-08-18 | 10 actor-stratified test clips × 7 labels, all at least 1 s, were normalised externally to 16 kHz mono. Pinned base direct whole-utterance score was `29/60` (`48.3%`), but actual resident WebSocket VAD/window/smoothing final score was `23/60` (`38.3%`), with all `70/70` clips speech-admitted and affect-observed plus `8` final abstentions. Direct report `/home/kaifaty/.cache/nextengine/emotion-calibration/resd-70-v2/emotion2vec-plus-base-report.v1.json`, SHA-256 `4dc1ef57ceb39c6ec21f4b9507dfaf104e32a4928ad755ae41c4bce954d569de`; timeline report `/home/kaifaty/.cache/nextengine/emotion-calibration/resd-70-v2/timeline-base-report.v1.json`, SHA-256 `726c58c1540547f012c1d8b34cee85a338bcbd1ebd7b9affd62fba44b50e024d`. | VAD does not explain this screen's quality loss; the observed 10-point gap is temporal/window/final-aggregation behavior. Keep v2 frozen, do not tune label/VAD/smoothing policy to it, and compare the exact direct/full-path contract with large before a policy experiment. |
| Pinned `emotion2vec_plus_large` direct A/B, 2026-08-19 | Official revision `6c303ba987b86b93193de93e34bb2b077a6bedc4` loaded successfully alongside the resident base service but scored `26/60` (`43.3%`) direct against frozen RESD-70 v2, five points below base's `29/60` (`48.3%`). It improved happy/sad but regressed angry/disgusted/fearful. External report `/home/kaifaty/.cache/nextengine/emotion-calibration/resd-70-v2/emotion2vec-plus-large-report.v1.json`, SHA-256 `152eaed20f0a7d6f36ae6f9ddbc52c67215099f9d4f79812a7736b6818e02a07`. | Reject large as a default replacement; do not interrupt the base resident service for an expensive full-path run. Reconsider only under an explicit downstream class weighting or if a modified candidate first passes the direct gate. |
| Pinned `Aniemore/wavlm-emotion-russian-resd` integration, 2026-08-19 | Exact standard-Transformers revision `7a4ca18b34adff59b56b451acc7ff44fc43a12dc` (safetensors SHA-256 `dabf15d84b451195346b8050102a7243b3f92276064195f6e34b65aaa06a12ab`) now has a local-only, no-remote-code adapter. Real Voxtral+WavLM WebSocket/VAD/scheduler/full-ASR replay completed all `70/70` v2 clips, admitted/observed all, and scored `45/60` (`75.0%`) on the six-class map, with two abstentions. Ready residency is 4,924 MiB process VRAM / 3,382 MiB free after 2.6 s Voxtral + 4.4 s WavLM loads; a two-run paced 5 s check has worker RTF p95 `0.609`, finish→final `761 ms`, capture-start→first affect `2.352 s`, stable emotion inference `21/24 ms` p50/p95. Direct/stub/joint/paced report SHA-256: `ed66c8e250695daf13805ef61888ed976f1aa35562ce1ab96fa93db1834c3489`, `8fdfc51f47474c1888b575f2d5c62b9e238a21bb60db071bb065c0106c8e572f`, `5add6a8be7b10c18245f6c24a3e481b3c7d54e2d1148fdcf14645ef59cf090e7`, `371a2a9012d07abc9b382bc86d9f992e052d3a2dc35ed9a6925f2f7687352582`. | The WavLM profile is accepted for local diagnostic use, preserving `enthusiasm`; no architecture/default-product promotion is claimed. Keep base as rollback, and do not call the unpaced joint p95 6.805 s result Live latency; first-affect timing needs microphone validation. |
| Pinned DPDFNet whisper-preserving discriminator, 2026-08-20 | Exact external 6.944-s WAV SHA-256 `e25a9a1ad8fd1d11c1e208222063b9ef1866869471343f255ff62d5c5417c7ae`: raw/gain-only/full-DPDF/final-whisper RMS `−46.47/−26.75/−29.01/−33.62` dBFS; nonzero ratios `0.8198/0.8198/0.4154/0.7749`; final whisper preprocess p95 `14 ms`, flush `3 ms`, exact `111,104` samples. Voxtral final text was empty for every route. | The aligned 12 dB dry safety floor closes complete over-suppression and stays real-time, but successful +19.72 dB gain did not restore recognition. Keep all processed routes diagnostic; next compare against whisper-capable ASR/adaptation on reference-transcribed Russian clips rather than tuning more gain. Detailed evidence: [research](../speech-input-front-end-research-2026-08-20.md). |
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

### D-006 — First implementation scope approved as A/A/A/A

- **Observation:** The user explicitly selected all four recommended scope options on 2026-08-18.
- **Decision:** Deliver the first vertical as a standalone authenticated localhost WebSocket service with explicit start/finish and `utterance` alignment.
- **Rejected for MVP:** Engine-facing IPC, mandatory automatic VAD, a `model_slot` completion gate and Unix-domain-socket-only transport.
- **Consequence:** Commit 0/1 may begin without another scope question; later increments remain separately measured and approved.
- **Reconsider when:** The standalone latency/resource gates pass and a concrete engine consumer authorizes promotion, or the selected transport cannot satisfy a reproducible local constraint.

### D-007 — FunctionGemma follows engine neural-capability integration in Phase 4

- **Observation:** FunctionGemma must be trained on actual engine/strategic-model capabilities, but those capabilities first need role, authority, validator, fallback and resource boundaries inside the engine.
- **Decision:** Keep Phase 2 focused on LLM/TTS. Phase 3 integrates only actual runnable models and runs the strategic model shadow-first, then freezes consumer-backed neural/strategic catalogs. Phase 4 trains and integrates FunctionGemma against those exact revisions. The eventual order remains `LLM planning → FunctionGemma compile → validated StrategicAgentGateway → LLM response → TTS`.
- **Rejected:** Generic base FunctionGemma integration before engine capability convergence, a universal neural registry without consumers, FunctionGemma-first routing, direct model execution/world access and speculative strategic tool schemas.
- **Consequence:** Phase 3 can validate existing models independently of FunctionGemma and supplies its training surface. The strategic backend remains replaceable and proposal-only; deterministic Strategic Agent/authored dialogue remain complete fallbacks.
- **Remaining uncertainty:** Exact LLM/TTS artifacts, joint 10-GiB resource profile, exact strategic-model interface, role classification, catalog breadth and Phase 4 thresholds.
- **Reconsider when:** Phase 3 cannot produce a stable consumer-backed catalog, or a different measured compiler makes FunctionGemma unnecessary without weakening validation/fallback boundaries.

### D-008 — Phase 3A is a presentation-only one-character dialogue demo

- **Observation:** The user made the first part of the large Phase 3 a scene
  where one character supports simple dialogue, explicitly without world
  context or integration with internal models.
- **Decision:** Reuse the clean `reference-alpha` relay keeper and the existing
  presentation-only semantic `Interact` open/close path, but never enter its
  authored `AcceptPending` transition. Use push-to-talk and the Phase 1/2
  resident services. The LLM sees only a fixed demo persona, bounded
  current-session history, final utterance, structured vocal affect and
  response limits. The session owns no gameplay state and is discarded on
  close.
- **Rejected for 3A:** World snapshots, Agent memory/goals, strategic neural
  models, tools, FunctionGemma, persistent memory, gameplay consequences, lip
  sync and new art/animation pipelines.
- **Consequence:** 3A can close on a three-turn microphone → subtitle/TTS demo
  plus fault/resource evidence. 3B generalizes only seams proven by that demo;
  3C audits/shadow-integrates internal models; 3D prepares Phase 4 catalogs.
- **Reconsider when:** 3A cannot be demonstrated through existing interaction,
  presentation and audio boundaries without adding authoritative state.

### D-009 — Admit the serialized joint CUDA profile for Phase 1

- **Observation/evidence:** Real pinned Voxtral Q4_K_M + emotion2vec runs peaked at 5870 MiB process VRAM, kept both load counts at one, and achieved worker-busy RTF below one in two-turn and 30 s paced runs.
- **Decision:** Keep both models in one resident process and one priority worker for the current local RTX 3080 profile; retain one active utterance and explicit finish.
- **Rejected:** CPU emotion fallback now, separate GPU processes, or weakening bounds to make unpaced bursts succeed.
- **Consequence/uncertainty:** Phase 1 service implementation is admissible; microphone acoustics, Russian quality and within-turn affect accuracy remain evaluation work, not residency blockers.
- **Reconsider when:** Representative live runs exceed the 1 GiB headroom/RTF gates, or model/runtime changes invalidate the exact profile.

### D-010 — Same-origin diagnostic UI and terminal turn bounds

- **Observation:** Raw protocol JSON obscures independent revisions, an overlong open microphone previously produced repeated nonterminal `TURN_TOO_LARGE` events after the 30 s in-memory ceiling, and a dashboard tab surviving service restart retained the old rotated token and failed its next hello with `AUTH_FAILED`.
- **Decision:** Serve a bundled Vue dashboard from the loopback speech process, keep transcript/raw affect/smoothed fusion as separate views on the sample clock, fetch the ephemeral token only through no-store same-origin bootstrap at mount and immediately before each session (so service restarts cannot leave a stale token), auto-finalize clients at the advertised bound, make actual overflow terminal exactly once, and apply bounded ASR admission backpressure before the GPU scheduler.
- **Rejected:** Inline word emotion tags without lexical timing, browser token entry, a separate permissive dev server, silently increasing the bounded turn or scheduler queue, continuing capture after overflow, or treating a transient browser delivery burst as terminal overload.
- **Consequence:** The UI remains an optional diagnostic projection and not gameplay authority. Conversations longer than 30 s require multiple utterances; bounded VAD remains a later measured increment.
- **Reconsider when:** A verified aligner supplies lexical timing or an approved multi-utterance consumer requires automatic endpointing.

### D-011 — Separate transport, decode and model-delay clocks

- **Observation/evidence:** The browser's block-local resampler added 94 ms over 12 s at 48 kHz; 80 ms transport plus a 240 ms partial-decode interval reduced paced finish and first-update latency without overload, while the first ASR revision still trails the first 1 s affect window.
- **Decision:** Keep the Voxtral quality-delay setting at 480 ms, feed 80 ms PCM, request partial decoding every 240 ms, preserve resampler phase across worklet blocks and flush the last block before finish. Report ASR and affect first-result latency separately.
- **Consequence:** Transport cadence no longer masquerades as model delay, browser audio time stays aligned, and later tuning can change adapter cadence without changing the facade.
- **Reconsider when:** Representative live Russian WER/revision-churn or RTF degrades, or a runtime exposes a lower-latency quality profile with measured parity.

### D-012 — Bound terminal diagnostics and fail closed on oversized events

- **Observation/evidence:** A long turn accumulated hundreds of model-job metrics; the 64 KiB protocol limit was exceeded after inference had completed, killing the sender task before `utterance.final`.
- **Decision:** Keep exact aggregates and only the last 64 job records in terminal metrics; treat any future `EVENT_TOO_LARGE` as one terminal protocol error instead of leaving the client waiting.
- **Consequence:** Long turns remain observable without unbounded wire payloads; benchmark aggregation reads the exact job summary when the tail is truncated.
- **Reconsider when:** The protocol gains a separately authenticated streaming metrics channel or a consumer requires complete per-job traces off-band.

### D-013 — Bound wrapper copies and append-only timeline updates

- **Observation/evidence:** Voxtral already owns a stateful native stream; the Python facade was copying the growing turn buffer on every 80-ms frame and repeating all affect observations in every event. The optimized 30-s paced run copies 0.96 MB once plus 7.36 MB of bounded affect windows, emits at most 3.3 KB events, and keeps scheduler depth at 2 with zero overloads.
- **Decision:** Keep the native Voxtral path untouched. Use `pcm_window()` for affect requests, emit append-only affect deltas merged by the Vue dashboard, bound event timing samples, and log ingress/model/event timings. Do not add external sentence trimming for current stateful models.
- **Consequence:** The former ~172 MiB cumulative Python PCM-copy path is removed; remaining delay is attributable to model work and serialized priority contention rather than growing wrapper payloads.
- **Reconsider when:** A replacement model lacks stateful streaming, retained audio or per-update compute grows with turn duration, or a measured live run shows wrapper/transport backpressure.

### D-014 — Embedded PCM preprocessing; no virtual capture-device dependency

- **Observation:** The eventual consumer is an embedded game host, not a Linux-only diagnostic application. The user explicitly rejected creating a PipeWire or other virtual microphone as part of the product path.
- **Decision:** A preprocessing adapter accepts host-captured PCM and produces timestamp/length-preserving derived PCM branches. It is in-process or a declared bounded host service; it never installs, creates or depends on a system virtual audio device. Its public seam stays model-neutral and reports exact algorithm/model identity, configuration, delay and bypass state. The implemented protocol defaults to `raw`; a configured resident preprocessor advertises explicit `gain_only`, full `enhanced` and attenuation-limited `whisper` routes, receives the calibrated noise floor only for its gain gate, and otherwise receives no per-turn calls.
- **Consequence:** The local Python prototype exposes the same model-neutral resident seam for evidence only; the future native Rust DSP must expose it directly. Raw is the selected control/fallback. ASR, neural activity and affect use separate derived branches; affect remains raw or AEC-only until its own labelled gate. WebRTC APM/AEC, Silero VAD and conservative DPDFNet/DeepFilterNet/RNNoise routes are independently evaluated; no enhancer is the native default yet. See [front-end research](../speech-input-front-end-research-2026-08-20.md).
- **Rejected/reconsider:** No virtual device, global AGC/shared affect stream or per-window CLI filtering. Reconsider only if the in-process A/B trial has no benefit, fails its resource/latency envelope, or a cross-platform host boundary requires a different adapter contract.

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1: both models fit and remain faster than realtime under serialized joint load | Confirmed on current exact profile: 5870 MiB peak and worker-busy RTF 0.530–0.690. | One Russian sample/host is not a broad deployment envelope. | Repeat after artifact/runtime/device change and on representative live dialogue. |
| H2: Voxtral token slots are accurate enough for clause-level affect attachment | Training uses explicitly aligned 80 ms audio/text streams. | Public APIs omit timing; slot offset/grouping error is unknown. | Expose token/control slots and compare to manually aligned Russian words. |
| H3: utterance-grade context already improves downstream LLM responses | It preserves vocal evidence honestly without timestamp invention. | Mixed emotion inside a turn may be smeared. | Blind downstream response evaluation: text-only versus turn affect versus timed spans. |
| H4: 1 s/250 ms emotion windows give useful Live transitions | Frozen RESD-70 v2 passes `70/70` VAD admissions/observations; WavLM's real one-process path is `45/60`, and its stable emotion jobs are `21/24 ms` p50/p95. | The paced WAV's `2.352 s` capture-start first-affect metric and acted clips do not measure microphone acoustics or transition boundary quality; `nikatonika` claims an incomparable internal validation score. | Direct-screen the candidate on the frozen six-class map, then run a labelled user-microphone checklist before changing any default; evaluate neural VAD only if a separate calibrated live gate fails. |

## Required context

Read in precedence order:

1. `docs/architecture/agent-routing.md`
2. `docs/architecture/adr/005-offline-first-ai-process-boundary.md`; `docs/architecture/06-ai-agents-perception-and-memory.md`
3. `docs/architecture/32-npc-cognition-intention-lifecycle-and-deterministic-behavior-inference.md`; `docs/architecture/adr/056-deterministic-strategic-agent-and-belief-driven-goap.md`
4. `docs/architecture/adr/073-deterministic-cognition-owner-vertical.md`; `docs/architecture/adr/074-systemic-strategic-agent-owner-vertical.md`
5. `docs/architecture/16-text-canonical-multimodal-dialogue-and-model-packs.md`; `docs/architecture/adr/017-text-canonical-multimodal-dialogue-and-replaceable-model-packs.md`
6. `docs/architecture/33-behavior-policy-training-evaluation-and-deployment-lifecycle.md`; `docs/architecture/34-model-training-environments-trajectories-and-consolidation-lifecycle.md`
7. `docs/architecture/adr/050-hierarchical-npc-cognition-and-learned-behavior-policy-boundary.md`; `docs/architecture/adr/053-engine-native-model-training-and-immutable-artifact-boundary.md`
8. `docs/architecture/adr/054-bounded-strategic-adaptation-and-two-tier-sleep.md`; `docs/architecture/18-player-interaction-ui-camera-localization-and-accessibility.md`
9. `docs/architecture/29-platform-host-and-application-session.md`; `docs/architecture/30-presentation-extraction-and-render-content.md`
10. `docs/architecture/08-audio-navigation-and-world-services.md`; `docs/architecture/adr/019-canonical-player-actions-and-presentation-authority.md`
11. `docs/architecture/adr/044-neutral-text-catalog-and-locale-fallback.md`; `docs/architecture/adr/047-simple-application-session-and-save-on-close.md`
12. `docs/architecture/adr/028-platform-session-and-presentation-authority.md`; `docs/architecture/09-tooling-sdk-and-observability.md`
13. `docs/architecture/11-security-licensing-and-governance.md`
14. `docs/development/voxtral-emotion2vec-facade-research-2026-08-18.md`
15. `docs/plans/2026-08-18-speech-timeline-service-implementation.md`
16. `docs/plans/2026-08-18-conversation-service-phase-2.md`
17. `docs/plans/2026-08-18-engine-neural-capability-integration-phase-3.md`
18. `docs/plans/2026-08-18-functiongemma-strategic-integration-phase-4.md`
19. `e839a38:lab/scripts/voxtral_microphone.py` and its tests
20. `tools/speech-timeline/README.md` and implementation

## Smallest next action

1. User-test the implemented `whisper` route after quiet calibration and compare its route-labelled WAV with RAW; the exact signal and latency metrics are already emitted in `utterance.final`.
2. Freeze comparable reference-transcribed normal/whisper/noise Russian takes externally and compare the current front-end with a whisper-capable ASR/adaptation using WER/CER and blind listening; the one existing clip already rejects further gain-only tuning.
3. Confirm the two unresolved Phase 2 choices and execute its Commit A without merging LLM/TTS implementation into `SpeechTimelineService`.
4. Keep VAD and Voxtral model-slot timing as separately measured increments; do not fabricate word spans meanwhile.
5. After Phase 2 evidence, implement Phase 3A first: clean `reference-alpha`
   relay keeper, semantic presentation-only dialogue open, fake then real
   `ConversationClient`, push-to-talk, subtitles/TTS, three-turn residency and
   fault/resource evidence.
6. Only after 3A closes, execute 3B capability hardening and 3C strategic-model
   audit/shadow integration; do not add world/memory/tool context to 3A.
7. Do not start Phase 4 until 3D freezes consumer-backed
    `NeuralCapabilityCatalog` and `StrategicSemanticCatalog` revisions plus the
    external corpus/evaluation seed.

## Do not retry

- One process per chunk/window or one model load per client.
- Treat the unpaced WavLM joint quality result as Live latency or as a production/default-model promotion.
- Treating 480 ms configured delay as measured end-to-end latency.
- Treating text arrival/commit time as acoustic word time.
- Emitting inline word emotion tags while capability is `none`/`utterance`.
- Recreating the Voxtral wrapper in this branch instead of using the tested `e839a38` source.
- Creating, installing or requiring PipeWire, a virtual microphone or another system audio device for the embedded path.
- Treating louder processed whisper, RMS, SNR proxy or DNSMOS alone as proof that ASR/affect quality improved.

## Handoff

- **Workspace state:** `codex/speech-timeline-service` contains the converged tested wrapper and complete standalone Phase 1 service/client/benchmark implementation; external models, profiles, ready files and reports remain outside Git.
- **Checks:** Run the focused Python/lab suites, `git diff --check`, lock consistency and final risk-scoped `host-check` before handoff.
- **Remaining risk:** User-spoken microphone acceptance, broader Russian ASR quality, VAD/model-slot alignment, emotion transition quality, and emotion2vec+ redistribution terms.
- **Promotion needed:** None; no Accepted architecture or roadmap change is authorized by this research.
