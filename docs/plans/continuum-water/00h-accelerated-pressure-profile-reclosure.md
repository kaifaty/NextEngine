# W0H — Accelerated pressure-profile reclosure

Status: `ACCELERATED_PRESSURE_ROOTS_FROZEN / W1_AUTHORIZED / RESEARCH_ONLY`.

## Purpose

The W0G-rooted Linux W1 corpus first fails in `CW-SEALED-001` step 2 because
the inherited projected active-set PCG reaches its 50th pressure-operator
application at `194,545 ppb`. A bounded
[pressure-solver research cycle](../../development/continuum-water-w1-sealed-pressure-solver-research-2026-08-18.md)
shows that every post-initial iteration changes at least one active row and
globally restarts the conjugate direction. At the 48,000-sample product scale,
the implementation therefore behaves as diagonally preconditioned steepest
descent rather than retaining a useful CG recurrence.

W0H changes only the deterministic algorithm used to solve the existing
non-negative pressure quadratic program. It does not change the W0F pressure
operator, right-hand side, density support, geometry, contact, capacities,
canonical publication or iteration-operation budget, and it leaves the W0G
energy contract byte-identical. The old PCG profile remains immutable evidence
and is an explicit parent through W0G.

## Pressure problem and coordinate transform

For the inherited density reconstruction, solve the same convex problem

```text
minimize 0.5 * k^T B k - b^T k
subject to k >= 0
B(k) = -dt^2 * D(P(k))
b = rho_adv - 1
```

Use the frozen positive diagonal factor only as a change of coordinates:

```text
inverse_diagonal_i = alpha_i / dt^2
S_i = sqrt(inverse_diagonal_i)
k_i = S_i * y_i
```

When `inverse_diagonal_i == 0`, the coordinate is disabled and its multiplier
is fixed to zero. Its unscaled residual is still included in both convergence
reductions. This preserves the inherited physical row rather than silently
dropping dispersed rows from acceptance; the case occurs in the dam-break
corpus.

## Fixed accelerated iteration

Start every substep from `y[-1] = y[0] = 0`, with no warm state. For iteration
`n` starting at one:

```text
beta_n = (n - 1) / (n + 2)
z_n = y_n + beta_n * (y_n - y_(n-1))
gradient(z_n) = gradient(y_n) + beta_n *
                  (gradient(y_n) - gradient(y_(n-1)))
y_(n+1) = max(0, z_n - 0.25 * gradient(z_n))
```

The affine gradient identity makes the extrapolated gradient exact for this
quadratic. Each iteration then performs exactly one pressure-operator
application at `y_(n+1)`. Reductions use ascending sample ID and an explicit
left fold.

The fixed step must also satisfy the local quadratic majorization witness for
the actual projected displacement:

```text
directional_curvature = dot(d, gradient(y_next) - gradient(z)) / dot(d, d)
directional_curvature <= 4
```

A zero displacement has curvature zero. A larger or non-finite value is typed
density non-convergence; there is no backtracking, retry or dynamic step
selection outside the frozen one-operator budget.

## Acceptance

Acceptance requires both existing physical compression convergence and the
projected first-order optimality condition:

```text
2 <= pressure_operator_applications <= 50
mean(max(rho_adv_after_pressure - 1, 0)) <= 100000 ppb
mean(projected_kkt_residual) <= 100000 ppb
```

For multiplier `k_i > 0`, the projected-KKT row is `abs(gradient_i)`. At the
zero bound it is `max(-gradient_i, 0)`. Both reductions use original,
unscaled coordinates. This prevents a compression-only acceptance from
hiding a materially non-stationary multiplier vector.

## Frozen solver projection

The following LF-terminated block is the complete W0H solver input. Its W0G
parent execution root is immutable.

```text
ACCELERATED_PRESSURE_PROFILE_V1_BEGIN
profile.id=diagonal-scaled-accelerated-projected-gradient-v1
parent.execution-profile=8f74f45e4952c419bd4fed9125ae9b22c873eaae28755ee38bef77922e3186f6
qp.operator=B(k)=-dt2*D(P(k))
qp.rhs=b=rho-adv-minus-one
qp.constraint=k-greater-than-or-equal-zero
coordinate.enabled=inverse-diagonal-greater-than-zero;scale=sqrt(alpha*inverse-dt2);k=scale*y
coordinate.disabled=inverse-diagonal-equals-zero;multiplier=fixed-zero;unscaled-residual-retained
initial.multiplier=zero
iteration.step=0.25;bits=0x3fd0000000000000
iteration.momentum=(iteration-1)/(iteration+2)
iteration.projection=max(0,extrapolated-(step*scaled-gradient))
iteration.operator-applications=one-per-iteration
iteration.reduction=ascending-sample-id-left-fold
majorization.directional-curvature-maximum=4
majorization.failure=typed-density-nonconvergence
convergence.minimum-iterations=2
convergence.maximum-iterations=50
convergence.mean-positive-compression-maximum-ppb=100000
convergence.mean-projected-kkt-maximum-ppb=100000
warm-start=disabled
dynamic-step-selection=disabled
ACCELERATED_PRESSURE_PROFILE_V1_END
```

## Root composition

W0H issues domain-separated roots for:

1. this whole document;
2. the exact marked accelerated-pressure block;
3. a child corpus root over the W0G corpus and W0H solver-profile roots;
4. one scenario root over its W0G scenario and W0H solver-profile roots;
5. a composite execution profile over the W0G execution profile, W0H
   document, solver-profile and child-corpus roots.

All W1 frames, external-reference headers and corpus-run roots produced after
W0H use its composite execution and scenario roots. They cannot collide with
the rejected PCG trajectory lineage. W0F and W0G documents and their constants
remain byte-identical.

## Required checks

W0H may authorize W1 continuation only if all checks pass:

1. The complete W0F/W0G parent chain verifies without edits.
2. Production and a separately written support-complete calculator match the
   hydro first step exactly on accepted iteration, compression residual,
   projected-KKT residual and maximum multiplier bits.
3. The fixed step and momentum schedule are exact, cold-started and perform
   one pressure-operator application per iteration.
4. A zero inverse-diagonal row remains fixed at zero but participates in the
   unscaled convergence metrics.
5. The majorization guard rejects a step whose directional curvature exceeds
   four.
6. The complete internal Linux discriminator passes hydro, exact free fall,
   dam break, repeated still tank, orifice, repeated sealed-48k and all three
   storage orders without a threshold change.
7. Two clean W0H closure reports have identical bounded semantic projections,
   excluding only wall-clock time and output path.

The internal discriminator gives no W1 corpus or ProductCheck credit. W1 still
requires clean rooted execution and independently generated hydro, dam-break
and orifice reference curves.

## Rejected alternatives and stop rules

- Do not raise the 50-application ceiling; the PCG-200 control needs 96
  applications on sealed step 2 and still has a larger projected-KKT residual
  at compression-only acceptance.
- Do not select the tested MPRGP or projected-CG-expansion variants; neither
  reaches a feasible CG step in the first 50 applications on the failing
  active-set sequence.
- Do not use step `0.5`; it violates its curvature bound in hydro step 6.
- Do not add warm start, hidden continuation state, backtracking or adaptive
  steps under these roots.
- If independent equality, clean W0H closure or the rooted W1 corpus fails,
  reject this profile and move to an active-set-aware multilevel
  preconditioner. Do not loosen compression, KKT, clearance, energy or
  reference thresholds.
- Do not start W2, GPU, PhysX coupling or runtime/public contracts before the
  complete W1 gate passes.

Windows repeatability is outside the current W1 execution scope by user
decision. It remains a deferred production-promotion gate and is not waived by
W0H.
