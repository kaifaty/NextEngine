# R5b capsule/world interactions — current task state

| Field | Value |
| --- | --- |
| Status | `ACTIVE / CONTRACT_FREEZE` |
| Updated | 2026-08-16 |
| Task key | `r5b-capsule-world-interactions` |
| Scope | One production grounded-capsule fixture covering a quantized box-profile slope, authored stairs, one pushable dynamic box, non-blocking sensors and deterministic fall-to-support recovery |
| Definition of done | The same canonical physics checkpoint runs through the reference-game player path, save/load and Replay V10; focused checks prove exact traversal, blocking, contact order, sensor continuity, dynamic-body continuation and post-fall locomotion without adding a second pose owner |
| Authority | Working context only; Accepted SPEC/ADR, `docs/roadmap.md`, checked-in schemas and exact ProductCheck results outrank this file |

## Resume in 60 seconds

- **Current conclusion:** R5b can reuse the current physical command, body, contact and `PhysicsCanonicalSnapshotV2` schemas. Dynamic-body pose/velocity and sensor-contact continuity already have canonical storage.
- **Why:** `PhysicsMotionKindV1::Dynamic`, `PhysicsParticipationV1::Sensor`, body state and sorted contact continuation are already public contracts; the missing capability is the bounded grounded-capsule adapter and one production consumer fixture.
- **Next action:** Add deterministic step-up/ground-snap, current-state dynamic-box collision/push and non-blocking sensor overlap to the grounded-capsule world, then instantiate the course in the reference game.
- **Current blocker:** None. Continuous rotated/mesh slopes, arbitrary rigid-body dynamics, ragdoll/get-up animation, root motion, BodySchema and learned control remain outside this cut.
- **Do not retry:** R141, R142, training/optimizer work, vendor/model state as authority, direct animation writes to physics pose or a speculative general character controller.
- **Reconsider when:** The production fixture proves that an exact future-affecting controller fact cannot be reconstructed from the canonical body/contact snapshot.

## Frozen bounded fixture

1. One upright kinematic capsule remains the sole avatar binding and accepts the existing cardinal locomotion intent.
2. Axis-aligned static boxes form two explicit traversal classes: a low-riser quantized slope profile and stairs. A grounded capsule may step or snap by at most `300,000` micrometres per edge; a taller obstacle remains blocking.
3. Exactly bounded axis-aligned `Dynamic` boxes may be pushed only by committed horizontal capsule displacement. The box moves no farther than collision clearance, publishes exact pose/velocity/body revision, and has no undeclared free-drift or mass model in this V1 adapter.
4. Axis-aligned static `Sensor` boxes never clip displacement. They use the existing canonical Begin/Persist/End continuity; participation is resolved from the immutable catalog rather than adding a same-version event field.
5. Fall/recovery means the upright capsule leaves support, integrates declared gravity, lands on a lower solid support with zero vertical velocity and can resume ordinary locomotion. Ragdoll, get-up clips and learned recovery are not claimed.
6. Terrain, dynamic and sensor collision decisions use canonical shape order and checked integer micrometre arithmetic. Registration/backend callback order cannot select an outcome.

## Product decisions

### D-001 — Reuse the canonical physics checkpoint

- **Observation:** Every future-affecting R5b fact is already representable as body pose/velocity/revision or contact continuity.
- **Decision:** Keep the existing command, step, snapshot, checkpoint and Replay V10 schemas. Controller support classification is derived each substep from committed geometry and state.
- **Rejected alternatives:** A parallel character-controller save segment, renderer-owned grounding state or an incompatible same-version field addition.
- **Consequences:** Save/load and replay continue from the same physics owner bytes; reconstruction must reproduce dynamic push, sensors and recovery exactly.

### D-002 — Bound terrain before general geometry

- **Observation:** The current grounded-capsule adapter has exact primitive box features but no activated collision-asset payload for heightfields or triangle meshes.
- **Decision:** R5b uses an authored axis-aligned box profile: low risers prove slope traversal semantics, and larger legal risers prove stairs. The controller's maximum step/snap height is a public fixed profile constant.
- **Rejected alternatives:** Pretending a renderer mesh is authoritative collision, adding a private heightfield side channel or admitting arbitrary rotations without exact feature mapping.
- **Consequences:** Continuous rotated/mesh slopes remain a later general-collision package; R5b reports only the bounded quantized slope fixture it actually runs.

### D-003 — Dynamic push is contact-directed and bounded

- **Observation:** The implemented V1 body descriptor exposes `Dynamic` state but omits accepted mass/inertia fields required for general rigid-body response.
- **Decision:** The capsule adapter may translate an axis-aligned dynamic box along the requested horizontal axis up to exact static/dynamic clearance. It publishes the resulting substep velocity and deterministically clears unsupported free drift.
- **Rejected alternatives:** Invented mass/impulse defaults, animation-driven props or direct gameplay pose mutation.
- **Consequences:** Push is physically owned and replayable, while arbitrary momentum, rotation, stacking and material response remain deferred.

### D-004 — Sensor and recovery remain physical facts

- **Observation:** Sensors require overlap continuity without response; capsule recovery requires no hidden state while the avatar remains upright.
- **Decision:** Sensors join canonical contact candidate ordering but never the solid sweep. Recovery is proven by gravity, landing contact, cleared vertical velocity and later accepted displacement.
- **Rejected alternatives:** Trigger callbacks that mutate gameplay, teleport recovery or an animation-only grounded flag.
- **Consequences:** Consumers can stage later commands from committed facts, and failures preserve the previous checkpoint atomically.

## Sequential implementation order

1. **Active:** freeze constants, supported descriptor subset and negative activation cases.
2. **Pending:** implement step-up/ground-snap and focused slope/stair/tall-blocker tests.
3. **Pending:** implement dynamic-box push plus blocked-push and checkpoint-reconstruction tests.
4. **Pending:** implement non-blocking sensor continuity and fall/landing/resumed-locomotion tests.
5. **Pending:** add the course to the production reference session and prove play/save/load/Replay V10 continuation.
6. **Pending:** run mapped checks, record honest conditional `NOT_RUN` results and close only R5b.

## Explicitly deferred from R5b

- Continuous rotated boxes, convex/mesh/heightfield terrain and a general rigid-body solver.
- Multiple avatar bindings, NPC locomotion commands, moving platforms, carrying, melee, trip/ragdoll and get-up animation.
- Root-motion admission, BodySchema projection, real skinning and expanded animation graphs.
- PhysX-only atomic production cutover, fresh `r5-physics-16.v1` gates, paired Windows/Linux evidence and full R5 completion.
- Learned policy training, export, inference and any resumption of the stopped R141/R142 lineage.

## Handoff

| Evidence | Result | Consequence |
| --- | --- | --- |
| Focused contracts/backend/reference-game tests | `PENDING` | R5b is not yet accepted. |
| `play`, `persistence-replay`, `content-package`, format, Clippy, `boundary-scan`, `host-check` | `PENDING` | Production integration and broad closure remain open. |
| `platform`, `performance`, real-SDK backend parity and paired Windows/Linux execution | `NOT_RUN` | No platform, performance, PhysX-only, Stage 0 or full-R5 claim is made by activation. |
