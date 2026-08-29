# Nonlocal corrected CUDA compensated-f32 objective assembly — revision 2

| Field | Value |
| --- | --- |
| Research ID | `NCGA2` revision 2 |
| Status | `FROZEN / IMPLEMENTATION_PENDING / REPORT_ONLY` |
| Supersedes | Revision 1 only as the candidate accumulation identity; revision 1 remains `REFUTED` |
| Parent checkpoint | `6119fd2c`; revision-1 failing report SHA-256 `9b000701b9282a4a8e8ff8846fad34676d26872e09497e72159608676d006353` |
| Candidate identity | `nuv-objective-assembly-owner-row-compensated-f32-r1` |
| Claim class | Same tiny objective energy/gradient/Hessian correspondence as revision 1 |
| Review budget | One independent initial review and at most one batched repair/re-review across the final revision-2 package |

## Causal basis

Revision 1 passed graph, density, isolated viscosity/surface, energy-derivative,
permutation and negative-identity gates. It failed only dense pressure-bearing
reductions. On the symmetric active-pressure fixture, analytical values near
zero became `2.8228759765625e-4` in one gradient component and
`3.4332275390625e-3` in one Hessian component. The combined fixture missed the
mixed Hessian bound narrowly at `2.274196e-4`.

The revision-2 hypothesis is falsifiable: stable compensated binary32 addition
closes those cancellation residues without changing a formula, graph, input,
tolerance, arithmetic product or output precision. Failure of any unchanged
gate ends NCGA2; no third accumulation identity is authorized.

## Immutable revision-1 boundary

Revision 2 retains byte-for-byte:

- profile, fixture generation/order and all enabled-term masks;
- exact current/reference integer graph predicates and canonical `SampleId`
  rows;
- host analytical and independent energy-only translation units;
- objective, gradient, exact Hessian, diagonal-block and HVP definitions;
- mixed/zero, derivative, symmetry and HVP bounds;
- six original wrong-formula/matrix/graph controls;
- cold-repeat count, admission limits and claim ceiling; and
- strict device arithmetic flags, RTX 3080 `sm_86` and CUDA 13.3 target.

No coordinate, coefficient, finite-difference step, expected output or
comparator route may change. Revision 1 remains independently buildable as the
named naive negative identity.

## Exact compensated recurrence

Every candidate floating reduction owns one independent two-float accumulator:

```text
sum = +0.0f
correction = +0.0f

add(value):
    adjusted = value - correction
    next = sum + adjusted
    correction = (next - sum) - adjusted
    sum = next

result = sum
```

All variables and operations above are binary32 under the unchanged strict CUDA
flags. The recurrence is Kahan-style compensation; it does not publish or use
`sum+correction`, binary64, FMA contraction, tensor cores, shared endpoint
scatter or a host recomputation.

One independent accumulator is used for:

1. each owner density;
2. each of four canonical owner-ordered energy reductions;
3. each gradient scalar;
4. each pressure-center Jacobian scalar;
5. each dense Hessian scalar;
6. each pressure directional reduction and final direct-HVP scalar.

Product evaluation, term order, center order, canonical neighbor order, matrix
row/column order and kernel staging remain revision 1. A reduction never shares
its correction with another scalar or persists across fixtures/runs.

The work receipt adds exact executed `compensated_additions` and
`compensation_initializations`. Every call to `add` increments the former; each
created accumulator increments the latter. Counts are sealed in the work root
and must be identical under the combined input permutation. Planned counts or
short-circuited additions do not count.

## New mandatory negative

`naive_f32_pressure` is the exact revision-1 device recurrence with no
compensation and all other revision-2 code paths unchanged. Through the common
comparator it must reproduce rejection on `active_pressure_cluster`; its first
gradient/Hessian mismatch class and revision-1 payload root must remain
observable. The harness may not special-case those expected scalar values.

The original six negatives remain mandatory. Passing the naive path, removing
the symmetric fixture or accepting only the combined case invalidates the
experiment.

## Ordered resolution gates

1. Rebuild revision 1 and reproduce the exact failing report hash.
2. Run revision 2 positives in the frozen order; stop at the first failure.
3. Require exact combined/permuted payload and work roots.
4. Reject naive f32 and all six original wrong identities through the common
   comparator.
5. Require ten cold candidate executions per process and two byte-identical
   fresh Release process reports/binaries.
6. Run `memcheck`, `initcheck`, `synccheck`, NCGA0, NCGA1 and the retained
   historical tiny CUDA control.
7. Freeze source/binary/report/fixture/payload/work roots and request one
   independent review of the exact snapshot.

`SUPPORTED_BOUNDED` requires every gate plus reviewer `GO`. Any unchanged
positive mismatch is `REFUTED`; a surviving review defect after one batched
repair is `INCONCLUSIVE`.

## Claim ceiling

Revision 2 can establish only that one stable compensated strict-binary32
implementation assembles the FCR objective energy, gradient and exact Hessian
on the frozen tiny corpus. It grants no sparse/matrix-free production operator,
factorization, preconditioner, nonlinear solve, trajectory, stability, visual
quality, full-solver timing, runtime integration, canonical GPU authority or
product-water claim. NCGP0 neighborhood-only timings remain a separate
diagnostic and cannot be added to an unmeasured solver total.

