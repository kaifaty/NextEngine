# NSR3-B4E2D7R4 step-norm trust inner research

Date: `2026-08-22`

Status: `RESEARCHED / CONTRACT_READY / PRIVATE_INNER_ONLY`

## Selected change

D7R3 selects a rejected-step-norm trust update, not a new optimizer. The
candidate retains the same objective, gradient, Hessian-vector product,
Steihaug truncated CG, stationarity stop, acceptance ratio and reject cap.

Only a rejected *interior* proposal changes radius ownership. With
`gTs=gradient dot step` and the D7R1 fixed-order direct actual reduction:

```text
delta_f = -direct_actual_reduction
denom   = 2 * (delta_f - gTs)
alpha   = -gTs / denom

Delta_candidate = min(max(alpha, 0.25) * ||s||, 0.5 * Delta_current)
```

The update is valid only when all operands are finite, `denom>0`, `alpha>0`,
`||s|| < 0.9*Delta_current` and the candidate radius is positive and strictly
smaller. Otherwise the exact old `0.25*Delta_current` update executes.

Acceptance deliberately remains based on the current raw total-energy
reduction and ratio. The factorized difference is used only to estimate the
new radius, so this stage does not silently replace the merit gate.

## Execution boundary

Implement a separate candidate inner function and command. Replay only the
exact post-outer-9 private state that defeated D7R. Record every trial,
including whether the new update or fallback owned the radius. The candidate
may update its private inner iterate, but cannot update multiplier, outer
state, public state or commit count.

The first rejection and new proposal must reproduce D7R3 exactly. The entire
private inner solve must converge within the unchanged 64-trial/8-reject
bounds, with every accepted step satisfying the unchanged raw admission gate.

A scalar negative control supplies a nonpositive interpolation denominator
and must select exact quarter-radius fallback. This prevents undefined
interpolation from becoming an accidental radius increase.

PASS yields only a tiny inner-policy candidate. The next contract must still
integrate it into the complete private AL outer/confirmation transaction and
prove pressure-state stability before any physical trajectory.
