# Task state: authoritative water volume and presentation water (R8c)

| Field | Value |
| --- | --- |
| Task | Implement ADR-100 option C: exact CPU `WaterVolume` for gameplay, presentation-only water for the renderer |
| Status | `ACTIVE / CONTINUUM-WATER-VOLUME-P1=PASS / CONTINUUM-WATER-FLOW-P1=PASS (R8d) / CONTINUUM-WATER-PRESENT-P1=PASS (WP1) / FREE_BODY_BOXES_DONE (WR1) / CONTINUUM-WATER-BUOYANCY-P1=PASS (WB1) / PROMOTION_DECISION_NEXT` |
| Branch | `codex/water-research` |
| Last updated | 2026-09-03 |

## Resume in 60 seconds

- **Decision C is implemented on the gameplay side.** `WaterVolumeSetV1`
  (definitions plus per-volume record state) is field 4 of
  `PhysicsWorldCheckpointV1` schema version 3 (2 at R8c), so it rides the physics leaf of
  every state root, save segment and replay compare point. Backends only
  carry it; the rigid step never reads it.
- **Command path.** `WaterVolumeCommandV1::SetLevel` is the ninth command
  kind (`core_r8c`, priority 290, capability
  `nextengine.capability.water-volume-level`). It commits
  `WaterVolumeChangedV1`, bumps the record revision and suspends the authored
  ramp; unknown/stale/out-of-extent/exhausted are stable rejections.
- **Query.** `WaterVolumeSetV1::submersion_at(point, tick)` returns the
  containing volume, exact depth and `Dry`/`Wading`/`Swimming`.
- **Reference basin.** `crates/reference-game/src/water.rs`: `4 x 2 m`,
  `2 m` deep at `x 4.5..8.5, z 1..3`, level `0.5 m`, swimming depth `1.2 m`.
- **Player consumer.** `next_reference_game::player_submersion` classifies
  the capsule foot point (pose minus radius plus half segment) against the
  committed table; the HUD status panel shows `Wading`/`Swimming` (text ids
  `nextengine.ui.text.hud.water.*`).
- **Still surface.** Authored quad mesh `0x7c` + material `0x7d` bound as an
  environment record (`REFERENCE_WATER_BASIN_ID`, layer 14) whose
  translation is `water_surface_translation(table, tick)`; the level
  command moves it in the ordinary presentation snapshot.
- **Check.** `cargo run -p xtask -- water-volume` runs
  `CONTINUUM-WATER-VOLUME-P1`: probe table, production locomotion walk
  (`-z` 5, `+x` 65, `+z` 25 ticks) to `[6.5, 0.9, 2.0] m`, Wading ->
  Swimming across the level command, surface translation `0.5 -> 1.5 m`,
  three rejections, checkpoint round trip, restore/continue, repeated
  generation.
- **Flow network (R8d, ADR-103).** `WaterFlowNetworkV1` is field 5 of the
  physics checkpoint (schema 3): cells are volumes, edges move water by
  head in one exact integer Jacobi step per tick after the rigid step;
  `WaterFlowCommandV1` (`SetGate`/`SetPump`/`SetSource`, tenth kind,
  priority 291, capability `nextengine.capability.water-flow-control`).
  Reference scene: vessels `0x7e`/`0x7f`, gate `0x80`, source `0x81`,
  sink `0x82`. `cargo run -p xtask -- water-flow` runs
  `CONTINUUM-WATER-FLOW-P1`.
- **Presentation stage (WP1, plan `continuum-water/09`).**
  `next_reference_game::compute_water_presentation_frame` is a pure
  function of the committed checkpoint (levels and edge fluxes) and a
  frame index: `32 x 16` grids per quad with a flux-driven ripple (cap
  `20 mm`) and a stateless ballistic jet at every gate/pipe mouth (one
  droplet per `0.5 L`, at most `4,096`). Vessel quads `0x7e`/`0x7f`
  (objects `0x84`/`0x85`) join the basin quad as environment records. The
  interactive worker publishes the frame beside the snapshot;
  `apps/game` declares the three quads as ADR-101 dynamic surfaces and
  the jet as the ADR-102 particle surface (`water_presentation.rs`) and
  takes `--capture-frame N --capture-png PATH` for one diagnostic PNG.
  `cargo run -p xtask -- water-present` runs
  `CONTINUUM-WATER-PRESENT-P1` (roots identical with and without the
  stage, capacities, purity, `98 us` release cost).
- **Free-body boxes (WR1, plan `continuum-water/10`).**
  `PhysicsBodyDescriptorV1.mass_microkilograms` (field 9; `1..=10^15`
  for `Dynamic`, `0` otherwise) and, in `GroundedCapsuleWorld` shared by
  both backends, exact vertical free-body motion for up to `16` dynamic
  boxes: gravity into `v_y`, a swept move against static solids, other
  boxes, the capsule bounds and carried boxes, inelastic support. The R5b
  push box declares `20 kg` and rests on the floor, so only the catalog
  hash moved the reference roots. Tests: `reference_world/tests/free_body.rs`.
- **Parity check repaired (2026-09-03):** `cargo run -p xtask --features
  physx -- physics-backend-parity` PASS (`100,000` substeps, `10,000`
  permutations); its fixture had rejected the sensor and query-only
  static shapes of the reference scene. WR1 G7 re-measured with the
  physics step alone: `116-121 us` mean, `124-174 us` steady maximum for
  sixteen boxes (plan 10 result).
- **Buoyancy batch (WB1, plan `continuum-water/08`, ADR-105).**
  `WaterBuoyancyProfileV1` is field 6 of the physics checkpoint (schema
  4); before every rigid step the runtime computes `WaterBuoyancyBatchV1`
  (exact clipped-bounds volume, `rho g V / hz` buoyancy, `-k rho V v /
  (1000 hz)` drag, ADR-081 tuple) and puts it into
  `PhysicsStepInputV2.external_impulses` (schema 3); the canonical world
  applies `J_y / m` once at the first substep. Reference crate: body
  `0x87`, `0.5 m`, `50 kg`, mesh `0x8d`, at `[6.5, 0.25, 2.0] m` in the
  basin. `cargo run -p xtask -- water-buoyancy` runs
  `CONTINUUM-WATER-BUOYANCY-P1` (settled immersion `0.196 m`, follows a
  `1.5 m` level, dry push box untouched, live/restored/repeated roots).
- **Next:** the four `CONTINUUM-WATER-*` checks pass on the reference
  host; the promotion decision (ADR-100/103/104/105 Accepted, SPEC-38
  Accepted for water) is the next step, then the remaining SPEC-38 2.2
  practices (lattice tier, activity stepping, rotational presentation)
  and wake/splash presentation for the crate (ADR-102 increment); the
  research CUDA tool stays a separate process behind the neutral stream.

## Required context

- Research note on engine and game water models:
  `docs/development/water-engines-research-2026-09-02.md`.
- `AGENTS.md`; routing rows "Physics world ..." and "Future continuum
  materials ..." in `docs/architecture/agent-routing.md`.
- ADR-100 / ADR-101 / ADR-102 / ADR-103 / ADR-104 (Proposed), SPEC-26 2.6, SPEC-38 2.2, SPEC-03 2.11, SPEC-21 2.2.
- Research side and live Vulkan bridge:
  `docs/development/task-state/nonlocal-gpu-full-step-performance.md`.

## Decisions

### D-001 — Water table inside the physics checkpoint, not a new owner segment

- **Observation:** a new owner segment (the world-activity template) touches
  save image/store, replay manifests, application roots and the
  persistence-replay runner (about 80 files across two commits); the physics
  checkpoint already is the Physical Embodiment save segment and SPEC-38
  anticipated "a composite successor to the physics checkpoint for the
  volume".
- **Decision:** `PhysicsWorldCheckpointV1` schema 1 -> 2 with
  `water_volumes` as field 4; hash domain `...checkpoint.v2`; `new()` keeps an
  empty table so every existing fixture compiles unchanged.
- **Rejected:** separate owner segment (cost without benefit for one table);
  water inside `PhysicsCanonicalSnapshotV2` (would enter PhysX/training
  snapshot hashing).
- **Reconsider when:** a consumer needs water state outside the physics
  world, or the articulated `PhysicsWorldCheckpointV2` lane needs water.

### D-002 — Level ramp is a pure function of the tick

- **Decision:** the authored ramp is evaluated from the tick in
  `effective_level`; no per-tick system, delta or schedule change. A
  committed level command suspends the ramp permanently.
- **Consequence:** no schedule manifest change (`core_r8c` keeps the R4d
  schedule) and no per-tick root churn for authored tides.

### D-004 — Still surface through the snapshot, not the ring

- **Observation:** a level-following flat surface needs no per-frame
  vertex payload; the presentation snapshot already carries exact
  fallback transforms for non-physics environment records.
- **Decision:** the basin quad is authored at local `y = 0` and bound with
  translation `[0, level, 0]`; the ADR-101 ring stays reserved for the
  solver-driven surface, which replaces vertex payload but not the binding.
- **Rejected:** authoring the quad at the initial level (would not follow
  commands or ramps); publishing ring updates for a flat quad (cost without
  benefit).

### D-006 — The specifications close the water ladder on the exact table and network

- **Observation:** the user asked whether the specifications still
  require heavy simulation now that the implementation is the exact
  network plus presentation tiers.
- **Evidence:** SPEC-38 1.x and ADR-076 still placed the particle
  reference, particle coupling, particle persistence and the GPU mirror
  before water promotion and kept a `48,000`-sample coupling scenario as
  the product fixture; nothing implemented reads a particle.
- **Decision:** ADR-104 (Proposed) closes the water authority ladder on
  ADR-100/ADR-103, keeps particle water presentation-only for V1, routes
  rigid coupling through exact levels (`CONTINUUM-WATER-BUOYANCY-P1`)
  and demotes the particle checks to research reports; SPEC-38 2.0,
  ADR-076 superseded in its water clauses, routing/traceability/roadmap
  updated.
- **Rejected:** keeping two authorities side by side; deleting the
  research lanes (they are the calibration oracle).
- **Reconsider when:** a consumer needs authoritative particle water,
  which is a new ADR with its own evidence.

### D-005 — Water mechanics live in an exact cell/edge network, not in particles

- **Observation:** the product wants Timberborn-class mechanics
  (dams, gates, channels, pumps, communicating vessels, flooding) and
  asked how to scale water by orders of magnitude; the particle solver
  scales with volume and is non-authoritative by ADR-100.
- **Evidence:** research calibration from plan 22 (`Cd 0.40..0.44` for a
  wall opening, `0.13` for a long lined duct, exit speed `0.5..0.55` of
  free fall); shipped games of this class run column or cell flow
  models; the R8c checkpoint already carries exact volumes.
- **Decision:** ADR-103 (Proposed): `WaterFlowNetworkV1` with cells as
  `WaterVolume`s and head-driven edges, one exact integer step per
  tick, field 5 of the physics checkpoint; presentation reads levels
  and, later, edge fluxes; first increment is the two-vessel scene under
  plan `continuum-water/07` with frozen gates.
- **Rejected:** particles as the mechanic owner; a floating-point
  shallow-water solver needing an execution profile; scripted levels only.
- **Reconsider when:** a dense map-wide grid is needed (it is a network
  with lattice cells and open edges) or a rigid-body coupling consumer
  lands.

### D-007 — The presentation stage is a pure function inside the reference game crate

- **Observation:** ADR-101/102 rings need per-frame payloads; SPEC-38 2.2
  practice 5 asks for edge-driven presentation and practice 6 for one
  writer per substance; the interactive worker already publishes one
  snapshot per generation to the desktop adapter.
- **Decision:** `compute_water_presentation_frame(volumes, network,
  bindings, tick, frame_index)` lives in `next_reference_game` and reads
  only `effective_level` and `edge_flux`; the worker attaches the frame to
  its published snapshot; `apps/game` converts it into ADR-101/102 updates
  with a monotonic sequence and republishes the last frame under menu
  republication. The jet is stateless (re-integrated from the flux each
  frame) so purity holds without a particle pool.
- **Rejected:** a stateful emitter in the adapter feed (would break G3 and
  the one-writer rule); feeding the ring from the research CUDA stream in
  the game root (a second process for a presentation effect); a solver
  inside the runtime tick (presentation state next to gameplay roots).
- **Reconsider when:** a wave layer needs history (SPEC-38 practice 7
  would then own a bounded presentation-only state with its own writer).

### D-008 — Dynamic boxes are exact vertical free bodies inside the shared canonical world

- **Observation:** ADR-105 needs a body that a reaction batch can move,
  the canonical world integrated only the capsule (boxes were push-only),
  and the PhysX backend supplies sweep queries to the same
  `GroundedCapsuleWorld`, so any box motion written there is shared by
  both backends by construction.
- **Decision:** mass as `mass_microkilograms` on `PhysicsBodyDescriptorV1`
  (field 9, always encoded; catalog hashes and pinned roots regenerated
  once); per substep every dynamic box adds `g / physics_hz` to `v_y`,
  sweeps `v_y / physics_hz` against static solids, other boxes, the
  capsule's axis-aligned bounds and carried boxes, and zeroes `v_y` on a
  cut sweep; horizontal motion stays push-only; up to `16` boxes.
- **Rejected:** a kinematic level-following rule for floating crates
  (no free fall, no support, no path to drag); PhysX dynamic actors for
  boxes (the backend would own motion the canonical world cannot
  reproduce); an exact capsule-shaped sweep for boxes over the avatar
  (an `isqrt` per pair for a rounding the capsule's own distance test
  already tolerates through the axis-aligned bound).
- **Reconsider when:** a consumer needs horizontal free motion, box-box
  chain pushes, restitution or friction, or reported box-support contacts.

### D-009 — The batch profile rides the checkpoint and the batch reads the staged table of its tick

- **Observation:** a restored world must compute the same batch as the
  live one, so the profile cannot be a code constant or a runtime option;
  and the runtime commits level commands in the command phase before the
  physical step of the same tick, so the table the step sees already
  carries them.
- **Decision:** `WaterBuoyancyProfileV1` is optional field 6 of
  `PhysicsWorldCheckpointV1` (schema 4; `None` means no batch); the batch
  binds the table as staged for the step (previous flow result plus the
  tick's committed level commands) and the previous tick's committed
  poses, with the water table hash and highest record revision as the
  source root/revision. The verification recomputes the batch
  independently every tick except the command tick, where it verifies the
  bound roots instead.
- **Rejected:** profile in the catalog (a catalog change for a water
  consumer); profile as a runtime bootstrap option (not saved with the
  world); computing the batch after the flow step of the same tick (the
  water side would read the rigid outcome it feeds).
- **Reconsider when:** several batch profiles per world or per-volume
  densities are needed.

### D-003 — Verification issues the level command as a `Tool` principal

- **Observation:** no gameplay mechanic sets a water level yet; the player
  principal must not carry the water capability.
- **Decision:** `water-volume` registers a SPEC-21 `Tool` principal with the
  water capability on the reference session bootstrap and drives the
  production admission path; game mechanics that change levels will use an
  `InternalSystem` principal at the Outcome barrier.

## Evidence

- `xtask water-flow` PASS 2026-09-02 (R8d, plan `continuum-water/07`):
  conservation exact over `1,800` ticks, vessel A drained by tick `868`
  (bound `1,680`), gate response within one tick, four rejections,
  checkpoint round trip, restored and repeated runs identical; matrix
  digest `a57b0ca3...`, final state root `41e84ebc...`, physics checkpoint
  `e93a8c35...`; step cost `183 us` release / `1,405 us` debug for `64`
  cells and `256` edges (G6 `50 us` not met). Values change with any
  profile/registry/content change; they are not golden.
- Pinned roots refreshed for the R8d registry and checkpoint schema 3:
  play ledger root `bdfba580...`, replay root `320cd4b5...`, creator-smoke
  lock `def43dcf...` and its five expectations; content counts unchanged
  (no new assets).
- `xtask water-volume` PASS 2026-09-02 (with the player walk and surface
  binding): matrix digest `fce46535...`, final state root `38372827...`,
  physics checkpoint `0efdf9d1...` (values change with any
  profile/registry/content change; they are not golden).
- `play`, `persistence-replay`, `content-package` PASS after the
  R8c registry/checkpoint root refresh (play ledger root, creator-smoke
  scenario expectations, replay pinned root, 39 roots / 125 entries,
  13 meshes / 12 materials, 10 rendered objects).
- `host-check`: fmt, clippy, workspace tests and boundary scan PASS across
  the final runs (the last consolidated run is re-executed after the
  `0x11` test byte rename; see the commit that follows).

## Do not retry

- GPU particle state as gameplay authority (ADR-100 alternatives).
- Reading presentation water from any command, query or root.

## Next action

0. Water mechanics (ADR-103): done in R8d (`CONTINUUM-WATER-FLOW-P1 =
   PASS`); the network tick rate is cross-checked against the world
   profile at activation and restore (2026-09-03). Presentation stage:
   done in WP1 (`CONTINUUM-WATER-PRESENT-P1 = PASS`, plan
   `continuum-water/09`; the human look gate G6 stays open until a walk
   to the vessels with the capture flags). Decided 2026-09-03 (D-007):
   the presentation stage first, then the free-body increment.
1. Free-body boxes: done in WR1 (plan `continuum-water/10`, SPEC-26
   2.7). Buoyancy batch: done in WB1 (plan `continuum-water/08`,
   ADR-105 0.2, SPEC-26 2.8, SPEC-03 2.12, SPEC-38 2.4). Next: the
   promotion decision for the water ladder (ADR-104), then the remaining
   SPEC-38 2.2 practices in ADR-103 order (lattice tier, activity
   stepping, rotational presentation) and a crate wake/splash increment.
2. Optional gameplay effect: motor speed scaling from the classification
   (needs its own bounded evidence; not part of C's authority split).
3. Keep ADR-100/103/104 Proposed until the four `CONTINUUM-WATER-*` checks pass, then accept them with the
   SPEC/routing/traceability updates.
