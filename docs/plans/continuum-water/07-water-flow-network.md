# Water flow network — first increment (communicating vessels)

| Field | Value |
| --- | --- |
| Research ID | `WF1` |
| Status | `RUN / G1-G7 PASS after revision 2 (G6 41-46 us release) / CONTINUUM-WATER-FLOW-P1 = PASS` |
| Parent | ADR-103 (Proposed); ADR-100 R8c `WaterVolume`; calibration from plan `nonlocal-gpu-full-step-performance/22` |
| Purpose | authoritative, exact, cheap water mechanics: cells joined by edges that move water by head |

## Frozen scope

- Contracts: `WaterFlowNetworkV1` (cells by `WaterVolume` id, edges
  `Open | Pipe | Gate | Pump | Source | Sink`, per-edge state, per-cell
  volume in cubic millimetres), exact Jacobi step per simulation tick,
  `WaterFlowCommandV1` (`SetGate`, `SetPump`, `SetSource`),
  `WaterFlowChangedV1`; field 5 of `PhysicsWorldCheckpointV1`, schema 3.
- Flux laws (integer, `g = 9.81 m/s^2` as `9_810_000 um/s^2`):
  - `Pipe`/`Gate`: `q = c_d * A * isqrt(2 g dh)` per tick, `c_d` in
    permille from the profile (default `400`; `130` for long lined ducts);
  - `Open`: `q = c_w * w * isqrt(g * h^3)` with `c_w = 385` permille;
  - limits: the water above the sill on the source side, and half the
    volume that equalises the two levels; per-cell largest-remainder
    scaling of outflows.
- Reference scene: vessel A `2 x 1.5 m` plan, floor `1.0 m`, level
  `1.5 m`; vessel B `2.8 x 1.5 m` plan, floor `0 m`, empty; one `Gate`
  pipe of `0.04 m^2` with invert at `0.6 m` above B's floor, `c_d = 400`;
  one `Source` into A at `0.5 L/s`, one `Sink` from B at `0.5 L/s`.
- Verification `xtask water-flow` (`CONTINUUM-WATER-FLOW-P1`): 1,200
  ticks, save at tick 600, restore and continue, replay; commands: close
  the gate at tick 300, reopen at tick 360.

## Frozen gates

| Gate | Definition | Pass |
| --- | --- | --- |
| G1 conservation | at every tick `sum(volumes) == initial + sources - sinks` exactly (cubic millimetres) | exact |
| G2 convergence | with the gate open, the two levels differ by `<= 1 mm` no later than tick `T = 2 * tau`, where `tau` is the analytic emptying time of the Torricelli law for the pipe at the initial head, and the common level equals the volume-weighted level within `1 mm` | pass |
| G3 gate | after `SetGate(0)` the pipe flux is `0` from the next tick; after reopening it resumes | pass |
| G4 determinism | live, restored-and-continued and replayed state roots identical on `game` and `headless` | pass |
| G5 rejections | unknown edge, wrong kind, stale revision, opening `> 1000` are rejected without any state change | pass |
| G6 cost | `step` for `64` cells and `256` edges on the reference host | `<= 50 us` |
| G7 no presentation read | the step and the commands compile without access to any presentation type (reviewed) | pass |

Do not change the flux laws, limits or coefficients after seeing the
results; a coefficient change is a new revision with its own evidence.

## Result (R8d, 2026-09-02)

`cargo run -p xtask -- water-flow` PASS (`CONTINUUM-WATER-FLOW-P1`):

| Gate | Result |
| --- | --- |
| G1 conservation | exact at every one of `1,800` ticks: initial `1,500,000,000 mm^3`, sources `29,998,800`, sinks `29,982,134`, final `1,500,016,666` — PASS |
| G2 convergence | vessel A at its floor (`1.000005 m`, the head that passes the `0.5 L/s` source) by tick `868`; analytic bound `840` ticks (Torricelli drain `26.8 s` plus the `2 s` gate closure), twice the bound `1,680` — PASS; vessel B ends at `0.357 m`, the level of all the water over its plan area |
| G3 gate | flux `0` from the tick after `SetGate(0)`, positive again after the reopen — PASS |
| G4 determinism | physics checkpoint round-trips byte-exactly at tick `900`; the restored runtime's per-tick physics hashes, events and final state root equal the live run's (`41e84ebc...`, physics `e93a8c35...`); the repeated generation is identical — PASS |
| G5 rejections | unknown edge, pump command on a gate, stale revision, opening `1001`: four stable rejections, no event, edge states unchanged — PASS |
| G6 cost | `64` cells / `256` edges: `183 us` maximum per step in a release build (`1,405 us` in debug) — FAIL against `50 us` |
| G7 no presentation read | reviewed: `water_flow.rs`, `physics_step.rs` and the command arm import no presentation type — PASS |

Apparatus corrections (recorded, not tuned): the plan's `1,200` ticks were
shorter than twice the analytic drain time of this scene, so the run is
`1,800` ticks with the save at `900`; the G2 wording assumed a common
level, but vessel A's floor (`1.0 m`) is above the level all the water
reaches in B (`0.357 m`), so the equilibrium of this scene is A drained to
its floor, which is what the gate now checks (the contract unit test
`communicating_vessels_equalise_when_floors_allow` covers the common-level
case with both floors at `0`: `0.625 m` within `1 mm`).

G6 reading: the step clones the network and the volume set (two
`BTreeMap`s of `64` and `256` entries) and evaluates the fluxes in `i128`;
`183 us` at `30 Hz` is `0.5%` of a frame. Candidates for a next revision:
in-place stepping and `i64` fast paths.

## Revision 2 — step cost (frozen 2026-09-03 before its run)

Plan `continuum-water/11` item R8d. Same law, same fluxes and levels,
same gate G6 (`<= 50 us` per step for `64` cells and `256` edges in
release). Changes:

- `isqrt_i128` uses the standard library's integer square root
  (`i128::isqrt`, the same floor value) instead of a 64-iteration binary
  search per edge;
- `WaterFlowNetworkV1::step_in_place(&mut self, &mut WaterVolumeSetV1)`
  steps without cloning the network and the volume set: the cells live
  in a vector for the step, per-cell outflow lists are vectors, the
  edge states and cell states are written in place, and the projected
  levels are checked against their volume extents directly (the only
  state-dependent clause of `WaterVolumeSetV1::validate`; the
  definitions and key sets do not change in a step). `step(&self, ..)`
  stays as clone plus `step_in_place` and returns the same value;
- the runtime's physics step and the G6 harness use `step_in_place`
  (apparatus: G6 measures the path the runtime runs).

Acceptance: the contract tests (including a new equality test of `step`
against `step_in_place` and of the two square roots on large radicands),
`water-flow` G1-G5 and G7 PASS with unchanged roots, `play` /
`persistence-replay` PASS.

### Revision 2 reading (2026-09-03)

Contract tests PASS, including the new equality of `step` and
`step_in_place` over `240` equalising steps and of the standard square
root against the former binary search up to `2^100`. `water-flow` PASS
with the final root `0d3bdf1c…` identical across runs; `play` /
`persistence-replay` PASS. G6 over three runs: maxima `43`, `46`,
`41 us`, mean `29`, `29`, `28 us` (`step_cost_mean_us`, new apparatus
field) — PASS against `50 us`, down from `183 us`. One implementation
detail inside the frozen scope, applied after a first reading of
`49-57 us`: the edge states are walked in lockstep with the edges
instead of one map lookup per edge (the two maps share a key set).
