# NSR3-B4E2D7R18R4 full normalized precancelled transaction research

Date: `2026-08-22`

Status: `RESEARCH_COMPLETE / CONTRACT_FROZEN / IMPLEMENTATION_NEXT`

## Question

Does the complete normalized private transaction recover the D7R13
confirmation/holdout semantics when its only solver change is the R3-certified
pairwise-precancelled actual-reduction numerator?

R3 proves one exact outer-1/trial-2 pair. It cannot establish what happens
after that trial is accepted: later positions, dual updates and near-floor
reductions are different from R2. A complete rollback-only transaction is the
smallest experiment that can expose those consequential states before a
nominal substep is even researched.

## Frozen solver delta

Retain R2's direct normalized objective, gradient, HVP, Steihaug step, trust
radius policy, stationarity limit, outer update and admission mapping. Replace
only the actual numerator used by ratio, acceptance and rejected-radius
interpolation:

```text
R2: active_delta = rounded_trial_active - rounded_current_active

R4: current_density and density_delta from sorted current/trial pair union
    current_constraint = current_density/rest_density - 1
    constraint_delta = density_delta/rest_density
    current_active_argument = u + current_constraint
    active_argument_delta = constraint_delta
    piecewise active-square delta
    PHR reduction = -0.5 * theta * active-square delta
```

The inertia delta and compensated center reduction remain unchanged. R4 must
call the separately named R3 evaluator; it may not alter the inherited R2
function or silently replace R2 regression behavior.

## Precision policy

Every R4-accepted trial receives the existing direct normalized
naive/compensated long-double audit under the 1024-ULP rule. A resolved
negative accepted sign is a contradiction.

A trial is a precision candidate-effect whenever R4 accepts it but either:

- direct binary64 `current.total-trial.total` would reject it; or
- R2's rounded-active divided numerator would reject it.

Every such pair receives the direct normalized naive/compensated binary128
audit under the 4096-ULP rule, exact pair membership and 5% candidate-relative
bound. This definition covers both the old raw-cancellation boundary and the
newly repaired R2 boundary.

## Correspondence and work

Execute the same five private transactions as R2:

```text
reference active
reference active repeat
reference inactive
aligned active
aligned inactive
```

Reference and aligned `theta` must remain exactly
`0x3fc5cccccccccccd`. Complete active/inactive roots must be byte-exact across
profiles and the reference repeat. The expected D7R13 semantics are frozen
before execution:

| Profile | Provisional | Confirmation | Holdout | Per active run work |
|---|---:|---:|---:|---:|
| active | `11` | `12` | `13` | `19` accepted / `0` rejected / `38` HVP |
| inactive | `0` | `1` | `2` | `0` accepted / `0` rejected / `0` HVP |

The exact work expectation is appropriate here: R4 claims arithmetic
correspondence with the already frozen D7R13 policy, not merely eventual
convergence under a different path.

All R2 invalid-prework, static binding, precision ledger, pair-union,
workspace lifecycle, nonnegative-`u`, primal monotonicity, confirmation,
holdout and forced rollback controls remain mandatory. The candidate uses no
all-pair path and binary128 remains an offline audit only.

## Routes

1. `PRECANCELLED_ACCEPTED_SIGN_CONTRADICTION`.
2. `PRECANCELLED_ORACLE_OR_REDUCTION_BOUND_REQUIRED`.
3. `FULL_NORMALIZED_PRECANCELLED_PRIVATE_STATE_CONFIRMED`.
4. `PRECANCELLED_INNER_POLICY_STILL_INSUFFICIENT`.
5. `PRECANCELLED_OUTER_STATE_FORMULATION_REQUIRED`.

The confirmation route requires all expected indices and exact work. A finite
result that misses them is classified by the inner/outer routes rather than
being promoted as a different but acceptable solver.

## Rejected alternatives

- Patch the existing R2 function in place: this would destroy the negative
  regression that localizes the repair.
- Run only the reference profile: cross-scale exactness is a central reason
  for the normalized representation.
- Audit only raw candidate effects: R3 shows a trial can agree with raw
  rejection while specifically contradicting the inherited divided formula.
- Relax the expected work if confirmation eventually occurs: that would hide
  an unresearched path change.
- Proceed to D7R19 after the one-pair R3 certificate: later accepted pairs and
  outer state have not yet been observed.

## Decision

Freeze R4 as a separate five-transaction private command. Only a confirmed
route may authorize research/freeze of D7R19 as one aligned nominal Dam
substep. It does not authorize D7R19 execution, another substep, a macro,
trajectory, timing, public state, GPU/runtime integration or production use.
