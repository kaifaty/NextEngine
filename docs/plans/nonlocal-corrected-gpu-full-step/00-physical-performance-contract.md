# Corrected Nonlocal CUDA full-step physical/performance contract

| Field | Value |
| --- | --- |
| Research ID | `NCGP1` revision 1 |
| Status | `FROZEN / IMPLEMENTATION_AUTHORIZED / REPORT_ONLY` |
| Architecture snapshot | commit `62e8bd1f922ce8b8a2d03dd25a0695ac8f212c92`; SPEC-38 and ADR-076 remain `Proposed`; ADR-081 remains `Accepted` |
| Mathematical parent | FCR0 corrected objective; reviewed NCGA0 terms and NCGA1 integer neighborhood; NCGA2--7 retained failures and bounded diagnostics |
| Engineering consumer | Decide whether a scalable corrected Nonlocal CUDA step for exactly 50,000 samples is physically admissible and fits `4 ms p95 / 6 ms p99` on the current RTX 3080 |
| Claim class | Finite/profile-bound numerical correspondence, physical trajectory and empirical performance |
| Review budget | One independent review and one batched-repair re-review at most |

## Claim and exact negation

The positive claim is that one standalone CUDA implementation of the corrected
FCR objective, dynamic neighborhood, analytical box boundary and matrix-free
trust-region Newton--CG step passes every physical gate below and, in two fresh
processes on the selected RTX 3080/CUDA 13.3 host, completes the exact 50,000
sample hot step at `p95 <= 4 ms` and `p99 <= 6 ms`.

The physical claim is negative if an admitted positive loses finiteness,
capacity, mass, boundary containment, correspondence, declared balance or the
fixed work budget. The performance claim is negative if physical admission
passes but either process misses either percentile after the two authorized
optimization cycles. Broken identity, oracle independence, controls, work
accounting, repeatability or review closure is `INCONCLUSIVE`.

NCGA2 remains `REFUTED` at its `2.624727856e-4` elementwise Hessian error
against the historical `2e-4` gate. NCGP1 neither edits nor relabels that
result. Its new physical observables are a different, separately frozen claim.

## Selected model and arithmetic

The profile ID is `nonlocal-water-50k-v1`:

```text
dt       = 1/240 s
spacing  = 0.05 m
horizon  = 0.15 m
mass     = 0.125 kg
rho0     = 1000 kg/m^3
kappa    = 9196.875
lambda   = 2.5
mu       = 1.7
gamma    = 3.5
gravity  = (0, 0, -9.81) m/s^2
capacity = 50,000 dynamic samples, 256 current neighbors per sample
```

Pressure, viscosity and surface energy are exactly FCR0. Density includes
self and admitted static boundary samples. Viscosity uses the immutable
reference graph and `omega=-dW/dr`; pressure and surface use the current graph.
The pressure clamp is `max(rho/rho0-1,0)`. No SISSM local matrix, Chebyshev
recurrence, Gauss--Newton substitution or clamped Hessian is admissible.

Primary device arithmetic stores positions, velocities, gradients and HVP
vectors in IEEE binary32 under `--fmad=false --prec-div=true
--prec-sqrt=true --ftz=false`. Energy, dot/norm reductions, `ared`, `pred` and
`rho` are binary64 under a fixed reduction tree. The only authorized arithmetic
discriminator promotes pressure density/coefficient/product evaluation to
binary64 while leaving stored state binary32; it runs only if the primary path
passes semantic controls but fails a pressure-operator physical gate.

## State, graph and capacity

Dynamic samples have stable ascending `u32 SampleId`, immutable reference
position, current/predicted position and velocity. The candidate owns SoA
device storage and a preallocated workspace. Host fixture construction,
admission and bounded final evidence capture are allowed; host graph building,
canonicalization, dense matrices and full-vector transfers inside a step are
forbidden.

Both reference and current graphs use exact signed micrometre positions for
cell assignment and the inclusive predicate `distance_squared <= horizon^2`.
Rows and neighbors are canonical ascending IDs. A current graph is rebuilt for
each outer evaluation and is immutable during its gradient/HVP transaction.
The reference graph is immutable for a substep. More than 256 neighbors,
integer overflow, duplicate IDs, nonfinite profile/state or allocation failure
rejects before a candidate state is published.

The retained correctness baseline is the reviewed NCGA1 two-pass 27-cell
builder. A separately reported single-traversal variant is authorized only by
the performance rule below: one 27-cell visit fills a fixed 256-ID row, then
ascending row sort, count scan and CSR compaction execute on device. It must
produce the same graph root and reject rather than truncate overflow.

## Matrix-free objective and solver

No dense Hessian may be allocated. The candidate evaluates energy, gradient,
diagonal and `H*v` directly on the CSR graphs. Pressure HVP uses a separately
stored center scalar `q=J*v`, then owner-row gather for `J^T q` plus the active
pressure geometric term. Reverse adjacency makes neighbor-center contributions
owner-gathered. Viscosity, surface and inertia remain local owner gathers.
Floating endpoint atomics are forbidden.

Tiny correspondence uses the unchanged unpreconditioned NCGA5 Steihaug--Toint
policy. The scalable performance profile uses the positive diagonal
preconditioner

```text
M[k] = max(abs(Hdiag[k]), mass/dt^2).
```

Trust radii, residual forcing, curvature/boundary exits, `rho` thresholds and
at most 64 outer trials remain NCGA5. Vectors stay on device; only fixed-size
scalar decision records cross to the host. The scalable work budget is selected
before timing by running total HVP ceilings `{32,64,128}` in that order over
the entire physical corpus and freezing the first passing ceiling. Exhausting
the selected ceiling is `WORK_BUDGET_EXCEEDED`, not convergence.

## Analytical basin and fixed corpora

The primary basin is an axis-aligned `3.0 x 2.5 x 1.5 m` box. Dynamic centers
are constrained to a radius-`spacing/2` inset. Boundary support is the unique
union of half-cell-centred lattice sites in the three-layer exterior shell,
`ceil(horizon/spacing)=3`. Boundary samples are fixed, contribute to density
and pressure, and own no inertia, viscosity or surface state. A swept segment
projection handles analytical-plane contact after a trial; it must never move
a particle outside the inset box.

The primary state contains `50 x 40 x 25 = 50,000` dynamic samples. Its first
center is `(0.275,0.275,0.025) m`; axes advance in `0.05 m` ascending-ID order.
Smaller exact profiles are `20 x 20 x 10 = 4,000` and
`40 x 20 x 20 = 16,000`. Coherent, fixed-permuted and one frozen advected
50k state share the same semantic profile.

Correctness executes in this order and stops on the first failed gate:

1. retained compressed pair and combined tetrahedron;
2. free fall and translation/rotation invariance;
3. viscosity decay and surface relaxation;
4. 4k hydrostatic hold, dam-break and orifice against an independent CPU f64
   matrix-free implementation;
5. exact 16k and 50k coherent/permuted/advected graph/capacity controls;
6. a 240-step 50k sealed-basin trajectory.

The independent CPU path is a separate translation unit using direct/all-pairs
graphs for tiny inputs and separately implemented CSR pair traversal for 4k.
It may share immutable DTOs and profile bytes, but no candidate graph, formula,
reduction, solver or boundary helper.

## Physical gates

- Pair/tetra final position difference from the independent host solve is at
  most `5 micrometres`; pressure-active signatures are exact.
- Matrix-free HVP has relative L2 error `<=1e-3` and cosine loss `<=1e-6`
  against the long-double dense tiny oracle.
- On the 4k trajectories, position RMSE is `<=2.5 mm` and maximum error is
  `<=5 mm` against the independent f64 path.
- Sample count and total mass are exact. Density RMSE is `<=5%` of `rho0` and
  maximum density error is `<=10%` of `rho0`.
- Normalized complete-system momentum residual is `<=1%` after gravity and
  boundary impulse accounting.
- Positive mechanical-energy excess and reversible-control energy drift are
  each `<=1%`.
- Maximum center penetration beyond the inset analytical boundary is
  `<=2.5 mm`; no particle escapes the closed box.
- Every result is finite, capacity-valid, work-sealed and permutation-stable.

These gates authorize performance measurement only. They do not change an
Accepted specification, select canonical water or prove visual quality.

## Mandatory wrong identities

The common comparator must reject: missing kernel `2/h`; half viscosity force;
wrong surface sign; first-HVP sign inversion; current/reference graph swap;
owner-only pressure that omits neighbor centers; strict `<` support radius;
disabled ghost/contact boundary; fixed-permutation identity loss; and
neighbor-capacity overflow. A control may not use a special expected-result
path.

## Performance protocol

The primary timed window starts with preallocated device state and ends after
dynamic graph rebuild, density/active set, energy/gradient/HVP work, trust
controller, boundary projection, integration and updated device state are
complete. Process startup, allocation, CPU oracle, JSON, rendering and PhysX
are excluded. Integration-copy cost is reported separately and grants no
runtime claim.

Each of two fresh processes executes 256 conditioning steps, 32 warmups and
128 measured CUDA-event samples with one synchronization per sample. Nearest
rank uses sorted indices `63/121/126` for p50/p95/p99. There is no outlier
removal, retry or best-process selection. Both processes must satisfy
`p95 <= 4 ms` and `p99 <= 6 ms`. Stage p50/p95 and exact work counts report
graph, energy/gradient, all HVPs, reductions/control and boundary/integration.

If the baseline misses and graph time exceeds 20% of the full-step median,
optimization cycle 1 replaces only the two-pass builder with the frozen
single-traversal variant. Cycle 2 may fuse compatible owner-row kernels, reuse
scratch and reduce launch/synchronization count. No third cycle and no change
to physics, workload, arithmetic selection, work budget or gates is allowed.

## Evidence, resolution and ceiling

Two fresh Release builds/runs must have byte-identical stripped binaries and
semantic reports. After numerical gates, run Compute Sanitizer `memcheck`,
`initcheck` and `synccheck`, plus exact NCGA0/NCGA1 and retained NCGA2--7
regressions. Freeze contract/source/binary/input/output/work/report hashes,
commands, compiler flags and host identity before review.

- `SUPPORTED_BOUNDED / PERFORMANCE_PASS`: all physical, performance,
  repeatability, sanitizer, control and independent-review gates pass.
- `PHYSICS_REFUTED`: an admitted physical positive fails before timing.
- `PERFORMANCE_REFUTED`: physical gates pass but the two-cycle candidate misses
  either percentile in either fresh process.
- `INCONCLUSIVE`: apparatus, identity, work, repeatability or allowed re-review
  cannot close.

A fresh reviewer receives the frozen contract, exact candidate snapshot and
raw artifacts without the author's desired verdict. One batched repair and one
re-review are the full budget; a surviving load-bearing defect closes NCGP1
`INCONCLUSIVE`.

Even `PERFORMANCE_PASS` establishes only standalone report-only Nonlocal CUDA
cost on the named host/profile. CPU DFSPH remains the only current product
candidate. Runtime, PhysX, rendering, persistence, game-frame integration,
Windows and canonical GPU authority remain outside this claim.
