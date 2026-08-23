# NSR3-B4E2D7R19R7 -- second guard-boundary replay contract

Status: `FROZEN / IMPLEMENTATION_NEXT / REPLAY ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r19r7-second-guard-boundary-replay|v1|parent=4534ba1939f73d78e183d23239f6621761d79493:67dfb6781a5840e228b63f3524bf4999ca6d1d9b650f8c461117575a8172611c:a2687bac0ba18b58ca4903047de6645c389174512452dad82c5e0d46320f5811|legacy=r5-stdoutfe0a75877bf539e37e11955f25cebb05821dcbacf7cb50eb120913367f664294;r2-stdout3dad88903f5f619d540587e805b35d63e2ef8c848e53e1ab87786c9e90587ba0|target=transaction-bf4e9dad4b6b8690905d07b3609dc6f0483205276b199ebcc3a354ff5a09cc5d;outer2;accepted9;rejected0;total-hvp227;failure-structural-budget-guarded-hvp-denied;first-later-denial-after-one-guard-convergence|capture=passive;predicted-current-u-theta-radius-gradient;outer-trial-solve-identity;first32-iterations;pre-denial-work|guard=prefix-finite-positive-interior;ratio-after32>eta;ratio-after32<=1.25eta;last8-next-ratios-strictly-decrease;clauses-separate;live-decision-unchanged|offline=same-steihaug;dimensionless-forcing;binary64-topology;no-preconditioner;first32-exact;cap128;no-model;no-trial;no-acceptance;no-precision|diagnostics=residual;curvature;boundary;orthogonality;conjugacy;ritz|controls=r19r6-parent-bytes;r19r5+r19r2-retained;target-state;capture-no-work;guard-decomposition;live-prefix-offline-exact;static-binding;finite;work;all-pair0;rollback|routes=second-guard-offline-nonfinite;second-guard-offline-negative-curvature;second-guard-offline-boundary;second-guard-offline-forcing-converged;second-guard-offline-cap-exhausted|precedence=nonfinite,negative,boundary,converged,cap|runs=2-clean-release-builds;1-process-each;byte-exact|work=control-parent-substeps1;candidate-substeps0;offline-hvp<=128;workspace1;precision0;trial0;acceptance0|second-substep=none;macro=none;trajectory=none;timing=none;public-commit=none;physics-mutation=none;guard-change=none;production-cap-change=none|credit=one-replay-only-later-guard-denial-diagnostic
```

Identity SHA-256:
`0f92e0253718b41fdfdc4de703ab0d276801fa350368c077e989f5860ee26dd5`.

## Required command

Add `--nonlocal-al-second-guard-boundary-replay`. It must:

1. reproduce D7R19R6 stdout SHA
   `67dfb6781a5840e228b63f3524bf4999ca6d1d9b650f8c461117575a8172611c`
   and semantic result
   `a2687bac0ba18b58ca4903047de6645c389174512452dad82c5e0d46320f5811`;
2. retain exact R5 and R2 stdout roots frozen in the identity projection;
3. retain R6 transaction root
   `bf4e9dad4b6b8690905d07b3609dc6f0483205276b199ebcc3a354ff5a09cc5d`
   and its `2/9/0/227` outer/accepted/rejected/total-HVP facts;
4. passively capture the first later `GUARDED_HVP_DENIED` solve after exactly
   one guarded HVP convergence, without changing R6 work or decisions;
5. record exact current/predicted/dual/theta/radius/gradient, static-support,
   outer/trial/solve identity, pre-denial counters and all first-32 recurrence
   iterations;
6. report the finite-positive-interior prefix, lower ratio, upper ratio and
   last-eight strict-decrease clauses separately and prove the live denial;
7. reject `ratio_after_32 <= eta` as a live forcing contradiction and require
   at least one frozen guard clause to explain the denial;
8. rebuild one exact sparse workspace and reproduce all first-32 offline
   recurrence projections before continuing;
9. continue only the captured recurrence under an offline cap of 128 HVPs,
   unchanged dimensionless forcing, binary64 topology and no preconditioner;
10. record finite/nonfinite, curvature, trust-boundary, forcing, residual,
    recurrence and CG-derived Ritz diagnostics;
11. consume no model HVP, trial, acceptance or precision audit and retain
    static binding, lifecycle, zero all-pair work and rollback;
12. run no candidate substep, transaction continuation, second substep,
    macro, trajectory, timing lane or public state mutation;
13. run one fresh process from each of two clean Release builds.

## Hard failures

Identity/parent bytes, target transaction/state/work identity, passive-capture
work, guard decomposition, live denial, first-32 replay equivalence, static
binding, finite/lifecycle, all-pair count, rollback, execution scope,
build/process repeat or route precedence mismatch is hard FAIL.

## Routes and precedence

1. `SECOND_GUARD_OFFLINE_NONFINITE`;
2. `SECOND_GUARD_OFFLINE_NEGATIVE_CURVATURE`;
3. `SECOND_GUARD_OFFLINE_BOUNDARY`;
4. `SECOND_GUARD_OFFLINE_FORCING_CONVERGED`;
5. `SECOND_GUARD_OFFLINE_CAP_EXHAUSTED`.

The report must expose the guard-denial clause independently of the offline
termination route.

## Authority boundary

This contract grants one replay-only diagnostic of the first later denied
guard boundary. It does not authorize a guard/cap change, model or trial use,
transaction continuation, another substep, macro, trajectory, timing,
performance claim, public state or runtime/production authority.
