# NSR3-B4E2D7R19R54 -- model-to-projection consistency contract

Date: `2026-08-25`

Status: `CLOSED PASS / BALL_BOX_PROJECTION_MODEL_REQUIRED / ROLLBACK ONLY`.

Parent: `af354acc`, R53 stdout SHA-256
`490669f7533c6b160a8426b029ee458cf80dc97439872f9b16bbf077a94b531a`,
semantic `4a778c2e546636c28f38d71a72f86e434c089fbc0a441b2babab1e519d86e80c`.

## Frozen identity

```text
nextengine.nonlocal.nsr3b4e2d7r19r54-model-projection-decomposition|v1|parent=af354acc:490669f7533c6b160a8426b029ee458cf80dc97439872f9b16bbf077a94b531a:4a778c2e546636c28f38d71a72f86e434c089fbc0a441b2babab1e519d86e80c:HIGH_PRECISION_RAW_RESIDUAL_CONFIRMED|source=r52-record64-2e0afdb4dbdac10cef5ba591b5764f7cce40ca414458e7f5b9c61cdbc83425f0;witness-ccc1084988fb56a829649f60c292210f24723a4932874923640d5bdbf99e8729;master-38d7a09afa7278d492e6c7981e7dd7359b482309d4d9df0efb01c84324e12e28;cache-f4e9368971d4b6feeb13874d0279dc73fd54ebbb523c75c5c2dd303acfc58cd5;pair-directed-f19cc1913ac2cf0061c9f6240f0f6378fc513108c77975e02cbd0f1f45f632ca|pipeline=recursive-master-predicted;direct-gram-binary64;compensated-gram-binary128;fresh-correction-pair-jvp;anchor-raw-plus-correction;fresh-unprojected-target-pair-jvp;captured-projected-directed;stable-topology|geometry=target-root;projected-root;target-norm;projected-norm;displacement-norm;maximum-displacement;changed-components;box-violations;ball-outside;analytic-clamp-check|rows=positive-count;maximum-and-row;worst-projected-row-value-per-stage;stage-difference-maximum-and-row;roots|selection=largest-exact-stage-gap;recursive-direct-first;direct-correction-second;anchor-target-third;target-projected-fourth;else-model-projection-consistent|controls=r53-parent-bytes;source;workspace;topology;master;lambda;gram;correction;target;geometry;row-comparison;work;rollback;route-precedence|routes=model-parent-rejected;model-source-rejected;model-workspace-rejected;model-topology-rejected;model-master-rejected;model-lambda-rejected;model-gram-rejected;model-correction-rejected;model-target-rejected;model-geometry-rejected;model-row-comparison-rejected;model-work-rejected;hildreth-explicit-residual-replacement-required;gram-correction-assembly-required;anchor-addition-reevaluation-required;ball-box-projection-model-required;model-projection-consistent-candidate|precedence=parent,source,workspace,topology,master,lambda,gram,correction,target,geometry,row-comparison,work,largest-gap-stable-order|work=parent-r53-replays1;moved-workspace1;workspace-release1;fresh-correction-pair-jvp1;fresh-target-pair-jvp1;pairpasses2;binary128-gram-fold1;geometry-fold1;new-row-vjp0;new-gram-jvp0;new-audit-jvp0;new-sweeps0;new-projection0;new-support-audits0;new-hvp0;new-model0;new-nonlinear-trial0;new-outer0|runs=2-clean-release-builds;1-process-each;byte-exact|r40-r41-r42-r43-r44-r45-r46-r47-r48-r49-r50-r51-r52-r53=unchanged;certificate=current-directed-unchanged;normal-witness-apply=none;r43-restoration-commit=none;filter-runtime-commit=none;restoration-exit=none;switching=none;trust-update=none;following-outer=none;tolerance=none;residual-replacement=none;correction-change=none;projection-change=none;gamma-change=none;capacity-change=none;state-mutation=none;timing=none;runtime=none;production=none|credit=one-private-model-projection-classification-only
```

SHA-256 (exact runtime literal, no terminal newline):
`14a4a9436b9e02babcadc9869c80749d77b1bf02fa5f02f1aefe52c091da91ca`.

## Hard gates

1. Reproduce exact R53 bytes, semantic and route before new work; retain exact
   R52 record, witness, 494-row master/cache and pair/directed root.
2. Build/release exactly one unchanged moved workspace and prove stable
   topology. Do not rebuild row bases or Gram columns.
3. Validate the 494 lambdas, recursive prediction and cached Gram mapping.
   Directly fold `u-G lambda` in binary64 and compensated binary128.
4. Execute exactly two fresh pair-once JVPs: assembled correction and
   unprojected target. Reuse captured anchor and projected-witness images.
5. Compare every relevant row at every stage, including the final projected
   worst row. Report positive counts, maxima, maximum stage gaps and roots.
6. Audit target/projected geometry analytically: roots, norms, displacement,
   changed components, box violations, ball status and clamp equivalence. Do
   not call the projection again.
7. Select the largest exact stage gap with stable tie order. No tolerance,
   normalization fit or post-observation route change.
8. Exact rollback. No residual replacement, correction/projection change,
   witness/state/filter commit, restoration exit, trust update, following
   outer, runtime arithmetic policy or timing.

Require two clean Release builds and byte-exact outputs. A PASS is one private
model-to-projection classification only, not compatibility, runtime permission
or production evidence.

Rationale:
[R54 research](../../development/nonlocal-nsr3b4e2d7r19r54-model-projection-research-2026-08-25.md).

Evidence:
[R54 closure](../../development/nonlocal-nsr3b4e2d7r19r54-model-projection-evidence-2026-08-25.md).
