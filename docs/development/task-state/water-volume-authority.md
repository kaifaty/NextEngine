# Task state: authoritative water volume and presentation water (R8c)

| Field | Value |
| --- | --- |
| Task | Implement ADR-100 option C: exact CPU `WaterVolume` for gameplay, presentation-only water for the renderer |
| Status | `ACTIVE / CONTINUUM-WATER-VOLUME-P1=PASS / CONTINUUM-WATER-FLOW-P1=PASS (R8d) / PLAYER_CLASS_AND_STILL_SURFACE_DONE / VESSEL_SURFACES_AND_SOLVER_SURFACE_NEXT` |
| Branch | `codex/water-research` |
| Last updated | 2026-09-02 |

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
- **Next:** surface meshes for the two vessels (presentation only), then
  the ADR-101 ring inside the game root fed by the presentation solver
  stream (`CONTINUUM-WATER-PRESENT-P1`, `RENDER-DYNSURF-P1`); the
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
   profile at activation and restore (2026-09-03). Plan
   `continuum-water/08` (ADR-105 buoyancy batch) is blocked by a
   prerequisite: the canonical world has no free rigid dynamics for
   boxes (no mass, no gravity, push-only). Decision pending: (a) an exact
   vertical free-body increment for dynamic boxes (mass in the
   descriptor, gravity, floor contact) under SPEC-26, then the batch; or
   (b) do `CONTINUUM-WATER-PRESENT-P1` first and return to buoyancy with
   (a). Then the SPEC-38 2.2
   practices in ADR-103 order (lattice tier, activity stepping,
   edge-driven presentation, rotational presentation, wave layer);
   surface meshes for the two vessels ride the first presentation
   increment.
1. Presentation solver in the game root: declare the basin mesh as an
   ADR-101 dynamic surface in `apps/game`, feed it from the neutral stream
   (`nonlocal-feasibility --game-surface-stream`) or a still fallback, and
   define `CONTINUUM-WATER-PRESENT-P1` around a bounded capture with
   identical gameplay roots with and without the solver.
2. Optional gameplay effect: motor speed scaling from the classification
   (needs its own bounded evidence; not part of C's authority split).
3. Keep ADR-100/103/104 Proposed until the four `CONTINUUM-WATER-*` checks pass, then accept them with the
   SPEC/routing/traceability updates.
