# Nonlocal corrected CUDA f32 consequence and mixed-pressure audit — revision 1

| Field | Value |
| --- | --- |
| Research ID | `NCGA3` revision 1 |
| Status | `FROZEN / IMPLEMENTATION_PENDING / REPORT_ONLY` |
| Parent checkpoint | `9127b7e3c18e71656d9ca3aa303cc5afb0f02161`; NCGA2 revision-2 failing report SHA-256 `5ac5b3e7990ac5cadd2b711119fac8bdc3ae27c2d1b1bafeb7296fe587377878` |
| Mathematical parent | Frozen FCR0/FCR1 objective and NCGA2 revision-1/2 fixtures, oracle and strict-f32 candidate |
| Engineering consumer | Decide whether the remaining strict-f32 Hessian mismatch is negligible for a bounded local step or whether pressure products require mixed precision before a solver experiment |
| Claim class | Tiny-fixture numerical-consequence screening; no solver, trajectory, performance or product-water authority |

## Question

NCGA2 revision 2 misses its elementwise Hessian correspondence gate at dense
index `51472`:

```text
reference = 20.023981996858438
strict f32 = 20.029237747192383
relative mixed error = 2.6247278562120636e-4
frozen NCGA2 gate = 2e-4
```

NCGA3 does not change that result or relax its gate. It asks a different,
product-oriented question: does the observed matrix error materially change
Hessian actions, a bounded regularized step or a short sequence of such steps?
It also localizes whether double accumulation alone or double
pressure-coefficient/product evaluation is the smallest arithmetic change that
removes the discrepancy.

## Competing hypotheses

| ID | Hypothesis | Prediction | Falsifier |
| --- | --- | --- | --- |
| H1 | The NCGA2 elementwise miss is locally negligible | strict-f32 HVPs and regularized steps remain within the screening band, preserve descent and accumulate only micrometre-scale drift | any probe exceeds the band, changes descent, or accumulates material drift |
| H2 | Final pressure reduction is still the dominant error | binary64 accumulation of otherwise binary32 pressure products closes the old elementwise miss and consequence metrics | product promotion is required or reduction-only remains outside the old gate |
| H3 | Pressure coefficient/product rounding is dominant | binary64 pressure coefficients/products and reduction from binary32 storage close the old gate while reduction-only does not | mixed products do not improve the failing scalar/operator metrics |
| H4 | The discrepancy is not confined to the known direction | one or more deterministic independent probes expose materially larger HVP or step error than the frozen NCGA2 direction | every probe remains within the screening band |

## Immutable boundary

NCGA3 retains the NCGA2 profile, eight fixtures, integer state, canonical
ordering, graphs, FCR objective, independent host `long double` analytical and
energy-only oracles, strict CUDA flags and RTX 3080/CUDA 13.3 target. Existing
NCGA2 sources, executable target, expected failure, payload roots and work roots
remain unchanged and must reproduce before NCGA3 evidence is interpreted.

The only positive fixture for consequence classification is
`combined_cluster`; `combined_cluster_permuted` must produce identical metrics.
The pressure-only symmetric cluster remains a cancellation control. The other
NCGA2 fixtures remain non-regression controls.

## Arithmetic variants

All variants ingest the same binary32 profile and position storage used by the
strict candidate. Non-pressure terms remain the unchanged compensated
binary32 NCGA2 values.

1. `strict_f32` is the frozen NCGA2 revision-2 candidate.
2. `f32_products_f64_reduction` evaluates density, compression, pressure
   Jacobians, normals, radial coefficients and individual pressure products in
   binary32, promotes each completed product to binary64, accumulates in
   binary64 and rounds the final pressure Hessian scalar to binary32.
3. `f64_pressure_products` promotes the admitted binary32 profile and decoded
   positions before radius, kernel derivative, compression, Jacobian, normal,
   radial-coefficient and pressure-product evaluation; pressure reduction is
   binary64 and the final assembled scalar is rounded to binary32.

No variant changes storage, graph membership, term order, center order, output
precision, formula, fixture, old comparator or non-pressure arithmetic. Device
binary64 use and executed binary32/binary64 operation counts are explicit in
the new receipt. These variants are diagnostics, not production profiles.

## Deterministic consequence probes

For the `300 x 300` combined Hessian, compare the host oracle and each variant
on these vectors in canonical degree-of-freedom order:

1. the frozen NCGA2 direction;
2. the normalized negative host gradient;
3. unit coordinates `171` and `172`, which span the failing scalar;
4. six fixed Rademacher vectors whose sign bit is
   `splitmix64(0x4e43474133000000 + probe*300 + index) & 1`, using the standard
   SplitMix64 add/xor/multiply sequence with constants `0x9e3779b97f4a7c15`,
   `0xbf58476d1ce4e5b9` and `0x94d049bb133111eb`;
5. three fixed smooth particle vectors with periods `7`, `11` and `17`, where
   scalar `(particle,axis)` is
   `sin(2*pi*(particle+1)/period + axis*pi/3)`.

Every vector is normalized to Euclidean norm one in host binary64. Report for
each probe:

```text
relative_l2 = ||H_candidate*v - H_reference*v||_2
              / max(||H_reference*v||_2, 1)
cosine_loss = 1 - cosine(H_candidate*v, H_reference*v)
maximum_component_error
```

The aggregate consequence band requires maximum `relative_l2 <= 1e-3` and
maximum `cosine_loss <= 1e-6`. The `1e-3` band is a report-only product
screening level aligned with the current water program's `0.1%` divergence
accuracy scale; it does not alter the NCGA2 correspondence requirement or
promote this different Nonlocal model.

## Regularized step discriminator

For the combined fixture define

```text
inertia_scale = m / dt^2 = 7200
A = H + inertia_scale * I
A * p = -gradient.
```

The added inertia-scale diagonal is fixed before execution and makes this a
bounded local response probe, not a claimed Nonlocal solver. Before solving,
apply the fixed projection `H_s=(H+H^T)/2`; report the removed antisymmetric
norm separately. Solve `H_s + inertia_scale*I` in host binary64 by deterministic
Cholesky without pivoting. Reject the probe if either matrix is not positive
definite or if either normalized residual against that exact projected system
exceeds `1e-10`.

The strict-f32 consequence band requires:

- relative step error `||p_f32-p_ref||_2/max(||p_ref||_2,1e-12) <= 1e-3`;
- cosine loss between steps `<=1e-6`;
- maximum per-particle step difference `<=5 micrometres`; and
- the independent `long double` objective decreases for both quantized steps.

Before applying a step, scale it uniformly so its maximum particle displacement
is at most `50 micrometres`, then round every coordinate update to the nearest
micrometre with `llround`. Evaluate the resulting state only through the
existing independent host objective/derivative implementation. Graphs,
pressure-active flags and branch-safety margins must remain unchanged.

## Short sequence

Starting from the same combined fixture, advance independent reference and
strict-f32 states for eight bounded regularized steps using the rule above.
Every accepted step must strictly reduce its own independent host objective.
The sequence passes the negligible-consequence screen only if:

- all eight steps are finite, positive-definite and residual-valid;
- graph and pressure-active identities remain unchanged;
- no step increases the independent objective;
- final maximum particle-position drift is `<=5 micrometres`; and
- final relative objective difference is `<=1e-3`.

The mixed variants are evaluated on the same one-step and eight-step rules if
strict f32 misses any consequence gate. No retry, adaptive damping, changed
step cap or additional trajectory is allowed.

## Mandatory controls and ordered execution

1. Rebuild and reproduce the exact NCGA2 revision-2 failure, roots and failing
   scalar before running the new harness.
2. A deliberate `hessian_entry_bias` control adds `1%` of the reference value
   at entries `(171,172)` and `(172,171)` and must be detected by the coordinate
   probes' maximum component-relative error `>1e-3` or by the regularized-step
   gate.
3. A deliberate `hvp_sign_flip` control negates the complete frozen-direction
   HVP and must fail the cosine gate.
4. Combined/permuted inputs must produce exact variant matrices, metrics,
   routes and receipts.
5. Two fresh Release builds/runs must be byte-identical. Run CUDA `memcheck`,
   `initcheck` and `synccheck` only after all numerical gates reach completion.
6. Preserve NCGA0, NCGA1 and NCGA2 results as unchanged regressions.

Stop on the first invalid input, nonfinite value, graph/active mismatch,
non-positive-definite regularized matrix, residual failure or undetected
negative control. Raw matrices and trajectories remain outside Git; checked-in
evidence contains bounded metrics and hashes only.

## Resolution and claim ceiling

- `F32_CONSEQUENCE_NEGLIGIBLE_BOUNDED`: strict f32 passes every HVP, step,
  descent, sequence, permutation and negative-control gate. This supports
  retaining f32 for the next separately frozen solver experiment despite the
  unchanged NCGA2 elementwise failure.
- `MIXED_PRESSURE_REQUIRED_BOUNDED`: strict f32 fails a consequence gate and
  `f64_pressure_products` passes it while restoring the old elementwise bound.
- `F32_CONSEQUENCE_MATERIAL`: strict f32 changes descent, exceeds a step/HVP
  band or accumulates more than the frozen drift limit.
- `INCONCLUSIVE`: apparatus independence, identity, conditioning, work receipt,
  repeatability or control detection is not closed.

Even a positive result is only a tiny local numerical screen. It establishes
no nonlinear solver convergence, multi-step physical trajectory, stability,
visual water quality, throughput, game frame time, runtime integration,
canonical GPU authority or product-ready water. SPEC-38 remains Proposed and
CPU DFSPH remains the only current V1 water candidate.
