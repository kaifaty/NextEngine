# Water flow network — first increment (communicating vessels)

| Field | Value |
| --- | --- |
| Research ID | `WF1` |
| Status | `FROZEN / NOT_RUN` |
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
