# Task state: authoritative water volume and presentation water (R8c)

| Field | Value |
| --- | --- |
| Task | Implement ADR-100 option C: exact CPU `WaterVolume` for gameplay, presentation-only water for the renderer |
| Status | `ACTIVE / CONTINUUM-WATER-VOLUME-P1=PASS / CONTINUUM-WATER-FLOW-P1=PASS (R8d) / CONTINUUM-WATER-PRESENT-P1=PASS (WP1) / FREE_BODY_BOXES_DONE (WR1) / CONTINUUM-WATER-BUOYANCY-P1=PASS (WB1) / WATER_V1_ACCEPTED_2026-09-03 / WATER_LOOK_NEXT` |
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
- **Accepted 2026-09-03.** ADR-100/103/104/105 Accepted, SPEC-38 3.0
  Accepted for water, terrain clauses in Proposed SPEC-39.
- **Water look L1 done (plan 12, 2026-09-03):** `water_surface` shader suite on `WaterSurface` rings (ADR-101 0.3).
- **Start at the water (2026-09-03):** `apps/game --interactive --start-at-water` starts a fresh session with `ReferenceSpawnOverrideV1::at_water()` (`LaunchRequestV1::spawn_override`: capsule at `[5.3, 0.9, 0.2] m` at the basin's south edge, camera yaw `0` looking across the water, pitch `-12`; a different bootstrap root for that session only, resumed sessions ignore it); combine with `--maximum-frames 122 --capture-frame 120 --capture-png` for a capture. The adapter keeps `DesktopRunOptions::scripted_input` (SDL event queue) for diagnostics that need real input.
- **Water look L2 + L3 done (plan 13, 2026-09-03):** the water pass after the opaque scene (`gpu_content/water.rs`, `water_scene` suite): scene colour copy, sampled scene depth, refraction, absorption, vertical-depth shoreline and foam; fallback prints `WATER_PASS_FALLBACK`.
- **Water look L4 done (plan 14, 2026-09-03):** ambient wave spectrum in the stage (`AMBIENT_WAVES`), animated detail normal in the water pass.
- **Water look L5 done (plan 15, 2026-09-03):** mirrored reflection pass (`b0_reflect` suite, set 3 binding 3 of the water pass).
- **Basin rim done (plan 16, 2026-09-03):** static body `0x89` with five box shapes (`REFERENCE_WATER_BASIN_RIM_BOXES_MICROMETRES`, `0.6 m` high, `0.15 m` thick, a `1 m` opening on the south side) and the compound mesh `0x8e` bound as an `Environment` presentation record; the scene roots moved (43 roots, 129 records, 17 meshes, 14 rendered objects) and every pinned count was refreshed.
- **Water look L6 + L7 done (plan 17, 2026-09-03, revision 2):** caustic term in `water_scene` (vertical-depth attenuation after an invisible revision 1), stage wake depression and splash droplets from the committed boxes (`floating_boxes`, `WaterFloatingBoxV1`; `compute_water_presentation_frame` gained the box input); stage cost `136 us` max.
- **Water look L8 done (plan 18, 2026-09-03):** DLSS-ready outputs in the desktop adapter: the scene renders into an offscreen HUD-less target copied to the swapchain before the UI overlay; the `gbuffer` suite (own set 0, `96`-byte push block, four `32`-bit attachments) writes albedo + group mask, normal + roughness, screen motion vectors (previous model per draw keyed by mesh/material/texture revisions and occurrence, previous jittered view-projection) and linear depth; `DesktopRunOptions::projection_jitter` / `apps/game --projection-jitter` applies the Halton(2, 3) jitter to the projection's `[8]`/`[9]` lanes; `--capture-buffer {color|scene|albedo|normal|motion|depth}` and `--capture-frames N` read the images back. The B0 contract, the frame plan hash and every root are unchanged. Motion vectors are exact per rendered frame, which means zero on the frames between two presentation ticks (about five frames per tick in release) and the full tick displacement on the straddling frame; a presentation-side pose interpolation is the follow-up if a temporal upscaler is ever integrated.
- **Acceleration done (2026-09-03, plan 11 section 2):** WB1 buoyancy batch `260 us` → `13 us` mean / `18-25 us` max (plan 08 revisions 2-3: identifiers once per batch, level cache, plan-rectangle reject, `text_id!` identifiers as `Arc<str>` — a representation change of `next_contracts::ids`, canonical bytes unchanged); R8d flow step `183 us` → `28 us` mean / `41-46 us` max (plan 07 revision 2: `WaterFlowNetworkV1::step_in_place`, `PhysicsWorldBackend::step_water_flow`, `i128::isqrt`). Records, fluxes, levels and every root unchanged; the cost reports carry a mean next to the gated maximum.
- **Lattice tier done (2026-09-03, plan 19, SPEC-38 3.1 practice 1):** `WaterLatticeRegionV1` (`crates/contracts/src/physics/water_lattice.rs`) builds face-sharing cells and open sills; the volume overlap rule is positive-measure; `CONTINUUM-WATER-LATTICE-P1 = PASS` through `xtask water-lattice` (exact conservation, downhill settle within `3.9 mm` of head, repeated and cloning runs identical, `18 us` mean step). The reference scene is unchanged.
- **Activity stepping done (2026-09-03, plan 20, SPEC-38 3.2 practice 2):** `WaterFlowActivityV1` (derived, never saved, reset with the table or network, empty after restore) and `step_in_place_with_activity`; `GroundedCapsuleWorld` steps with it. Roots identical to the always-stepped run with and without a wake; `water-flow` restored-run and `persistence-replay` PASS. Frozen G2/G5 thresholds FAIL by reading: under the exact weir law a settled sill reaches flux `0` only below about `6 um` of head (first rest `6.8` minutes after the water arrives, `59` percent of sills at `20` minutes, step `17` to `14 us`).
- **Decided 2026-09-03 (D-010):** the quiescence clause of the flux law is deferred; recorded as a future optimisation in plan `continuum-water/11` section 4, to be revisited only with a lattice larger than the current bounds.
- **Edge-driven presentation done (2026-09-03, plan 21, SPEC-38 3.3 practice 3):** `WaterPresentationFrameV1::edges` carries one `WaterEdgePresentationV1` per edge with non-zero flux (jet, fall, sill, mouth); falls over open sills shed droplets through the jet's lane (`emit_stream`); nothing per cell. `water-present` digest of surfaces and droplets identical (`a8904f22…`), `water-lattice` revision 3 PASS (`56` records max, `37 us` mean stage).
- **Rotational presentation done (2026-09-03, plan 22, SPEC-38 3.4 practice 4, authored vortex):** `vortices_of` / `vortex_height` in the stage: a whirlpool over vessel B's sink in its ring (dip from the exact flux, two-arm spiral), inside the cap; `water-present` PASS with roots identical (`85 us` mean). The human look at vessel B (G3) is open. The shallow-water grid variant stays planned.
- **ADR-106 Accepted 1.0 (user, 2026-09-04)** on the plan 23 probe and the plan 24 demo; the lane continues with plan 25 (emission from exact data and absorption at the level).
- **Decided 2026-09-04 (D-011):** the water authority stays the exact CPU model (table, flow network, buoyancy batch); NVIDIA PhysX becomes the water *presentation* lane (PBD particle fluids) behind the existing PhysX boundary, optional and vendor-specific, with the current ring-and-droplet presentation as the fallback; no gameplay read, no root change. Research note `docs/development/water-physx-presentation-research-2026-09-04.md`, ADR-106 (Proposed) and plan `continuum-water/23` (the probe: GPU SDK profile, bridge probe, `xtask physx pbd-probe`) written 2026-09-04; ADR-106 is accepted on the probe's evidence, the lane in the game is plan 24.
- **Roadmap (2026-09-04):** plan `continuum-water/31` orders the remaining water work by the player's impression (the camera under the surface, sound, wetness, gates and pumps, tilting floats, event waves, big water, the lane's finish, the ground, far water, then the player in the water and the player's wake); each item becomes its own frozen plan when picked.
- **Plan 32 done (2026-09-04):** the reference pond, sunk into the ground north of the spawn (`x −9..1`, `z 5..9.5`, floor `−1.5 m`, level `−0.1 m`, five steps at the west end), the first body that holds the third-person camera under the level and the first `Swimming` classification of the scene; the ground is four strips around the hole; a fourth surface ring; `--start-at-pond`. Roots moved: `play` `255dec16…`, `water-present` `0abd6769…`, `persistence-replay` `aba2347c…`; content pins refreshed once. The choice of the pond over a longer basin or a closer camera is the user's (2026-09-04).
- **Plan 33 done (2026-09-04):** the camera under the surface: a pure submersion test over the water rings, the `water_under` fullscreen pass (fog by the path through the water, clipped at the level), the ring's back face with Snell's window and the mirror beyond the critical angle, the droplet layer skipped while submerged, `submerged_frames` on the session line. Revision 1 (plan 13's absorption) failed the look by reading; revision 2 gives the underwater path its own clear-pool absorption. Roots unchanged from plan 32. Plan 31 item 1 is done; next is item 2 (water heard).
- **Plan 34 done (2026-09-04):** water heard through the accepted clip path: a `noise-loop` synth kind and two reference clips, looped flow emitters per edge record with the loudness class by flux band, splash cues on a box's upward threshold crossing at the level (presentation-only memory in the driver), a low-pass when the camera point is under a level. The roots moved (`play` `39527e4f…`, `persistence-replay` `a49a7267…`, `water-present` `9813212f…`) because the two clips move the project's composition lock, which the RPG aggregate of the authoritative state carries: every content change moves the roots, the water readings of the checks are identical. Content pins refreshed once. Plan 31 item 2 done; next is item 3 (wetness).
- **Plan 35 done (2026-09-04):** the wet band above every ring's level as a fullscreen pass of the water pass (darkening and a sun gloss from the depth normal within `15 cm` over the level inside the widened plan; the ring plans in a uniform of the water set). No root change. Plan 31 item 3 done; next is item 4 (gates, pumps, sources and sinks as play).
- **Plan 36 done (2026-09-04):** the gate lever: a reference-game water-gate system principal with the flow-control grant issues `WaterFlowCommandV1::SetGate` on its own stream (the tick as the sequence) when the player taps interact within `1 m` of the lever; the HUD prompts close/open; the driver test closes and reopens the gate through the command log. Roots moved (the lever body and the stream; the play ledger pin refreshed). Plan 31 item 4 first cut done (pumps, sources and sinks follow the same path); next is item 5 (tilting floats).
- **Plan 37 done (2026-09-04):** drift without tilt: ADR-105 revision 1.1 (the drag acts on the velocity relative to the cell's current, the currents from the flow network's fluxes) and SPEC-26 2.9 (dynamic boxes apply all three impulse components; a supported box keeps the push-only rule, an unsupported box sweeps `x` and `z`); D-008 amended. Contract test for the currents and the relative drag, world tests for drift, the wall and the resting box, a lattice drift scenario. Tilt and rotation stay an ADR of their own. Plan 31 item 5 first cut done; next is item 6 (waves that answer events).
- **Plan 38 done (2026-09-04):** waves that answer events: a wave grid per ring in the game's water feed (the ring's own vertices, the wave equation with reflective edges, damping, relaxation, a `15 mm` cap under the catalog's `20 mm`), excited by the stage's box and edge records; the stage stays pure and its digest unchanged. Plan 31 item 6 done; next is item 7 (big water).
- **Plan 39 done (2026-09-04):** the water showcase: the floor grows to `60 × 60 m`; a lake (`20 × 14 m`, level `2.25 m`) behind a dam with a spillway sill at `2.2 m`, a stream of three lattice terraces (`0.5`, `1.2`, `2.0 m` floors) falling into the pond, a spring into the lake and a drain from the pond (`3 L/s`) that hold the levels, banks with a `12`-step stair to the bank top; eight rings (the adapter's bound); `--start-at-falls` and `--start-at-lake`. Roots moved (content and authority); pins refreshed once. Plan 31 item 7 first cut done (the large-ring profile and LOD stay item 10).
- **Scene look roadmap recorded (2026-09-05):** the user reads the reference scene as "a bare Minecraft"; the look work outside the water (HDR chain, shadows, GTAO, TAA, PBR materials, environment, post) is its own task-state `scene-look.md` with the roadmap `docs/plans/look/00-scene-look-roadmap.md`.
- **Plan 42 done (2026-09-05):** far water (plan 31 item 10): a ring grid profile per surface (`WaterSurfaceGridV1`, the lake at `64 x 32`, `0.32 x 0.45 m` cells, the capacities at the large grid's) and the water pass's detail normal fading with distance (`12 / (12 + d)`). Play, persistence-replay and audio-scene roots unchanged; water-present stage cost `2 805 µs` mean in debug. Open: a camera-dependent ring resolution, a transparent-object sort (nothing to sort yet). Plan 31 items 1-10 done; items 11 and 12 (the player in the water, water as a mechanic) stay last per D-013.
- **Plan 41 done (2026-09-05):** water and the ground (plan 31 item 9): the `Seep` edge kind (ADR-103 1.1) carries water from a wetted cell into a ground-water cell at a capacity per plan area and stops when the table reaches the surface; the showcase closes its cycle (pond `48 µm/s` and lake `3 µm/s` seep into a hidden ground cell `0xf4`, the spring pumps `3 L/s` back into the lake; the six-cell total exact). Roots: play `64ac5dac81d8`, persistence-replay `7fb9966adcbc`, water-present `6fbf15e511fc`, audio-scene `ed0da7bedc3a`. Open: a per-material soak rate (no terrain material catalog), a well in the scene. Next: plan 31 item 10 (far water, the large-ring profile and LOD).
- **Plan 40 done (2026-09-04):** the PhysX lane recreates its fluid `300` frames after a failure (at most three attempts) and blends every particle's kernel with its own previous one through carried ids (flicker measure `26 ‰` on the pour demo); `recoveries` in the report. Open: the analysis on the GPU, an AMD host, the retention decision (ADR-106, D-012). Plan 31 item 8 done; next is item 9 (water and the ground).
- **Decided 2026-09-04 (D-013):** the player's own interaction with the water (buoyancy, swimming, wading, the player's wake) goes to the bottom of the roadmap; swimming will be a separately trained model on PD controllers over the SPEC-37 motors, and the water's forces on the body are built as that model's environment when the project asks.
- **Decided 2026-09-04 (D-012):** no player preference (SPEC-18) for the PhysX water lane for now; the run option stays the request until the lane's retention is decided.
- **Plan 23 run (2026-09-04):** `xtask physx setup` profile v2 builds `libPhysXGpu_64.so` from the pinned 5.9 sources with the host CUDA 13.3 (three recorded source patches for the 12.8-to-13 gap: architecture list, `cuCtxCreate`, nvcc-13 host stubs in the kernel wrangler); `xtask physx pbd-probe` (`PHYSX-WATER-PRESENT-R1`): RTX 3080, `16,384` fluid particles, `120` frames, step `1.5-1.8 ms` mean / `3.9-4.4 ms` max, readback `70-80 us`, fail-closed without the library, run-to-run positions differ; G4 FAIL by reading (`3` particles beyond the `0.1 m` margin at the worst frame). `physics-backend-parity` and the `physx-sdk` tests PASS on the v2 SDK; the bridge ABI stays `4` (the motor closure pins it). ADR-106 stays Proposed until the user reads the probe.
- **Plan 24 first cut (2026-09-04, developer demo):** a persistent GPU fluid in the bridge (`NativeFluid`) and the game feature `physx-water` with `--physx-water`: a block of PhysX water drops onto the basin's exact level inside the rim and splashes through the ADR-102 particle pass every frame; fail-closed to the stage's droplets without the GPU library. Run: `cargo run --release -p next_game --features physx-water -- --interactive --physx-water`. The lane's real increments (emission at edges, absorption at the surface, body boundaries, kernels, run option, statistics) are listed in plan 24 to be frozen.
- **Plan 25 done (2026-09-04):** the lane emits from the exact inputs (the crate's waterline while it moves, jets and falls inside the basin box) and absorbs settled particles on the level (`ne_physx_fluid_set`, `NativeFluid::set`); `--physx-water` runs the lane, `--physx-water-pour` adds the demo block. Clean `900`-frame session: peak `608` particles, all absorbed by the end, `764 us` mean per frame; unit tests for the emission and absorption rules. G6 partial: a sheet of particles lingers while the crate keeps bobbing. Plan 26 done (2026-09-04): the frame's floating boxes are kinematic colliders in the fluid (`set_box` / `clear_box`, `16` slots); the pour session shows `0` particles inside the crate at peak `5,270`, `781 us` mean per frame; roots and the present digest unchanged. Plan 27 done (2026-09-04): neighbour counts and cluster sizes from the fluid's own density (dense linked-cell grid, `853 us` mean at the splash) feed the ADR-102 spray layer (revision 3 thresholds); the splash renders as streaked droplets over a surface sheet. Plan 28 done (2026-09-04): the lane is in the game's run report (`RunReportV1::presentation_fluid`: presence, the probe's reason, bounded statistics), absent when not requested; a broken library path yields `active: false` with the bridge's reason and a passing run. Plan 29 done (2026-09-04): a fluid failure demotes the lane to the stage's droplets with the reason in the report (`--physx-water-fail-after` injects it), a device loss is survived (`--inject-device-loss-after-frames` passes the adapter's injection through); the adapter's reflection pass read the skinned vertex stream before it was prepared (fixed, recorded in plan 29). Plan 30 done (2026-09-04): anisotropic kernels from the fluid's weighted covariance (Yu and Turk) fill the particle pass's `kernels`; revision 1 (Jacobi, single thread) failed the 2.5 ms gate at 3.7 ms, revision 2 (closed-form eigen, cell-sorted sweep on up to four threads) passes at 2.2 ms with the same kernel definition. Open from plan 24: the ring's blending under a thick layer (the composite already attenuates the ring by the fluid's thickness; a separate increment only if a capture shows the ring through the block); the player preference is deferred by D-012 (accepted 2026-09-04) until the lane's retention is decided.
- **Plan `continuum-water/11` is complete** except the deferred items of its section 4 (gameplay gates/pumps, player buoyancy, the quiescence threshold D-010, SPEC-38 terrain housekeeping). Next candidates: raise the lattice bounds when a region needs them; the shallow-water presentation grid; pose interpolation for motion vectors (plan 18 note).
- Research note on engine and game water models:
  `docs/development/water-engines-research-2026-09-02.md`.
- `AGENTS.md`; routing rows "Physics world ..." and "Future continuum
  materials ..." in `docs/architecture/agent-routing.md`.
- ADR-100 / ADR-103 / ADR-104 / ADR-105 (Accepted 2026-09-03), ADR-101 / ADR-102 (Proposed), SPEC-26 2.8, SPEC-38 3.0, SPEC-39 0.1 (terrain), SPEC-03 2.12, SPEC-21 2.2; next actions in `docs/plans/continuum-water/11-next-actions.md`; water look research in `docs/development/water-look-dlss-research-2026-09-03.md`.
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
- **Amended 2026-09-04 (plan `continuum-water/37`, ADR-105 1.1, SPEC-26
  2.9):** a supported box (its downward sweep cut) keeps the push-only
  horizontal rule; an unsupported box applies all three impulse
  components and sweeps `x` then `z` against the same obstacles; the
  batch's drag acts on the velocity relative to the cell's current from
  the flow network. Rotation stays identity (tilt is an ADR of its own).
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

### D-010 — No quiescence threshold in the flux law for now (deferred)

- **Observation:** with the activity set of plan 20 the roots are
  identical to the always-stepped run, but the exact weir law with
  integer truncation reaches flux `0` only under about `6 um` of head,
  so settled sills rest minutes after visual settling (`59` percent at
  `20` minutes). A threshold ("a per-tick flux below `X` records `0`")
  would rest them in seconds.
- **Decision (user, 2026-09-03):** not now. The clause changes ADR-103's
  law and every recorded root, leaves a residual head "staircase" of the
  threshold's size, scales with cell area if stated in volume, chatters
  at the boundary, and would silently swallow weak sources, sinks and
  pumps unless they are excluded. At the current bounds the always-active
  step costs `17-44 us`, so the practice has nothing to save yet.
- **Recorded for later (plan 11 section 4):** if a lattice larger than
  the current `64` cells / `256` edges is authored, introduce the
  threshold as an authored network field (`0` = off, encoded only when
  non-zero so existing hashes stay), stated as a head in micrometres
  rather than a volume, with rate edges (source, sink, pump) exempt, and
  recorded as an ADR-103 revision with regenerated pins.

### D-012 — No player preference for the PhysX water lane for now (deferred)

- **Observation:** plan 24 item 4 named a player preference (SPEC-18)
  for the lane. `PlayerPreferenceProfileV1` is versioned local data with
  a fixed six-field canonical segment and a content hash; a new toggle
  means schema version 2, a migration of stored profiles, the store's
  import/export paths and SPEC-18 itself, all for a research lane whose
  retention ADR-106 leaves to the demo evidence.
- **Decision (user, 2026-09-04):** not now. The request stays on the run
  option (`--physx-water`, plans 25-30) until the lane is either retained
  as a shipped presentation option or dropped; the preference becomes its
  own plan when that decision is made.
- **Reconsider when:** the lane is retained for players (then SPEC-18
  gains the toggle, default off, with the schema bump).

### D-013 — The player's interaction with the water goes last (deferred)

- **Observation:** plan 31's first draft put the player in the water at
  the top (buoyancy and drag on the capsule, swimming as a SPEC-37
  locomotion mode, wading, climbing out) with the player's wake next.
- **Decision (user, 2026-09-04):** the player's interaction with the
  water is the lowest priority. Swimming will be a separately trained
  model driving the motors through PD controllers; the water supplies
  that model's environment (exact forces on the body, the immersion
  record) when the project needs it, and the roadmap's other items do
  not wait for it.
- **Reconsider when:** the swimming-model project starts and asks for
  the water's forces; then item 11 of plan 31 is frozen as its own plan.

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
   ADR-105, SPEC-26 2.8, SPEC-03 2.12, SPEC-38). Water V1 accepted
   2026-09-03. Next: plan `continuum-water/11` in order — the water look
   (L1-L8), the cost revisions, the scale practices; gameplay gates/pumps
   and player buoyancy deferred.
2. Optional gameplay effect: motor speed scaling from the classification
   (needs its own bounded evidence; not part of C's authority split).
3. Done 2026-09-03: ADR-100/103/104/105 accepted after the four `CONTINUUM-WATER-*` checks passed, with the
   SPEC/routing/traceability updates.
