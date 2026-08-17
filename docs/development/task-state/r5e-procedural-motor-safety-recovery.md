# R5e procedural motor safety and recovery — completed task state

| Field | Value |
| --- | --- |
| Status | `COMPLETE` |
| Updated | 2026-08-17 |
| Task key | `r5e-procedural-motor-safety-recovery` |
| Scope | Admit one bounded production procedural motor route over the exact R5d player body projection from the shared player/NPC projection set, with deterministic actuator safety and recovery behavior, while capsule Physics remains the sole active transform owner |
| Definition of done | The reference gameplay path consumes exact projection roots, deterministically bounds or rejects unsafe motor output, demonstrates a recovery branch through production inputs, and passes focused plus play/persistence checks without enabling a learned route or another mutable owner |
| Authority | Working context only; Accepted SPEC/ADR, `docs/roadmap.md`, checked-in runtime profiles and exact ProductCheck results outrank this file |

## Resume in 60 seconds

- **Current conclusion:** R5e is complete. One immutable/stateless `CapsuleAnimation` controller is exact-bound to the R5d player projection, action-layout and actuator-safety roots. It admits the existing closed cardinal Q15 command while the capsule is vertically stable and clamps requested horizontal locomotion to zero during Physics-owned fall/landing recovery.
- **Why:** The production live path now consumes the controller before root-motion/direct physical command publication. Invalid projection, input or body state fails closed; save/load during a fall reconstructs the same recovery decision from canonical Physics velocity and resumes held movement after landing.
- **Next action:** Start a separate bounded R5f production base-skinning/render-rig route. Active articulation, general SPEC-27 route state and learned control remain later cuts.
- **Current blocker:** None for R5e.
- **Do not retry:** Learned-policy activation, persisting compiled projections, or directly mutating transforms from Motor; none is authorized by R5e.
- **Reconsider when:** Accepted architecture or an observable production requirement proves that the existing root-motion/capsule command boundary cannot express the bounded route.

## Current evidence

| Evidence | Result | Consequence |
| --- | --- | --- |
| `docs/development/task-state/r5d-body-schema-projection.md` | `PASS`: exact player/NPC projection roots are live and rederived after restore | R5e can consume projections without a new save owner |
| Capsule procedural motor unit suite | `PASS`: exact projection/profile binding, deterministic idle/tracking/recovery and malformed candidate/body rejection | The route is immutable, stateless and fail-closed at its public Motor boundary |
| Production reference fall/recovery fixture | `PASS`: controller clamps horizontal air steering, direct/restored checkpoints remain exact and held movement resumes after stable landing | Recovery is derived from Physics; no new owner or Replay successor is required |
| `play`, `persistence-replay`, `content-package` | `PASS`: existing authoritative roots and 36-root/122-record package closure remain exact | The bounded route does not change the command, physics, content or Replay schemas |
| Conditional interactive-frame performance report | `NOT_RUN (TargetUnavailable)`: the current Linux host lacks the required `desktop-sdl-ash` target; the report-only runtime still completed 900 ticks | No performance claim is promoted; later supported-host R5 evidence remains open |
| Full `host-check` | `PASS` on `x86_64-unknown-linux-gnu`, Rust 1.97.1 | Workspace compilation, strict Clippy, all Rust/doc tests and repository boundary gates are green |

## Verification closure

| Command or gate | Result |
| --- | --- |
| `cargo fmt --all -- --check`; `git diff --check` | `PASS` |
| `cargo clippy -p next_motor -p next_reference_game --all-targets -- -D warnings` | `PASS` |
| `cargo test -p next_motor --all-targets --no-fail-fast` | `PASS`: 63 Motor tests plus example targets |
| `cargo test -p next_reference_game --test physical_animation --no-fail-fast` | `PASS`: production root-motion consumer remains exact |
| `cargo test -p next_reference_game --test collision_visibility --no-fail-fast` | `PASS`: five production fixtures, including recovery save/restore and prior mid-push continuation |
| `cargo run -p xtask -- play` | `PASS`: 32 ticks, 52 domain events, 23 RPG events; state root `0d8f846843a6972e0af92d6237a310552307763a7f640208d6496cbf8138c3b2` |
| `cargo run -p xtask -- persistence-replay` | `PASS`: 20 ticks, two generations; final state root `a40ec7f39ab88b2f5d150b0431e3e4f6373fe8680272e6c25842a264a6389662` |
| `cargo run -p xtask -- content-package` | `PASS`: 122 records, 64 chunks, 36 roots; composition lock `33952d1df2dfa1aaec0a188778d31a465bb1f846f87c16fefba20ab37d1e0af7` |
| `cargo run -p xtask -- boundary-scan` | `PASS`: all six repository boundary checks |
| `cargo run -p xtask -- performance --scenario interactive-frame-soak --mode report` | `NOT_RUN (TargetUnavailable)`: no hard timing claim; report-only live route completed 900 ticks at state root `3df16f0a62f7276e5ee365e5dd39bde9a595b8d4df25a3e3ddbfdde30767d86a` |
| `cargo run -p xtask -- host-check` | `PASS`: canonical JSON result on `x86_64-unknown-linux-gnu`, Rust 1.97.1 |

## Resolved hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1: procedural output can reuse the R5c root-motion command and capsule application path | R5c already validates source/body/revision/capability and delegates transform authority to Physics | None for the bounded capsule route | `RESOLVED / CONFIRMED`: exact forward tracking stays on R5c; other admitted cardinal movement stays on the existing physical command path |
| H2: actuator safety can remain a derived, stateless clamp/reject decision | R5d publishes exact projection/action/safety roots; Physics snapshot already persists the vertical recovery fact | A future learned/full-articulation route still needs complete SPEC-27 state/action publication | `RESOLVED / CONFIRMED` for R5e only: bind the immutable controller to the roots, reject malformed candidates atomically, and derive recovery from current Physics velocity |

## Decisions

### D-001 — Keep R5e at the accepted CapsuleAnimation tier

- **Decision:** Add a stateless controller that produces an admitted cardinal capsule action or a zero recovery action; do not publish joint actuation into the active scene.
- **Reason:** ADR-027 permits a validated capsule controller while full articulation remains disabled at `Prototype`; the roadmap explicitly leaves active articulation cutover after R5e.
- **Consequence:** R5e can prove a real procedural fallback without transferring transform authority or adding a saved owner.

### D-002 — Recovery is derived from committed Physics state

- **Decision:** Any non-zero vertical capsule velocity selects `ProceduralRecovery` and clamps both horizontal command channels to zero. Stable zero vertical velocity resumes the original cardinal command on the next logical tick.
- **Reason:** R5b already stores and replays the exact vertical velocity and landing result. Adding a duplicate motor phase/hysteresis field would create a second future-affecting fact with no present need.
- **Consequence:** Save/load during a fall reconstructs the same recovery decision from the Physics checkpoint; no Replay successor is needed.

### D-003 — Projection roots bind compatibility, not transform ownership

- **Decision:** Controller activation exact-compares the subject projection hash, compiler profile, action-layout hash and actuator-safety root and derives an immutable controller-profile hash from that closure plus the capsule locomotion profile.
- **Reason:** R5d projections are the authoritative compatibility witness for this bounded body generation, but they are reconstructible content-derived evidence rather than mutable state.
- **Consequence:** Tampered/mismatched projection or invalid non-cardinal input fails before command publication; a valid decision still reaches Physics only through the existing command/admission path.

## Required context

Read these sources in precedence order before acting:

1. `docs/architecture/agent-routing.md`, `README.md`, SPEC-00/01 and glossary.
2. Routed SPEC-05/14/21/26/27/28/35 and their Accepted ADRs.
3. `docs/roadmap.md` R5 scope and `docs/development/task-state/r5d-body-schema-projection.md`.
4. Current R5c command, R5d projection, capsule physics and persistence/replay implementation.

## Next action

1. Define R5f as one production base-skinning/render-rig consumer over the existing skeleton/body projection while preserving Physics as the gameplay transform owner.
2. Keep active articulation, non-identity retarget, physical IK and learned/general Motor routes outside R5f unless a distinct accepted consumer admits them.
3. Do not promote full `ANIM-ROOT-MOTION-P1`, `MOTOR-SAFETY/STATE/ROUTE-P1` or full R5 until their declared matrices actually run.

## Do not retry

- Activating learned policy or training manifests — the lineage remains stopped and grants no production authority.
- Letting Motor write authoritative transforms — capsule Physics is the sole active transform owner.
- Treating compiled body projections as mutable/saved state — R5d established them as deterministic derived evidence.

## Handoff

- **Workspace state:** R5e Motor controller, reference live integration, production recovery/restore fixture and current architecture/roadmap checkpoint form one coherent change.
- **Checks:** Focused Motor/reference suites, full Motor targets, strict scoped Clippy, `play`, `persistence-replay`, `content-package` and `boundary-scan` pass. The final workspace `host-check` status is recorded above before commit.
- **Remaining risk:** This is a capsule-only recovery rule. Active articulation, real skinning, general supervisor/PolicyState semantics, cross-target parity and supported-host performance evidence remain explicitly open.
- **Promotion needed:** None. R5e consumes the already Accepted CapsuleAnimation, Physics ownership, projection and current-only compatibility boundaries without adding a public wire format or mutable authority.
