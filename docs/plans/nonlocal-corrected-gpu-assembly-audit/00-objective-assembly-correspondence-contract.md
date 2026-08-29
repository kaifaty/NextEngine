# Nonlocal corrected CUDA objective assembly correspondence — revision 1

| Field | Value |
| --- | --- |
| Research ID | `NCGA2` revision 1 |
| Status | `FROZEN / IMPLEMENTATION_PENDING / REPORT_ONLY` |
| Architecture snapshot | `bb73075c`; SPEC-38 and ADR-076 remain `Proposed`; ADR-081 remains `Accepted` |
| Mathematical parent | FCR0/FCR1 objective `nuv-variational-fcr1` and reviewed NCGA0 full-pair terms |
| Engineering consumer | Decide whether the corrected CUDA lineage may proceed from exact neighborhoods to a separately frozen matrix-free operator/solver audit |
| Claim class | Tiny finite-set objective energy, gradient and Hessian assembly correspondence |
| Review budget | One independent initial review and at most one batched repair/re-review |

## Why this is not the historical SISSM matrix

The historical per-particle SISSM source and local `3x3` split are not the
candidate in this experiment. FCR3-B2 stopped that solver lineage after its
pressure-bearing recurrence produced non-descent or fixed-budget quality
failures. Reproducing the same split on the GPU would establish translation of
an already rejected algorithm, not progress toward a corrected solver.

NCGA2 instead assembles observables owned directly by the reviewed variational
objective:

1. density and the four objective-energy components;
2. the exact objective gradient (with `source = -gradient` only as a named
   diagnostic alias);
3. the exact dense Hessian on tiny fixtures and its per-particle diagonal
   `3x3` blocks; and
4. one fixed Hessian-vector product derived from that matrix.

No inverse, factorization, preconditioned step, SISSM recurrence, line search
or state update is part of this contract.

## Frozen arithmetic profile and state

The report-only Nonlocal audit profile is the NCGA0/FCR profile, not the
SPEC-38 V1 DFSPH product profile:

```text
h = 0.15 m
r0 = 0.05 m
m = 0.125 kg
dt = 1/240 s
rho0 = 1000 kg/m^3
kappa = 9196.875
lambda = 2.5
mu = 1.7
gamma = 3.5
```

Every sample has a unique `u32 SampleId`, reference position `x`, inertial
prediction `y_star`, current candidate `y` and one fixed direction `d`.
Positions are admitted as signed integer micrometres in
`[-1000000000,1000000000]` and decoded once to strict binary32 on the device.
The host oracle evaluates the same exact rational micrometre inputs in `long
double`. Empty inputs, duplicate IDs, nonfinite profile values, zero/nonpositive
physical constants, capacity excess and arithmetic overflow reject before any
observable is published.

Current density/pressure/surface membership uses the exact integer predicate

```text
|y_i-y_j|_um^2 <= 150000^2,
```

while viscosity membership uses the independent reference predicate on `x`.
Self contributes exactly once to density and never becomes a pair. Rows and
matrix blocks are published in ascending `SampleId` order. Input array position
has no semantic meaning.

## Frozen objective and derivatives

The kernel, compression clamp, viscosity and surface potential are exactly the
FCR0 contract. With one visit per undirected pair:

```text
E = m/(2 dt^2) sum_i |y_i-y_star_i|^2
  + kappa/2 sum_i max(rho_i/rho0 - 1, 0)^2
  + sum_{i<j in G_x} m*omega/(rho0*dt)
      [mu |P_t delta_ij|^2 + lambda/2 |P_n delta_ij|^2]
  + sum_{i<j in G_y} 2*gamma*m^2*C(|y_i-y_j|).
```

`gradient` is the analytical derivative of this exact scalar objective.
`Hessian` is its analytical derivative at the frozen state away from density,
support and surface-spline branch boundaries. It includes both pressure terms
`kappa J^T J` and `kappa*s_i*H(rho_i/rho0)`, the full viscosity curvature and
the radial/tangential surface curvature. It is not clamped positive and is not
the historical SISSM or block-Gauss-Newton matrix.

The dense matrix is row-major over canonical `(SampleId,x/y/z)` degrees of
freedom. The reported diagonal blocks are exact slices of that matrix. The HVP
must equal a dense matrix-vector product evaluated in the declared row order.

## Independent host and CUDA implementations

The host oracle is a new translation unit using direct checked `O(N^2)` graph
construction, `long double` energies/gradients and direct dense block assembly.
It may consume immutable fixture DTOs but must not call CUDA helpers, NCGA0
force evaluators, `variational_reference.cpp`, historical SISSM code or the
candidate's pair-contribution helpers.

A second host translation unit implements energy only and performs centered
directional finite differences. It must not call the analytical gradient or
Hessian implementation. It checks both

```text
dE/depsilon = gradient dot d
d2E/depsilon2 = d^T Hessian d
```

at predeclared fixture-specific steps that do not cross a membership or spline
branch.

The CUDA candidate uses strict binary32 arithmetic on RTX 3080 (`sm_86`) and
CUDA 13.3 with `--fmad=false --prec-div=true --prec-sqrt=true --ftz=false`.
It constructs both exact integer graphs on device, evaluates density in stable
neighbor order and gives each output row one owner. No floating atomic,
unordered endpoint scatter, host-built candidate graph, host canonicalization
or hidden binary64 device arithmetic is permitted. Tiny dense assembly may be
quadratic/cubic in fixture size; it is evidence apparatus, not a production
algorithm.

## Frozen fixtures

The ordered corpus is generated from explicit integer records:

1. `isolated_inertia` — one nonzero ID, no pair energy, exact inertia block;
2. `inactive_pressure_cloud` — density below the clamp, zero pressure
   energy/gradient/Hessian;
3. `reference_current_support_crossing` — one pair inside only `G_x` and one
   inside only `G_y`, distinguishing viscosity from current terms;
4. `oblique_viscosity` — non-axis-aligned normal plus independent normal and
   tangential increments;
5. `surface_two_branches` — radii strictly inside both surface-polynomial
   branches with attraction and repulsion;
6. `active_pressure_cluster` — 100 unique samples on a `5x5x4`, `1 mm`
   lattice with every pressure center strictly above rest density;
7. `combined_cluster` — the same canonical IDs with fixed non-affine `x`,
   `y_star`, `y` and direction perturbations, all four terms enabled; and
8. `combined_cluster_permuted` — identical semantic state under a fixed affine
   input permutation.

The two combined fixtures must have identical canonical graph, observable and
work roots. Every fixture publishes minimum distance to the density clamp,
kernel support and surface branch boundaries; a derivative check is invalid if
its finite-difference interval reaches any boundary.

Maximum admitted samples are 128 and maximum dense scalar entries are
`(3*128)^2`. Capacity validates before allocation.

## Exact gates and tolerances

For every positive fixture:

- current/reference owner rows, offsets and neighbor IDs equal the independent
  integer oracle byte-for-byte;
- all values are finite and pressure active flags are exact;
- density, each energy component, gradient, dense Hessian, diagonal blocks and
  HVP pass `abs(candidate-reference) <= 2e-4*max(1,abs(reference))`;
- reference zeros require candidate absolute value `<=2e-6` except dense
  Hessian zeros, which use `<=2e-5`;
- the host analytical gradient and Hessian pass their independent energy-only
  first/second directional derivative gates at relative error `<=2e-7` and
  `<=2e-5` respectively;
- dense-Hessian antisymmetry is `<=2e-5*max(1,max_abs(H))` and internal
  gradient closure is `<=2e-6*max(1,sum_norm_internal)`; and
- direct dense `H*d` and the independently accumulated candidate HVP agree
  under the main mixed bound.

The first failed mandatory gate ends the positive claim. Tolerances, fixture
positions, enabled terms and derivative steps cannot change after observing a
candidate result.

## Mandatory negative identities

Every control runs through the same comparator and must be rejected:

1. `source_shaped_gradient` omits the physical `2/h` conversion in `dW/dr`;
2. `directed_edge_viscosity` uses half-force/half-curvature coefficients;
3. `owner_pressure_only` omits the neighbor-center pressure contribution;
4. `gauss_newton_pressure_only` omits the active pressure geometric Hessian;
5. `sissm_local_matrix` substitutes the stopped diagonal SISSM split for the
   exact objective Hessian; and
6. `current_graph_viscosity` uses `G_y` instead of `G_x`.

At least one named fixture is frozen for each rejection. A special
control-only expected path is forbidden.

## Work, repeatability and evidence

The result seals exact counts for graph key/distance work, density kernel
evaluations, energy pair visits, gradient pair visits, active pressure centers,
pressure participant products, viscosity/surface curvature visits, dense
entries written, HVP products and host/device transfers. A count represents
executed work, not planned work. Early rejection publishes only the causally
completed prefix.

Ten cold allocation/execution runs and two fresh Release builds must produce
byte-identical canonical payload and work roots. CUDA `memcheck`, `initcheck`
and `synccheck` must report zero errors. NCGA0, NCGA1 and the retained
historical tiny CUDA control run without source or expected-root changes.

The evidence report freezes the contract/source/binary/stdout/fixture/graph/
observable/work roots, compiler flags, GPU/toolchain identity, repository
snapshot and parent-to-snapshot diff. A fresh reviewer receives only the
contract, exact candidate snapshot and neutral review request; the candidate
is read-only during review.

## Resolution and claim ceiling

- `SUPPORTED_BOUNDED`: all positive, derivative, negative, repeatability,
  sanitizer and regression gates pass and an independent reviewer returns
  `GO`.
- `REFUTED`: an admitted positive differs from the independent oracle; retain
  the first mismatch and stop before operator/solver work.
- `INCONCLUSIVE`: identity/environment closure fails, an oracle shares
  candidate logic, required executed work is invisible, or one load-bearing
  defect survives the single repair/re-review.

Even a reviewed positive result establishes only tiny strict-binary32 objective
assembly correspondence for this Nonlocal audit profile. It establishes no
matrix-free production operator, factorization, nonlinear solver, trajectory,
stability, visual water, throughput, runtime integration, canonical GPU
authority or product-ready water. SPEC-38's V1 product candidate remains CPU
DFSPH and disables viscosity and surface tension.

