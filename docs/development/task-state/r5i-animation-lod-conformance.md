# R5i animation LOD conformance — task state

| Field | Value |
| --- | --- |
| Status | `COMPLETE` |
| Updated | 2026-08-18 |
| Task key | `r5i-animation-lod-conformance` |
| Scope | Close the current bounded R5 `ANIM-LOD-P1` route over the existing player/NPC physical-animation and R5g presentation consumer |
| Definition of done | A production LOD planner and Linux-runnable ProductCheck execute exactly 10,000 LOD/cadence/resource/publication transitions; every due root-intent probe runs; every publication is complete or absent; render variation changes zero authoritative roots |
| Authority | Working context only; Accepted SPEC/ADR, `docs/roadmap.md`, checked-in contracts and exact ProductCheck results outrank this file |

## Resume in 60 seconds

- **Current conclusion:** R5i is complete. The fixed `ANIM-LOD-P1` matrix
  passes twice with identical report roots and transcript digest through the
  production physical-animation, presentation-extraction and B0 frame-planning
  path. R5g's deformation selector remains separate because its `Culled`
  record deliberately carries a complete local-joint pose.
- **Closed cut:** the five currently Accepted evaluation levels
  (`FullPose`, `ReducedPose`, `HeldPresentationPose`, `IntentOnly`,
  `CulledPresentation`) run for the bounded R5 humanoid owner, including fixed
  reduced cadence and bounded held/bind/cull resource fallback.
- **Schema boundary:** this cut adds reconstructible runtime selection and
  projection types, not a new durable animation graph, content, save or Replay
  schema. The exact R5 profile is fixed and revision-bound in code; a general
  creator-authored LOD profile remains a later consumer-backed successor.
- **Production consumer:** the reference-game character-skinning assembly uses
  the planner for its normal `FullPose` path. ProductCheck drives every
  other level through the same physical-animation owner and atomic
  `PresentationSnapshotV3` publication path.
- **Host boundary:** Linux is the active development host under ADR-082.
  Windows, THOTH and PhysX Stage 0 are intentionally deferred and do not block
  this cut.
- **Next action:** perform a new explicit R5 completion audit before selecting
  another bounded cut. General graphs, non-identity retargeting, physical IK,
  creator-authored LOD profiles and active articulation remain separate gates.

## Locked boundary

1. `PhysicalAnimationOwnerV1` remains the only future-affecting animation
   owner; Physics remains the only committed body-pose writer.
2. LOD selection is reconstructible presentation work. It may read the
   committed physical-animation/Physics snapshots but cannot mutate either.
3. `ReducedPose` samples at the fixed profile cadence without presentation IK;
   off-cadence ticks reuse only a bounded compatible pose, then bind or cull.
4. `HeldPresentationPose` refreshes the presentation root from current Physics
   while retaining a bounded prior complete local pose. It never becomes an
   animation cursor or save owner.
5. `IntentOnly` and `CulledPresentation` publish no pose. The due root-intent
   probe still executes before every LOD decision in the conformance matrix.
6. Missing optional clip/IK/presentation resources select exact held, bind or
   cull fallback. Missing/invalid publication retains the prior complete
   snapshot and consumes no sequence.
7. Renderer cadence and target/cache revision are downstream variations only;
   30/60/144 Hz repeat the newest complete snapshot.
8. General layered graphs, non-identity retargeting, physical IK, active
   articulation, injury/severity and learned control remain out of scope.

## Fixed 10,000-cycle matrix

Each of 1,000 isolated ten-cycle generations executes this exact order:

| Offset | Requested work | Expected publication |
| --- | --- | --- |
| 0 | `FullPose`, all resources | sampled complete pose |
| 1 | `ReducedPose`, off cadence | bounded held complete pose |
| 2 | `ReducedPose`, due cadence | sampled complete pose without presentation IK |
| 3 | `HeldPresentationPose` | bounded held complete pose |
| 4 | `IntentOnly` | complete snapshot with no character pose |
| 5 | `CulledPresentation` | complete snapshot with no character pose |
| 6 | `FullPose`, clip unavailable | bounded held resource fallback |
| 7 | `ReducedPose`, clip unavailable and no held input | bind-pose fallback |
| 8 | `FullPose`, clip/bind unavailable and no held input | cull/no-pose fallback |
| 9 | valid full projection plus invalid scene/skinning closure | reject candidate and retain prior exact snapshot |

Cadence rotates through 30/60/144 Hz and render target revision rotates through
a fixed set. Every cycle samples the same due root intent before and after
presentation work, compares Runtime, ledger, Physics and physical-animation
snapshots exactly, and appends all immutable observations to one matrix digest.

## Completion audit

| Candidate | Audit result | Decision |
| --- | --- | --- |
| Current bounded `ANIM-LOD-P1` | At selection time this was mandatory R5/v1 behavior with physical-animation and R5g consumers, but no production animation-work selector or full transition corpus | `SELECTED AS R5I` |
| General graph/non-identity retarget/physical IK | Requires broader descriptors and production consumers absent from the current bounded owner | `DEFER` |
| Active articulation cutover | Depends on deferred PhysX Stage 0 readiness/evidence | `DEFER` |
| Injury/severity deformation | Accepted direction but post-baseline and not required by the current R5 surface exit | `DEFER` |

## Verification closure

| Check | Result | Boundary proved |
| --- | --- | --- |
| `cargo run --locked --release -p xtask -- animation-lod` (two independent runs) | `PASS` twice: `10,000` cycles; requests Full/Reduced/Held/IntentOnly/Culled `4,000/3,000/1,000/1,000/1,000`; projections sampled/held/bind/no-pose `3,000/3,000/1,000/3,000`; publications complete/rejected-retained `9,000/1,000`; due intents and authority-isolation checks `10,000/10,000`; resource fallbacks `3,000`; B0 frame plans `26,665`; matrix digest `4ada362926fefcd636cf1b86f69863c82895dc9bf4c6f80c3b5a91c232aba59b` | The exact bounded-profile `ANIM-LOD-P1` matrix is deterministic and complete. |
| Motor, reference presentation and LOD-conformance focused Rust tests | `PASS`: `7 + 2 + 2` tests | Profile bounds, sample/hold/bind/no-pose selection, stale/cross-world hold rejection, production composition, exact partition and reproducible smoke are covered. |
| Format and scoped strict Clippy | `PASS` | Motor, reference-game, verification and tooling additions are formatted and warning-free on all targets. |
| `cargo run --locked -p xtask -- play` | `PASS`: `32` ticks; state root `4fbc6f43c4e58e46d26f844f16a466cf762dbd41793ab6e1ddf7cd34ebcd9f8b`; ledger hash `e66f0788511d48f75698a4efc3ef0b874d5013ecef1b4ba60c0d6fed1c2c3167` | Live production behavior and command authority remain intact. |
| `cargo run --locked -p xtask -- persistence-replay` | `PASS`: `20` ticks/two generations; final state root `277bfac663836c8c1455cb18840b1e450fb0cdced12f0bea6fb7e234f89c0e07`; ledger root `ad6234b7fbbb4d7fde733ac7c448bd2440297fe5aef46b039325b9c44084508b` | The unchanged ten-owner save/replay closure remains exact. |
| `cargo run --locked -p xtask -- content-package` | `PASS`: `123` records, `64` chunks; content manifest `70cef7998c07a2224b11f7712dab1085cda6dea89d698c37534be6f5e5c70fc7`; composition lock `2c5b466d95ed6e6cb636a7850e984a98d4e444627f6a04b9f2b71be670f689b4` | Runtime-only LOD selection leaves the durable project/content closure unchanged. |
| Full `cargo run --locked -p xtask -- host-check` | `PASS` on `x86_64-unknown-linux-gnu`, Rust `1.97.1` | Workspace build, strict Clippy, all Rust/doc tests and repository boundary checks are green. |

The fixed LOD profile revision is
`921cbc0a07c98ee0da9f8164243ee824edc8b50cc9316162debf3fca9360abaf`.
Final roots are:

- Runtime state: `fc28729da6d4a9b2b75d91c49b233e01abc2bc91f97bd6e877b2e36a2428046a`;
- command ledger: `7a1bce6e7da7c9875eac0a7b4972bf0616085cffd8fa6ad793eb1187eb259928`;
- Physics checkpoint: `5b6ff6a6c56d6d508bb25e2a14b99057bee5e41f2c0d87d3ffe7040fd571c4a8`;
- physical-animation snapshot: `f23fc300e70df27483de843c0178a081e75ed951000dac07256020aa81d96191`;
- presentation snapshot: `1185ada8fff0748db6d0cb5eeec83378100d45abb1bad7322e080c6d81d1d93e`;
- B0 frame plan: `f684731ae50b94a0ee1e3fa80f699904fc1c48a9c3033c6921f78867ed8a00bb`.

Linux platform/render and Performance V5 were not rerun because R5i changes
neither a renderer/backend nor an established measured hot path; its B0
composition boundary is exercised by the exact 26,665-frame matrix. Optional
capture was not requested and is not an acceptance prerequisite. Windows and
THOTH are intentionally `NOT_RUN (WindowsHostDeferred)` under ADR-082.

## Decisions

### D-001 — The current LOD profile is reconstructible runtime policy

- **Decision:** Freeze the bounded R5 cadence and maximum held age in
  `PhysicalAnimationLodProfileV1`; do not add a durable creator-facing content,
  save or Replay schema.
- **Reason:** One production humanoid consumer proves the exact scheduling
  behavior but does not yet justify a general authoring vocabulary or format.
- **Consequence:** The profile is revision-bound and testable now; multiple
  graph/layer profiles remain a consumer-backed successor rather than an
  unused compatibility obligation.

### D-002 — Held projections bind exact Physics identity and refresh roots

- **Decision:** Reuse a held local pose only when subject, owner profile, LOD
  profile, logical age, Physics world/catalog and mapped joints match exactly;
  rebuild its presentation roots from the current committed body pose.
- **Reason:** A held presentation pose may save animation work, but it cannot
  freeze movement, cross a Physics generation or become an authority owner.
- **Consequence:** Incompatible or stale input follows the declared
  held-to-bind-to-cull fallback without mutating animation or Physics state.

### D-003 — No-pose publication is omission, and faults retain atomically

- **Decision:** `IntentOnly`, requested cull and exhausted fallback omit both
  the character scene binding and skinning record. Invalid scene/skinning
  closure rejects the candidate and retains the prior complete snapshot and
  sequence exactly.
- **Reason:** Publishing half of a skinned character would make renderer
  behavior order-dependent and could expose a partial LOD transition.
- **Consequence:** Every accepted candidate is complete, while every rejected
  candidate is observationally identical to the prior presentation snapshot.

## Do not retry

- Do not reuse R5g `CharacterDeformationLodV1::Culled` as proof that animation
  pose evaluation was skipped; that record deliberately remains complete.
- Do not add a durable creator-facing LOD/content/save schema without a real
  authoring and activation consumer.
- Do not schedule Windows/THOTH or PhysX Stage 0 work during this Linux cut.
