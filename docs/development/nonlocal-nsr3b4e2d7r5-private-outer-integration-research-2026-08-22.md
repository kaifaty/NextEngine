# NSR3-B4E2D7R5 private outer-AL integration research

Date: `2026-08-22`

Status: `RESEARCHED / CONTRACT_READY / PRIVATE_TRANSACTION_ONLY`

## Question

D7R4 proves that step-norm-aware radius ownership repairs the exact inner
`REJECT_LIMIT` in two trials. It does not prove that the nested unilateral PHR
augmented-Lagrangian sequence stabilizes its pressure state.

The next experiment must answer:

> If every inner call uses the selected candidate, can the unchanged D7R outer
> protocol produce two consecutive dimensionally admissible pressure updates
> within its original 14-update cap?

## Integration boundary

Create a separate private outer function. Replace only
`solve_al_vector_inner` with `solve_al_vector_inner_step_norm`. Retain exactly:

- `beta=1226.25 J` and every density/PHR formula;
- maximum 14 outer updates;
- primal `<=1e-8`;
- inner stationarity `<=1e-8`;
- complementarity `<=1e-9`;
- absolute multiplier update `<=1e-8 J`;
- equivalent pressure update `<=8e-5 Pa`;
- position update `<=1e-8 dx`;
- non-negative multipliers and monotone primal violation;
- two immediately consecutive admissible updates before confirmation.

The first eight outer records and state roots must remain exactly D7/D7R. The
new policy is expected to activate only at the later failed inner state; if it
changes the prefix, integration is invalid.

## Outcomes

### Confirmed pressure state

If two consecutive outer updates meet every gate, retain the second state as a
private confirmation candidate. Immediately rerun one private warm outer
update from it and require the same absolute pressure-state gates. This is a
holdout, not a third commit.

### Outer cap

If every inner call passes, primal remains monotone and the 14-update cap is
reached without confirmation, report the complete residual/update sequence.
Only an exact improving tail may route to outer-cap/nested-accuracy research;
the cap is not increased here.

### Inner failure

Any new inner reject/min-radius/outer-limit or nonfinite result is preserved as
`INNER_POLICY_INSUFFICIENT`. It does not authorize a second trust-policy tweak.

## Publication boundary

All position/multiplier states remain private. Even a confirmed candidate has
`public_commit_count=0`; it authorizes a separate dense-AL holdout contract,
not nominal Dam/Hydro. Inactive/reset, forced rollback and every D7--D7R4
command remain exact.

This ordering prevents a tiny symmetric fixture from becoming production
authority merely because its original failure was repaired.
