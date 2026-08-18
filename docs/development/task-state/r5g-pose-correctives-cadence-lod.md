# R5g pose correctives and cadence/LOD isolation — task state

| Field | Value |
| --- | --- |
| Status | `COMPLETE` |
| Updated | 2026-08-18 |
| Task key | `r5g-pose-correctives-cadence-lod` |
| Scope | Extend the current R5f player/NPC surface with a small exact authored pose-corrective set and prove that presentation cadence and deformation LOD alter no authoritative owner or root |
| Definition of done | Exact corrective content cooks and activates atomically; B0 applies deterministic full/reduced correctives with base-skinning fallback; sampled/held/bind and full/reduced/base/cull permutations remain complete and preserve gameplay/save/replay roots; mapped ProductChecks pass |
| Authority | Working context only; Accepted SPEC/ADR, `docs/roadmap.md`, checked-in content and exact ProductCheck results outrank this file |

## Resume in 60 seconds

- **Current conclusion:** R5g is complete as a bounded presentation-only
  successor over R5f. The current-only base-skinning content and V3 character
  subprojection now carry the exact corrective/LOD route without a new pose,
  animation, save or Replay owner.
- **Implementation direction:** author sparse mesh-local deltas driven by one
  named render-joint translation relative to bind pose. Apply them before the
  existing fixed-point LBS in canonical corrective/vertex order.
- **LOD boundary:** `FullCorrectives`, `ReducedCorrectives`,
  `BaseSkinningOnly` and `Culled` select presentation work only. Correctives
  declare essential/detail class; reduced evaluates only essential data.
- **Cadence boundary:** 30/60/144 Hz consumers repeat the newest complete V3
  snapshot. Cadence is not serialized into content, pose records or gameplay
  state and cannot select an animation tick.
- **Fallback:** invalid/unavailable corrective evaluation falls back atomically
  to sampled R5f base skinning; invalid sampled skinning retains the authored
  bind surface. Held pose is a complete prior presentation pose, not hidden
  animation state.
- **Next action:** perform the R5 completion audit before selecting another
  bounded cut. General animation/root-motion corpora, active articulation,
  injury/severity and hard performance evidence remain separate open gates.

## Locked boundary

1. Physics remains the gameplay-transform owner; Physical Animation remains
   the only future-affecting animation owner.
2. Corrective IDs, drivers, LOD classes and sparse vertex deltas are immutable
   content-addressed presentation data with finite explicit bounds.
3. Driver input is derived only from the complete local render-joint pose and
   the exact profile bind transform. Camera, wall time, GPU state and worker
   completion are forbidden inputs.
4. Corrective accumulation uses checked integer arithmetic and one canonical
   order. No floating epsilon, backend blend shape or unordered merge enters
   the public result.
5. The player and reference NPC share one exact profile and fallback chain.
6. `PresentationSnapshotV3` remains the atomic publication; no partial
   corrective palette or vertex stream may be exposed.
7. Injury/severity/topology variants, load-aware deformation, secondary
   tissue, active articulation, learned control and full
   `CHARACTER-EMBODIMENT-P1` remain out of scope.

## Sequential implementation order

1. **Completed:** freeze bounded corrective records, canonical encoding,
   current-only authoring shape and malformed rejection.
2. **Completed:** author the reference locomotion corrective set and close exact
   mesh/skeleton/BodySchema/corrective activation.
3. **Completed:** carry pose source and deformation LOD through the immutable V3
   character record and recovery codec.
4. **Completed:** evaluate correctives before fixed-point LBS, retain sampled-base
   and bind fallbacks, and make cull produce no renderer work.
5. **Completed:** prove full/reduced/base/held/cull and 30/60/144 Hz permutations
   change presentation hashes where expected and zero authoritative roots.
6. **Completed:** run mapped checks, record exact evidence and update normative
   current checkpoint plus roadmap without widening the claim.

## Promotion evidence

| Check | Result | Boundary proved |
| --- | --- | --- |
| Focused Rust suites | `PASS`: contracts `221`, project content pipeline `15`, presentation `46`, render `9`, reference visual presentation `2` | Canonical round trips, malformed content, recovery, exact corrective weights/fallback and production LOD/cadence permutations are covered. |
| Format and scoped strict Clippy | `PASS` | New current-only contracts and all affected consumers are warning-free on all targets. |
| `cargo run --locked -p xtask -- content-package` | `PASS`: `123` records, `64` chunks; content manifest `70cef7998c07a2224b11f7712dab1085cda6dea89d698c37534be6f5e5c70fc7`; composition lock `2c5b466d95ed6e6cb636a7850e984a98d4e444627f6a04b9f2b71be670f689b4` | One Essential/two Detail correctives cook, reopen and activate through the production project path. |
| `cargo run --locked -p xtask -- play` | `PASS`: `32` ticks; state root `4fbc6f43c4e58e46d26f844f16a466cf762dbd41793ab6e1ddf7cd34ebcd9f8b`; ledger hash `e66f0788511d48f75698a4efc3ef0b874d5013ecef1b4ba60c0d6fed1c2c3167` | Live production behavior and authority remain intact. |
| `cargo run --locked -p xtask -- persistence-replay` | `PASS`: `20` ticks/two generations; final state root `277bfac663836c8c1455cb18840b1e450fb0cdced12f0bea6fb7e234f89c0e07`; ledger root `ad6234b7fbbb4d7fde733ac7c448bd2440297fe5aef46b039325b9c44084508b` | The unchanged ten-owner save/replay closure remains exact. |
| `cargo run --locked -p xtask -- visual-smoke --output target/visual-smoke-r5g` | `PASS`: six fixed frames under project lock `2c5b466d95ed6e6cb636a7850e984a98d4e444627f6a04b9f2b71be670f689b4`; representative spawn/combat/pause captures inspected | Displayless presentation remains readable; captures are supporting human evidence, not an authority oracle. |
| `cargo run --locked -p xtask --features desktop-sdl-ash -- platform` | `PASS`: portable contract and SDL/Ash candidate; eight rendered objects; state root `4fbc6f43c4e58e46d26f844f16a466cf762dbd41793ab6e1ddf7cd34ebcd9f8b` | The current Linux hardware renderer consumes the complete project without authority drift. |
| `cargo run --locked --release -p xtask --features desktop-sdl-ash -- performance --scenario r2-alpha-render --mode report --output target/performance-r2-linux-first-20260818` | `PASS / REPORT_ONLY` under ADR-082: R2 v3 completed `3,600` warm-up and `21,600` measured frames across six Linux Vulkan windows, `50,400` timestamp queries and zero deadline misses; complete Linux RSS/I/O/device evidence has no unavailable fields; report SHA-256 `4370267ae4506e3383079d7d8b16c51bfa5b0fb63ccea070647bc9c845de3789` | The current Linux renderer hot path executes instead of returning the former Windows-only `NOT_RUN`. The changeset worktree was dirty, so this remains development evidence and makes no hard timing, baseline, Windows or B-12 claim. |
| Full `cargo run --locked -p xtask -- host-check` | `PASS` on `x86_64-unknown-linux-gnu`, Rust `1.97.1` | Workspace build, strict Clippy, all Rust/doc tests and repository boundary checks are green. |

## Decisions

### D-001 — Pose-space authoring remains small and explicit

- **Decision:** Each corrective names one render-joint translation axis, a
  signed zero/full activation interval, LOD class and sorted sparse per-vertex
  mesh-local deltas.
- **Reason:** The current neutral clips expose exact fixed-point local
  translations, and the reference mesh is small. This is sufficient for a
  real authored consumer without introducing a general blend-shape graph.
- **Consequence:** Rotation-driven, multi-driver, load/injury and neural
  correctives remain future consumer-backed successors.

### D-002 — Presentation LOD is explicit, renderer cadence is not state

- **Decision:** Publish the selected deformation LOD and sampled/held/bind
  source with the complete character record, while renderer cadence only
  repeats complete snapshots.
- **Reason:** LOD changes which presentation work is requested; cadence does
  not create new simulation or animation facts.
- **Consequence:** Snapshot/frame hashes may differ across LOD, while command,
  ledger, physics, save and replay roots must remain exact.

## Explicitly deferred

- Injury-conditioned limp/guarded/fall/crawl/drag corrective breadth and all
  SPEC-36 condition/severity/topology inputs.
- General animation graph/retarget/physical-IK and complete ANIM corpora.
- DQS, load-aware recruitment, cloth/tissue, neural/contact deformation.
- 16/64/distant hard performance budgets and PhysX Stage 0 promotion.
