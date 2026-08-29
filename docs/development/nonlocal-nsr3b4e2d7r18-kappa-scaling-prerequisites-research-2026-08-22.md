# NSR3-B4E2D7R18 kappa scaling prerequisites research

Date: `2026-08-22`

Status: `RESEARCH_COMPLETE / CONTRACT_FREEZE_NEXT`

## Question

Why does the first aligned nominal AL substep make almost no primal progress
even though every inner trust solve reaches about `1.5e-12` stationarity in
one accepted trial and two HVPs, and what is the smallest safe next test?

## D7R17 observation

D7R17 changes the time step from the reference `1/240 s` to the aligned Dam
substep `1/(240*78) s`, but retains `kappa=1226.25`. Its objective is

```text
E(y) = M/(2 dt^2) ||y-y_hat||^2
     + sum_i (max(0, lambda_i + kappa c_i(y))^2-lambda_i^2)/(2 kappa).
```

At zero multiplier the second term is
`kappa/2 * max(c_i(y),0)^2`. With fixed geometry, mass and density
constraint, the relative AL/inertia curvature is governed by

```text
kappa * dt^2 / (M * L^2),
```

where `L` is the fixed length scale. Therefore reducing `dt` by `78` while
holding `kappa` fixed weakens the AL term relative to inertia by
`78^2 = 6084`.

This matches the observed mechanism: inner Newton--Krylov solves are cheap and
tight, but each outer multiplier update moves the primal state only slightly.
Increasing the outer cap would spend work around a changed dimensionless
problem rather than test the missing scale.

## Exact scale candidate

For fixed geometry, preserve the reference ratio with

```text
kappa_sub = kappa_ref * (dt_ref/dt_sub)^2.
```

The selected constants are exactly representable:

| Quantity | Decimal | binary64 bits |
|---|---:|---:|
| `dt_ref` | `0.0041666666666666666` | `0x3f71111111111111` |
| `kappa_ref` | `1226.25` | `0x4093290000000000` |
| `dt_ref/dt_sub` | `78` | exact |
| squared ratio | `6084` | exact |
| `dt_sub` | `5.341880341880342e-5` | `0x3f0c01c01c01c01c` |
| `kappa_sub` | `7460505` | `0x415c75a640000000` |
| `kappa*dt^2` | `0.021289062500000001` | `0x3f95cccccccccccd` |
| inertia scale, reference | `7200` | exact |
| inertia scale, substep | `43804800` | exact |

Both profiles produce the same binary64 `kappa*dt^2`; their inertia scales
differ by exactly `6084`.

This is a formulation candidate, not yet a selected production coefficient.
The nominal solve must not run until the implementation can express and audit
the candidate without silently retaining global `KAPPA` in one of its paths.

## Implementation audit

The sparse transaction currently receives explicit `dt` but reads the global
`KAPPA` independently in:

- workspace PHR active coefficient, energy and gradient;
- Hessian-vector product curvature;
- current/trial divided reduction;
- long-double and binary128 energy/sign audits;
- outer multiplier update and scaled dual-change accounting.

These operations must share one validated immutable scale object. The
workspace must retain the exact `kappa` bits used to build its coefficient
tape, so an HVP cannot accidentally combine a workspace from one penalty with
another penalty. Zero, negative, NaN and infinity must reject before static
support work, neighborhood/pair construction, precision evaluation, HVP or
outer update.

Historical dense and source-line controls may retain the frozen global
constant. D7R18 changes only the D7R14--D7R17 sparse candidate lineage.

## Smallest discriminator

D7R18 is a prerequisite, not a nominal solve:

1. introduce explicit finite-positive `{dt,kappa}` through the complete
   sparse candidate call graph;
2. preserve D7R16 bytes and directly regress D7R17 from a clean build;
3. reuse the D7R15 tiny active fixture with identical position, prediction and
   direction at reference and scaled profiles;
4. require dense/sparse equivalence within each profile;
5. require density, constraint and active-set identity across profiles and
   the analytically expected `6084` scaling of PHR, inertia, gradient, HVP and
   divided-reduction quantities under scaled multiplier state;
6. execute both independent long-double and binary128 paths at the explicit
   scale and require sign/membership agreement;
7. prove that a one-ULP mutation of `kappa_sub` breaks the exact
   nondimensional product and at least one active numerical root;
8. reject every invalid scale before work and retain zero all-pair candidate
   calls with at most two live workspaces.

No outer transaction is required for the scaled profile. The invalid-scale
outer entry may be called only to prove pre-work rejection.

## Rejected next actions

- Increase the outer cap: D7R17 shows only about 2% relative primal reduction
  in 16 updates and identifies a dimensional mismatch first.
- Replace AL or add contact projection: neither tests the diagnosed scale and
  would conflate independent solver/boundary questions.
- Run a `kappa` sweep: it would tune after observing the nominal result. The
  only candidate is derived from the unchanged reference dimensionless ratio.
- Run both constant- and scaled-`kappa` nominal lanes in one command: it
  repeats the expensive negative control. D7R17 is already exact evidence;
  one clean direct regression is sufficient after the prerequisite change.
- Promote `7460505` to production: the algebra establishes a candidate scale,
  not physical/corpus validity or acceptable conditioning.

## Decision

Freeze D7R18 as explicit-`kappa` propagation and a tiny dimensionless scaling
oracle with no nominal substep. If it passes, freeze a separate D7R19 command
that executes exactly one scaled private nominal substep under the same
structural budgets and route precedence as D7R17.

The performance roadmap changes accordingly: the next bottleneck experiment
is D7R19 only after D7R18 proves the scale plumbing. No timing, macro,
trajectory, public state, runtime or production authority is granted.
