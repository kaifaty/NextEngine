# NSR3-B4E2D7R19R6 -- guarded-residual full transaction contract

Status: `FROZEN / IMPLEMENTATION_NEXT / ONE PRIVATE FIRST SUBSTEP`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r19r6-guarded-residual-private-transaction|v1|parent=95e2fa24f528395c1faf619a2a2d3b442367cad3:fe0a75877bf539e37e11955f25cebb05821dcbacf7cb50eb120913367f664294:ae751940bf195fdf70aab9f1a5df6ce986decfc9c4f3c5fbeb26bf95d43e8880|legacy=r4-stdout798e8005aaa503eedb7a90ed9fd3590230cec7bbadf85536b39ac135ac7cb3a0;r3-stdouta1038937496f31ed64008eb8e366763e2da9875e3935194955d68604a7b23771;r2-stdout3dad88903f5f619d540587e805b35d63e2ef8c848e53e1ab87786c9e90587ba0|policy=ordinary-direct-model;guarded-residual-model-only-after-grace|budget=base-recurrence32;guard-extra1;absolute-step33;total512;outer16;trials-per-update16;workspaces288;precision64|guard=first32-finite-positive-interior;ratio-after32-in-eta-to-1.25eta;last8-decrease;hvp33-must-converge|precision=binary64-owned-membership;long-double-every-accepted;binary128-candidate-effect-only|anchors=trials0-4-r2-binary64-exact;trial5-current54bafbf48d0798438fd9baad9fb91e67c7b5cf6b12e37c4bb1d49694384ddf8a-step74a9b58726d5d0279699498c41a70fb0e199f45e531d11befd2bc2dfe692d2bd-trial932ce178a6238025c8de6ea907d9966638fa0f5abef7780a15fb9c8171f67ea2-predicted0x3bc27dd9b2871ea7-divided0x3bc27dd8dc16d400-ratio0x3feffffe8ce92225-precisiona58caa2c1cb83c23dbcc15b8d2daf243de7749e6e692a92369141bbfd741f8db-radius0x3f8999999999999a|work-ownership=ordinary-recurrence+direct-model;guarded-recurrence+residual-model;all-pair0|controls=r19r5-parent-bytes;r19r4+r19r3+r19r2-retained;legacy-policy-bytes;first-six-anchors;guard-provenance;model-ownership;divided-repeat;precision;static-binding;structural-work;finite;mass;boundary;impulse;rollback|routes=normalized-nominal-precision-contradiction;normalized-nominal-structural-watchdog-exhausted;normalized-nominal-solver-not-confirmed;normalized-nominal-boundary-penetration;normalized-nominal-impulse-ledger-mismatch;normalized-nominal-substep-shadow-confirmed|precedence=precision,watchdog,solver,boundary,ledger,confirmed|runs=2-clean-release-builds;1-process-each;byte-exact|candidate-nominal-substeps=1;second-substep=none;macro=none;trajectory=none;timing=none;public-commit=none;physics-mutation=none;production-cap-change=none|credit=one-private-guarded-residual-first-substep-transaction-only
```

Identity SHA-256:
`e71a9fced6ffa724380cfd3a4dc7e00354738fe56168aa1a8f1242dc9da9a895`.

## Required command

Add `--nonlocal-al-guarded-residual-private-transaction`. It must:

1. reproduce D7R19R5 stdout SHA
   `fe0a75877bf539e37e11955f25cebb05821dcbacf7cb50eb120913367f664294`
   and semantic result
   `ae751940bf195fdf70aab9f1a5df6ce986decfc9c4f3c5fbeb26bf95d43e8880`;
2. retain exact R4/R3/R2 stdout roots frozen in the identity projection;
3. keep the historical direct-model trust-completion policy as the default
   and reproduce its parent bytes exactly;
4. run one separate candidate transaction with explicit guarded-residual
   completion and binary64-owned precision membership;
5. enforce base recurrence cap `32`, one guarded HVP, absolute per-step cap
   `33`, total HVP `512`, outer `16`, trials/update `16`, workspace `288` and
   precision-audit `64` caps;
6. permit HVP 33 only under the frozen R5 predicate and require convergence on
   that HVP; use residual-derived model image only when grace was actually
   used;
7. retain direct model HVPs for every ordinary solve and record recurrence/
   model/grace ownership separately;
8. reproduce R2 trials `0..4` exactly and the complete frozen R5 trial-5
   anchor before admitting any later candidate observation;
9. retain pairwise pre-cancelled divided repeat, binary64-owned precision,
   static binding, finite/mass, lifecycle/work, boundary/impulse, rollback and
   route-precedence controls;
10. execute one candidate first substep, no second substep/macro/trajectory/
    timing lane and no public state mutation;
11. run one fresh process from each of two clean Release builds.

## Hard failures

Identity/parent/legacy bytes, policy provenance, either first-six anchor,
unauthorized residual model use, guard or budget ownership, divided repeat,
precision, static binding, finite/mass, work/lifecycle, all-pair count,
rollback, build/process repeat or route precedence mismatch is hard FAIL.

## Routes and precedence

1. `NORMALIZED_NOMINAL_PRECISION_CONTRADICTION`;
2. `NORMALIZED_NOMINAL_STRUCTURAL_WATCHDOG_EXHAUSTED`;
3. `NORMALIZED_NOMINAL_SOLVER_NOT_CONFIRMED`;
4. `NORMALIZED_NOMINAL_BOUNDARY_PENETRATION`;
5. `NORMALIZED_NOMINAL_IMPULSE_LEDGER_MISMATCH`;
6. `NORMALIZED_NOMINAL_SUBSTEP_SHADOW_CONFIRMED`.

The selected route describes only the private candidate transaction.

## Authority boundary

This contract grants one guarded-residual private first-substep transaction.
It does not authorize public state commit, a production cap/policy, another
substep, macro, trajectory, timing, performance claim or runtime/production
authority.
