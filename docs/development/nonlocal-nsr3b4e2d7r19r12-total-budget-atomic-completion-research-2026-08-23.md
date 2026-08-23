# NSR3-B4E2D7R19R12 total-budget atomic completion research

Date: `2026-08-23`

Status: `COMPLETE / PASS ACCEPTANCE CANDIDATE / SHADOW ONLY`

## Question

D7R19R11 proves that the Krylov solve interrupted at the transaction-wide
HVP-512 boundary is healthy and reaches the existing dimensionless forcing
criterion on HVP 24. The live prefix already paid for 14 of those HVPs.

The smallest unresolved question is therefore not whether the whole private
transaction should receive a larger budget. It is whether the already-started
outer-5/trial-1 solve can be completed as one bounded atomic unit and produce
a numerically classified trial under the existing model, reduction, precision,
acceptance and trust-radius policies.

## Facts inherited from R10 and R11

```text
live total-HVP boundary       512
target outer / trial / solve  5 / 1 / 20
live recurrence prefix        14 HVP
offline recurrence total      24 HVP
post-boundary recurrence debt 10 HVP
ordinary direct model debt     1 HVP
projected live total          523 HVP
```

The exact boundary, recurrence, prefix, full offline trace and returned-step
roots are already closed by R11. Every offline iteration is finite,
positive-curvature and trust-interior. The solve reaches forcing at
`0.8478329061 eta`; its 24-dimensional Ritz condition is `26.6716`. There is
no evidence for a preconditioner or for changing the per-solve policy at this
boundary.

The `24` HVPs physically executed by the offline control are not the proposed
live reserve. Fourteen reproduce work already present at the live boundary.
Any completion-policy accounting must expose both numbers and credit only the
ten-HVP tail as new recurrence debt.

## Alternatives

| Alternative | Benefit | Defect | Decision |
|---|---|---|---|
| Raise the transaction cap unconditionally | Simplest implementation | Gives every later solve an unbounded entitlement and mixes this trial with unknown downstream work | Reject |
| Resume CG from a serialized HVP-14 state | Executes only the ten-HVP tail | The live capture does not freeze all mutable recurrence state as a restart ABI; adding one now would test a different mechanism | Defer |
| Recompute the exact recurrence, then complete one shadow trial | Uses the already-verified deterministic replay and existing solver semantics | Physically repeats 14 HVPs in the control | Select |
| Replace the model image by the residual identity | Avoids the direct model HVP | R11 converges inside the ordinary base solve; residual-model grace is neither needed nor the live policy for this case | Reject |
| Add a preconditioner | Might help other systems | R11 exposes no conditioning signal and would change the recurrence under test | Reject |
| Continue the private transaction after this trial | Directly tests eventual confirmation | Conflates atomic completion with an unknown number of later solves and commits | Reject |

## Selected experiment

R12 is one rollback-only shadow completion of the exact captured trial:

1. reproduce R11 stdout and semantic bytes through an internal capture that
   leaves the public R11 report unchanged;
2. bind the exact R10 boundary and exact R11 24-HVP recurrence/step;
3. require exact first-14 correspondence and classify exactly ten recurrence
   HVPs as post-boundary debt;
4. rebuild the binary64-owned sparse current workspace and reproduce the
   captured gradient;
5. evaluate one ordinary direct `H(step)` and compute
   `predicted = -g dot step - 0.5 step dot H(step)`;
6. form exactly one `current + step` trial and one trial workspace;
7. compute the existing raw, inherited-divided and selected precancelled
   divided reductions; repeat the selected reduction only to close
   deterministic equality;
8. apply the existing binary64 acceptance threshold
   `predicted > 0`, `reduction > 0`, `rho >= 0.1`;
9. on a would-accept trial, run the existing binary64-owned long-double audit;
   run binary128 only when precancellation changes the acceptance decision;
10. classify the existing trust-radius update once, release both workspaces
    and prove rollback of every captured input;
11. do not write the candidate position, dual state, radius or any transaction
    counter back to the parent capture.

The raw and inherited reductions are comparators needed to identify whether
precancellation caused a candidate effect. They do not create extra trials or
acceptances. Binary128 remains a candidate-effect oracle only; it is not a
runtime policy.

## Precision and route semantics

A binary64 would-accept decision is only an acceptance candidate. The shadow
classifies it as:

- precision contradiction when a required precision audit resolves negative;
- oracle required when the required audit is finite but unresolved, or a
  candidate-effect binary128 audit does not validate the selected reduction;
- acceptance candidate only when all required audits resolve positive and,
  for binary128, its candidate bound passes;
- rejected when binary64 acceptance conditions are not met.

Nonfinite trial/model data and a nonpositive direct model are earlier routes.
Parent identity, recurrence/prefix/step correspondence, divided repeat,
precision-ledger, lifecycle, rollback and execution-scope defects are hard
FAIL rather than physical classifications.

Route precedence is:

1. `TOTAL_BUDGET_ATOMIC_NONFINITE`;
2. `TOTAL_BUDGET_ATOMIC_NONPOSITIVE_MODEL`;
3. `TOTAL_BUDGET_ATOMIC_PRECISION_CONTRADICTION`;
4. `TOTAL_BUDGET_ATOMIC_ORACLE_REQUIRED`;
5. `TOTAL_BUDGET_ATOMIC_REJECTED`;
6. `TOTAL_BUDGET_ATOMIC_ACCEPTANCE_CANDIDATE`.

## Work and authority boundary

The control may physically execute the R11 parent replay, two new sparse
workspaces, one direct model HVP, one trial classification, one long-double
audit iff the trial would accept, and one binary128 audit iff it is a
precancellation candidate effect. Its projected live completion cost is
exactly `10 recurrence + 1 model = 11 HVP`, producing projected total `523`.

R12 does not authorize a cap increase, restart ABI, transaction continuation,
state commit, another solve, another substep, macro, trajectory, timing,
performance claim, runtime policy or production use. Its result can justify
only research and freezing of a later soft-cap/completion-reserve policy.

The selected experiment is frozen by the
[D7R19R12 atomic-completion contract](../plans/nonlocal-nonlinear-solver-research/03b4e2d7r19r12-total-budget-atomic-completion-contract.md).
Its reproducible result is recorded in the
[D7R19R12 evidence](nonlocal-nsr3b4e2d7r19r12-total-budget-atomic-completion-evidence-2026-08-23.md).
