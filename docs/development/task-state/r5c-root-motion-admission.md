# R5c root-motion admission — current task state

| Field | Value |
| --- | --- |
| Status | `COMPLETE` |
| Updated | 2026-08-17 |
| Task key | `r5c-root-motion-admission` |
| Scope | One bounded production path in which sampled animation root displacement becomes a revision-bound intent that capsule physics accepts, clips or rejects while remaining the sole transform owner |
| Definition of done | The reference player consumes root displacement through production animation and physics paths; focused vectors prove accepted, clipped and rejected motion plus exact save/load/Replay continuation without claiming BodySchema, skinning, learned control or full R5 completion |
| Authority | Working context only; Accepted SPEC/ADR, `docs/roadmap.md`, checked-in schemas and exact ProductCheck results outrank this file |

## Resume in 60 seconds

- **Current conclusion:** R5c is complete. A standalone canonical `RootMotionIntentV1` command validates the exact registered animation profile/clip source plus tick/body/profile closure, then lowers into the existing `AcceptedLocomotionIntentV2`; capsule physics remains the sole transform writer.
- **Why:** Production forward input now consumes an authored exact root curve; focused vectors prove accepted, collision-clipped, stale and unregistered-source rejection, while save/load regenerates the same proposal and Replay V10 records the full command without a schema successor.
- **Next action:** Start a separate bounded R5d package for consumer-backed `BodySchema` projection. Real skinning, general graph/yaw root motion and full `ANIM-ROOT-MOTION-P1` remain later work.
- **Current blocker:** None for R5c.
- **Do not retry:** R141/R142, training or optimizer work, animation-owned teleports, vendor/model state as authority, or a speculative generic graph/controller framework.
- **Reconsider when:** Exact existing contracts cannot represent admission outcome or future-affecting continuation without one minimal current-only successor.

## Current evidence

| Evidence | Result | Consequence |
| --- | --- | --- |
| `docs/roadmap.md` R5; R5a/R5b task states | `PASS` planning prerequisite | Root-motion admission owns the next WIP slot; BodySchema, skinning and learned routes remain later cuts. |
| `git status --short --branch` before R5c | `PASS` clean workspace | New R5c work can be isolated from user changes. |
| Canonical contract/motor/runtime focused vectors | `PASS` | The proposal round-trips, samples exact fixed-tick deltas without owner mutation, and commits only through the existing physical step. |
| Production cooker/reference live consumer | `PASS` | Authoring v2 cooks the 31-key forward curve; normalized `W` input enters Runtime as `RootMotionIntentV1` and moves only through the capsule. |
| Exact continuation and Replay V10 | `PASS` | Direct/restored owners regenerate equal proposals, the recorded command is replayed, and final physics/owner/ledger roots converge without a new owner or format. |

## Verification closure

| Command or gate | Result | Evidence |
| --- | --- | --- |
| `cargo test -p next_contracts physical_animation` | `PASS` | Canonical root-motion round-trip and stable yaw/profile rejection vectors pass. |
| `cargo test -p next_motor physical_animation` | `PASS` | Exact fixed-tick sampling and non-mutating proposal generation pass. |
| `cargo test -p next_runtime root_motion` | `PASS` | Open-space acceptance, collision clipping, both legal fractional cadence quanta, stale-body, out-of-cadence interval and unregistered-source rejection pass without animation pose authority. |
| `cargo test -p next_project --test content_pipeline` | `PASS` | Authoring v2 cooks the exact 31-key locomotion root curve and `core_r5c` project lock. |
| `cargo test -p next_reference_game --test physical_animation` | `PASS` | Normalized forward input emits `RootMotionIntentV1` and moves the player only through Runtime/Physics. |
| `cargo run -p xtask -- play` | `PASS` | 32 ticks, 52 domain events and 23 RPG events converge; authoritative state root is `06c1d82156895fb3a03c8fd70e144e91fb718d2f11d6b6c8fea797c200a65109`. |
| `cargo run -p xtask -- content-package` | `PASS` | 121 records and 64 chunks activate under composition lock `a50c96a18edda07401772d1e75bed1a2d3e61171009536bd24e0e482ca1c50e1`. |
| `cargo run -p xtask -- physics-collision` | `PASS` | Six gameplay ticks and twelve substeps converge at `[0, 900000, 200000]`. |
| `cargo run -p xtask -- persistence-replay` | `PASS` | Direct/restored/Replay V10 execution converges after 20 ticks at state root `1332ce773b2ea566fe2709aedb15d73bc0f0de473ea8813dad16b25efc2e49f3`. |
| `cargo run -p xtask -- host-check` | `PASS` | Rust 1.97.1, format, workspace clippy with warnings denied, all workspace/doc tests and boundary scan pass. |
| Full `ANIM-ROOT-MOTION-P1` 10 000-cycle matrix | `NOT_RUN / OPEN` | R5c proves the bounded production path only; the roadmap gate is not promoted. |
| Broad performance/platform matrix | `NOT_RUN / OPEN` | R5c changes no physics backend, platform surface or declared performance baseline; the later R5/Stage 0 obligations remain open. |

## Decisions that still constrain the work

### D-001 — Preserve the capsule as the sole transform writer

- **Observation:** Accepted architecture defines animation root motion as a proposal and assigns collision transform ownership to capsule physics.
- **Evidence:** `docs/architecture/adr/027-physics-motor-and-animation-layering.md` and `docs/roadmap.md` R5.
- **Decision:** Every sampled root displacement must cross a typed, revision-bound admission seam and be committed only as a physics result.
- **Rejected alternatives:** Direct animation pose writes, post-teleport collision correction, renderer-time motion, or a second saved transform owner.
- **Consequences:** Admission tests must distinguish requested displacement from committed displacement and prove clipping/rejection without animation-side correction.
- **Uncertainty:** None for the bounded R5c path; the inventory confirmed that an additive command kind fits without a physics/replay successor.
- **Reconsider when:** A newer Accepted ADR explicitly changes pose ownership.

## Resolved hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1: Existing animation owner snapshot can carry R5c without a replay schema successor | `phase_ticks`, profile/clip revisions and next tick reproduce the next exact sample; Replay V10 stores the complete command plus animation compare segments | None in the bounded consumer | `RESOLVED / CONFIRMED` by direct/restored proposal equality and Replay V10 convergence. |
| H2: One minimal current-only intent contract is required | Glossary/SPEC-28 name `RootMotionIntentV1`; Runtime and the reference game are immediate consumers | Reusing the old direction-only command would lose source/revision evidence | `RESOLVED / CONFIRMED`: additive command kind; physical step/result schemas remain unchanged. |

### D-002 — Add a command kind, not another physics or replay owner

- **Observation:** Existing `PhysicalCommandV1` and `PhysicsStepInputV2` already provide the validated axial capsule path and exact applied displacement, while Replay V10 records direct command bodies and `PhysicalAnimationSnapshotV1` compare segments.
- **Evidence:** Current command archive/replay code, R5a persistence integration and R5b locomotion clipping tests.
- **Decision:** Encode `RootMotionIntentV1` as its own ingress command schema. Runtime validates target/tick/intent/body/profile closure and lowers a valid bounded forward sample into `AcceptedLocomotionIntentV2`; physics and applied-result schemas remain byte-compatible.
- **Rejected alternatives:** Widening the closed physical-command V1 union in place, inventing a second transform/result owner, or adding a replay successor with no new future state.
- **Consequences:** The current determinism bundle gains one command-kind registry entry, while save/replay owner arity and the physical-animation snapshot remain unchanged.
- **Uncertainty:** None for the bounded forward reference profile; lateral/yaw/general graph root motion remains later scope.
- **Reconsider when:** A later consumer requires non-axial displacement, authored yaw or a distinct motor profile.

### D-003 — Reconstruct source admission from the project lock

- **Observation:** Non-zero graph/clip hashes alone do not prove that a proposal came from the active animation source, while persisting another mutable profile owner would force unnecessary save/replay schema work.
- **Evidence:** Capability authority is already reconstructed outside mutable world state on bootstrap/restore, and the exact project lock pins the profile inputs and locomotion clip revision.
- **Decision:** Bind `(issuer, subject)` to the exact active profile and clip hashes in `AuthorityRegistry`; root admission requires that pair in addition to the ordinary physical capability and body/tick checks.
- **Rejected alternatives:** Trusting arbitrary non-zero source hashes, storing source admission in Physics/RPG state, or advancing Replay solely to carry immutable bootstrap authority.
- **Consequences:** Tampered/unregistered source revisions reject with `ANIM_ROOT_MOTION_REJECTED` and zero pose/event mutation; save/replay owner arity remains ten.
- **Uncertainty:** None for the single current reference profile.
- **Reconsider when:** Multiple simultaneously active animation profiles for one principal/subject require a bounded project-authored selection rule.

### D-004 — Validate the fixed-rate quantum without inventing clip-phase parity

- **Observation:** Locomotion graph phase can begin at any world tick. At 30 Hz its exact sampled interval is therefore one of `33_333` or `33_334` microseconds, but `RootMotionIntentV1` does not carry a separate graph `phase_ticks` field from which Runtime could derive which residue applies.
- **Evidence:** The focused cadence vectors accept both legal quanta and reject an out-of-cadence value; the long-held production movement test crosses graph-state and world-tick offsets while retaining every 100 000 µm capsule step.
- **Decision:** Runtime validates `interval_us` against the bounded floor/ceil cadence set, while source tick/revision checks independently prove freshness and Replay records the exact sampled value.
- **Rejected alternatives:** Deriving clip phase from world-tick parity, relabelling a phase-sampled delta with a different interval, or widening V1 solely to encode redundant cursor state.
- **Consequences:** A valid phase-offset root proposal is not spuriously dropped; values outside the declared cadence still reject with zero physical mutation.
- **Uncertainty:** None for the bounded 20/30/60 Hz capsule profiles.
- **Reconsider when:** A future non-uniform/general graph consumer needs Runtime to validate the exact curve interval from an explicit cursor.

## Required context

Read these sources in precedence order before acting:

1. `AGENTS.md`, `docs/architecture/agent-routing.md`, architecture README, SPEC-00, SPEC-01 and glossary.
2. Routed physics/animation/replay SPECs and ADRs, especially SPEC-05/26/28 and ADR-027/046.
3. `docs/roadmap.md` R5 plus completed R5a/R5b task states.
4. Current contracts, `next_motor`, reference physics, reference-game and application/replay consumers.

## Next action

1. Freeze R5d as a separate consumer-backed `BodySchema` projection package before editing shared physical descriptors.
2. Keep R5c forward-only; lateral/yaw/general graph root motion requires a new consumer and focused admission design.
3. Do not promote the full `ANIM-ROOT-MOTION-P1` until its declared 10 000-cycle accepted/rejected/retried/save/replay/LOD matrix actually runs.

## Do not retry

- Animation-authoritative displacement — rejected by ADR-027; reconsider only after an explicit superseding Accepted ADR.
- R141/R142 learned lineage — `INVALID / STOP_NO_RETRY`; reconsider only with new downstream authority recorded in the roadmap.

## Handoff

- **Workspace state:** R5c implementation, current architecture checkpoint and roadmap are complete in one coherent change.
- **Checks:** Focused contract, motor, runtime, cooker/live and exact persistence/replay checks pass; final workspace commands are recorded in `Verification closure`.
- **Remaining risk:** Full 10 000-cycle `ANIM-ROOT-MOTION-P1`, non-forward/yaw roots, BodySchema, real skinning and broad R5 performance/platform evidence remain explicitly open.
- **Promotion needed:** None. The implementation consumes the already Accepted SPEC-28/ADR-027 authority split and ADR-046 current-only format policy.
