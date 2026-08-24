# NSR3-B4E2D7R19R30 linearized range-projection evidence

Date: `2026-08-24`

Status: `PASS / LINEARIZED_RANGE_PROJECTION_CANDIDATE / REPORT ONLY`

## Outcome

R30 closes a matrix-free, zero-damping LSMR projection over the exact R29
operator restricted to the 1,420 violated R28 rows. Exact scalar column-norm
scaling leaves all 18,000 coordinate columns nonzero. Four analytic dense
controls pass before the nominal solve.

```text
LSMR stop                         COMPATIBLE
iterations                        388 / 512
pair passes                       780 / 1028
||b||                             8.1139951163476689e-08
||q||                             5.0778391645710737e-16
||q|| / ||b||                     6.2581245018751602e-09
||C^T q||                         1.3912509730218889e-16
cross-energy orthogonality        2.6757413893334442e-13
Pythagorean relative defect       5.3514799548257377e-13
```

The direct compatible rule passes. Estimated normal residual never increases.
The candidate therefore finds no substantial component of the violated-row
RHS outside `Range(restrict_violated(SPACING * Jc))` at the frozen numerical
accuracy.

## Globalization boundary

Range compatibility does not make the iterate an admissible correction:

```text
dimensionless preimage RMS        12966.351593601192
dimensionless preimage maximum    510946.65161832742
new positive inactive rows        450
maximum predicted positive        178317.72255805245
```

The full-row response shows severe active-set leakage. R30 does not apply the
preimage, evaluate a moved nonlinear state, classify a floor or authorize a
following outer. The next causal question is whether a predeclared
fraction-to-boundary rule permits meaningful predicted progress along this
direction. If not, a constrained inequality active-set/trust-region
subproblem is required instead of scalar damping.

## First diagnostic retained

The first nominal run hard-failed the original angle-cosine orthogonality
check even though `q` was roundoff-sized and the Pythagorean defect passed.
The exact values and repair rationale are retained in the
[first-diagnostic record](nonlocal-nsr3b4e2d7r19r30-first-diagnostic-2026-08-24.md).
The repair changed only normalization to the cross-energy
`|p^Tq|/max(||b||^2,tiny)`; the `1e-10` gate, solver tolerances, scaling and
iteration cap are unchanged. Raw angle cosine remains report-only.

## Reproducibility

Research/contract commit: `1e439397`.

Implementation commit: `b1eb1dde`.

Two clean Release builds:

- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r30-a.BKJHXC`;
- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r30-b.Qf0lo6`.

Both binaries are `7,006,400` bytes, have SHA-256
`931ec8bf8c44bd013175134cee3cdcc8955c987045106fd79617f36b9e2052d6`
and GNU build ID `2053bb7c07c3e79bc60fa841736c964944cc38d3`.

Fresh one-process outputs:

- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r30-a.sG1p3F`;
- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r30-b.6M1n7j`.

Both exit `0`, have empty stderr and reproduce `3,099` stdout bytes exactly:

```text
stdout SHA-256  34cf7a56d5c449dca20bf1ab800e061f62bfa0e8145996a6c032e080519596ad
semantic        41c3e833a7c618281d94219e059bfbab85cc6631f2bc916478856484ece4808b
route           LINEARIZED_RANGE_PROJECTION_CANDIDATE
```

These are correctness/reproducibility runs, not timing or performance
measurements.

## Next action

Research and freeze R31 as a read-only fraction-to-boundary/globalization
discriminator over the exact R30 direction and full R29 response. Do not
apply the direction, evaluate nonlinear moved state, select a post-observation
step fraction, execute another outer, change penalty/caps/policy, run a
substep/macro/trajectory or claim runtime/production authority.
