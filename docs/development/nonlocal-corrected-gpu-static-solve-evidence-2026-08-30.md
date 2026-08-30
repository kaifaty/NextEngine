# Nonlocal corrected CUDA full-static-solve evidence — 2026-08-30

Status:
`AUTHOR_REFUTED / F32_STATIC_SOLVE_MATERIAL / REVIEW_NOT_RUN / REPORT_ONLY`

## Outcome

The repaired NCGA5 apparatus is valid, repeatable and negative-control closed.
The strict-f32 CUDA evaluator does **not** complete either retained static
solve. Both cases make accurate early progress and finish geometrically close
to the independent reference, but their f32 objective loses the sign and
resolution of the next actual reduction. The controller then rejects the same
state until the minimum radius.

| Case | Reference terminal | strict f32 terminal | Final state drift | Objective difference |
| --- | --- | --- | ---: | ---: |
| compressed pair | raw gradient success, `4` accepted | `2` accepted, `21` rejected, minimum radius | `0.0352562 um` | `2.08248e-6` |
| combined tetrahedron | independent scale-aware reference floor, `12` accepted | `11` accepted, `21` rejected, minimum radius | `0.00346181 um` | `3.49972e-6` |

The strict compressed residual is `R_x=1.3229809701e-5`, outside the frozen
`1e-5` numerical-floor band, and its objective difference exceeds the frozen
`1e-6` band. The combined residual is `5.7310109566e-7`, but its objective
difference also exceeds `1e-6`. The declared result is therefore
`F32_STATIC_SOLVE_MATERIAL`, not a successful or accepted floor-limited solve.

This classification is about the solver transaction, not visible game-water
error. The sub-micrometre state drift says the current arithmetic is already
close physically on these two tiny cases; it does not permit changing the
predeclared gate after seeing the result.

## Causal trace

For the compressed pair, the third strict-f32 step has predicted reduction
`1.2282437911e-10` but measured actual reduction `-1.1445954442e-6`. For the
combined case, the first persistent late failure has predicted reduction
`6.1295565097e-8` and actual reduction `-5.2154064178e-7`. Later accepted f32
states reduce the combined residual further, then the same sign/resolution
failure repeats.

No pressure active-set change, nonfinite value or negative-curvature path
causes the failure. Both CUDA cases execute residual-terminated, positive-
curvature Krylov paths; the combined path reaches four inner HVPs. This closes
the exact NCGA4 uncertainty: the solver boundary is energy/globalization
resolution near convergence, not failure to execute multi-iteration CG.

## Reference apparatus history

Revision 1 remained immutable `INCONCLUSIVE`. Its independent `long double`
dense combined reference reached `R_x=9.5411881976e-11` but did not reproduce
the historical matrix-free binary64 terminal ratio, which the NSR1 evidence
already identifies as noise-dominated. Revision 2 applied one predeclared
apparatus repair: the pre-existing NSR2-A1 scale-aware reference criterion
`R_x<=1e-8`. A logical pre-run corrigendum assigned the multi-HVP gate to the
combined/corpus path; compressed `4 outer / 8 total HVP` proves one inner plus
one prediction HVP per step.

The repaired host apparatus passed. It used no CUDA candidate metric to set a
threshold, and every strict-f32 candidate gate stayed unchanged.

## Controls and exact identities

- canonical/permuted CUDA inputs produced bit-identical solve roots;
- first-HVP sign inversion changed the solve root and was rejected;
- nonfinite continuous state was rejected before device work;
- predicted-only and reference-only controls changed their owned energy/path;
- pressure active signatures stayed unchanged in all positive paths.

| Artifact | SHA-256/root |
| --- | --- |
| NCGA5 revision-1 contract | `6771ad7769010785f3473c2f9f68753d2985f77c25dc39ddf5845c0c52ab84cb` |
| reference repair contract | `d2a98ff8b1bc51ff0503f37c5b50e06adc6db409841718e799ad2550305ac16b` |
| corpus corrigendum | `9a8becb63ab2b224f23165c2da843483046ec3a963e227e792e31820eaa09e97` |
| accepted raw report, both clean runs | `fa1e5f059fd60aeed6e5ad67094c9c5aa9c17dc1b7444c23491bf990068bc03c` |
| semantic result | `ed822d16e61bc5a9ecd70fc5b62d94584bae0abc10433ed1d8f3b3bff9c22593` |
| stripped Release binary, both clean builds | `cff3133766d9634a4986d5ff60da9f62d916b645a8f1e57fb2194136627a8c7d` |

Implementation checkpoint commit `6340cacf28c1c9170e28c3dd8a08061b3f3c89e7`,
tree `868db0f97e32df7f64d31aa2bbad997081c26afb`. Final driver SHA-256 is
`6e079c3326cb0b4d97e86f09461729a9b7c12c800250c820a577586f19a06299`.

## Verification

Two fresh Ninja Release directories built byte-identical executables and
emitted byte-identical reports. CUDA Compute Sanitizer `memcheck`, `initcheck`
and `synccheck` each reported `ERROR SUMMARY: 0 errors`.

| Retained check | Result | stdout SHA-256 |
| --- | --- | --- |
| NCGA0 corrected terms | `PASS` | `5342fb400c07d429645f66ebb1f596d3e1a2fcd69e16dc5f8d6640c700cfbcd7` |
| NCGA1 neighborhoods | `PASS` | `0a89d92ceff561f6e8f128299ead356abaa7820cd05025fd2c1f6b3d85906b51` |
| NCGA2 strict assembly | expected `FAIL`, exit `1` | `5ac5b3e7990ac5cadd2b711119fac8bdc3ae27c2d1b1bafeb7296fe587377878` |
| NCGA3 consequence | `PASS` | `0329f8b5848ba06b46f90af4028406af1b7b8ab68d2426a16f82912a6d4fc44a` |
| NCGA4 trust prefix | `PASS` | `be6236be48ab4e75cccbfdc53343f7ce41d77ab44524d7927afd04398d447cb0` |
| retained CPU NSR1 | `PASS` | `520258ef7f711d90bb4e1ab1e7c0fc8c9ca12002aff0287223901626cc50678f` |

No workspace ProductCheck was run: this is an isolated C++/CUDA report-only
research target and changes no Rust runtime or product profile. Performance
timing was intentionally not run after the failed solve gate.

## Decision and next boundary

Do not start the physical trajectory and do not accept strict f32 by widening
the gate. The smallest causal experiment is GPU binary64 energy evaluation at
the exact f32 stored state/profile while retaining the strict-f32 gradient,
Hessian and controller. If that closes the retained static solves, it selects
a mixed-energy candidate; otherwise pressure coefficient/product promotion is
the next discriminator.

Independent review remains `NOT_RUN`; the user did not request a reviewer for
NCGA5. SPEC-38 remains Proposed and CPU DFSPH remains the V1 product-water
candidate.

## Claim ceiling

NCGA5 refutes strict-f32 full-solve sufficiency only on two tiny fixed-graph
static cases. It does not establish a physical trajectory, dynamic neighbors,
contacts/boundaries, a GPU-resident solver, scalable cost, 50k throughput,
frame time, runtime integration or game-ready water.
