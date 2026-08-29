# TTS engine integration specification — current task state

| Field | Value |
| --- | --- |
| Status | `COMPLETE` |
| Updated | `2026-08-29` |
| Task key | `tts-engine-integration-spec` |
| Scope | Design a bounded, optional TTS vertical from canonical dialogue text through an isolated `ai-host` and streaming `AudioScene` source to the ordinary spatial mixer/device path, including voice/model-pack, IPC, buffering, replay, security, resource and ProductCheck contracts. |
| Definition of done | A new Proposed SPEC/ADR is indexed and routed; affected audio/dialogue architecture and traceability agree on ownership, exact contracts, fallbacks and promotion evidence; documentation-only validation passes. |
| Authority | Working context only; `AGENTS.md`, Accepted SPEC/ADR and exact model-evaluation evidence outrank this file. |

## Resume in 60 seconds

- **Current conclusion:** TTS must generate dry mono PCM in the optional external `ai-host`, then enter the same engine-owned `AudioScene` and mixer path as authored clips. Direct `ai-host → SDL` playback would bypass spatialization, acoustic context, voice admission and device recovery and is rejected.
- **Status choice:** The design remains `Proposed` under ADR-046 until a production NPC speech consumer, offline fallback, stream-fault matrix, spatial capture and renderer-co-resident resource profile pass. It does not promote all of SPEC-16/ADR-017.
- **First candidate:** VoxCPM2 compiled PyTorch is the current quality candidate, not a shipping default. The measured RTX 3080 result has useful first PCM and acceptable warm RTF, but lacks p95 corpus evidence, stable identity across emotion controls and renderer-co-residency proof.
- **Next action:** Implement T0/T1 from SPEC-47: engine-owned protocol contracts and one supervised mock/real host request with authored fallback, without yet adding propagation breadth.
- **Current blocker:** None.
- **Do not retry:** Do not put model/vendor types in `crates/contracts`, make generated duration authoritative, commit weights/WAV/cache output, infer safe GPU coexistence from standalone VRAM, or treat one mean timing as a p95 gate.

## Decisions that constrain the design

### D-001 — Presentation-owned streaming speech

- **Observation:** The current mixer consumes only cooked clip revisions, while the desktop sink receives already mixed stereo PCM.
- **Evidence:** `AudioEmitterRecordV1` contains `clip_revision`; `AudioMixerV1` resolves only `NeutralAudioV1`; `DesktopAudioOutputV1` queues final stereo S16 windows.
- **Decision:** Add a future current-only successor source union for cooked clips and bounded speech streams. PCM enters before attenuation/panning/directivity/occlusion/reverb and never bypasses the mixer.
- **Consequence:** `AudioScene` and presentation own stream/playback state; simulation, dialogue and replay hashes do not.

### D-002 — External resident host with bounded protocol

- **Observation:** ADR-005 requires TTS in optional `ai-host`; model load dominates cold latency and standalone model VRAM is material on a 10 GiB GPU.
- **Evidence:** VoxCPM2 compiled PyTorch warm RTF `0.707`, streaming mean first PCM `56.9 ms`, observed peak GPU up to `8,455 MiB`; cold/compiled initialization is much slower.
- **Decision:** Use one resident, prewarmed local host, one active request plus at most one prefetched sentence for the initial profile, exact pack/runtime hashes and bounded length-prefixed IPC. Readiness is published only after validation, load, warm-up and joint resource admission.
- **Consequence:** Host/model faults select authored voice or subtitle fallback without blocking a simulation tick.

### D-003 — Model quality and engine integration are separate gates

- **Observation:** Nano-vLLM reaches RTF `0.249` but has worse intonation; PyTorch has better intonation but does not meet the existing SPEC-16 RTF `≤0.5` target.
- **Decision:** Preserve `MODEL-TTS-P1` as the quality/performance gate and add separate protocol/stream/spatial/offline/resource checks. No runtime is promoted from speed alone.
- **Consequence:** The engine vertical can be implemented and evaluated without declaring VoxCPM2 a shipped default.

### D-004 — Preserve mainline architecture identities during integration

- **Observation:** Mainline assigned `SPEC-36` and `ADR-075` to functional anatomy while the isolated TTS branch still used those IDs.
- **Decision:** Integrate the unchanged bounded TTS proposal as `SPEC-47` and `ADR-099`, and update every index, route, traceability and cross-document link in the same squash.
- **Consequence:** Functional-anatomy authority remains intact and the TTS proposal has unique current identities; do not restore the old TTS IDs.

## Required context

1. `AGENTS.md`, `docs/architecture/agent-routing.md`, architecture baseline/product/system/glossary.
2. SPEC-02/08/09/11/16/17/30 and ADR-002/005/017/028/046.
3. `docs/development/task-state/voxcpm2-tts-demo.md` and the Fish/CosyVoice model reports.
4. Current `AudioSceneSnapshotV1`, `AudioMixerV1` and desktop SDL audio sink implementation.

## Remaining uncertainty

- Exact p95 TTFA/RTF and voice-quality corpus results for the selected profile.
- Stable authorized character voice under controlled cloning and emotion variation.
- Renderer + TTS concurrent GPU budget and device-loss/host-restart behavior.
- Whether the engine-native zone/reverb baseline is sufficient before an optional propagation adapter is evaluated.

## Handoff

- **Normative result:** Proposed SPEC-47 and ADR-099 define the bounded vertical; SPEC-08, SPEC-16, SPEC-30, ADR-017, index, routing, glossary and traceability are synchronized without changing current shipped status.
- **Checks:** Five focused Python test files pass 28 `unittest` cases; all added Python files compile; `git diff --check` passes; every relative link in all changed documents resolves; 147 scanned SPEC/ADR IDs are unique. `pytest` is unavailable in the host Python, so the equivalent stdlib runner was used. Cargo, `host-check`, `play`, `persistence-replay`, `content-package`, `platform` and `performance` are `NotRun(NoEngineRuntimeChange)`.
- **Remaining risk:** All TTS ProductChecks remain `NOT_RUN`; no production stream/source contract, host supervisor, non-locking callback, room response or renderer-co-resident profile exists yet.
- **Smallest next action:** T0 engine-owned canonical protocol/negative vectors, followed by T1 one resident host request/cancel/crash path and a subtitle fallback in the production application coordinator.
