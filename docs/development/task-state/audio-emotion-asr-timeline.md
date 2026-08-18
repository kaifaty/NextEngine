# Audio emotion + ASR timeline — current task state

| Field | Value |
| --- | --- |
| Status | `PHASE_1_IMPLEMENTED`; manual microphone acceptance remains |
| Updated | `2026-08-18` |
| Task key | `audio-emotion-asr-timeline` |
| Scope | Phase 1 prototypes Voxtral/emotion2vec; Phase 2 adds replaceable LLM/TTS; Phase 3A proves them in a one-character simple-dialogue scene before 3B-3D boundary/internal-model work; prerequisite-gated Phase 4 fine-tunes FunctionGemma. |
| Definition of done | A resident prototype exposes versioned transcript/affect/fusion revisions, declares timing precision, avoids model reload between clients and reports joint latency/resource evidence. |
| Authority | Working context only; Accepted ADR-005 and repository architecture outrank this file. SPEC-16/ADR-017 remain Deferred Proposed. |

## Resume in 60 seconds

- **Current conclusion:** Phase 1 now implements the resident optional `SpeechTimelineService` with model-neutral Voxtral/emotion2vec adapters, bounded scheduling, three independently revisioned tracks, authenticated loopback WebSocket, microphone client and benchmark runner.
- **Why:** The existing Voxtral wrapper already performs real streaming inside one invocation, while warm emotion2vec inference is fast. Reloading either model per connection/window is the avoidable delay.
- **Critical limit:** Current Voxtral public APIs return streaming text but no lexical timestamps. `transcribe.cpp` reports timestamp kind `NONE`; its Voxtral `audio_committed_ms` remains zero during feed and is not a text boundary.
- **Implementation plan:** `docs/plans/2026-08-18-speech-timeline-service-implementation.md` has approved scope `A/A/A/A`: standalone authenticated localhost WebSocket, explicit finish first and honest `utterance` alignment; VAD/model-slot remain later increments.
- **Phase 2 plan:** `docs/plans/2026-08-18-conversation-service-phase-2.md` adds a `ConversationService` facade for large-LLM dialogue and TTS only; two Phase 2 scope choices remain pending.
- **Phase 3 plan:** `docs/plans/2026-08-18-engine-neural-capability-integration-phase-3.md` now starts with 3A: one existing character, push-to-talk, session-local persona/history, subtitles/TTS and no world/internal-model/tool context. 3B-3D then harden proven boundaries, shadow-integrate the strategic model and freeze catalogs.
- **Phase 4 plan:** `docs/plans/2026-08-18-functiongemma-strategic-integration-phase-4.md` fine-tunes FunctionGemma only after Phase 3 freezes a consumer-backed strategic catalog and corpus seed.
- **Next action:** Run one user-spoken microphone acceptance through `next-speech-timeline microphone`, then reopen the two Phase 2 LLM/TTS scope choices; VAD/model-slot remain separate increments.
- **Current blocker:** No service implementation blocker. A meaningful live-microphone quality/latency run requires a user-spoken utterance; automated evidence uses the external 4.5 s Russian WAV and 30 s paced soak.
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
| `codex/speech-timeline-service`, Phase 1 fake suite | 45 Python service tests + 14 preserved Voxtral probe tests pass | Auth/version/size/state/fault boundaries, second-session residency, cadence, no-word-span fusion and client/benchmark paths are executable without weights. |
| Joint RTX 3080 run, exact artifacts, 2026-08-18 | 2 × 4.5 s paced turns; load counts Voxtral/emotion `1/1`; worker-busy RTF p95 `0.690`; first chunk→update p95 `1.363 s`; finish→final p95 `0.970 s`; process peak `5870 MiB` VRAM | Both CUDA models remain resident with >1 GiB headroom; explicit-finish vertical passes. Report: `/tmp/nextengine-speech-timeline-phase1-benchmark.json` (external, content-free). |
| 30 s paced soak, 2026-08-18 | worker-busy RTF `0.530`; first update `1.136 s`; finish→final `1.352 s`; no reload | Paced streaming remains faster than realtime without unbounded ASR backlog. Unpaced burst separately fails closed as `SERVICE_OVERLOADED`. |
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

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1: both models fit and remain faster than realtime under serialized joint load | Confirmed on current exact profile: 5870 MiB peak and worker-busy RTF 0.530–0.690. | One Russian sample/host is not a broad deployment envelope. | Repeat after artifact/runtime/device change and on representative live dialogue. |
| H2: Voxtral token slots are accurate enough for clause-level affect attachment | Training uses explicitly aligned 80 ms audio/text streams. | Public APIs omit timing; slot offset/grouping error is unknown. | Expose token/control slots and compare to manually aligned Russian words. |
| H3: utterance-grade context already improves downstream LLM responses | It preserves vocal evidence honestly without timestamp invention. | Mixed emotion inside a turn may be smeared. | Blind downstream response evaluation: text-only versus turn affect versus timed spans. |
| H4: 1 s/250 ms emotion windows give useful Live transitions | Warm inference is ~8 ms and windows overlap densely. | Utterance-pooled classifier may smear or flicker. | Labeled Russian within-turn transition corpus with boundary/F1 and churn metrics. |

## Required context

Read in precedence order:

1. `docs/architecture/agent-routing.md`
2. `docs/architecture/adr/005-offline-first-ai-process-boundary.md`
3. `docs/architecture/06-ai-agents-perception-and-memory.md`
4. `docs/architecture/32-npc-cognition-intention-lifecycle-and-deterministic-behavior-inference.md`
5. `docs/architecture/adr/056-deterministic-strategic-agent-and-belief-driven-goap.md`
6. `docs/architecture/adr/073-deterministic-cognition-owner-vertical.md`
7. `docs/architecture/adr/074-systemic-strategic-agent-owner-vertical.md`
8. `docs/architecture/16-text-canonical-multimodal-dialogue-and-model-packs.md`
9. `docs/architecture/adr/017-text-canonical-multimodal-dialogue-and-replaceable-model-packs.md`
10. `docs/architecture/33-behavior-policy-training-evaluation-and-deployment-lifecycle.md`
11. `docs/architecture/34-model-training-environments-trajectories-and-consolidation-lifecycle.md`
12. `docs/architecture/adr/050-hierarchical-npc-cognition-and-learned-behavior-policy-boundary.md`
13. `docs/architecture/adr/053-engine-native-model-training-and-immutable-artifact-boundary.md`
14. `docs/architecture/adr/054-bounded-strategic-adaptation-and-two-tier-sleep.md`
15. `docs/architecture/18-player-interaction-ui-camera-localization-and-accessibility.md`
16. `docs/architecture/29-platform-host-and-application-session.md`
17. `docs/architecture/30-presentation-extraction-and-render-content.md`
18. `docs/architecture/08-audio-navigation-and-world-services.md`
19. `docs/architecture/adr/019-canonical-player-actions-and-presentation-authority.md`
20. `docs/architecture/adr/044-neutral-text-catalog-and-locale-fallback.md`
21. `docs/architecture/adr/047-simple-application-session-and-save-on-close.md`
22. `docs/architecture/adr/028-platform-session-and-presentation-authority.md`
23. `docs/architecture/09-tooling-sdk-and-observability.md`
24. `docs/architecture/11-security-licensing-and-governance.md`
25. `docs/development/voxtral-emotion2vec-facade-research-2026-08-18.md`
26. `docs/plans/2026-08-18-speech-timeline-service-implementation.md`
27. `docs/plans/2026-08-18-conversation-service-phase-2.md`
28. `docs/plans/2026-08-18-engine-neural-capability-integration-phase-3.md`
29. `docs/plans/2026-08-18-functiongemma-strategic-integration-phase-4.md`
30. `e839a38:lab/scripts/voxtral_microphone.py` and its tests
31. `tools/speech-timeline/README.md` and implementation

## Smallest next action

1. Run one user-spoken turn with the service and microphone client; record capture-to-first-update and subjective ASR/affect sanity without persisting audio by default.
2. Confirm the two unresolved Phase 2 choices and execute its Commit A without merging LLM/TTS implementation into `SpeechTimelineService`.
3. Keep VAD and Voxtral model-slot timing as separately measured increments; do not fabricate word spans meanwhile.
4. After Phase 2 evidence, implement Phase 3A first: clean `reference-alpha`
   relay keeper, semantic presentation-only dialogue open, fake then real
   `ConversationClient`, push-to-talk, subtitles/TTS, three-turn residency and
   fault/resource evidence.
5. Only after 3A closes, execute 3B capability hardening and 3C strategic-model
   audit/shadow integration; do not add world/memory/tool context to 3A.
6. Do not start Phase 4 until 3D freezes consumer-backed
    `NeuralCapabilityCatalog` and `StrategicSemanticCatalog` revisions plus the
    external corpus/evaluation seed.

## Do not retry

- One process per chunk/window or one model load per client.
- Treating 480 ms configured delay as measured end-to-end latency.
- Treating text arrival/commit time as acoustic word time.
- Emitting inline word emotion tags while capability is `none`/`utterance`.
- Recreating the Voxtral wrapper in this branch instead of using the tested `e839a38` source.

## Handoff

- **Workspace state:** `codex/speech-timeline-service` contains the converged tested wrapper and complete standalone Phase 1 service/client/benchmark implementation; external models, profiles, ready files and reports remain outside Git.
- **Checks:** Run the focused Python/lab suites, `git diff --check`, lock consistency and final risk-scoped `host-check` before handoff.
- **Remaining risk:** User-spoken microphone acceptance, broader Russian ASR quality, VAD/model-slot alignment, emotion transition quality, and emotion2vec+ redistribution terms.
- **Promotion needed:** None; no Accepted architecture or roadmap change is authorized by this research.
