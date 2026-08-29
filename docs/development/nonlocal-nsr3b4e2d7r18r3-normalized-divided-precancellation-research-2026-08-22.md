# NSR3-B4E2D7R18R3 normalized divided precancellation research

Date: `2026-08-22`

Status: `CLOSED / PASS / NORMALIZED_DIVIDED_PRECANCELLATION_CANDIDATE`

## Question

D7R18R2 proves that the complete normalized transaction is byte-exact across
the reference and aligned physical profiles, but it rejects active outer `1`,
trial `2`. Is that rejection caused specifically by subtracting two rounded
normalized active states, and does the D7R10 pairwise precancellation identity
restore the independently certified positive reduction without changing a
trust decision?

This stage is a replay-only formula discriminator. It cannot accept the target
trial, continue the nonlinear solve with the candidate value, update `u`, or
publish state.

## Observed boundary

The frozen R2 target has:

| Quantity | Value |
|---|---:|
| active transaction root | `c76bea9f1bff57c16e27a08c4fc51dad7029af1730c11ac80e568ebe112601a5` |
| failing inner root | `b7f44b27a51e6249b0d88b29cd08af42a7490e69ea5fed4e081cf64f48b44515` |
| outer / trial | `1 / 2` |
| stationarity | `4.3508730711255175e-10` |
| step norm | `1.0282158542755083e-11` |
| predicted reduction | `3.163331278531801e-22` |
| raw reduction | `-3.0715532249809066e-19` |
| current normalized divided reduction | `-3.0714726551614456e-19` |

The corresponding D7R13 dimensional divided reduction is
`2.277579377453211e-18`. Dividing by the exact reference objective scale
`M/dt^2=7200` yields `3.1633046909072375e-22`. The independent D7R13
binary128 result maps to approximately `3.1633252799233112e-22`. The
normalized model prediction is therefore already consistent with both prior
certificates; the current normalized actual-reduction formula is not.

## Algebraic reclosure

The normalized PHR term for one center is

```text
Pbar = theta/2 * (max(0, u+c)^2 - u^2).
```

Let `c0` be the current constraint and let `dc` be the constraint delta
computed directly from pairwise kernel deltas over the current/trial pair
union. Define:

```text
a0 = u + c0
da = dc
a1 = a0 + da
```

Then the square delta is evaluated without subtracting rounded support
states:

```text
if a0 > 0 and a1 > 0:  d2 = da * (2*a0 + da)
if a0 > 0 and a1 <= 0: d2 = -a0*a0
if a0 <= 0 and a1 > 0: d2 = a1*a1
otherwise:              d2 = 0

PHR reduction = -0.5 * theta * d2
```

The normalized inertia reduction remains

```text
-0.5 * squared_norm_delta(current_position-predicted, step).
```

This is exactly the dimensional D7R10 identity under
`lambda=kappa*u`, `theta=kappa*dt^2/M`, followed by division by the positive
objective scale `M/dt^2`:

```text
d(lambda+kappa*c) = kappa*dc
-d(active_dimensional^2)/(2*kappa) / (M/dt^2)
    = -0.5*theta*d(active_normalized^2).
```

The candidate must retain D7R10's compensated current-density and
density-delta accumulators, piecewise kernel delta, support/segment crossing
counts and sorted static current/trial pair union. Reusing
`trial.support.active_u-current.support.active_u` is explicitly forbidden.

## Smallest discriminator

R3 first reproduces complete R2 stdout bytes. It then replays the unchanged
reference active transaction only far enough to recover the already recorded
target pair from the immutable R2 result. The replay must reproduce the active
root, failing inner root, outer/trial indices and the frozen binary64 numeric
anchors before evaluating the candidate.

For the same exact current/trial positions, prediction and outer-1 `u`, R3
evaluates:

1. the inherited rounded-state normalized divided reduction;
2. the pairwise-precancelled candidate twice;
3. direct normalized long-double naive/compensated energy reductions;
4. direct normalized binary128 naive/compensated energy reductions;
5. the same candidate under independently derived reference/aligned `theta`.

The binary128 sign resolves only when naive and compensated signs agree and
both magnitudes are at least 4096 binary128 ULPs of the total-energy scale.
The candidate and the model prediction must each be within 5% relative of the
compensated binary128 reduction. The candidate trust ratio must be at least
the inherited `0.1`, but this is an observation only and cannot affect the
replay.

Long double retains the R2 1024-ULP rule as an additional observable. Exact
binary64/extended pair membership, exact reference/aligned candidate roots,
zero all-pair candidate calls, invalid-prework rejection and workspace
lifecycle are hard controls.

## Routes

1. `NORMALIZED_PRECANCELLATION_SIGN_CONTRADICTION`: the independent oracles
   resolve opposite signs, or binary128 resolves the target negative despite
   the positive model.
2. `NORMALIZED_PRECANCELLATION_BOUND_REQUIRED`: binary128 is unresolved or
   the candidate is not positive and within 5% of its compensated result.
3. `NORMALIZED_MODEL_OR_DERIVATIVE_RECLOSURE_REQUIRED`: the candidate is
   certified, but the predicted reduction is not positive and within 5%, or
   the resulting observation-only ratio is below `0.1`.
4. `NORMALIZED_DIVIDED_PRECANCELLATION_CANDIDATE`: the candidate and model are
   both certified against a resolved-positive binary128 result and are exact
   across the two normalized profiles.

## Rejected alternatives

- Retry the full normalized transaction with the same formula: R2 already
  makes that result deterministic.
- Reduce the minimum radius or relax stationarity/acceptance: those changes
  hide the first arithmetic divergence instead of repairing it.
- Use direct `current.total-trial.total`: R2 measures exactly why this is not
  a valid near-floor numerator.
- Run binary128 in the solver: the oracle remains offline evidence only.
- Accept the target and see whether the solver converges: that is a later,
  separately frozen transaction stage if and only if R3 selects the candidate.

## Decision

Freeze R3 as a no-acceptance replay of one exact pair. A candidate route may
authorize research/freeze of a complete normalized transaction using the
precancelled formula. It does not authorize that transaction, D7R19, a nominal
substep, macro, trajectory, timing, runtime binary128, public state or
production use.

## Closure

R3 selects the candidate reproducibly; see the
[dated evidence](nonlocal-nsr3b4e2d7r18r3-normalized-divided-precancellation-evidence-2026-08-22.md).
The candidate is positive, within `1.18e-5` relative of the compensated
binary128 result and exact across reference/aligned normalized profiles. The
next stage is a separately frozen full private transaction using this formula;
the replay itself remains unable to accept a trial.
