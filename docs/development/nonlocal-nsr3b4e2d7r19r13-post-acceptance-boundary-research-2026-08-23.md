# NSR3-B4E2D7R19R13 post-acceptance boundary research

Date: `2026-08-23`

Status: `COMPLETE / PASS OUTER BOUNDARY / SHADOW ONLY`

## Question

D7R19R12 proves that the solve interrupted at HVP 512 can finish with ten
recurrence HVPs plus one direct model HVP and yields a precision-confirmed
accepted trial. It deliberately stops before mutating the private inner or
outer transaction.

An accepted trust-region trial is not automatically a safe transaction stop.
The inner solver normally promotes its workspace, returns to the loop head and
checks stationarity. If stationarity is still above `1e-10`, it must start
another trust solve. Only after inner completion does the outer update rebuild
the final state, update the dual and classify its convergence fields.

The next smallest question is therefore whether the R12 trial reaches that
inner boundary and whether outer 5 can be computed without another HVP.

## Existing control flow

```text
accepted trial
  -> promote trial workspace
  -> stationarity check
     -> above 1e-10: another trust solve is required
     -> at/below 1e-10: inner returns
         -> rebuild final workspace
         -> compute constraint / dual / outer state
         -> completed outer boundary
```

The final workspace rebuild is redundant as an optimization matter, but it is
part of the current frozen implementation. R13 must reproduce that path rather
than credit a hypothetical workspace reuse.

## Alternatives

| Alternative | Benefit | Defect | Decision |
|---|---|---|---|
| Raise the live cap now | R12 shows the needed HVP debt | Does not establish a resumable or completed transaction boundary | Reject |
| Continue the full private transaction | Directly observes later convergence | Admits an unknown number of new solves and destroys causal attribution | Reject |
| Declare trial acceptance a checkpoint | No extra work | Current solver has no serialized inner workspace/restart ABI | Reject |
| Inspect accepted-state stationarity only | Determines whether another solve is needed | Does not prove the existing outer update can close when stationarity passes | Insufficient alone |
| Replay stationarity and conditionally replay the current outer-finalization path | Answers both boundary questions with zero HVP | Repeats one final workspace build | Select |

## Selected discriminator

R13 is a rollback-only child of exact R12:

1. expose an internal R12 capture without changing public R12 bytes;
2. bind the exact accepted trial root, parent transaction, outer-5 start,
   predicted state, dual, theta, static identity and projected HVP total 523;
3. build one binary64-owned workspace at the accepted trial and compute the
   exact normalized stationarity;
4. if stationarity exceeds `1e-10`, stop and classify that another trust solve
   is required;
5. otherwise release the accepted workspace, rebuild the same final workspace
   exactly as the current outer-update path does and require the two
   evaluations to match exactly;
6. compute the existing outer fields: primal, stationarity, dual change,
   complementarity, position update, dual range, finite and admissible;
7. compare outer-5 primal with the completed outer-4 primal for monotonicity;
8. classify the completed outer as nonmonotone, not admissible or admissible;
9. release all workspaces and prove every parent/candidate input unchanged.

No HVP, model, divided reduction, precision audit, trust-radius update, trial
admission or later outer/transaction work is allowed in R13.

## Routes and decision meaning

Precedence is:

1. `POST_ACCEPTANCE_BOUNDARY_NONFINITE`;
2. `POST_ACCEPTANCE_INNER_REQUIRES_SOLVE`;
3. `POST_ACCEPTANCE_OUTER_NONMONOTONE`;
4. `POST_ACCEPTANCE_OUTER_COMPLETE_NOT_ADMISSIBLE`;
5. `POST_ACCEPTANCE_OUTER_COMPLETE_ADMISSIBLE`.

The first route is a numerical stop. The second means a soft global HVP cap
cannot stop safely after R12 without either a larger bounded reserve or a
frozen suspend/resume ABI. Any of the last three proves a completed outer
boundary with no new HVP; admissibility only determines convergence progress,
not whether the boundary is structurally complete.

## Authority boundary

R13 may execute the exact R12 parent, one accepted-state evaluation and, only
when inner stationarity passes, one existing-path final-state evaluation. It
does not authorize a soft-cap policy, live cap change, state commit, another
trust solve/trial, later outer update, substep, macro, trajectory, timing,
runtime policy or production use.

The selected discriminator is frozen by the
[D7R19R13 post-acceptance boundary contract](../plans/nonlocal-nonlinear-solver-research/03b4e2d7r19r13-post-acceptance-boundary-contract.md).
Its reproducible result is recorded in the
[D7R19R13 evidence](nonlocal-nsr3b4e2d7r19r13-post-acceptance-boundary-evidence-2026-08-24.md).
