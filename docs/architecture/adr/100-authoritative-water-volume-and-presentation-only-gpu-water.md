# ADR-100: Authoritative water volume and presentation-only GPU water dynamics

| Field | Value |
|---|---|
| ID | ADR-100 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-09-03 |
| Proposal date | 2026-09-02 |
| Last verified | 2026-09-03 |
| Normative dependencies | [SPEC-00](../00-product-contract.md), [SPEC-01](../01-system-architecture.md), [SPEC-04](../04-rendering-and-platform.md), [SPEC-21](../21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-26](../26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-30](../30-presentation-extraction-and-render-content.md), [SPEC-38](../38-continuum-material-physics.md), [ADR-003](003-vulkan-renderer-and-shader-toolchain.md), [ADR-028](028-platform-session-and-presentation-authority.md), [ADR-046](046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-076](076-continuum-material-physics-track.md), [ADR-081](081-world-dynamics-gap-closure-and-promotion-guardrails.md), [ADR-090](090-linux-only-v1-and-indefinitely-deferred-windows.md), [ADR-101](101-presentation-only-dynamic-surface-ring.md) |
| Supersedes | ADR-076 clauses "GPU DFSPH is optional correspondence-only work", the crate-coupled first consumer and "debug points/spheres are sufficient presentation", for water V1 only |
| Superseded by | none |

## Context

ADR-076 selected CPU `f64` DFSPH as the sole canonical water candidate and
allowed GPU work only as a correspondence mirror. It also fixed the exit: if
`50k` misses the `4/6 ms` standalone stop target after two evidence-backed
optimization cycles, the track remains research-only and GPU authority
requires an explicit new decision.

The evidence since then:

- the serial CPU reference passed `CONTINUUM-WATER-REF-P1` on Linux, but its
  W2 scaling stops at about `270 ms` per sealed-48k step with eight workers,
  `67x` above the target (`docs/roadmap.md`, water row);
- the original five-iteration Nonlocal GPU model steps `48,000` samples in
  `3.9--5.0 ms` on the RTX 3080 witness, passed a bounded dynamic visual
  corpus, and now drives the production SDL3/Ash Vulkan renderer live through
  a one-directional process boundary (task-state
  `nonlocal-gpu-full-step-performance`, D-038 through D-048);
- watching that live surface exposed and then resolved a wall stall that the
  accepted corpus never covered: boundary density support is required, and
  fixed boundary samples must support density only.

That GPU model is `f32`, executed by a vendor toolchain (CUDA) outside the
Cargo workspace, and reproducible only on the same device and driver. Making
it gameplay authority would require a GPU `CanonicalFloatExecutionProfile`,
a Vulkan compute port and same-device-only replay, none of which the product
needs for the first water experience.

## Decision

### Water V1 has two owners with one direction of flow

1. **`WaterVolume` is the authoritative water.** Physical Embodiment owns a
   bounded set of sealed water regions in the portable CPU core. Each region
   binds an exact integer extent, a still-water level, an optional authored
   level ramp and a profile revision. Gameplay queries (submersion depth
   at a point, wading/swimming classification, buoyancy for a later
   consumer) read only this volume through exact snapshot-bound water
   queries on the physics owner, in the SPEC-26 query discipline. Its state
   changes only through validated `WorldCommand` transactions (and, for
   volumes that are cells of the ADR-103 network, through that network's
   exact per-tick step), publishes exact roots, saves and replays with the
   world and needs no floating-point execution profile.
2. **Presentation water is non-authoritative.** A separate presentation
   dynamics stage may animate the free surface of a `WaterVolume` with any
   solver, including the Nonlocal GPU candidate, and publish a bounded
   surface (height field plus mask over a fixed pixel grid) to the renderer
   through the ADR-101 dynamic surface path. Nothing flows back: no command,
   event, query, save, reaction or root reads presentation water. Renderer
   cadence, device loss, driver, GPU vendor, missing capability or a stopped
   presentation process cannot change gameplay; the fallback is the still
   surface of the authoritative level.

### Presentation dynamics reach the engine only through engine-owned boundaries

The research CUDA solver stays outside the Cargo workspace as developer
tooling; its stream is a versioned neutral artifact across a process boundary
(ADR-005 style). A shipped presentation solver must run in the Vulkan render
backend under SPEC-04/ADR-003 (compute in the same B0/E1 capability tiers) or
through the same neutral artifact boundary; no CUDA, vendor or process
dependency enters `game`, `headless` or public contracts. `headless` never
executes presentation water.

### Frozen findings that any water solver candidate inherits

- Floor and wall neighbourhoods need boundary density support. Fixed boundary
  samples contribute to density and the incompressibility term only;
  viscosity and surface terms skip them. Analytic contact alone is
  insufficient (evidence: D-047/D-048).
- The candidate presentation surface is a top-down height field over a fixed
  pixel grid of one quarter particle spacing: sphere-cap projection, every
  8-connected component of at least one particle footprint (the accepted
  research corpus keeps the earlier largest-component stage and its roots),
  one 3x3 close, local fill, a 5x5 grayscale closing of the height and one
  bilateral pass with range sigma equal to the particle radius. It cannot represent overhangs or spray; those remain optional later
  stages. GPU and CPU implementations are equivalent when masks and mesh
  counts are identical and depths agree within `1 um`.

### First consumer

The first water consumer becomes one sealed basin with an authoritative
`WaterVolume`, player wading/swimming classification from its queries, and
live presentation dynamics on its surface. The PhysX crate coupling of
ADR-076 moves to a later consumer under its own ADR; until then no continuum
reaction batch exists and PhysX remains the sole rigid writer.

## Implementation (R8c, first increment)

- `WaterVolumeSetV1` (definitions plus per-volume record state) is field 4
  of `PhysicsWorldCheckpointV1`, whose schema version became `2` in R8c
  (segment `v2`, hash domain `nextengine.physics-world-checkpoint.v2`) and
  is `3` since ADR-103 (segment `v3`, domain `...v3`). It therefore
  enters the physics leaf of every state root, save segment and replay
  compare point without a new owner segment. Rigid backends only carry it.
- `WaterVolumeCommandV1::SetLevel` is the ninth command kind
  (`core_r8c`, priority `290`, capability
  `nextengine.capability.water-volume-level`, either phase, no subject
  target). It commits `WaterVolumeChangedV1`, bumps the record revision and
  suspends the authored ramp; unknown volume, stale revision, level outside
  the extent and revision exhaustion are stable rejections.
- `WaterVolumeSetV1::submersion_at(point, tick)` is the exact query:
  volumes are disjoint, depth is `level - y`, a depth at or above the
  authored swimming depth classifies as `Swimming`, otherwise `Wading`, a
  point above the level or outside every volume is `Dry`.
- The reference scene declares one basin (`4 x 2 m`, `2 m` deep, level
  `0.5 m`, swimming depth `1.2 m`) east of the rock proxies.
- The player consumer classifies the capsule foot point every published
  frame through the same query; `Wading`/`Swimming` ride the HUD status
  panel as a label and never write gameplay state.
- The still-surface fallback is real: an authored basin quad
  (`4 x 2 m`, catalog mesh `0x7c`, material `0x7d`) is bound as an
  environment record whose translation is the exact effective level of the
  authoritative basin at the published tick. It renders through the ordinary
  presentation snapshot; an ADR-101 ring may later animate the same catalog
  mesh without touching its identity or bounds.
- `xtask water-volume` runs `CONTINUUM-WATER-VOLUME-P1` including the
  production locomotion walk into the basin, the classification before and
  after a level command and the surface binding translation.
- WP1 (plan `continuum-water/09`): the presentation stage
  `compute_water_presentation_frame` in the reference game crate is a
  pure function of the committed checkpoint and a frame index (surfaces
  following the exact levels with a flux-driven ripple capped at `20 mm`,
  a stateless ballistic jet from the exact gate flux); the interactive
  worker publishes it beside the snapshot and `apps/game` feeds the
  ADR-101 ring (three quads) and the ADR-102 particle pass.
  `xtask water-present` runs `CONTINUUM-WATER-PRESENT-P1` (PASS).

## Consequences

- SPEC-38 gains the two-owner authority split, the density-only boundary
  rule, the presentation surface path and the new first consumer; the CPU
  DFSPH reference remains the research oracle lane, not a shipping promise.
- SPEC-26 records the water table inside the physics world checkpoint and
  the exact water query; SPEC-03 records the checkpoint schema bump. Pre-R8c
  saves are current-only alpha artifacts under ADR-046 and are rejected
  before mutation.
- SPEC-30/SPEC-04 admit the ADR-101 dynamic surface path and developer frame
  capture as presentation-private mechanisms.
- The `4/6 ms` standalone stop target now applies to the presentation
  solver's own budget row, not to gameplay correctness; a missed budget lowers
  surface cadence or sample count, never gameplay.
- Determinism, save, replay and cross-target roots cover `WaterVolume` and,
  since ADR-103, `WaterFlowNetworkV1`.

## Product checks

| ID | Scenario | Expected behavior | Fallback |
|---|---|---|---|
| `CONTINUUM-WATER-VOLUME-P1` | `xtask water-volume`: activate the reference basin, probe authored points, raise the level through the production command path, reject stale/out-of-extent/unknown commands, round-trip the physics checkpoint, restore and continue. | Exact probe table, one event per commit, unchanged table on rejection, byte-exact checkpoint, restored and live runs reach the same root, repeated generation identical; `play`/`persistence-replay` keep `game`/`headless` parity. | Reject the region before activation; keep the dry variant. |
| `CONTINUUM-WATER-PRESENT-P1` | `xtask water-present`: compute the presentation stage from the committed checkpoint for `600` ticks on one runtime and compare with a runtime that never sees it; drive the game root with the stage for a bounded capture. | Gameplay roots identical every tick with and without the stage, every update within the declared capacities and bounds, the stage pure and within `1 ms` per frame, one catalog/frame plan per run, capture diagnostic only (PASS, WP1; the human look gate is a walk to the vessels with `--capture-frame`/`--capture-png`). | Still surface at the authoritative level. |

## Considered alternatives

- GPU-canonical water authority — rejected for V1: same-device-only
  reproducibility and a vendor toolchain contradict SPEC-21 exactness and
  the Rust/Vulkan portable core; it stays a possible later ADR.
- Keep CPU DFSPH as the only path — rejected as the product answer: `67x`
  over budget after two cycles leaves no playable water.
- Presentation water feeding buoyancy or contact — rejected: it would make
  renderer/GPU state gameplay authority.
