# NCGA3 f32 consequence audit — regularized-step apparatus repair

| Field | Value |
| --- | --- |
| Research ID | `NCGA3` revision 2 |
| Status | `FROZEN / REPAIR_IMPLEMENTATION_PENDING / REPORT_ONLY` |
| Supersedes | Revision 1 only for its invalid fixed `7200` Cholesky shift; all arithmetic variants, probes, thresholds, controls and claim ceilings remain unchanged |
| Revision-1 outcome | `INCONCLUSIVE / REFERENCE_REGULARIZED_MATRIX_NOT_POSITIVE_DEFINITE` |
| Parent contract | `02-f32-consequence-and-mixed-pressure-contract.md` at commit `9000d5bf` |

## Observed apparatus failure

The first NCGA3 execution reproduced NCGA2 exactly and completed every HVP and
mixed-arithmetic comparison, but `H_reference + 7200 I` was not positive
definite. Revision 1 therefore published no step or sequence result. It did
not authorize adaptive damping or a retry-to-green value.

The already observed non-step metrics remain immutable:

```text
strict-f32 maximum probe relative L2       1.0643170379823261e-6
f32-products/f64-reduction element error   2.5942468278833446e-4
f64-pressure-products element error        4.7195113035303216e-5
```

Revision 2 repairs only the positive-definiteness guarantee before one fresh
execution.

## Fixed norm-bounded regularization

At each state, form the independently evaluated host matrix
`H_ref_s=(H_ref+H_ref^T)/2` and compute

```text
row_bound = max_i sum_j abs(H_ref_s[i,j])
inertia_scale = m/dt^2 = 7200
regularization = row_bound + inertia_scale
A_variant = (H_variant + H_variant^T)/2 + regularization I.
```

The same `regularization` derived from the reference at that exact immutable
state is used for the reference, strict-f32 and both mixed variants. For a real
symmetric matrix, every eigenvalue magnitude is bounded by `row_bound`, so the
reference system has minimum eigenvalue at least `inertia_scale`. Candidate
positive definiteness is still checked and may fail; no further shift is
allowed.

This bound is intentionally conservative. It tests whether the matrix mismatch
changes a stable local response, not whether an undamped Newton step is a valid
Nonlocal solver. The report records `row_bound`, `regularization`, Cholesky
residual and the removed antisymmetric norm.

For each short-sequence state, derive the bound from the independent host
matrix of that same state before evaluating the selected candidate. Reference
and candidate paths may later occupy different integer states, but arithmetic
selection never supplies the bound.

## Unchanged gates and stop rule

The `1e-3` HVP/step band, `1e-6` cosine band, `5 micrometre` one-step and final
drift limits, `50 micrometre` step cap, eight-step length, energy-descent rule,
permutation gate, controls and mixed-arithmetic definitions remain byte-for-byte
revision 1.

If the reference or any required candidate is still not positive definite, a
residual exceeds `1e-10`, a control is missed or a topology/active set changes,
NCGA3 closes `INCONCLUSIVE`. There is no second apparatus repair. A positive
result remains tiny local numerical evidence only and grants no solver,
trajectory, performance, runtime or product-water authority.
