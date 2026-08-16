# R5b capsule/world interactions — current task state

| Field | Value |
| --- | --- |
| Status | `COMPLETE` |
| Updated | 2026-08-17 |
| Task key | `r5b-capsule-world-interactions` |
| Scope | One production grounded-capsule fixture covering a quantized box-profile slope, authored stairs, one pushable dynamic box, non-blocking sensors and deterministic fall-to-support recovery |
| Definition of done | The same canonical physics checkpoint runs through the reference-game player path, save/load and Replay V10; focused checks prove exact traversal, blocking, contact order, sensor continuity, dynamic-body continuation and post-fall locomotion without adding a second pose owner |
| Authority | Working context only; Accepted SPEC/ADR, `docs/roadmap.md`, checked-in schemas and exact ProductCheck results outrank this file |

## Resume in 60 seconds

- **Current conclusion:** R5b is accepted: the production reference session owns a visible, traversable slope/stair/push/sensor/fall course, and an exact mid-push checkpoint restores through the production driver with identical continuation.
- **Why:** Five focused backend fixtures, the live production course, `play`, `persistence-replay`, `content-package`, `physics-collision` and the full `host-check` all pass without changing public step/snapshot or Replay V10 schemas.
- **Next action:** Freeze a separate bounded R5c root-motion admission package: sampled root displacement remains an intent that physics may clip or reject, while the physics capsule stays the sole transform writer.
- **Current blocker:** None for R5b. Continuous rotated/mesh slopes, arbitrary rigid-body dynamics, ragdoll/get-up animation, root motion, BodySchema, real skinning and learned control remain outside this completed cut.
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

1. **Complete:** freeze constants, supported descriptor subset and negative activation cases.
2. **Complete:** implement step-up/ground-snap and focused slope/stair/tall-blocker tests.
3. **Complete:** implement dynamic-box push plus blocked-push and checkpoint-reconstruction tests.
4. **Complete:** implement non-blocking sensor continuity and fall/landing/resumed-locomotion tests.
5. **Complete:** add the visible course to the production reference session and prove exact play/save/load/Replay V10 continuation from an active dynamic push.
6. **Complete:** run mapped checks, record conditional `NOT_RUN` results and close only R5b.

## Explicitly deferred from R5b

- Continuous rotated boxes, convex/mesh/heightfield terrain and a general rigid-body solver.
- Multiple avatar bindings, NPC locomotion commands, moving platforms, carrying, melee, trip/ragdoll and get-up animation.
- Root-motion admission, BodySchema projection, real skinning and expanded animation graphs.
- PhysX-only atomic production cutover, fresh `r5-physics-16.v1` gates, paired Windows/Linux evidence and full R5 completion.
- Learned policy training, export, inference and any resumption of the stopped R141/R142 lineage.

## Handoff

| Evidence | Result | Consequence |
| --- | --- | --- |
| `cargo test -p next_contracts physics --quiet`; `cargo test -p next_physics_api --all-targets --quiet` | `PASS` — 13 contract tests and 30 backend tests, including five R5b fixtures | Constants, supported subset, slope/stair traversal, tall blocking, bounded dynamic push, sensors, fall/recovery and exact reconstruction are green. |
| `cargo test -p next_reference_game --test collision_visibility production_capsule_course_traverses_and_restores_mid_push --quiet`; `cargo test -p next_reference_game --test visual_presentation --quiet` | `PASS` — exact restored continuation through tick 80; ten initial scene records and nine physics-owned presentation bindings | The course is reached through production input, and the visible static course/dynamic box consume the same authoritative physics state. |
| `cargo run -p xtask -- content-package` | `PASS` — 121 records, 64 chunks, 12 meshes/cooked meshes, zero fallback-material draws in the reference frame | Authored visual assets, activation and the production gameplay consumer close together. |
| `cargo run -p xtask -- play` | `PASS` — 32 ticks, 52 events, authoritative state root `448a94a345526d8dcfb43f2f6bc6bf3940c4cbb1367963d1b136724730253c7d`, ledger root `fc3d05e211a39dcb7953944460abf4dad536230e0eb1551e7b87bd9693457daf` | Existing gameplay outcome remains exact with the R5b bodies present. |
| `cargo run -p xtask -- persistence-replay` | `PASS` — 19 ticks, two generations, final state root `b337277a3c4b1e95e472923e35eb5ae86f63820e4766ab7852cbba6208188cef`, ledger root `e53e69edf29994ed50d3b7997bcb71e7d7d23c9bad441ffba8b53e8a60f03ad3` | Save/load and Replay V10 reconstruct all existing owners plus R5b physics exactly. |
| `cargo run -p xtask -- physics-collision` | `PASS` — 6 gameplay ticks, 12 substeps, `3/15/2` begin/persist/end events, checkpoint hash `af11dae48353deb37cba7d9b4dd0201e3d98445f46aadfda9731a1e7639b1425` | The prior collision fixture retains its accepted semantics. |
| `cargo run -p xtask -- host-check` | `PASS` — format, workspace strict Clippy, full workspace tests and `boundary-scan` on `x86_64-unknown-linux-gnu`, Rust `1.97.1` | R5b closes its broad local regression boundary. |
| `platform`, `performance`, real-SDK backend parity and paired Windows/Linux execution | `NOT_RUN` | R5b acceptance makes no platform, performance, PhysX-only, Stage 0 or full-R5 claim. |
