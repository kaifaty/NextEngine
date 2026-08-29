# NSR3-B4E2D7R19R9 -- tiered-grace private transaction contract

Status: `FROZEN / IMPLEMENTATION_NEXT / PRIVATE FIRST SUBSTEP ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r19r9-tiered-grace-private-transaction|v1|parent=0e0041b2:def805d3ff7b9b596dd00ec0ff07cd6490dc0f483baa37efc9b8bff511a503d5:65b9a51e231f51dd9d693ba55b63b0bfd3c4b16ccd5e0c9c298f7f1df7578f63|legacy=r7-stdoutdb0e5e733b57c9d6b5ffe0ad9fb641de1cee78e2fd05fe8cbf890fcae591fcdb;r6-stdout67dfb6781a5840e228b63f3524bf4999ca6d1d9b650f8c461117575a8172611c;r5-stdoutfe0a75877bf539e37e11955f25cebb05821dcbacf7cb50eb120913367f664294;r2-stdout3dad88903f5f619d540587e805b35d63e2ef8c848e53e1ab87786c9e90587ba0|policy=ordinary-direct-model;tier1-residual-after-hvp33;tier2-residual-after-hvp34;explicit-research-only|budget=base-recurrence32;tier1-extra1;tier2-extra2;candidate-absolute-step34;total512;outer16;trials-per-update16;workspaces288;precision64|tier1=prefix-safe;ratio32-in-eta-to-1.25eta;last8-decrease;hvp33-must-converge|tier2-entry=prefix-safe;ratio32-in-1.25eta-to-2eta;last8-decrease;allow-hvp33|tier2-continuation=hvp33-safe-interior;ratio33-in-eta-to-1.5eta;q33<=0.75;updated-last8-decrease;allow-hvp34;hvp34-must-converge|model=ordinary-direct-H-step;tiered-r-final-minus-g;zero-model-hvp-after-grace|precision=binary64-owned-membership;long-double-every-accepted;binary128-candidate-effect-only|anchors=trials0-4-r2-binary64-exact;trial5-current54bafbf48d0798438fd9baad9fb91e67c7b5cf6b12e37c4bb1d49694384ddf8a-step74a9b58726d5d0279699498c41a70fb0e199f45e531d11befd2bc2dfe692d2bd-trial932ce178a6238025c8de6ea907d9966638fa0f5abef7780a15fb9c8171f67ea2-predicted0x3bc27dd9b2871ea7-divided0x3bc27dd8dc16d400-ratio0x3feffffe8ce92225-precisiona58caa2c1cb83c23dbcc15b8d2daf243de7749e6e692a92369141bbfd741f8db-radius0x3f8999999999999a|work-ownership=ordinary-recurrence+direct-model;tier1-33+residual-model;tier2-34+residual-model;all-pair0|controls=r19r8-parent-bytes;r19r7+r19r6+r19r5+r19r2-retained;legacy-policy-bytes;first-six-anchors;tier-provenance;model-ownership;divided-repeat;precision;static-binding;structural-work;finite;mass;boundary;impulse;rollback|routes=normalized-nominal-precision-contradiction;normalized-nominal-structural-watchdog-exhausted;normalized-nominal-solver-not-confirmed;normalized-nominal-boundary-penetration;normalized-nominal-impulse-ledger-mismatch;normalized-nominal-substep-shadow-confirmed|precedence=precision,watchdog,solver,boundary,ledger,confirmed|runs=2-clean-release-builds;1-process-each;byte-exact|candidate-nominal-substeps=1;second-substep=none;macro=none;trajectory=none;timing=none;public-commit=none;physics-mutation=none;production-policy-change=none|credit=one-private-tiered-grace-first-substep-transaction-only
```

Identity SHA-256:
`2d7bba5ca68fb92aa154d7de10e950769ae8d722e1a91e5ae7a58ac6f1421a65`.

## Required command

Add `--nonlocal-al-tiered-grace-private-transaction`. It must:

1. reproduce R8 stdout SHA
   `def805d3ff7b9b596dd00ec0ff07cd6490dc0f483baa37efc9b8bff511a503d5`
   and semantic result
   `65b9a51e231f51dd9d693ba55b63b0bfd3c4b16ccd5e0c9c298f7f1df7578f63`;
2. retain exact R7/R6/R5/R2 roots frozen in the identity projection;
3. leave the direct and one-shot guarded public policies byte-exact;
4. run one separate private first-substep transaction with an explicit
   tiered-grace completion policy;
5. preserve base recurrence HVP 32 and admit HVP 33/34 only through the exact
   R8 tier-1/tier-2 predicates;
6. require tier 1 to converge on HVP 33 and tier 2 to converge on HVP 34;
7. use `r_final-g` and zero model HVP only after a converged grace path;
8. retain one direct model HVP for every ordinary solve converging by HVP 32;
9. reproduce exact R2 trials `0..4` and the exact R5 trial `5` anchor;
10. expose exact per-tier attempts, admissions, continuations, HVPs,
    convergences, denials and model ownership;
11. enforce candidate limits `16/16/34/512/288/64`, with 34 available only
    to this explicit research policy;
12. retain divided-repeat, binary64 membership, accepted-trial long-double
    audit and candidate-effect binary128 policy;
13. retain static binding, finite values, mass, boundary, impulse, lifecycle,
    zero all-pair work, rollback and route precedence;
14. run no second substep, macro, trajectory, timing lane, public mutation or
    production-policy change;
15. run one fresh process from each of two clean Release builds.

## Hard failures

Identity/parent bytes, legacy-policy bytes, first-six anchors, tier predicate
or provenance, model ownership, divided repeat, precision, structural work,
lifecycle, static binding, finite/mass/boundary/impulse controls, rollback,
execution scope, build/process repeat or route precedence mismatch is hard
FAIL.

The transaction may stop at a new exact structural or physics boundary. That
route is evidence, not a hard failure, when every required control above
passes.

## Routes and precedence

1. `NORMALIZED_NOMINAL_PRECISION_CONTRADICTION`;
2. `NORMALIZED_NOMINAL_STRUCTURAL_WATCHDOG_EXHAUSTED`;
3. `NORMALIZED_NOMINAL_SOLVER_NOT_CONFIRMED`;
4. `NORMALIZED_NOMINAL_BOUNDARY_PENETRATION`;
5. `NORMALIZED_NOMINAL_IMPULSE_LEDGER_MISMATCH`;
6. `NORMALIZED_NOMINAL_SUBSTEP_SHADOW_CONFIRMED`.

## Authority boundary

This contract grants one private first-substep transaction under an explicit
tiered research policy. It does not authorize a public state commit, second
substep, runtime/production policy, macro, trajectory, timing or performance
claim.
