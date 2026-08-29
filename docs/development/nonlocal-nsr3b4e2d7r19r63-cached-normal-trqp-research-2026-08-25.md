# NSR3-B4E2D7R19R63 cached-normal TRQP research

Date: `2026-08-25`

Status: `RESEARCH COMPLETE / BOUNDED PROJECTION REFERENCE SELECTED /
CONTRACT NEXT`.

## The current tangential model is a projection problem

R62 publishes exact R43 as the private next iterate and retains the R58
dimensionless witness `n` as the normal step of the next compatible TRQP.
Trust-region filter-SQP now asks for a tangential step `t` and composite step

```text
s = n + t.
```

For the current normalized problem, the filter objective is pure inertia:

```text
f(x) = 0.5 * ||x - x_hat||^2.
```

At R43, with dimensionless composite step `s` and physical map
`x = y + SPACING*s`, this is exactly

```text
f(y + SPACING*s)
  = 0.5 * SPACING^2 * ||s - target||^2,

target = (x_hat - y) / SPACING.
```

The Hessian in step coordinates is therefore `SPACING^2 * I`. No approximate
Hessian, HVP, Krylov solve or augmented-Lagrangian curvature belongs in this
TRQP. The density term is the constraint; including its PHR curvature in the
objective would undo the R46 filter coordinate separation.

The complete linearized TRQP is the Euclidean projection

```text
minimize    0.5 * ||s - target||^2
subject to  c(R43) + A*s <= 0
            contact_lower <= s <= contact_upper
            ||s||_2 <= 0.0625.
```

The cached normal `n` is a certified feasible anchor inside this set. Once the
TRQP consumes it, the reported tangential component is `t=s-n`; `n` is neither
applied alone nor recomputed.

## Why this is the correct next boundary

Fletcher et al. decompose the SQP step as `s=n+t`, require the normal to satisfy
the linearized constraints, and then seek tangential model reduction while
the composite step remains linearly feasible and inside the trust region
(equations 2.4--2.7). Their generalized Cauchy construction is a projected
gradient path from the normal point. For the present identity-Hessian
quadratic, projection of `target` solves the entire convex TRQP reference, not
merely one Cauchy direction.

This does not make R44/R45 reusable. Their direction is the contact projection
of the complete augmented-Lagrangian gradient and was tested against composite
AL merit. R63 instead uses the pure inertial objective and preserves density
as a linear inequality, matching the filter formulation.

## Selected bounded reference

Use the existing R51 494-row basis/Gram cache and stable row order. It is
already rooted at the exact R43 operator and avoids new row VJP/Gram builds in
the first discriminator.

Run cyclic Hildreth density projections followed by one exact
`contact-box intersection trust-ball` Dykstra block per cycle:

```text
s_0          = target
density dual = 0
box dual     = 0

for each master row i:
    lambda_i <- max(0, lambda_i + residual_i / ||a_i||^2)
    s        <- s - delta_lambda_i * a_i

q            <- s + box_dual
s            <- projection_ball_intersection_box(q)
box_dual     <- q - s
residual     <- fresh c + A*s
```

Freeze exactly 64 cycles and checkpoints `8,16,32,64`, matching the existing
bounded Dykstra reference but solving a different objective. At each checkpoint
run the topology-owned R61 all-row binary64 audit. Selection and failure are
predeclared:

```text
CACHED_NORMAL_PROJECTED_TRQP_CANDIDATE
TANGENTIAL_MASTER_EXPANSION_REQUIRED
TANGENTIAL_CERTIFICATE_REFINEMENT_REQUIRED
TANGENTIAL_PROJECTION_DEPTH_REQUIRED
TANGENTIAL_MODEL_REDUCTION_REJECTED
```

The first certified checkpoint with strict inertial reduction from the cached
normal may be selected. Any raw/directed positive outside the 494-row master
requires active-set expansion; zero raw positives with bound-only positives
requires certificate-aware refinement. A positive master residual after the
fixed depth is not silently tolerated.

## Geometry and validation

Contact bounds are rebuilt at R43. Each projection must be finite and inside
the `0.0625` ball. The selected composite endpoint
`R43 + SPACING*s` is materialized only diagnostically and must pass all 36,000
R43-to-endpoint contact tests. The model reduction is evaluated exactly from
pure inertia at the normal endpoint and composite endpoint.

The R62 payload is consumed once into a private TRQP record containing
`normal`, `composite`, `tangential`, target/model roots and the selected
certificate. Runtime position/filter/trust remain unchanged.

## What R63 deliberately does not do

R63 performs no nonlinear density trial at the composite endpoint, no
switching/filter acceptance, no trust response, no multiplier/Hessian update
and no following outer. Those actions require a separately frozen R64
globalization transaction after a linearly certified TRQP candidate exists.

For production performance, the captured full-vector R51 rows are only a
reference. A later implementation should derive sparse row gradients directly
from flat topology and update overlapping rows through a particle-to-row
inverted index, a standard sparse active-set/competitive-programming pattern.
That optimization must be validated against the reference before replacement.

## Primary source

- Fletcher, Gould, Leyffer, Toint and Waechter,
  [Global Convergence of a Trust-Region SQP-Filter Algorithm for General
  Nonlinear Programming](https://www.numerical.rl.ac.uk/media/people/nick-gould/FletGoulLeyfToinWach02_siopt.pdf),
  equations 2.2--2.15 and Algorithm 2.1.

