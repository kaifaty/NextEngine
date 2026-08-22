# NSR3-B4E2D7R10 private divided-reduction inner research

Date: `2026-08-22`

Status: `RESEARCH_COMPLETE / CONTRACT_FROZEN / NOT_RUN`

## Question

D7R9R1 proves the divided-difference numerator only on the already observed
D7R7 trials. Does using it for private trust decisions let the three failed
inners converge, and are the signs of candidate-created acceptances visible
to the independent extended evaluator?

This is the first stage allowed to change a decision, but only inside a
transaction that is always rolled back. It is not an outer AL integration and
does not advance the Dam trajectory.

## Why integrate before another formula study

The eleven resolved D7R8 trials show that analytic model and divided actual
reduction agree closely once cancellation is removed. Rechecking the same
fixed trials cannot tell us what happens after the first corrected acceptance.
The next falsifiable experiment must therefore generate the newly reachable
states while retaining all existing trust-region safeguards.

The twelve unresolved old trials still matter. An 80-bit oracle cannot call
their signs under the conservative D7R8 rule, and D7R9R1 does not provide a
formal binary64 forward-error bound. Private integration may observe such
acceptances, but cannot promote them. This separates two outcomes:

- the candidate reaches the requested stationarity using only resolved-
  positive accepted signs;
- it converges numerically, but production use first needs an error
  certificate or stronger offline oracle for one or more accepted signs.

That distinction follows the bounded-noise trust-region concern described by
[Sun and Nocedal](https://arxiv.org/abs/2201.00973), while preserving the
algebraic divided-difference remedy selected in
[D7R9 research](nonlocal-nsr3b4e2d7r9-divided-difference-research-2026-08-22.md).

## Frozen solver delta

Start independently from the exact `eta=1e-8`, `1e-9` and `1e-10` pre-failure
states. Retain the D7R4 inner mechanics:

```text
maximum trials       64
maximum rejects       8
minimum trust radius  1e-14
initial radius        0.25 dx
acceptance ratio      0.1
shrink / grow ratios  0.25 / 0.75
maximum radius        2 dx
```

Gradient, analytic HVP, Steihaug step, predicted reduction and topology are
unchanged. Replace only the actual numerator used by ratio, acceptance and
rejected-radius interpolation with the D7R9R1 divided difference. Raw and old
direct reductions remain observables.

Every divided evaluation runs twice and must be bit-exact. Every candidate-
accepted trial is also reevaluated by the independent D7R8 long-double
formula. The audit calls a sign resolved only under the existing agreement
and 1024-extended-ULP rule. A resolved negative accepted sign is a direct
candidate contradiction. An unresolved sign is not called wrong, but blocks
integration authority.

## Routes

1. `RESOLVED_SIGN_CONTRADICTION`: any candidate-accepted trial has a resolved
   negative extended reduction.
2. `PRECISION_CERTIFICATE_REQUIRED`: no contradiction, all three inners
   converge, and at least one accepted sign is unresolved.
3. `PRIVATE_INNER_CONVERGENCE_CANDIDATE`: all three converge and every
   accepted sign is resolved positive.
4. `INNER_POLICY_STILL_INSUFFICIENT`: no contradiction, but at least one
   inner reaches an unchanged reject/trial/radius failure.

The run must also prove that at least one trial rejected by the inherited raw
rule becomes a candidate acceptance; otherwise it has not exercised the
selected change. Every route rolls all position/multiplier state back and
authorizes no outer update.

## Expected continuation

- A precision-certificate route leads to a bounded error-free expansion or
  stronger oracle study only for accepted near-floor deltas.
- A clean convergence route leads to a separately frozen complete private AL
  integration with pressure-state gates.
- A remaining policy failure is diagnosed from its first exact new state.
- A resolved contradiction stops this candidate.

