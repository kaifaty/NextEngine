# ADR-103: Authoritative water flow network (cells, edges, exact integer step)

| Field | Value |
|---|---|
| ID | ADR-103 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-09-03 |
| Proposal date | 2026-09-02 |
| Last verified | 2026-09-03 |
| Normative dependencies | [SPEC-00](../00-product-contract.md), [SPEC-03](../03-assets-world-streaming-and-persistence.md), [SPEC-21](../21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-26](../26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-38](../38-continuum-material-physics.md), [ADR-046](046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-081](081-world-dynamics-gap-closure-and-promotion-guardrails.md), [ADR-100](100-authoritative-water-volume-and-presentation-only-gpu-water.md) |
| Supersedes | none (extends the ADR-100 `WaterVolume` owner) |
| Superseded by | none |

## Context

ADR-100 made `WaterVolume` the only authoritative water: sealed regions
with a still level and an authored ramp, changed by `SetLevel` commands.
That is enough for wading and swimming, not for water as a game mechanic:
dams and their failure, floodgates, channels and irrigation networks,
pumps and towers, communicating vessels and siphons, drought and
flooding. Those need water that moves between places under its own head
and stays exact, saved, replayed and cheap.

Particles cannot be that owner (ADR-100: `f32`, vendor toolchain, device
reproducibility), and they scale with volume. Water mechanics in shipped
games of this class (Timberborn is the reference the product names) run a
column or cell flow model on a grid or graph: volumes per cell, exchange
by head difference, deterministic, save-friendly, with cost proportional
to the number of cells and edges rather than to the amount of water.

The research lane already produced the calibration such a model needs:
discharge coefficient `0.40..0.44` for a wall opening flush with the
floor and `0.13` for a long lined four-cell duct (plan 22, NGQ8), exit
speed `0.5..0.55` of free fall for the head (revisions 5-9).

## Decision

### `WaterFlowNetworkV1` is the second authoritative water record

1. **Cells are `WaterVolume`s.** A flow network references existing
   `WaterVolumeDefinitionV1` records by id; each becomes a cell with a
   floor (`minimum.y`), a plan area (`(max.x - min.x) * (max.z - min.z)`)
   and a stored volume. The effective level of a cell is
   `floor + volume / area` in exact integer arithmetic (volume in cubic
   millimetres, area in square millimetres, level in micrometres). The
   R8c `SetLevel` command remains the authored override: it rewrites the
   committed level, and the cell's volume is recomputed from that level at
   the next flow step (`synced_record_revision`); cells cannot carry an
   authored ramp.
2. **Edges carry water by head.** An edge joins two cells (or a cell and
   the outside) and is one of:
   - `Open`: a sill of width `w` at height `s` (weir / free surface
     exchange); flow `q = c_w * w * sqrt(g) * h^(3/2)` with `h` the head
     of the higher level above the sill;
   - `Pipe`: an orifice of area `A` with invert height `s` and a
     discharge coefficient `c_d`; flow `q = c_d * A * sqrt(2 g dh)` with
     `dh` the difference of the two levels (or of the level and the
     invert when the lower side is below it); a pipe carries water in
     either direction, which is what makes communicating vessels
     equalise;
   - `Gate`: a `Pipe` scaled by an opening in permille that commands set;
   - `Pump`: a signed constant rate up to a maximum head, on or off by
     command;
   - `Source` / `Sink`: a constant rate from or to the outside (rain,
     inflow, evaporation, drain).
   Coefficients are profile constants in the network record, never code
   constants; the defaults are the research calibration (`c_d = 0.40`
   for short openings, `0.13` for long lined ducts, `c_w = 0.385`).
3. **One exact step per simulation tick.** The step is Jacobi: every
   edge flux is evaluated from the state at the start of the tick with
   integer square roots, each `Open`/`Pipe`/`Gate` flux is limited to the
   water available above the sill on its source side and to half the
   volume that would equalise the two levels (a `Pump` by its source
   cell's volume, a `Sink` by its cell's volume, a `Source` unbounded),
   then per cell the outgoing fluxes are scaled
   down (integer, largest-remainder) so a cell never goes negative, then
   all fluxes apply. Total volume changes only by sources and sinks and
   the difference is exact. Edges evaluate in ascending edge-id order,
   which also fixes the largest-remainder tie-break; there is
   no floating point anywhere, so `game` and `headless` produce identical
   roots and no execution profile is needed.
4. **Commands and events.** `WaterFlowCommandV1` (`SetGate`, `SetPump`,
   `SetSource`; the last sets the rate of a `Source` or a `Sink`) is the
   tenth command kind, in the water capability family, validated against
   the edge kind, opening range (`0..=1000`), rate range (`0..=1 m^3/s`)
   and record revision; each commit publishes `WaterFlowChangedV1` and
   bumps the edge revision. Unknown edge, wrong kind, stale revision,
   opening out of range, rate out of range and revision exhaustion are
   the six stable rejections.
5. **Checkpoint.** `WaterFlowNetworkV1` (the integration tick rate, edge
   definitions, per-edge state, per-cell volumes with the water-volume
   record revision each was last synchronised with) is field 5 of
   `PhysicsWorldCheckpointV1`,
   whose schema version becomes `3`. It rides the physics leaf of every
   state root, save segment and replay compare point; rigid backends only
   carry it. Bounds: at most `MAX_WATER_VOLUMES` cells and `256` edges
   per network, one network per world in this increment.
6. **Queries.** Gameplay keeps reading `WaterVolumeSetV1::submersion_at`;
   the network only moves the levels those queries see. Two additional
   exact queries: the flux of an edge over the last tick and the volume
   of a cell.
7. **Presentation is unchanged in authority.** The still surface, the
   ADR-101 ring and the ADR-102 particle pass read cell levels as they
   read the level today; a later presentation stage may spawn particles
   at edges (a jet at a pipe mouth, a fall over a sill) from the edge
   flux, never the other way round.

### What stays outside this decision

- A dense shallow-water grid over the whole map is a network whose cells
  are a regular lattice and whose edges are all `Open`; the record
  admits it, the first increment does not build it.
- Wave, ripple and flow presentation on cell surfaces is a later
  presentation increment under ADR-101/ADR-102.
- Coupling to rigid bodies (buoyancy, drag) is the
  `CONTINUUM-WATER-BUOYANCY-P1` consumer of ADR-104, decided by ADR-105
  with plan `continuum-water/08`.

### Later increments under this ADR (SPEC-38 2.1 practices)

Ordered by value, each with its own frozen plan and evidence:

1. `WaterFlowLatticeV1`: an authored regular lattice of cells with
   `Open` edges to the four neighbours, declared cell size and bound,
   materialised into the same network record (floods, channels,
   terrain-following water).
2. Activity-based stepping: a derived rest set so unchanged cells and
   edges are skipped without changing any root; rebuilt on restore.
3. Edge-driven presentation: the particle pass spawns jets and falls from
   gate, sill and pipe-mouth fluxes; the still surface and ring read cell
   levels.
4. `CONTINUUM-WATER-BUOYANCY-P1` (ADR-104): buoyancy and drag from exact
   cell levels through the one-pass reaction batch.
5. Rotational presentation: a presentation-only shallow-water grid fed by
   levels and fluxes, or an authored vortex around a `Sink` edge.
6. A wave layer over cell surfaces.

## Implementation (R8d, first increment, plan `continuum-water/07`)

- `crates/contracts/src/physics/water_flow.rs`: `WaterFlowEdgeKindV1`,
  `WaterFlowEdgeV1`, `WaterFlowEdgeStateV1`, `WaterFlowCellStateV1`,
  `WaterFlowNetworkV1` (`from_edges`, `validate`, `validate_against`,
  `apply_command`, `step`, canonical record, `cell_volume`, `edge_flux`,
  `total_volume`), exact helpers (`isqrt_i128`, `level_from_volume`,
  `volume_from_level`, largest-remainder scaling), `WaterFlowCommandV1`,
  `WaterFlowChangedV1`, `WaterFlowRejectionV1`.
- `PhysicsWorldCheckpointV1` schema `3` (segment `v3`, hash domain
  `nextengine.physics-world-checkpoint.v3`) with field 5; the network's
  cells must be declared volumes without an authored ramp; a cell whose
  volume state carries a newer record revision (an authored `SetLevel`)
  is resynchronised from the level at the next step; an overfull cell
  reads as full and keeps its excess for the following ticks.
- `WaterFlowCommandV1` is the tenth command kind (`core_r8d`, priority
  `291`, capability `nextengine.capability.water-flow-control`, either
  phase); the exact step runs in the physics owner's tick after the
  rigid step (`physics_step.rs`); the schedule manifest is unchanged.
- Reference scene: vessel A (`0x7e`, `2 x 1.5 m` on a `1 m` shelf, level
  `1.5 m`) and vessel B (`0x7f`, `2.8 x 1.5 m` on the floor, empty)
  joined by the gate `0x80` (`0.04 m^2`, invert `0.6 m`, `c_d = 0.40`),
  the source `0x81` and the sink `0x82` at `0.5 L/s`; both vessels sit
  east of the basin, away from the locomotion walk, and have no surface
  mesh yet.
- `xtask water-flow` runs `CONTINUUM-WATER-FLOW-P1` (plan 07 gates):
  `1,800` ticks, gate closed at tick `300` and reopened at `360`, save at
  `900`, restore and continue, four rejections, repeated generation.

## Consequences

- Water becomes a system the product can build mechanics on at constant
  cost per edge, saved and replayed with the world, without touching the
  presentation solvers.
- Every state root changes once (schema 3); the pinned goldens are
  refreshed in the same increment with the evidence.
- The step at the record bounds costs `183 us` in a release build (plan 07
  G6, `50 us` not met); it clones two maps and computes in `i128`, and is
  report-only until a consumer freezes a bound.
- SPEC-38 gains the network owner and the calibration table; SPEC-26
  gains the two queries; the routing and traceability rows for water name
  `CONTINUUM-WATER-FLOW-P1`.

## Product checks

| ID | Scenario | Expected behavior | Fallback |
|---|---|---|---|
| `CONTINUUM-WATER-FLOW-P1` | Two vessels at different levels joined by a pipe, a gate on the pipe, a source and a sink; step for a bounded number of ticks on `game` and `headless`, save at the middle, restore, continue, replay. | Total volume equals the initial volume plus sources minus sinks exactly at every tick; vessel A drains to within `1 mm` of its floor no later than twice the analytic Torricelli drain time plus the gate closure (the common-level case is the contract unit test `communicating_vessels_equalise_when_floors_allow`); closing the gate stops the exchange within one tick; identical roots live, restored and replayed; unknown edge, wrong kind, stale revision and out-of-range opening rejected without mutation; no presentation state is read. | The network is absent from the world and cells keep their authored levels (R8c behaviour). |

## Considered alternatives

- Particles as the mechanic owner: rejected by ADR-100 (not exact,
  device-bound, cost scales with volume).
- A floating-point shallow-water solver with a canonical execution
  profile: rejected for this increment; integer arithmetic gives the same
  mechanics without a profile and with exact conservation.
- Scripted levels only (R8c as is): rejected; it cannot express
  interaction between bodies of water.
