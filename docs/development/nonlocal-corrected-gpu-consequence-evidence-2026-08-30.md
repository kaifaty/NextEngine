# Nonlocal corrected GPU f32 consequence evidence — 2026-08-30

| Field | Value |
| --- | --- |
| Research ID | `NCGA3` revision 2 |
| Author result | `AUTHOR_SUPPORTED_BOUNDED / F32_CONSEQUENCE_NEGLIGIBLE_BOUNDED` |
| Independent review | `NOT_RUN` |
| Parent strict result | NCGA2 revision 2 remains `REFUTED / COMPENSATED_F32_INSUFFICIENT` |
| Product status | `REPORT_ONLY`; no solver, trajectory, performance, runtime or product-water authority |
| Host | Linux x86-64, RTX 3080 (`sm_86`), CUDA compiler/runtime 13.3 |

## Outcome

The remaining strict-f32 elementwise Hessian mismatch is negligible under the
frozen tiny local consequence screen. It remains a real NCGA2 correspondence
failure, but it does not materially change any of thirteen deterministic
Hessian actions, the common norm-regularized local step or eight quantized
local steps on the combined fixture.

The arithmetic decomposition also closes the causal question:

- binary64 reduction of completed binary32 pressure products barely changes
  the failing element (`2.62473e-4 -> 2.59425e-4`) and remains outside the old
  `2e-4` element gate;
- evaluating pressure coefficients/products and their reduction in binary64
  from unchanged binary32 storage reduces maximum element error to
  `4.71951e-5`, well inside the old gate; but
- that mixed profile does not improve the already tiny operator or local-step
  consequence enough to justify paying for device binary64 before a real
  separately frozen solver experiment demonstrates a need.

The bounded engineering decision is therefore to retain strict f32 as the
first arithmetic candidate for a future solver experiment. Mixed pressure
products remain a known fallback, not a selected production profile.

## Frozen metrics

| Metric | Strict f32 | f32 products + f64 reduction | f64 pressure products | Gate |
| --- | ---: | ---: | ---: | ---: |
| maximum element mixed error | `2.624727856e-4` | `2.594246828e-4` | `4.719511304e-5` | old NCGA2 `2e-4` (reported, not changed) |
| maximum probe relative L2 | `1.064317038e-6` | `1.066700313e-6` | `1.243485005e-6` | `<=1e-3` |
| maximum probe cosine loss | `9.325873407e-15` | `9.325873407e-15` | `3.330669074e-16` | `<=1e-6` |
| regularized-step relative L2 | `2.000435459e-6` | `2.001709323e-6` | `2.147920001e-6` | `<=1e-3` |
| maximum particle-step difference | `0.00623564 um` | `0.00623689 um` | `0.00619669 um` | `<=5 um` |
| eight-step final integer drift | `0 um` | `0 um` | `0 um` | `<=5 um` |

The worst strict probe is deterministic `rademacher_4`. The strict step's
normalized residual is `1.35695e-16`; its removed antisymmetric Frobenius ratio
is `3.08837e-10`. The common reference row bound is
`3,953,549.1816811049`, giving the frozen regularization
`3,960,749.1816811049`. Both reference and candidate energy decrease at every
step. The independent reference objective moves from
`14,175.010660251486` to `14,036.789821732249`; the strict and reference final
integer states and objective are exact at the published resolution.

The original failing scalar remains unchanged:

```text
index       = 51472 = (row 171, column 172)
reference   = 20.023981996858438
strict f32  = 20.029237747192383
mixed error = 2.6247278562120636e-4
```

## Apparatus revision

Revision 1 completed all HVP and arithmetic comparisons but correctly stopped
`INCONCLUSIVE`: `H_reference + 7200 I` was not positive definite. No step or
sequence value from that run was accepted.

Revision 2 changed only the regularization rule before re-execution. It uses
the symmetric reference infinity-norm row bound plus the same `7200` inertia
scale for all variants. This guarantees a positive reference system without
selecting a value from candidate success. Every threshold, fixture,
arithmetic variant, observed HVP metric and negative control remained
unchanged. There is no further apparatus repair budget.

## Mixed arithmetic work receipts

Both arithmetic variants execute the same logical work:

```text
density terms             10,000
Jacobian terms             59,400
pressure outer products 9,000,000
geometric products        356,400
binary64 accumulations  9,356,400
f32 output rounds          90,000
```

The reduction-only variant records `9,356,400` completed f32 products and zero
f64 products. The mixed-product variant records the inverse. Combined and
permuted inputs reproduce exact matrix and work roots.

| Variant | Matrix root | Work root |
| --- | --- | --- |
| strict f32 | `3388f7c978120ee03a6e883b24c682b8db8e879ecffd7b17dd5c2f35cbd85217` | existing NCGA2 receipt |
| f32 products + f64 reduction | `6836007e664d33286af0fc9f6856cf7cad052f7e7469b2dbb1f93161f8e32a42` | `bc76b2b29a7b8d7c5455c560b75d7277ed495f0caecb8f81e1c7bd195aee7128` |
| f64 pressure products | `ddf2a0e22a969c7d7fb3aa7f5a79f24cc6c1b1bbf6ecafa2bf1c542b37d0e8a1` | `ce9c3758949fc9457a7805065805820c11d43697a59ca851f32a780f2792fe08` |

The `1%` symmetric failing-entry bias and full frozen-HVP sign flip are both
rejected. Combined/permuted matrices and receipts are exact.

## Identity and repeatability closure

| Artifact | SHA-256 |
| --- | --- |
| implementation commit | `0af08ba58c4faf0a659d2ebb4b19ab3d920bd01c` |
| implementation tree | `2d7ce3ee9b0089917a69655f806bbfc158696493` |
| implementation parent | `3e0a17599f47948cf2526b20123c673ae3cd6f8f` |
| parent-to-implementation diff | `0237f5d29637ed48658c4a60aec2d91a9a18ffc062363f98458a9fdb1280230b` |
| revision-1 contract | `acbcfffdc8dcc8854e2e32ed75a47c2ce85b99608417b0d0fff17f3936b56b88` |
| revision-2 repair contract | `870064ec4a4505a6e4a213130c6f322eedc806fa0c420a5534845ac9cc46cf2d` |
| CMake target source | `84e9b14b2f60d92ca4ee85f5dcaa17cb6eb1d2b543f60ecd2695b6734172c63c` |
| consequence header | `db0b4529f56087965d76e3fa119c6fd8caaf556210f1fd1dc7907dd496b26dee` |
| mixed CUDA source | `68581445552a4243d34037da803f403afe4e6532d0227a97b597418a1118d6ca` |
| consequence harness | `a9a785aa39ffee9126399d5b19881a864a6490b49e6bcada3b60ee0176d6b10c` |
| clean Release binary A/B | `c175f7f56d3aa1e61946d06d3301397a71c12df85f9d1d5147372824fe3541d7` |
| exact stdout A/B | `0329f8b5848ba06b46f90af4028406af1b7b8ab68d2426a16f82912a6d4fc44a` |

Fresh build directories:

- `/tmp/nextengine-ncga3-final-a-vlJ63X`;
- `/tmp/nextengine-ncga3-final-b-kBBzqs`.

The two stripped binaries and two stdout streams are byte-identical. The
unchanged NCGA2 target remains byte-identical to its frozen binary
`913c7424...d5e73` and reproduces its exact failing stdout
`5ac5b3e7...7377878`.

## Checks

- clean Release NCGA3 build/run A and B: `PASS`, byte-identical;
- NCGA2 exact failure/non-regression: `PASS` as an expected failure;
- Compute Sanitizer `memcheck`, `initcheck`, `synccheck`: `PASS`, zero errors;
- NCGA0 revision-2 self-test stdout:
  `5342fb400c07d429645f66ebb1f596d3e1a2fcd69e16dc5f8d6640c700cfbcd7`;
- NCGA1 self-test stdout:
  `0a89d92ceff561f6e8f128299ead356abaa7820cd05025fd2c1f6b3d85906b51`;
- historical CUDA tiny self-test: `11/11` cases pass and all reused-instance
  comparisons are exact; its report includes diagnostic fields and is not
  claimed byte-identical.

## Decision and remaining risk

NCGA3 author evidence supports treating the small NCGA2 elementwise miss as
negligible for this bounded local response. The next admissible engineering
step is a separately frozen corrected nonlinear/operator solver experiment
that starts with strict f32 and retains mixed pressure products as a named
fallback/control.

This result is not independently reviewed and is not a solver trajectory. The
strong norm regularization is intentionally conservative, the sequence is only
eight `50 um`-capped steps on one 100-sample fixture, and no game-scale timing
was measured. Full solver convergence, long physical stability, visual water,
50k assembly/solve cost, frame time, runtime integration and product readiness
remain open.
