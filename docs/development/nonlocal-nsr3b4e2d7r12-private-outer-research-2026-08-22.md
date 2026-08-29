# NSR3-B4E2D7R12 private divided outer continuation research

Date: `2026-08-22`

Status: `CLOSED / PASS / PRIVATE_PRESSURE_STATE_CONFIRMED`

## Question

Does the D7R10 divided-reduction inner remove the actual blocker in the
pressure-state solve, or does the complete private outer continuation still
fail the unchanged physical gates?

D7R11 certifies only the three exact acceptances that repair the observed
inner floors. It does not prove that newly reachable outer states converge,
stay primal-monotone or produce two consecutive admissible pressure states
plus a holdout. The next experiment must therefore integrate the selected
binary64 numerator into the outer continuation without admitting a frame or
publishing state.

## Selected lane and fork

Resume from the exact D7R5 post-outer-7 position/multiplier prefix and run only
the `eta=1e-10` continuation through at most outer index 63.

This lane is selected before execution because:

- its outer-10 state already passes stationarity, complementarity and position
  gates and misses the common primal/dual/pressure bound by only `1.023944x`;
- its outer-11 failure is one of the exact inner floors repaired by D7R10;
- `eta=1e-11` and `1e-12` reach the identical failed state and add no
  discriminating information;
- `eta=1e-8` exhibits a five-update primal cycle, while `eta=1e-9` fails
  earlier and is not the nearest pressure-state boundary.

The exact prefix keeps prior accepted work fixed and isolates the consequence
of the selected numerator at and after the known floor. A later promotion
stage must still prove the full initial-to-final transaction.

## Frozen solver delta

Keep `beta=1226.25`, gradient, analytic HVP, Steihaug model, trust radii,
trial/reject limits, multiplier update and all dimensional gates unchanged.
Inside every continuation inner, use the D7R10 divided reduction only for:

```text
actual/predicted ratio
trial acceptance
rejected-radius interpolation
```

Raw and prior direct reductions remain observables. Every accepted trial that
the raw-total rule would have rejected is independently reevaluated by the
D7R11 fixed-order and compensated binary128 formulas. Such a trial is usable
only when its sign resolves positive above 4096 total-energy ULPs, pair
membership agrees and divided magnitude error is at most 5%.

## Unchanged pressure-state gate

An outer record is admissible only when all existing bounds pass:

```text
positive constraint          <= 1e-8
inner stationarity           <= 1e-10
complementarity              <= 1e-9
absolute multiplier change   <= 1e-8 J
equivalent pressure change   <= 8e-5 Pa
position update              <= 1e-8 dx
multiplier                   >= 0
primal sequence              monotone
```

Selection requires two consecutive admissible records and one subsequent
warm holdout under the same gates. Confirmation, holdout and every generated
state remain private and are rolled back.

## Frozen classifications

1. `BINARY128_SIGN_CONTRADICTION`: any candidate-created acceptance resolves
   negative.
2. `ORACLE_OR_REDUCTION_BOUND_REQUIRED`: no contradiction, but a candidate-
   created acceptance is unresolved or exceeds the frozen magnitude error.
3. `PRIVATE_PRESSURE_STATE_CONFIRMED`: every audit passes, two consecutive
   pressure states and the holdout are admissible.
4. `INNER_POLICY_STILL_INSUFFICIENT`: no preceding route, but a continuation
   inner reaches an unchanged trial/reject/radius limit.
5. `OUTER_STATE_FORMULATION_REQUIRED`: no preceding route; inners remain
   finite, but monotonicity, confirmation, holdout or the outer cap fails.

This is a correctness/convergence discriminator, not a speed measurement.
It records work counts for later performance planning but authorizes no CPU
or wall-clock A/B on the shared host.

## Expected continuation

- Confirmation leads to a separate full private transaction/replay contract,
  then structural performance work before a dedicated-host timing lane.
- An inner failure returns to local solver policy research at its first exact
  newly reachable state.
- An outer-formulation result keeps the divided numerator but studies the
  pressure-state update/gate mechanism rather than tuning caps after the fact.
- A sign or error-bound route stops outer integration and strengthens the
  arithmetic certificate only for the newly accepted pairs.

## Closure

D7R12 passes; see the
[dated evidence](nonlocal-nsr3b4e2d7r12-private-outer-evidence-2026-08-22.md).
Outer 11/12 satisfy the unchanged two-state gate and outer 13 is an admissible
holdout. Proceed to a separately frozen full private transaction from the
original state; do not promote the prefix-only result.
