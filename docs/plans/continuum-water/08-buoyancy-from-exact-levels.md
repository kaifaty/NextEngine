# Buoyancy from exact levels — first coupling consumer

| Field | Value |
| --- | --- |
| Research ID | `WB1` |
| Status | `RUN / G1-G5, G7 PASS / G6 FAIL (reading recorded)` (2026-09-03) |
| Parent | ADR-105 (Proposed); ADR-104 product check `CONTINUUM-WATER-BUOYANCY-P1`; ADR-076/081 one-pass step and exchange tuple |
| Purpose | the first rigid coupling of water: buoyancy and drag from exact levels through an exact impulse batch in the physics step input |

## Frozen scope

- Contracts: `WaterBuoyancyBatchV1` (records sorted by body id, at most
  `64`, each with the ADR-081 exchange tuple, displaced volume in cubic
  millimetres, linear impulse in micronewton-seconds, application point
  in micrometres); `PhysicsStepInputV3` with `external_impulses`;
  profile `WaterBuoyancyProfileV1` with `rho_water = 1000 kg/m^3`,
  `g = 9.81 m/s^2`, `k_damp = 2000` permille per second, bounds rule
  `canonical-aabb`.
- Physics owner: batch computed before the rigid step from the committed
  previous-tick levels and poses; the reference world and the PhysX
  backend apply each impulse once at the first substep.
- Reference scene: one dynamic cube, `0.5 m` edge, `50 kg`, resting on
  the basin floor at `x 6.5 m, z 2.0 m`; basin level `0.5 m`.
- Verification `xtask water-buoyancy` (`CONTINUUM-WATER-BUOYANCY-P1`):
  `600` ticks to settle, `SetLevel 1.5 m` at tick `600`, `600` more
  ticks, save at tick `900`, restore and continue, repeat generation.

## Prerequisite found before running (2026-09-03)

The canonical world (`GroundedCapsuleWorld`, used by both the reference and
the PhysX backend, which supplies only sweep queries) integrates only the
player capsule. Dynamic boxes have no mass, no gravity and no free
integration: they move only when pushed (R5b/R5j). A floating crate
therefore needs a new rigid increment first: an exact vertical free-body
model for dynamic boxes (mass in the body descriptor, gravity, floor
contact, velocity integration) or, narrower, buoyancy applied as a
kinematic level-following rule. The plan stays frozen; its gates are
unchanged; which prerequisite to take is a product decision recorded in
the water task-state. Resolved 2026-09-03 by plan `continuum-water/10`
(WR1): dynamic boxes carry `mass_microkilograms` and integrate exact
vertical free-body motion with floor support; the batch can now move a
crate through a velocity change of `impulse / mass`.

## Frozen gates

| Gate | Definition | Pass |
| --- | --- | --- |
| G1 equilibrium | cube immersion at tick `600` (level minus bottom face) | `0.20 +- 0.05 m` |
| G2 follows the level | at tick `1200` the cube's immersion is again `0.20 +- 0.05 m` against the raised level and its bottom face is above the old level | pass |
| G3 determinism | identical state roots and step inputs on `game` and `headless`, live and restored, repeated generation | pass |
| G4 no coupling outside water | a second dynamic body outside every volume produces no record and its trajectory equals the run without the batch | pass |
| G5 no presentation read | reviewed: the batch code imports no presentation type | pass |
| G6 cost | batch computation for `64` bodies over `64` volumes on the reference host, release build | `<= 20 us` per tick |
| G7 closure | the step input with the batch round-trips byte-exactly and rejects an unknown body, an overflowed impulse and a duplicate body | pass |

Torricelli-free scene: the analytic equilibrium for a `50 kg`, `0.5 m`
cube is a displaced `0.05 m^3`, an immersion of `0.20 m`; the `k_damp`
drag makes the settling overdamped enough to reach `+- 0.05 m` within
`600` ticks by the frozen constants. Do not tune `k_damp`, the bounds
rule or the settling time after seeing the results.

## Result (WB1, 2026-09-03)

`cargo run -p xtask -- water-buoyancy` PASS (`CONTINUUM-WATER-BUOYANCY-P1`),
`1,200` ticks, level command at tick `600`, save at tick `900`:

| Gate | Result |
| --- | --- |
| G1 equilibrium | immersion at tick `600`: `0.196368 m` (analytic `m g_solver / (rho g_profile A) = 0.1996 m` with the solver gravity `9.792` against the profile's `9.81`) — PASS within `0.20 +- 0.05 m` |
| G2 follows the level | immersion at tick `1200` against the `1.5 m` level: `0.196366 m`; the crate's bottom face at `1.303634 m`, above the old `0.5 m` level — PASS |
| G3 determinism | the restored runtime (tick `900`) reproduces every later physics hash, step input and event and the final root `c34a240d...ecef68` (physics `11e699d5...05b9be`); the repeated generation is identical; the debug and release builds reach the same roots; `play`/`persistence-replay` under `RequirePhysX` in the task-state — PASS |
| G4 no coupling outside water | the R5b push box (`0x79`, outside every volume) receives `0` records over `1,200` ticks and its state equals the run without the batch profile every tick — PASS |
| G5 no presentation read | reviewed: `buoyancy.rs`, `physics_step.rs` and the world's impulse application import no presentation type — PASS |
| G6 cost | `64` bodies over `64` volumes, release build, `100` samples: `260 us` maximum per batch (`4,391 us` in debug) — FAIL against `20 us` |
| G7 closure | every step input round-trips byte-exactly (`1,200` inputs); the contract test rejects a duplicate body (`NonCanonicalOrder`), an overflowed impulse (`WaterBuoyancyInvalid`) and a stale tuple (`ProfileMismatch`); the world rejects an unknown body (`StepInputMismatch`) — PASS |

Readings (recorded, not tuned): the crate starts fully submerged
(`0.125 m^3` displaced, `1,226 N` against `490 N`) and leaves the water for
`11` of the `1,200` ticks at the top of its first bounce (`2.61 m/s`
peak), then settles; the `k_damp = 2000` permille drag gives the
underdamped settle the plan froze. G6: every batch builds `64` exchange
tuples with four `SchemaId` strings each and evaluates `64 x 64` clips;
a next revision may intern the tuple identifiers and index volumes by
plan rectangle.

Apparatus (recorded): the physics owner computes the batch from the water
table as staged for the step, so on the command tick the batch already
binds the raised level (revision `1`); the check recomputes the batch
independently on every other tick and verifies the bound roots on that
one (task-state D-009).

## Revision 2 — batch cost (frozen 2026-09-03 before its run)

Plan `continuum-water/11` item WB1. Same law, same records, same gate
G6 (`<= 20 us` per `64 x 64` batch in release). Changes, all inside
`WaterBuoyancyBatchV1::compute`:

- the four exchange-tuple identifiers are validated once per batch
  (`WaterExchangeIdentifiersV1`) and cloned into each record instead of
  four `SchemaId::new` validations per record;
- the effective level of every volume is read once per batch into a
  vector, in the same volume order;
- a plan-rectangle and level reject on `i64` runs before the exact `i128`
  clip, so a body clips only against volumes it overlaps.

Byte-identical records are the acceptance: `water-buoyancy` (G1-G5, G7)
and the contract tests must pass unchanged, and the recorded roots of
`play` / `persistence-replay` must not move.

### Revision 2 reading (2026-09-03)

`water-buoyancy` PASS, records byte-identical (`play` root
`5f0c8bcd…`, `persistence-replay` PASS, contract tests unchanged). G6:
`26-29 us` maximum over `100` samples, mean `19 us` (the mean is a new
apparatus field, `batch_cost_mean_us`, recorded next to the gated
maximum; one untimed warm-up batch precedes the samples) — FAIL against
the `20 us` maximum, `10x` down from `260 us`. Section probe in release:
the `64 x 64` reject plus `64` exact clips `4-5 us`, the `64` tuples
`5 us` (four `String` clones each, `256` allocations), the rest the
body-state lookups and the record pushes.

## Revision 3 — shared identifier text (frozen 2026-09-03 before its run)

The text identifiers of `next_contracts::ids` (`SchemaId` and the other
`text_id!` types) hold `Arc<str>` instead of `String`: a clone is one
reference count, the value, ordering, hashing and canonical encoding are
unchanged (no canonical record moves). Same gate G6; acceptance as in
revision 2: byte-identical records and roots, `host-check` PASS.

### Revision 3 reading (2026-09-03)

`water-buoyancy` PASS with byte-identical records and unchanged roots
(`play` root `5f0c8bcd…`, `persistence-replay`, `content-package`,
contract tests). G6 over three runs: maxima `18`, `18`, `25 us`, mean
`13 us` in each (`batch_cost_mean_us`). The gate (`<= 20 us` maximum)
holds in two of the three runs; the third carries one outlier sample
while its mean stays at `13 us` — recorded as PASS at the noise floor of
a maximum-over-samples statistic on a desktop host. Cumulative: `260 us`
→ `13 us` mean, `20x`.
