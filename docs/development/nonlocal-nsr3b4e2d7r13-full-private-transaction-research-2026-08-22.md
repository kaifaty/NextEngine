# NSR3-B4E2D7R13 full private divided transaction research

Date: `2026-08-22`

Status: `RESEARCH_COMPLETE / CONTRACT_FROZEN / NOT_RUN`

## Question

Does one consistent divided-reduction solver confirm the pressure state when
started from the original private state, rather than inheriting D7R5's first
eight outer updates?

D7R12 is strong causal evidence, but its prefix was produced with the older
`eta=1e-8` inner. Several prefix records stop above `1e-10` stationarity,
including outer 1 at `4.35e-10` and outer 4 at `7.03e-10`. A production
candidate cannot silently combine two inner accuracies just because the
continuation succeeds. The next bounded experiment must use the selected
`eta=1e-10` and divided numerator from outer 0 onward.

## Frozen transaction

Start from the original `corner-box-2x2x2` compressed `0.99` state with zero
multipliers. Run outer indices 0 through at most 63. Every inner uses the
D7R10 mechanics and `eta=1e-10`; only actual reduction for ratio, acceptance
and rejected-radius interpolation differs from the pre-D7R10 solver.

Retain exactly:

```text
beta                    1226.25
maximum inner trials    64
maximum rejects         8
minimum trust radius    1e-14
initial / maximum       0.25 dx / 2 dx
accept/shrink/grow       0.1 / 0.25 / 0.75
maximum outer index     63
```

Every candidate-effect acceptance is audited by the D7R11 binary128 oracle.
The complete active transaction runs twice in process and must have one exact
root. A separate inactive `compressed=1.01` transaction must confirm without
moving particles or creating multipliers.

## Pressure and safety gates

The pressure-state gate, two-consecutive confirmation and same-gate holdout
are unchanged from D7R12. The test records every inner trial, HVP and outer
state. It rolls active and inactive public inputs back and publishes nothing.

The result may compare structural work with D7R12/D7R5, but no wall or CPU
timing is admitted on the shared host. This distinction matters for the
performance roadmap: only after the consistent full transaction is correct
can its dominant work be optimized without accelerating a mixed solver.

## Frozen classifications

1. `BINARY128_SIGN_CONTRADICTION`.
2. `ORACLE_OR_REDUCTION_BOUND_REQUIRED`.
3. `FULL_PRIVATE_PRESSURE_STATE_CONFIRMED`.
4. `INNER_POLICY_STILL_INSUFFICIENT`.
5. `OUTER_STATE_FORMULATION_REQUIRED`.

Precedence and meanings match D7R12, except the confirmation route now covers
the complete original-state transaction and requires the inactive control.

## Expected continuation

- Confirmation freezes one private state/output contract and then advances to
  a single nominal-frame shadow integration; it still does not authorize a
  trajectory or runtime publication.
- A work increase is recorded structurally and becomes input to the next
  performance roadmap stage, not a reason to weaken accuracy after seeing it.
- Any other route returns to the exact first failing state under the already
  frozen arithmetic and pressure gates.

