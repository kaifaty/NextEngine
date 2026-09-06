# NCGP9 complete 4k product-correctness corpus

| Field | Value |
| --- | --- |
| Research ID | `NCGP9` revision 1 |
| Status | `FROZEN / CORRECTNESS_ONLY / PERFORMANCE_BLOCKING` |
| Frozen before implementation | 2026-08-31 |
| Authorized parent | reviewed NCGP8 repair `8f7d9850`, H8A / independent GO |
| Architecture status | SPEC-38/ADR-076 remain Proposed; ADR-081 guardrails remain Accepted |
| Claim class | finite/profile-bound standalone 4k correctness; no 16k/50k, frame-time or runtime claim |

## Question and stop boundary

Can the reviewed corrected compensated FCR2 GPU solver complete the entire
one-second 4,000-sample corpus for hydrostatic hold, dam break and an orifice
jet while preserving same-state formula correspondence, physical invariants,
permutation identity, bulk water fields and visible sphere geometry?

NCGP9 changes no physics coefficient, solver rule, arithmetic representation,
HVP ceiling, boundary rule or reviewed observer threshold. It does not time the
solver. The first valid failure stops the ordered corpus. A complete NCGP9 PASS
authorizes only separately frozen 16k/50k capacity and correctness work. CPU
DFSPH remains the product fallback.

The historical NCGP4 5 mm maximum and NCGP6 2.5 mm stable-ID p99 failures
remain exact facts. Stable-ID trajectory errors remain reported and sealed but
are diagnostic after the mandatory same-state first step; NCGP9 does not
rewrite either old result.

## Exact solver and profile

All scenarios use:

```text
profile_id        = nonlocal-water-50k-v1
dt                = 1/240 s
spacing           = 0.05 m
horizon           = 0.15 m
mass              = 0.125 kg
gravity           = (0, 0, -9.81) m/s^2
dynamic samples   = 4,000
ghost layers      = 3
GPU arithmetic    = corrected compensated FCR2 binary32 (hi,lo)
CPU arithmetic    = independent binary64/long-double reference
solver             = unpreconditioned Steihaug--Toint
total HVP ceiling = 128 per step
steps             = 240 per scenario
```

The profile coefficients are the exact corrected-FCR values already sealed by
NCGP8. Dynamic and ghost inputs are explicitly canonicalized through binary32
before both CPU and GPU routes. Corrected and stable-ID-permuted GPU executions
start from one immutable input byte set. No vector, position, Hessian or field
round-trip may influence the GPU step.

## Frozen ordered corpus

Run exactly in this order, with a fresh workspace and CPU state for each:

1. `hydrostatic-hold`: canonical `20 x 20 x 10` lattice;
2. `dam-break`: canonical `10 x 20 x 20` lattice;
3. `orifice-jet`: canonical `20 x 20 x 10` lattice; initial velocity is
   `(1.5,0,0) m/s` exactly when
   `(y-0.75)^2 + (z-0.25)^2 <= 0.15^2`, otherwise zero.

The existing generators are part of the frozen input definition. Sample IDs
may only be permuted in the explicit permutation route. Basin geometry,
contact, ghosts and current-neighbour graph rebuild remain identical across
the three routes. Each accepted GPU point rebuilds the current graph; the graph
is fixed within one energy/gradient/HVP evaluation.

## Admission and retained controls

Before the corpus, all exact reviewed controls must pass:

- corrected profile and input admission;
- compensated graph and strict-radius control;
- corrected operator/HVP, including inactive and mixed pressure centres;
- free fall, translation/rotation invariance, viscosity decay and surface
  relaxation;
- analytical contact/ghost boundary and post-finalize rollback;
- product-gate apparatus and NCGP7 retained bulk evaluator;
- reviewed NCGP8 corrected-axis observer controls and root recomputation.

This retains the required negative mutations: missing `2/h`, half viscosity,
wrong surface sign, HVP sign inversion, current/reference graph swap, omitted
neighbouring pressure centre, strict `<` radius, disabled ghost support,
disabled contact, permutation identity loss and capacity overflow.

NCGP9 adds real end-to-end controls for skipped scenario, swapped scenario
order, skipped visible step, skipped bulk checkpoint, mutated worst-step metric,
mutated work count, CPU-oracle failure, corrected/permuted disagreement and
result-root mutation. Labels or expected-hash changes without executing the
real path are insufficient.

## Per-step execution order

For each scenario and step `1..240`:

1. execute corrected GPU, stable-ID-permuted GPU and independent CPU steps;
2. capture public/compensated states and work receipts;
3. reject solver, nonfinite, capacity, rollback, input or identity failure;
4. verify corrected/permuted GPU state, density, active signature, result and
   work roots exactly;
5. evaluate particle count, mass, density, momentum, energy and containment;
6. build CPU, GPU and permuted NCGP8 visible-sphere images and compare them;
7. at steps `60,120,180,240`, additionally build the retained 50 mm and 25 mm
   three-dimensional bulk fields for CPU/GPU/permuted state;
8. seal the step receipt before advancing CPU or GPU state.

No later result may be computed after the first failing gate. The failure
receipt seals the last accepted state and candidate/permuted rollback state.

## Same-state and physical gates

The first accepted step of every scenario must retain the strict formula-level
gate:

```text
CPU/GPU position maximum      <= 5 micrometres
HVP relative L2               <= 1e-3
HVP cosine loss               <= 1e-6
pressure active-ID signature  exact
corrected/permuted GPU result exact
```

For every accepted step:

```text
particle count and total mass       exact
density correspondence RMSE         <= 5%
density correspondence maximum      <= 10%
normalized momentum residual        <= 1%
positive mechanical-energy excess  <= 1%
centre penetration beyond inset     <= 2.5 mm
NaN/capacity/work overflow          forbidden
corrected/permuted state/work roots exact
```

The retained physics suite, rather than a one-way trajectory surrogate,
supplies the required reversible-energy-drift gate `<=1%`. Viscous dam/orifice
trajectories are not falsely called reversible. Scenario reports still seal
initial/final energy, external work, positive excess and all reductions.

Stable-ID position RMSE/p50/p95/p99/maximum and active-signature differences
between independently evolved CPU/GPU trajectories are diagnostic after step
1. They cannot independently reject or admit NCGP9.

## Visible-surface gates

Use the exact reviewed NCGP8 observer on every accepted step:

```text
view direction = -z
image plane    = x-y, 240 x 200
sphere radius  = 25 mm
pixel pitch    = 12.5 mm
```

For every step of every scenario:

```text
silhouette symmetric difference <= 1%
depth RMSE                       <= 6.25 mm
depth p95                        <= 12.5 mm
depth p99                        <= 25 mm
material component count        exact CPU/GPU
largest-component fraction diff <= 1%
satellite-area fraction diff    <= 1%
GPU/permuted image/topology     exact
```

Hydrostatic hold additionally requires CPU and GPU satellite-area fractions
each `<=1%`. Dam break and orifice may physically form multiple visible
regions, so their absolute satellite fractions are reported but are not forced
to the hydrostatic single-region value. No image alignment, smoothing, hole
filling, sample reassignment, alternate resolution or retry is permitted.

Each scenario seals worst values and their first step. `depth_max` remains a
diagnostic warning. The exact `507 mm` NCGP8 step-112 warning is not erased.

## Retained three-dimensional bulk gates

At steps `60,120,180,240`, build exact NCGP7 50 mm and 25 mm fields but use only
their valid three-dimensional bulk channels:

```text
mass total variation            <= 1%
density RMSE                    <= 1%
normalized velocity RMSE        <= 1%
centre-of-mass error            <= 2.5 mm
GPU/permuted field/work roots   exact
```

The NCGP7 `y`-height surface columns remain emitted and sealed for regression,
but never select NCGP9. Corrected visible-surface gates above are the only
surface authority.

## Work and root closure

For each scenario, route and step, seal and publish:

- canonical input, ghost, profile, state, density, active-ID and compensated
  state roots;
- solver result/work, graph, boundary/contact and snapshot receipts;
- exact HVP/outer/accepted/rejected work and all transfers;
- physical metric inputs, reductions, work and metrics roots;
- visible contribution/mask/depth/component/image/comparison roots and every
  validation/reduction/flood/root count;
- both retained bulk fields/comparisons and their exact work roots at the four
  checkpoints;
- worst-step records, scenario receipt/root and ordered corpus root;
- contract/profile/input/source commit/tree/binary/compiler/CUDA environment,
  allocated device bytes and exact command.

The JSON is versioned and finite on every route. CUDA/runtime construction,
upload, replay, oracle or observer failure is `APPARATUS_INCONCLUSIVE`, never
silent output or physics evidence. There are no allocations in the GPU hot
step beyond the preallocated workspace; CPU oracle/observer allocations are
outside future performance timing but remain counted in correctness receipts.

## First-specific classification

Apply this precedence:

1. identity/input/CPU-oracle/evidence failure, non-matching corrected/permuted
   failure, or malformed receipt -> `APPARATUS_INCONCLUSIVE`;
2. identical admitted corrected/permuted GPU work/capacity/solver failure with
   healthy CPU oracle and exact rollback -> `PHYSICS_REFUTED_BOUNDED`;
3. first valid same-state, physical, visible or bulk gate failure ->
   `PHYSICS_REFUTED_BOUNDED`, naming scenario, step and gate;
4. all 720 steps and 24 bulk field pairs pass -> `CORPUS_PASS`.

No partial-scenario or prefix PASS exists. Dam break and orifice remain
`NOT_RUN` if hydrostatic hold fails first.

## Evidence protocol and successor

After local development closure:

- freeze one exact source commit and source tree;
- build in two fresh Release directories;
- execute two fresh complete corpus processes sequentially;
- require byte-identical binaries and output, except independently explained
  absolute-source-path binary salt in detached review builds;
- run retained controls and Compute Sanitizer memcheck/initcheck/synccheck;
- obtain one independent read-only review;
- allow one batched repair and one re-review; any remaining load-bearing defect
  closes NCGP9 `INCONCLUSIVE`.

`CORPUS_PASS` still does not measure performance. It only authorizes a new
frozen 16k/50k capacity/correctness stage. Complete 50k timing remains blocked
until that stage and the 240-step 50k sealed-basin run pass. `docs/roadmap.md`
changes only if verified evidence materially changes R8 product status.
