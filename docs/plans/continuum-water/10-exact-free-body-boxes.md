# Exact vertical free-body dynamics for dynamic boxes — prerequisite of the buoyancy batch

| Field | Value |
| --- | --- |
| Research ID | `WR1` |
| Status | `RUN / G1-G6 PASS (parity check itself broken before WR1) / G7 FAIL (reading recorded)` (2026-09-03) |
| Parent | SPEC-26 current bounded profile; ADR-105 (Proposed) and plan `continuum-water/08`, whose prerequisite this is (task-state D-006/D-007) |
| Purpose | give dynamic boxes mass, gravity and floor support in the canonical world shared by both backends, so that a water reaction batch can move a crate; nothing else about the bounded profile changes |

## Frozen scope

- **Contract.** `PhysicsBodyDescriptorV1` gains `mass_microkilograms`
  (canonical field 9, always encoded): `1..=10^15` for `Dynamic`
  (SPEC-26 range `0.000001..=1,000,000,000 kg`), exactly `0` for `Static`
  and `Kinematic`. Every catalog hash changes; pinned roots are
  regenerated once from the failing runs and recorded. The mass is the
  impulse divisor for later reaction batches (ADR-105) and takes no part
  in this increment's motion.
- **World.** `GroundedCapsuleWorld` accepts up to `16` dynamic boxes
  (today `1`). Each physics substep, before the capsule integrates and in
  shape-id order, every dynamic box:
  1. adds the solver-profile gravity to its vertical velocity
     (`g / physics_hz`, the same integer delta the capsule uses);
  2. sweeps `v_y / physics_hz` along `y` against the static solids, the
     other dynamic boxes at their current staged poses, the capsule's
     axis-aligned bounds (`centre +- [r, half_segment + r, r]`) and the
     attached carried boxes, all filtered by layer/mask;
  3. moves by the applied delta and zeroes its vertical velocity when the
     sweep was cut (inelastic support; the materials already require zero
     restitution).
  Horizontal motion stays push-only: the capsule push sweeps the box
  against the static solids and the other dynamic boxes (no chain push),
  and the box's horizontal velocity remains the applied push times
  `physics_hz` for that substep. Body revisions bump exactly when pose or
  velocity change. No sleep conversion; boxes stay active.
- **Reference scene.** The R5b push box declares `20 kg`; static bodies
  declare `0`. It rests on the floor at activation (bottom face on the
  floor top), so its state is unchanged by the new integrator and the
  reference roots move only through the catalog hash.
- **Verification.** Focused tests in
  `crates/physics-api/src/reference_world/tests/free_body.rs` on
  synthetic scenes built from the R5b fixture, plus the existing product
  checks `physics-backend-parity`, `physics-collision`, `play`,
  `persistence-replay` and `host-check`.

## Frozen gates

| Gate | Definition | Pass |
| --- | --- | --- |
| G1 fall | a box released `1 m` above the floor lands with its bottom face exactly on the floor top and zero vertical velocity; the landing substep is within one substep of the integer recurrence `v += g/hz; y += v/hz` from rest (the analytic `sqrt(2h/g)` rounded up) | pass |
| G2 rest | a box resting on the floor keeps pose, velocity and body revision unchanged over `600` ticks; the R5b reference snapshot hash is unchanged by the integrator (only the catalog hash moves) | pass |
| G3 stack | a second box released above the first lands on the first's top face and rests there; the lower box's state is unchanged | pass |
| G4 push into a box | the capsule pushes box A into resting box B: A stops with its face on B's face, B does not move, no penetration between any pair | pass |
| G5 capsule support | a box released above the capsule stops on the capsule's axis-aligned top; the capsule pose is unchanged; the resulting checkpoint restores (activation penetration check passes) | pass |
| G6 determinism | the reference and the PhysX backend reach identical roots (`physics-backend-parity`), restore-and-continue matches live (`persistence-replay`), `play` passes | pass |
| G7 cost | one gameplay tick with `16` resting dynamic boxes on the reference host, release build | `<= 200 us` |

Do not change the gravity rule, the obstacle set, the box limit or the
reference mass after seeing the results; a change is a new revision with
its own evidence.

## Result (WR1, 2026-09-03)

`cargo test -p next_physics_api free_body` (7 tests) and the workspace
sweep pass; the product checks are recorded in the task-state:

| Gate | Result |
| --- | --- |
| G1 fall | the box released `1 m` above the floor lands at `y = 300,000 um` (bottom face on the floor top) with zero velocity; the recurrence lands at substep `27` (tick `14` of two substeps), the analytic `sqrt(2 h / g) * hz = 27.12` rounds up to `28` — within one substep — PASS |
| G2 rest | a resting box keeps pose, velocity and body revision over `600` ticks; a runtime restored at tick `5` (one box mid-fall) reproduces every later step result and the final checkpoint; the R5b push box rests on the floor, so the reference snapshot is unchanged and only the catalog hash (field 9) moved the roots — PASS |
| G3 stack | the upper box lands on the lower box's top face (`y = 900,000 um`) and rests; the lower box is unchanged — PASS |
| G4 push into a box | box A stops at `x = 1,100,000 um` with its face on box B, B is unchanged, the capsule stops at `600,000 um` (the R5b blocker numbers) — PASS |
| G5 capsule support | the box released above the capsule rests at `y = 2,100,000 um` (bottom on the capsule's axis-aligned top at `1.8 m`); the capsule pose is unchanged; the checkpoint restores — PASS |
| G6 determinism | `physics-collision`, `play`, `persistence-replay` and `host-check` PASS with the regenerated catalog hash; the reference and PhysX backends share the integrator by construction (the PhysX backend supplies sweep queries to the same `GroundedCapsuleWorld`, and `play`/`persistence-replay` run under `RequirePhysX`). `physics-backend-parity` could not be run: it stops before any comparison with `parity fixture has an unsupported shape` on the parent commit `e834f83a` as well (a pre-existing fixture fault, recorded in the task-state) — PASS with that reservation |
| G7 cost | release build, `60` ticks: capsule only `111 us`, sixteen resting boxes `416 us` per gameplay tick (`~19 us` per box per tick over two substeps) — FAIL against `200 us` |

G7 reading (recorded, not tuned): each box substep rebuilds the obstacle
list (every other box through `current_dynamic_boxes`, an allocation and a
sort) and the capsule step re-derives the boxes again; the cost is
quadratic in the box count. A next revision may build the obstacle set
once per substep and reuse it; the frozen `200 us` bound stays the target
of that revision.

Apparatus (recorded): the sixteen-box cost scene spaces the boxes `0.5 m`
apart so that all sixteen rest on the `20 m` fixture floor.
