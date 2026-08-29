# NSR3-B4E2D7R19R14 soft-cap suspension research

Date: `2026-08-24`

Status: `RESEARCH COMPLETE / PROJECTION PASS / TOKEN VALIDATION NEXT`

## Question and budget audit

R13 proves that the in-flight outer completes at total HVP `523`, but the
existing budget treats `512` as a hard per-HVP stop. A useful policy must
distinguish admission of new work from completion of already-owned work.

The current `maximum_hvp_per_trust_step = 34` counts both recurrence and
ordinary direct-model HVPs. The target trial began at total `498`; therefore:

```text
soft admission boundary      512
trial start total             498
trial hard allowance           34
dynamic trial hard ceiling    532
actual recurrence              24
actual direct model             1
actual trial total             25
completed total               523
soft-cap overshoot              11
unused trial allowance          9
```

The exact trial is bounded without raising the global hard ceiling for later
work. R13 then proves inner and outer completion with zero additional HVP.

## Selected policy projection

R14 does not yet modify the solver. It freezes a projection over exact R13:

1. an inner trial may start only while `total_hvp < soft_limit`;
2. admission records `trial_start_total` and grants the existing per-trial
   hard allowance, producing dynamic ceiling `start + 34`;
3. once admitted, recurrence/model HVPs are charged to that trial even when
   cumulative total crosses the soft limit;
4. exceeding the dynamic ceiling remains a hard structural failure;
5. after trial completion, inner must complete without another trial for the
   current narrow policy to suspend safely; otherwise return an explicit
   unsupported-inner-suspension route;
6. outer finalization may execute because it consumes zero HVP and R13 proves
   its exact finite boundary;
7. before outer 6 admission, `523 >= 512` returns a resumable outer-boundary
   suspension instead of convergence, failure or public commit;
8. the continuation token owns exact private position/dual, `next_outer=6`,
   previous primal/admissibility/provisional state, predicted/theta/static
   identities and cumulative/slice ledgers;
9. resume starts a new budget epoch but must retain cumulative accounting and
   revalidate every token identity. R14 does not execute resume.

The policy is deliberately fail-closed for any over-cap accepted trial whose
inner stationarity still requires another solve. General inner suspension
would need a separately frozen serialization ABI.

## Routes

1. `SOFT_CAP_PROJECTION_INVALID`;
2. `SOFT_CAP_ATOMIC_RESERVE_EXCEEDED`;
3. `SOFT_CAP_INNER_SUSPENSION_UNSUPPORTED`;
4. `SOFT_CAP_OUTER_BOUNDARY_INCOMPLETE`;
5. `SOFT_CAP_OUTER_BOUNDARY_SUSPENDED`.

Only the last route creates a continuation-token candidate. It is not a state
commit and does not authorize outer 6.

## Authority boundary

R14 may reproduce exact R13 and build one deterministic policy/token
projection. It performs zero new workspace, HVP, model, trial, precision,
outer or physics work. It does not change budget code, execute resume, admit
outer 6, commit state, run another substep/macro/trajectory/timing or claim
runtime/production readiness.

The policy projection is frozen by the
[D7R19R14 soft-cap suspension contract](../plans/nonlocal-nonlinear-solver-research/03b4e2d7r19r14-soft-cap-suspension-contract.md).

The projection passes with the evidence recorded in the
[D7R19R14 soft-cap suspension evidence](nonlocal-nsr3b4e2d7r19r14-soft-cap-suspension-evidence-2026-08-24.md).
