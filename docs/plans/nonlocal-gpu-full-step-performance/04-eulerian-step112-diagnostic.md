# NCGP7 Eulerian step-112 field diagnostic contract

| Field | Value |
| --- | --- |
| Research ID | `NCGP7` revision 1 |
| Status | `FROZEN / DIAGNOSTIC_ONLY / REPORT_ONLY` |
| User decision | Explicitly authorized on 2026-08-31 after NCGP6 |
| Architecture snapshot | commit `3aa914159ca34be5fab4ba9f54ed81a2ef9841ab`, tree `ce3eaf6e7ab6c394e955ebf0d2935d69540925ec`; SPEC-38/ADR-076 Proposed; ADR-081 Accepted guardrails |
| Frozen parent | NCGP6 `REFUTED_BOUNDED`, exact hydrostatic first failure step 112 |
| Engineering consumer | decide whether a later product gate should compare macroscopic water quantities rather than stable particle identities |
| Claim class | finite/profile-bound diagnostic and hypothesis discrimination |
| Budget | one field apparatus, one exact witness replay, two clean reproductions and one independent read-only review before any positive product decision |

## Exact question and claim ceiling

NCGP6 establishes that at hydrostatic step 112 the stable-ID position p99 is
`2.576882866 mm > 2.5 mm`, while position RMSE is `0.780996279 mm`, the first
same-state step differs by at most `0.113995 um`, corrected/permuted GPU state
is exact and all retained physical invariants pass.

NCGP7 asks only whether the CPU and corrected GPU step-112 states represent
the same macroscopic water distribution on two predeclared fixed spatial
grids. It does not change or waive NCGP4/NCGP6, select a production metric,
admit dam/orifice/50k, or permit performance timing. A positive diagnostic can
only support asking for a separate product decision.

## Frozen witness and arithmetic

Replay the exact NCGP6 hydrostatic input from step 0 through accepted step 112
using the unchanged corrected FCR2 profile, canonical binary32 `(hi,lo)` GPU
state, independent CPU long-double solver, unpreconditioned Steihaug--Toint
rules and total HVP budget 128. The corrected and stable-ID-permuted GPU routes
must remain exact. No trajectory tolerance participates in this replay: the
known NCGP6 p99 failure is the witness, not a reason to stop before depositing
its accepted state.

Field construction is diagnostic host work after the accepted state snapshot.
It is outside every future GPU hot-step timing window. Inputs, transfers,
field work and all results are sealed separately.

## Canonical Eulerian grids

Use two node-centred Cartesian grids over the complete analytical basin
`[0,3.0] x [0,2.5] x [0,1.5] m`:

```text
coarse spacing = 0.05 m;  dimensions = 61 x 51 x 31
fine spacing   = 0.025 m; dimensions = 121 x 101 x 61
```

For each finite in-basin sample position, use trilinear cloud-in-cell weights
to its eight enclosing nodes. Deposit, in SI units:

```text
node_mass            += mass * weight
node_density_moment  += mass * density * weight
node_momentum        += mass * velocity * weight
```

Create explicit contribution records and sort them by node index followed by
the binary64 position/density/velocity value tuple. `SampleId` is excluded
from this ordering and from field identity. Sum each channel in `long double`
and round once to binary64 for the field record. This makes the diagnostic
independent of particle naming without making the simulation state itself
identity-free.

Each field must conserve its input mass within `8 * epsilon_binary64` relative
error. Nonfinite input, out-of-basin input, invalid dimensions, failed mass
closure or an empty field is an apparatus failure, not evidence for either
physical hypothesis.

## Field observables

For GPU field `G` and CPU field `C`, total mass `M`, sample mass `m`, rest
density `rho0`, and characteristic velocity `spacing/dt = 12 m/s`, compute:

```text
mass_tv = 0.5 * sum_nodes(abs(G.mass - C.mass)) / M

density_rmse = sqrt(
  sum_common min(G.mass,C.mass) * (G.rho-C.rho)^2
  / sum_common min(G.mass,C.mass)) / rho0

velocity_rmse_normalized = sqrt(
  sum_common min(G.mass,C.mass) * |G.velocity-C.velocity|^2
  / sum_common min(G.mass,C.mass)) / (spacing/dt)

center_of_mass_error = |G.center_of_mass - C.center_of_mass|
```

A node is common when both masses are at least `m/4`. Report the maximum node
mass difference in units of `m`, but do not use that single-node maximum to
select a hypothesis.

## Canonical free-surface map

Independently bilinearly deposit each sample mass to the four enclosing `x-z`
grid columns. Sort contributions by column, height and the remaining binary64
value tuple, excluding `SampleId`. A column is wet when its deposited mass is
at least `m/4`. Its surface height is the smallest sample-centre `y` whose
ascending cumulative column weight reaches 99% of that column mass.

Compare the wet-column sets by symmetric-difference/union fraction. On the
common wet columns report nearest-rank surface-height RMSE, p95 and maximum.
Empty common support or failed column-mass closure is an apparatus failure.

## Competing hypotheses and predeclared bands

The bands are diagnostic, not a product acceptance contract. They are tied to
the existing 5 cm sample spacing and 5% physical-field ceiling rather than to
the observed step-112 values.

### `H7A_SUPPORTED_BOUNDED` — identity separation with close fields

Both grids must satisfy all of:

```text
mass_tv                         <= 1%
density_rmse                    <= 1%
velocity_rmse_normalized        <= 1%   // <= 0.12 m/s
center_of_mass_error            <= 2.5 mm
wet-column symmetric difference <= 1%
surface-height RMSE             <= 12.5 mm
surface-height p95              <= 25 mm
```

The exact GPU permutation, field mass, retained momentum/energy/containment
observables and all identity/work roots must also pass.

### `H7B_SUPPORTED_BOUNDED` — clear macroscopic divergence

The coarse 5 cm grid selects H7B if any of these clear-error bands is reached:

```text
mass_tv                         >= 5%
density_rmse                    >= 5%
velocity_rmse_normalized        >= 5%
center_of_mass_error            >= 25 mm
wet-column symmetric difference >= 5%
surface-height RMSE             >= 50 mm
surface-height p95              >= 100 mm
```

### `H7C_INCONCLUSIVE` — resolution/threshold sensitivity

Every other valid result, including one grid satisfying the H7A band while
the other does not, is H7C. No interpolation, third resolution, sample
deletion, phase alignment or retry may turn H7C into H7A inside revision 1.

## Apparatus controls

Before the witness replay, require:

1. identical input fields produce bit-exact roots and zero comparison metrics;
2. arbitrary stable-ID relabelling with unchanged physical samples preserves
   both grid and surface roots;
3. a whole-state `+0.05 m` vertical translation, kept inside the basin,
   selects H7B rather than H7A;
4. deleting one sample fails exact particle/mass closure;
5. a grid-spacing mutation, omitted density or momentum channel, changed
   surface quantile, field-record mutation and work-count mutation each
   change or invalidate the result;
6. corrected/permuted GPU state and Eulerian roots are exact on the witness.

Controls consume real field construction and comparison. Merely binding a
variant label into an expected hash is insufficient.

## Work and evidence closure

Seal for each route and resolution:

- state, density and active-signature input roots;
- grid dimensions/spacing and all node/column record roots;
- samples validated, CIC records emitted/sorted/reduced, node comparisons,
  column records emitted/sorted/reduced, quantile reads and root derivations;
- every metric and hypothesis predicate;
- profile/input/source commit/tree/binary/compiler/environment identity;
- exact command, GPU/CPU step work, snapshot transfers and allocated bytes.

The result schema is versioned. Two clean Release builds and byte-identical
replays are required before interpretation. One independent read-only review
may request one batched apparatus repair and one re-review. A remaining
load-bearing defect closes NCGP7 `INCONCLUSIVE`.

## Stop and promotion boundary

- NCGP6 remains `REFUTED_BOUNDED` under its own frozen p99 gate.
- NCGP7 does not run dam-break, orifice, 16k, 50k or timing.
- Do not change physics, solver, HVP ceiling, stable IDs or NCGP6 tolerances.
- H7A permits only a new explicit product decision on quantities of interest.
- H7B keeps the physical/correspondence blocker.
- H7C requires a new diagnostic revision, not an in-place tolerance change.
- CPU DFSPH remains the product fallback; no public/runtime/PhysX/renderer
  contract changes.
