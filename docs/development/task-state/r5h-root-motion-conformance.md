# R5h root-motion conformance — task state

| Field | Value |
| --- | --- |
| Status | `COMPLETE` |
| Updated | 2026-08-18 |
| Task key | `r5h-root-motion-conformance` |
| Scope | Close the existing `ANIM-ROOT-MOTION-P1` gate with one fixed 10,000-cycle production-path matrix over the bounded forward R5c/R5e capsule route |
| Definition of done | A Linux-runnable ProductCheck executes exactly 4,000 accepted, 2,000 rejected, 1,000 exact retry, 1,000 checkpoint/restore and 2,000 animation-LOD-isolation cycles through canonical proposals, procedural motor/safety, Runtime receipts, Physics outcomes and exact replay; faults publish no partial body mutation |
| Authority | Working context only; Accepted SPEC/ADR, `docs/roadmap.md`, checked-in contracts and exact ProductCheck results outrank this file |

## Resume in 60 seconds

- **Current conclusion:** R5h is complete. The fixed
  `ANIM-ROOT-MOTION-P1` matrix passes twice with identical report roots and
  transcript digest over the production R5c/R5e route. The cut adds
  verification and tooling only; it does not add an owner, command,
  save/replay schema or wider root-motion vocabulary.
- **Fixed matrix:** each ten-cycle block is four ordinary accepted proposals,
  one rotating canonical-valid proposal fault, one exact retry of that
  rejected command, one second rotating fault, one checkpoint/restore
  continuation and two LOD-isolation continuations. Exactly 1,000 blocks run.
- **Authority boundary:** `PhysicalAnimationOwnerV1` proposes; the project-
  bound `CapsuleProceduralMotorControllerV1` supplies safety evidence; Runtime
  validates and receipts; Physics remains the only body-pose writer.
- **LOD boundary:** R5h verifies only that full/fallback animation projection
  availability cannot affect a root-motion command or authoritative outcome.
  It does not close the separate general `ANIM-LOD-P1` corpus.
- **Host boundary:** the ProductCheck runs on the active Linux host. Windows,
  THOTH and Stage 0 remain deferred under ADR-082 and are not R5h blockers.
- **Next action:** perform a new R5 completion audit before selecting the next
  bounded cut. Injury/severity, general graph/retarget/physical IK, general
  animation LOD and active articulation remain separate open gates.

## Locked boundary

1. The matrix uses the activated neutral project, production animation owner,
   project-backed procedural controller, canonical `WorldCommand`, Runtime
   admission, reference Physics checkpoint/restore and `RuntimeReplayDriver`.
2. Verification reads immutable public snapshots and receipts. No test-only
   mutable Runtime, Physics, ECS or renderer access is added.
3. Every submitted command and embedded proposal round-trips through canonical
   bytes before execution.
4. Rejected and exact-retry cycles preserve the complete controlled body state
   and admit no physical intent. Fresh accepted cycles can be fully applied,
   collision-clipped or blocked only by Physics.
5. Checkpoint/restore preserves Runtime and physical-animation snapshots before
   the next proposal. Replay reproduces every one of the 10,000 reports exactly.
6. Fault selection and retry cadence are logical and fixed. Functional or
   deterministic failures are never retried to green.
7. Lateral/yaw roots, general graph/retarget/physical IK, active articulation,
   learned control, injury/severity and general animation LOD remain out of
   scope.

## Completion audit

| Candidate | Audit result | Decision |
| --- | --- | --- |
| Full `ANIM-ROOT-MOTION-P1` | Mandatory R5/v1 gate; R5c/R5e production consumer and save/replay path already exist | `SELECTED AS R5H` |
| Injury/severity deformation | Accepted product direction, but exact condition schemas and vertical remain Proposed/post-baseline | `DEFER` |
| Active articulation cutover | Depends on the deferred PhysX Stage 0 readiness/evidence boundary | `DEFER` |
| General graph/non-identity retarget/physical IK | Multiple broad contracts and consumers remain absent | `DEFER` |
| General `ANIM-LOD-P1` | Wider scheduling/residency corpus than the R5g/R5h authority-isolation probes | `DEFER` |

## Verification closure

| Check | Result | Boundary proved |
| --- | --- | --- |
| `cargo run --locked --release -p xtask -- animation-root-motion` (two independent runs) | `PASS` twice: `10,000` cycles; `4,000` ordinary accepted, `2,000` rejected, `1,000` retry, `1,000` save/load and `2,000` LOD; `9,000` motor/safety decisions; `10,000` exact replays; full/clipped/blocked outcomes `2,000/1,000/4,000`; fault/retry no-mutation `3,000`; matrix digest `ae4dddfe019234859a49221f3ac00784c84f4fc0ec0d0332c0642e6c4899c34a` | The full bounded-forward `ANIM-ROOT-MOTION-P1` matrix is deterministic and complete. |
| Root-motion focused Rust tests | `PASS`: `2` passed, including the exact partition and 30-cycle production smoke across all six fault kinds | The fast workspace path covers every category and rejection boundary without embedding the full milestone workload. |
| Format and scoped strict Clippy | `PASS` | Verification and tooling additions are formatted and warning-free on all targets. |
| `cargo run --locked -p xtask -- play` | `PASS`: `32` ticks; state root `4fbc6f43c4e58e46d26f844f16a466cf762dbd41793ab6e1ddf7cd34ebcd9f8b`; ledger hash `e66f0788511d48f75698a4efc3ef0b874d5013ecef1b4ba60c0d6fed1c2c3167` | Live production behavior and command authority remain intact. |
| `cargo run --locked -p xtask -- persistence-replay` | `PASS`: `20` ticks/two generations; final state root `277bfac663836c8c1455cb18840b1e450fb0cdced12f0bea6fb7e234f89c0e07`; ledger root `ad6234b7fbbb4d7fde733ac7c448bd2440297fe5aef46b039325b9c44084508b` | The unchanged ten-owner save/replay closure remains exact. |
| `cargo run --locked -p xtask -- content-package` | `PASS`: `123` records, `64` chunks; content manifest `70cef7998c07a2224b11f7712dab1085cda6dea89d698c37534be6f5e5c70fc7`; composition lock `2c5b466d95ed6e6cb636a7850e984a98d4e444627f6a04b9f2b71be670f689b4` | The verification-only cut leaves the activated project and content closure unchanged. |
| `cargo test --locked -p xtask --no-fail-fast` | `PASS`: `104` library tests, `44` command tests and doc-tests | The new strict report/command surface does not regress tooling. |
| Full `cargo run --locked -p xtask -- host-check` | `PASS` on `x86_64-unknown-linux-gnu`, Rust `1.97.1` | Workspace build, strict Clippy, all Rust/doc tests and repository boundary checks are green. |

Final Physics checkpoint hash is
`9073bf88eb493a7eaa5fcb5c2ae013da29cbc5426b753278bb5c6347a204c193`;
final command-ledger hash is
`56beb6187bc34bbbdb9225381dfcff33dceca3456398ea8644395b1e29a451a7`.
Linux renderer/performance was not rerun because R5h changes neither the
presentation consumer nor its hot path. Windows/THOTH is intentionally
`NOT_RUN (WindowsHostDeferred)` under ADR-082.

## Decisions

### D-001 — Fixed generations keep history length out of the work input

- **Decision:** Execute 1,000 independent ten-cycle Runtime/animation/replay
  generations, while binding every per-cycle result into one canonical matrix
  transcript.
- **Reason:** The gate is about proposal, admission, Physics, persistence and
  replay boundaries. A monotonically growing command archive would make prior
  ledger length an accidental workload input rather than add coverage.
- **Consequence:** Every block has the same bounded history shape; the final
  digest still binds all 10,000 ordered observations and rotating faults.

### D-002 — Collision coverage is a fully validated fixture property

- **Decision:** Move the neutral capsule to the fixed clipping start before
  Runtime activation, then rebuild and validate the catalog, genesis Physics
  snapshot and checkpoint through public constructors.
- **Reason:** This yields full, partially clipped and blocked outcomes through
  the real reference Physics path without mutable test access or teleporting a
  live body.
- **Consequence:** The matrix proves all three lawful Physics outcomes while
  production ownership and checkpoint validation remain unchanged.

## Do not retry

- R141/R142 learned lineage remains `INVALID / STOP_NO_RETRY`.
- Windows/THOTH execution remains deferred until explicit pre-R7 bring-up.
- Do not widen R5h into new animation, motor, physics or persistence schemas
  unless the fixed existing path proves incapable of expressing the gate.
