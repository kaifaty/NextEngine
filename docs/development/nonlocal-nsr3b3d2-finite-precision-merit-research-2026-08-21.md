# NSR3-B3D2 finite-precision merit research -- 2026-08-21

Status: `RESEARCH_COMPLETE / CONTRACT_REQUIRED / B3_RETRY_BLOCKED`

Parent D1 fails only after displacement ownership has made the inactive
reconstruction path well conditioned. At each first failure, the trust model
predicts a positive decrease of `4.55e-24--1.25e-20`, while the inherited
absolute energy floor is `2.27e-13`. Reaction residual is still above its
unchanged mixed limit, so neither accepting nor rejecting the step without a
new discriminator is justified.

## Exact identity behind the residual

For displacement-owned inertia,

```text
delta* = h*(v+h*g)
F(delta) = Phi(x+delta) + M/(2h^2) ||delta-delta*||^2
```

and the smooth momentum defect is

```text
R(delta) = M/h*(delta-delta*) + h*grad Phi(x+delta)
         = h*grad F(delta).
```

Thus the reaction gate is not an unrelated heuristic. It is the first-order
optimality residual of the same objective in impulse units. Near a stationary
point, objective decrease is quadratic in the small correction while `R` is
linear. Finite precision can therefore lose the objective difference before
it loses the stationarity signal.

## Competing hypotheses

### H1 -- accumulated-total cancellation only

Evaluate the unchanged endpoint difference term by term:

```text
Delta I = -M/(2h^2) * sum p dot (2*(delta-delta*) + p)
Delta Phi = kappa/2 * sum (c_old-c_new)*(c_old+c_new)
Delta F = Delta I + Delta Phi.
```

This avoids subtracting two accumulated totals. A gamma-bound over the exact
binary64 inputs must certify `Delta F > 0`; comparison with a `long double`
evaluation of the same factored expression checks the transcription.

### H2 -- endpoint geometry is itself unresolved

If `x+delta+p` rounds to `x+delta`, the pressure endpoint is unchanged in the
current representation even though the owned displacement and inertia state
changed. The diagnostic therefore publishes changed position components,
step/ULP ratios, density change and compression active-set correspondence. A
factored scalar alone cannot repair an unresolved geometric endpoint.

### H3 -- use the same objective's stationarity merit at the floor

When the energy sign is ambiguous but the active set is unchanged, evaluate

```text
G(delta) = 1/2 ||R(delta)||^2.
```

This is a globalization merit for the first-order equation `grad F=0`, not a
new physical energy. A later candidate may switch to it only inside the
already frozen energy-floor region, and only if the trial reaches the existing
reaction limit or produces a separately certified residual decrease. Negative
curvature, support change, non-finite state, or an unresolved residual sign
must still reject.

## Decision tree

```text
factored Delta F has certified positive sign and matches extended precision
  -> FACTORED_OBJECTIVE_DIFFERENCE_CANDIDATE

otherwise, same active/support topology and trial R reaches existing limit
  -> FLOOR_STATIONARITY_MERIT_CANDIDATE

otherwise, intended displacement is below endpoint geometry resolution
  -> LOCAL_GEOMETRY_OWNERSHIP_REQUIRED

otherwise
  -> NUMERICAL_GLOBALIZATION_STOP
```

The diagnostic uses the six exact first-floor states from D1. It does not run
past them and cannot select B3R directly.

## Decision

Execute the frozen
[B3D2 contract](../plans/nonlocal-nonlinear-solver-research/03b3d2-finite-precision-merit-contract.md).
Do not alter the reaction tolerance, pressure model, contact order or old
energy-floor behavior while diagnosing the boundary.
