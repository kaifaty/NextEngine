# Nonlocal corrected CUDA f64-pressure evidence — 2026-08-30

Status:
`AUTHOR_REFUTED / MIXED_PRESSURE_STATIC_SOLVE_FAILED / PERFORMANCE_BLOCKED / REVIEW_NOT_RUN / REPORT_ONLY`

## Outcome

Binary64 pressure density, coefficients, analytical gradient and exact Hessian
products improve the compressed-pair residual by another `26.0x` over f64
energy alone, but the frozen two-case static-solve gate still does not close.
The precision ladder therefore stops at NCGA7.

| Case | f64 energy `R_x` | f64 energy + pressure `R_x` | Mixed state drift | Objective difference | Result |
| --- | ---: | ---: | ---: | ---: | --- |
| compressed pair | `2.77070e-6` | `1.06370e-7` | `0.00917917 um` | `1.97257e-6` | misses `1e-7` and `1e-6` bands |
| combined tetrahedron | `3.00034e-7` | `4.37830e-7` | `0.00295592 um` | `3.08388e-6` | residual regresses and objective band misses |

The pressure-mixed pair is only `6.37%` above the residual threshold, but that
threshold was frozen before the run. The combined case accepts one additional
trial yet has a worse final residual than energy-only. Widening either gate now
would be result-fitting rather than evidence.

All paths are finite, monotone on accepted transactions and preserve pressure
activity. The absolute position differences are tiny for a game, but the
solver still cannot establish its own stopping condition robustly. This is not
a demonstrated visual physics failure; it is a failure of the current
arithmetic/controller package to earn a trajectory claim.

## Controls, work and reproducibility

- the three CUDA contributions — full f32/f64-energy, strict pressure-only and
  f64 pressure-only — are executed and sealed separately;
- canonical/permuted results are bit-identical;
- rounding pressure outputs to f32 and first-HVP inversion are rejected;
- invalid continuous state is rejected;
- the energy-only NCGA6 report remains byte-identical; and
- multi-HVP residual CG remains exercised.

| Artifact | SHA-256/root |
| --- | --- |
| frozen contract | `ccd0ee3506099cf4c02f8745659142fb4dc0944856b83ce30034d8a452249825` |
| raw report, both clean runs | `a6bdbe0fb1a78ab091cdfb0e93566a1d71c20207b148732f2c899c43516000c4` |
| semantic result | `6c327d52bf38b6c5e0804d8fb9f35d772fbf2c715cd8f955f000ab6883ce4ba6` |
| stripped Release binary, both clean builds | `8696f968de4dc18cfea18dd3db94f41b5bdf9e37be4f8296765a40b0ef0b31e6` |

Implementation checkpoint commit `2fce8c6dcf9176af5502af86e2a540c2ce2550bd`,
tree `73c0568e18f64349dad9eb7c603f941f8ab3f84a`. CUDA source SHA-256 is
`e1e66575230962c6054a6de2ccd7f39e604f73d4542564dc9373360181094cbb`;
driver SHA-256 is
`54e79df53dde02b7cdff6db34ed43a48d8a3030bf78c7d74d7686574ffe81fc3`.

Two fresh Ninja Release builds produced byte-identical binaries and reports.
Compute Sanitizer `memcheck`, `initcheck` and `synccheck` each reported zero
errors. NCGA0--6 and retained CPU NSR1 reproduced their exact expected stdout
hashes, including default NCGA5 `fa1e5f...fc03c` and NCGA6
`87eeef...fc82`.

## Performance consequence

No end-to-end timing is reported because the correctness gate failed and the
current harness is structurally non-scalable:

- `50,000` particles have `150,000` scalar degrees of freedom;
- a dense Hessian has `22.5 billion` entries — about `90 GB` in f32 or
  `180 GB` in f64, before gradients, graphs and solver workspace;
- the current active-pressure `J^T J` assembly is dense and asymptotically
  cubic in particle count for a fully materialized matrix; and
- every host CG matvec is quadratic in degrees of freedom and requires the
  dense matrix to cross the device/host boundary.

This cannot fit or run as game water on the RTX 3080 used for the experiments.
The separately measured scalable neighbor-builder stage remains useful:
`50k` p95 was about `1.019--1.184 ms`, but that measurement excludes energy,
gradient, matrix-free HVP, nonlinear iterations and integration.

The smallest performance-capable design is a local-neighborhood, matrix-free
HVP with GPU-resident trust-region CG and a bounded preconditioner. It must be
validated against the tiny reference before a 50k end-to-end benchmark. The
serial f64 diagnostic kernels in NCGA6/7 are causal oracles, not production
kernels.

## Decision

Stop arithmetic promotion and do not start the physical trajectory or quote a
full frame time. Return to the controller/model boundary and define a
game-oriented physical acceptance contract independently of the failed
near-machine-precision stop. If that contract accepts a scale-aware terminal
state, implement the matrix-free GPU operator and then measure complete
assembly+solve+integration at 16k/50k. Otherwise repair the solver/model first.

Independent review remains `NOT_RUN`. SPEC-38 remains Proposed and CPU DFSPH
remains the V1 product-water candidate.

## Claim ceiling

NCGA7 refutes the current mixed-pressure static-solve package on two tiny
fixed-graph cases. It does not prove that the visible water behavior is bad;
it also does not establish a trajectory, scalable solver, 50k throughput,
frame time, runtime integration or game-ready Nonlocal water.
