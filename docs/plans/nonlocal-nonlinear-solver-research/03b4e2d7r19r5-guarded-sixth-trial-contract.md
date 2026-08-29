# NSR3-B4E2D7R19R5 -- guarded sixth-trial shadow contract

Status: `EXECUTED / PASS / SIXTH_TRIAL_RESIDUAL_MODEL_ACCEPTANCE_CANDIDATE / SHADOW ONLY / NO STATE`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r19r5-guarded-sixth-trial-shadow|v1|parent=70cf041211dc5042d34b0bd6c58d2d53f3fef5c4:798e8005aaa503eedb7a90ed9fd3590230cec7bbadf85536b39ac135ac7cb3a0:99b77c1e0d4750911af11a5cc9933d512dc22ee3041a873eabccc2a8389f2cd0|legacy=r3-stdouta1038937496f31ed64008eb8e366763e2da9875e3935194955d68604a7b23771;r2-stdout3dad88903f5f619d540587e805b35d63e2ef8c848e53e1ab87786c9e90587ba0|target=current-54bafbf48d0798438fd9baad9fb91e67c7b5cf6b12e37c4bb1d49694384ddf8a;predicted-36112dde1e0b274c5b9216f4818b82977a0c80dc257478c111b0f0a9390d2d7e;step-74a9b58726d5d0279699498c41a70fb0e199f45e531d11befd2bc2dfe692d2bd;radius0x3f8999999999999a|grace=base-recurrence-hvp32;extra1;absolute33;eligible=first32-finite-positive-interior;ratio-after32>eta;ratio-after32<=1.25eta;last8-next-ratios-strictly-decrease|model=residual-derived-r-final-minus-g;direct-H-step-oracle-control;direct-predicted-bits0x3bc27dd9b2871ea7;candidate-predicted-exact|trial=current+step;static-sparse;pairwise-precancelled-divided;binary64-owned-long-double-audit|accept=trial-finite;predicted>0;divided>0;ratio>=0.1;precision-resolved-positive|radius=existing-policy-unchanged;shadow-only|controls=r19r4-parent-bytes;r19r3+r19r2-retained;target-roots;grace-provenance;model-correspondence;divided-repeat;precision;static-binding;finite;work;rollback|routes=sixth-trial-precision-contradiction;sixth-trial-grace-insufficient;sixth-trial-residual-model-acceptance-candidate;sixth-trial-residual-model-rejected|precedence=precision,grace,accepted,rejected|runs=2-clean-release-builds;1-process-each;byte-exact|work=parent-recurrence33;new-direct-oracle-hvp1;candidate-model-hvp0;workspaces2;long-audits1;trial-formation1;state-commit0|candidate-nominal-substeps=0;macro=none;trajectory=none;timing=none;public-commit=none;physics-mutation=none;production-cap-change=none|credit=one-shadow-sixth-trial-only
```

Identity SHA-256:
`2e84206805c247933c2dae475aa0162bcbe0fe265442e79e14dcf5ef9fcb830e`.

## Required command

Add `--nonlocal-al-guarded-sixth-trial-shadow`. It must:

1. reproduce D7R19R4 stdout SHA
   `798e8005aaa503eedb7a90ed9fd3590230cec7bbadf85536b39ac135ac7cb3a0`
   and semantic result
   `99b77c1e0d4750911af11a5cc9933d512dc22ee3041a873eabccc2a8389f2cd0`;
2. retain the exact R3/R2 stdout roots frozen in the identity projection;
3. bind current/predicted/step/radius roots and the exact 32-HVP prefix;
4. require all first-32 records finite, positive-curvature and interior, the
   after-32 residual ratio in `(eta, 1.25*eta]`, and eight strictly decreasing
   trailing next-residual ratios;
5. grant exactly one recurrence HVP, require convergence at HVP 33 and execute
   no candidate model HVP;
6. derive the model image from `r_final-g`, retain exactly one direct oracle
   HVP and require candidate/direct predicted bits
   `0x3bc27dd9b2871ea7`;
7. form exactly one shadow trial, build current/trial static sparse workspaces
   and compute the pairwise pre-cancelled divided reduction twice;
8. apply the existing acceptance and radius formulas without mutating current
   state or continuing the solver;
9. on would-accept, execute exactly one binary64-owned long-double audit and
   require finite resolved-positive sign unless selecting the frozen precision
   contradiction route;
10. retain exact static binding, work/lifecycle, all-pair-zero, parent/target
    rollback and route precedence;
11. run one fresh process from each of two clean Release builds.

## Frozen work

```text
parent recurrence HVPs       33 (control, not candidate work)
new direct oracle HVPs       1
candidate model HVPs         0
current/trial workspaces     2 / 2 releases / <=2 live
long-double audits           1 only when would-accept
shadow trials formed         1
state commits                0
all-pair calls               0
```

Divided-repeat evaluation and residual-image vector work add no HVP credit.

## Routes and precedence

1. `SIXTH_TRIAL_PRECISION_CONTRADICTION`;
2. `SIXTH_TRIAL_GRACE_INSUFFICIENT`;
3. `SIXTH_TRIAL_RESIDUAL_MODEL_ACCEPTANCE_CANDIDATE`;
4. `SIXTH_TRIAL_RESIDUAL_MODEL_REJECTED`.

Identity/parent/target, guard provenance, model correspondence, divided repeat,
finite, precision execution, work/lifecycle, static binding, rollback,
build/process repeat or route-precedence mismatch is hard FAIL.

## Authority boundary

This contract grants one shadow sixth-trial classification. It does not grant
a production cap change, transaction mutation/continuation, state commit,
another candidate nominal substep, macro, trajectory, timing, performance
claim or runtime/production authority.

## Result

The shadow passes reproducibly and selects
`SIXTH_TRIAL_RESIDUAL_MODEL_ACCEPTANCE_CANDIDATE`; see the
[dated evidence](../../development/nonlocal-nsr3b4e2d7r19r5-guarded-sixth-trial-evidence-2026-08-23.md).
The divided ratio is `0.9999993088`, long-double sign resolves positive and the
radius remains exact. Research/freeze a separate full private first-substep
transaction candidate next; this shadow itself commits and continues nothing.
