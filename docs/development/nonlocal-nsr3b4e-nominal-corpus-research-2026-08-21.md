# NSR3-B4E nominal-corpus research -- 2026-08-21

Status: `COMPLETE / STAGED_ALIGNMENT_AND_COST_LADDER_SELECTED / NO_TRAJECTORY_YET`

## Question

How can the packaged Nonlocal pressure candidate be compared with the newly
attested DFSPH references without confusing geometry, canonical publication,
solver diagnostics, execution cost and physical error in one long run?

## Current capability boundary

The B4C4C1 package proves the selected Newton--Krylov/KKT pressure solver,
joint fluid/static-support cell neighborhood, immutable support index,
single-owner flat CSR tape, adaptive macro transaction, micrometre canonical
publication and rollback on the tiny P1/P2 lanes. It does **not** contain a
6,000-sample nominal scenario command.

The underlying candidate capacities already admit the nominal shape:

| Capacity | Candidate limit | Hydro / Dam requirement |
|---|---:|---:|
| fluid samples | 50,000 | 6,000 / 6,000 |
| static support samples | 32,768 | 5,824 / 16,384 |
| neighbors per fluid sample | 160 | must be measured below 160 |
| pair storage | `160 * fluid` | must be measured below 960,000 |

The packaged path disables its all-pairs audit for candidate transactions, so
a nominal run can remain cell/CSR based. It nevertheless performs a 48-HVP
spectral estimate for an active macro frame, then up to four refinement levels
and many nonlinear HVPs. Capacity is therefore not a runtime-cost proof.

## Exact correspondence and deliberate exclusions

Hydro and Dam share the following physical inputs with the new DFSPH profile:

- 6,000 samples, `dx=0.05 m`, radius `0.025 m`, mass `0.125 kg` and
  `rho0=1000 kg/m^3`;
- `dt=1/240 s`, gravity `(0,-9.81,0) m/s^2` and zero initial velocity;
- identical initial centres and stable ID order `iy-iz-ix`;
- the same two-layer outer-complement coordinate set and hard centre
  clearance;
- Hydro `20x15x20` fluid in a 1 m cube and Dam the same fluid in a
  `4x1x1 m` box;
- exact output steps `0..1200/every24` and `0..720/every4`.

The following values are not comparable and receive no cross-solver gate:

- DFSPH pressure/divergence iterations and residuals versus Nonlocal
  trust/KKT/HVP counters;
- DFSPH density extrema versus the Nonlocal unilateral compression energy;
- per-particle position error after nonlinear free-surface evolution;
- raw boundary reaction values from different boundary formulations.

Orifice is excluded. The selected Nonlocal pressure candidate has only a
closed outer box; it has no sided internal support/aperture topology. Giving
it the DFSPH orifice curve would test missing B4O physics, not B4E pressure
water.

## Canonical observable bridge

Both sides are compared only at the same integer macro step. No interpolation,
time warp, fitted scale or discarded transient is allowed.

DFSPH binary64 positions are converted to signed micrometres with the same
ties-to-even canonical conversion used by the Nonlocal macro publisher.
Candidate and reference then use checked nearest-rank q99 with zero-based
index 5,939, exact sample count/mass and integer centre-of-mass sums. This
avoids making raw binary64 rounding or persistent particle identity a physical
metric.

Pre-observation thresholds retain the already accepted water-corpus scale:

- normalized curve RMSE at most 5%;
- maximum normalized curve error at most 10%;
- mechanical-energy and momentum residual at most 1%;
- maximum centre-clearance penetration at most 2.5 mm;
- Nonlocal positive density strain at most `1e-3` and selected KKT/ledger
  residual gates unchanged.

Dam compares normalized front
`min(4 m,q99(x)+0.025 m)/4 m` and height
`min(1 m,q99(y)+0.025 m)/1 m`. Hydro retains its analytical late COM gates
and additionally compares normalized q99 height with the same 5%/10% curve
budget. This new Hydro curve is independent of the late-COM condition and may
not be tuned after observation.

## Cost-aware execution ladder

1. **B4E0 alignment preflight:** no solver. Reconstruct both nominal
   geometries, exact IDs/projection roots, support coordinate sets, constants,
   schedules, initial canonical aggregates and flat-neighborhood capacity.
   Include ID, boundary-coordinate and schedule mutations.
2. **B4E1 one-macro resource probe:** run Hydro macro step 1 twice from fresh
   processes. Require deterministic physics/work roots and selected solver
   gates, but grant no DFSPH comparison credit because step 1 is not a
   reference output.
3. **B4E2 first-output pilots:** run Dam through step 4 first, then Hydro
   through step 24. Compare only their first reference outputs and repeat each
   candidate process. A first physics/capacity failure stops the ladder.
4. **B4EP cost decision:** use separately reported process wall/RSS facts to
   project the full 720/1,200-step runs. If the combined projection exceeds
   four machine-hours or peak RSS exceeds 4 GiB, classify
   `PERFORMANCE_REMEDIATION_REQUIRED`; do not call it a physics failure and do
   not start a long run.
5. **B4E3 full Hydro:** execute twice, require byte-identical deterministic
   reports and all analytical/reference gates.
6. **B4E4 full Dam:** execute twice only after Hydro passes; require the front
   and height curve gates.

Free-fall, still-tank, sealed and storage-order controls remain required for a
later complete nominal pressure corpus, but do not block the first external
DFSPH comparison. They have analytical/self-reference authority rather than
new external payloads. Orifice remains B4O.

Wall time is an execution-scheduling discriminator, not a physical tolerance.
Timing does not enter deterministic reports, and the four-hour boundary may
redirect work to B4EP optimization but cannot turn a physical FAIL into PASS.

## Decision

Freeze B4E0 as an exact zero-trajectory alignment gate. Do not add the nominal
trajectory command, parse a reference curve or execute a solver until that
preflight passes twice. A B4E0 PASS may authorize only B4E1 research and
contract design.
