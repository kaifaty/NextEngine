# NSR3-B4E2D7R19R13 -- post-acceptance boundary contract

Status: `FROZEN / PASS OUTER COMPLETE NOT ADMISSIBLE / SHADOW ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r19r13-post-acceptance-boundary|v1|parent=3405ec25:e57ba96aca2c3114ea2c9c10fa2728d8c91dd7ac7969010f381a245219175cee:3e08a082789053fb5209c4622b391db647328ee9f6582612fd936d8b9ec30f18|legacy=r11-stdoutbd05f7ac3ca76425dbfda1e43882f48adcb4c83addf1fe4ddeaacd37ca8a8efe;r10-stdout15719465931a21421e094abde1200d685bee44e4b112dc314d5857f863074953;r9-stdoutf1cb461d270e1642e9c0220c60fc59c64cb5bb69039f293ed8e9eade7f6ed1f0;r8-stdoutdef805d3ff7b9b596dd00ec0ff07cd6490dc0f483baa37efc9b8bff511a503d5;r7-stdoutdb0e5e733b57c9d6b5ffe0ad9fb641de1cee78e2fd05fe8cbf890fcae591fcdb;r6-stdout67dfb6781a5840e228b63f3524bf4999ca6d1d9b650f8c461117575a8172611c;r5-stdoutfe0a75877bf539e37e11955f25cebb05821dcbacf7cb50eb120913367f664294;r2-stdout3dad88903f5f619d540587e805b35d63e2ef8c848e53e1ab87786c9e90587ba0|target=outer5;trial1;accepted-trial-58dadefd7426e9f80188b694557a02efcaf005dd67abfff5e337b90f214be5a8;long-double-701b0ebe63e44b72a940816c5459819c839ebdf57f5aeaf636ec569273002433;projected-total-hvp523;rho-bits0x3ff0000003c90373;radius-bits0x3f8999999999999a|inner=accepted-state-binary64-owned-workspace;stationarity-limit1e-10;no-new-trust-solve|outer=existing-final-workspace-rebuild-if-inner-complete;constraint-dual-state;previous-outer4-primal-monotonicity;admission-unchanged|controls=r19r12-parent-bytes;transitive-parents-retained;target-roots;outer-start-and-parent-transaction;accepted-state;stationarity;conditional-final-repeat;outer-fields;static-binding;finite;lifecycle;all-pair0;rollback|routes=post-acceptance-boundary-nonfinite;post-acceptance-inner-requires-solve;post-acceptance-outer-nonmonotone;post-acceptance-outer-complete-not-admissible;post-acceptance-outer-complete-admissible|precedence=nonfinite,inner-solve,nonmonotone,not-admissible,admissible|runs=2-clean-release-builds;1-process-each;byte-exact|work=control-parent-substeps1;new-workspaces1-or2;new-hvp0;new-model0;new-trial0;new-divided0;new-precision0|transaction-continuation=none;later-outer=none;second-solve=none;second-substep=none;macro=none;trajectory=none;timing=none;public-commit=none;physics-mutation=none;live-total-cap-change=none;production-policy-change=none|credit=one-post-acceptance-inner-outer-boundary-classification-only
```

Identity SHA-256:
`e7ec6101fa06454082ca70a6bf799850146f884076b285d14978206bead0ad34`.

## Required command

Add `--nonlocal-al-post-acceptance-boundary`. It must:

1. reproduce exact R12 stdout SHA
   `e57ba96aca2c3114ea2c9c10fa2728d8c91dd7ac7969010f381a245219175cee`
   and semantic result
   `3e08a082789053fb5209c4622b391db647328ee9f6582612fd936d8b9ec30f18`;
2. retain exact R11/R10/R9/R8/R7/R6/R5/R2 roots frozen above;
3. bind outer `5`, trial `1`, accepted trial root
   `58dadefd7426e9f80188b694557a02efcaf005dd67abfff5e337b90f214be5a8`,
   precision root, rho/radius bits and projected total HVP `523`;
4. bind the unchanged parent transaction and exact outer-5 start,
   predicted/dual/theta/static inputs;
5. build one accepted-state workspace and compute stationarity with the
   existing `1e-10` threshold;
6. admit no new trust solve when stationarity exceeds the threshold;
7. only when stationarity passes, release and rebuild the existing-path final
   workspace, requiring exact accepted/final evaluation equality;
8. compute the unchanged outer-5 constraint/dual state, finite/admissible
   fields and monotonicity against completed outer 4;
9. classify exactly one route under the frozen precedence;
10. consume one or two new workspaces according to the branch, zero HVP/model/
    trial/divided/precision/all-pair candidate work and release everything;
11. prove rollback of all R12/R11 inputs, parent transaction, candidate trial,
    projected accounting and computed shadow outer state;
12. run no later solve/outer/transaction work, state commit, substep, macro,
    trajectory, timing or policy change;
13. run one fresh process from each of two clean Release builds.

## Hard failures

Identity/parent bytes, transitive retention, target or outer-start roots,
accepted-state evaluation, stationarity computation, required final repeat,
outer-field reconstruction, static binding, finite/lifecycle, work/all-pair,
rollback, execution scope or build/process repeat mismatch is hard FAIL.

## Routes and precedence

1. `POST_ACCEPTANCE_BOUNDARY_NONFINITE`;
2. `POST_ACCEPTANCE_INNER_REQUIRES_SOLVE`;
3. `POST_ACCEPTANCE_OUTER_NONMONOTONE`;
4. `POST_ACCEPTANCE_OUTER_COMPLETE_NOT_ADMISSIBLE`;
5. `POST_ACCEPTANCE_OUTER_COMPLETE_ADMISSIBLE`.

## Authority boundary

This contract grants one rollback-only post-acceptance boundary
classification. It does not authorize a soft/live cap change, checkpoint ABI,
state commit, another trust solve/trial, later outer/transaction work, timing,
runtime policy or production authority.

Closed by the
[D7R19R13 post-acceptance boundary evidence](../../development/nonlocal-nsr3b4e2d7r19r13-post-acceptance-boundary-evidence-2026-08-24.md).
