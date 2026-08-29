# NSR3-B4E2D7R19R11 -- total-budget offline replay contract

Status: `FROZEN / IMPLEMENTATION_NEXT / REPLAY ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r19r11-total-budget-offline-replay|v1|parent=750719f1:15719465931a21421e094abde1200d685bee44e4b112dc314d5857f863074953:30f339247a76484071494b26db54c385a0bea637c4f9e744628a0ff83371b30c|legacy=r9-stdoutf1cb461d270e1642e9c0220c60fc59c64cb5bb69039f293ed8e9eade7f6ed1f0;r8-stdoutdef805d3ff7b9b596dd00ec0ff07cd6490dc0f483baa37efc9b8bff511a503d5;r7-stdoutdb0e5e733b57c9d6b5ffe0ad9fb641de1cee78e2fd05fe8cbf890fcae591fcdb;r6-stdout67dfb6781a5840e228b63f3524bf4999ca6d1d9b650f8c461117575a8172611c;r5-stdoutfe0a75877bf539e37e11955f25cebb05821dcbacf7cb50eb120913367f664294;r2-stdout3dad88903f5f619d540587e805b35d63e2ef8c848e53e1ab87786c9e90587ba0|target=boundary-002f3b63423de2e0db8715b599e064043502c3881ba89cb55aaf7394bee850df;recurrence-a900c9452fa173f1c60fdf7c670de747653742348340056f05d7336cbaaa974c;outer5;trial1;solve20;live-prefix14;termination-budget;final-ratio37.960285973054738eta|offline=same-binary64-owned-sparse-workspace;same-gradient-radius-dimensionless-forcing;steihaug;no-preconditioner;first14-exact;cap128|diagnostics=full-iteration-roots;residual;curvature;boundary;orthogonality;conjugacy;ritz|controls=r19r10-parent-bytes;transitive-parents-retained;target-input-roots;gradient-exact;first14-prefix-exact;static-binding;finite;work;all-pair0;rollback|routes=total-budget-offline-nonfinite;total-budget-offline-nonpositive-curvature;total-budget-offline-boundary;total-budget-offline-forcing-converged;total-budget-offline-cap-exhausted|precedence=nonfinite,curvature,boundary,converged,cap|runs=2-clean-release-builds;1-process-each;byte-exact|work=control-parent-substeps1;candidate-substeps0;offline-hvp<=128;workspace1;model0;trial0;acceptance0;precision0|transaction-continuation=none;second-substep=none;macro=none;trajectory=none;timing=none;public-commit=none;physics-mutation=none;live-total-cap-change=none;production-policy-change=none|credit=one-replay-only-total-budget-recurrence-diagnostic
```

Identity SHA-256:
`9cf1761428463b72901c61503466128576e113b9a75f21da3149dda8a7af7a40`.

## Required command

Add `--nonlocal-al-total-budget-offline-replay`. It must:

1. reproduce R10 stdout SHA
   `15719465931a21421e094abde1200d685bee44e4b112dc314d5857f863074953`
   and semantic result
   `30f339247a76484071494b26db54c385a0bea637c4f9e744628a0ff83371b30c`;
2. retain exact R9/R8/R7/R6/R5/R2 roots frozen in the identity projection;
3. bind exact R10 boundary root
   `002f3b63423de2e0db8715b599e064043502c3881ba89cb55aaf7394bee850df`
   and recurrence root
   `a900c9452fa173f1c60fdf7c670de747653742348340056f05d7336cbaaa974c`;
4. retain outer `5`, trial `1`, solve `20`, 14-HVP `BUDGET` prefix and all
   captured current/predicted/gradient/dual/theta/radius/static roots;
5. rebuild one exact binary64-owned sparse workspace and reproduce the
   captured gradient exactly;
6. rerun unchanged Steihaug with dimensionless forcing, no preconditioner and
   an offline cap of 128 recurrence HVPs;
7. require every first-14 iteration projection and prefix root to match R10
   before later diagnostics count;
8. expose the full offline trace, residual, curvature, trust-boundary,
   orthogonality, adjacent A-conjugacy and CG-derived Ritz diagnostics;
9. classify exactly one terminal route through the frozen precedence;
10. consume exactly one offline workspace, at most 128 HVPs and zero model,
    trial, acceptance, precision or all-pair candidate work;
11. retain static binding, finite/lifecycle, rollback and execution scope;
12. run no transaction continuation, candidate/second substep, macro,
    trajectory, timing, public mutation, live cap or production-policy change;
13. run one fresh process from each of two clean Release builds.

## Hard failures

Identity/parent bytes, target/input roots, exact first-14 correspondence,
offline recurrence, terminal uniqueness/precedence, static binding,
finite/lifecycle, work/all-pair, rollback, execution scope or build/process
repeat mismatch is hard FAIL.

## Routes and precedence

1. `TOTAL_BUDGET_OFFLINE_NONFINITE`;
2. `TOTAL_BUDGET_OFFLINE_NONPOSITIVE_CURVATURE`;
3. `TOTAL_BUDGET_OFFLINE_BOUNDARY`;
4. `TOTAL_BUDGET_OFFLINE_FORCING_CONVERGED`;
5. `TOTAL_BUDGET_OFFLINE_CAP_EXHAUSTED`.

## Authority boundary

This contract grants one replay-only recurrence diagnostic. It does not
authorize a model/trial, transaction continuation, live total-budget change,
state commit, another substep, timing, performance claim or runtime/production
authority.
