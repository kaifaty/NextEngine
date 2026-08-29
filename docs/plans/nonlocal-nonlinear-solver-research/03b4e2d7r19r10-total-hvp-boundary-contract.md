# NSR3-B4E2D7R19R10 -- total-HVP boundary diagnostic contract

Status: `FROZEN / IMPLEMENTATION_NEXT / PASSIVE DIAGNOSTIC ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r19r10-total-hvp-boundary-diagnostic|v1|parent=978f5839:f1cb461d270e1642e9c0220c60fc59c64cb5bb69039f293ed8e9eade7f6ed1f0:40e152b8f45f2d2b7b654d0bc629aa76ed160f6c9a9467db1097a5fef877934d|legacy=r8-stdoutdef805d3ff7b9b596dd00ec0ff07cd6490dc0f483baa37efc9b8bff511a503d5;r7-stdoutdb0e5e733b57c9d6b5ffe0ad9fb641de1cee78e2fd05fe8cbf890fcae591fcdb;r6-stdout67dfb6781a5840e228b63f3524bf4999ca6d1d9b650f8c461117575a8172611c;r5-stdoutfe0a75877bf539e37e11955f25cebb05821dcbacf7cb50eb120913367f664294;r2-stdout3dad88903f5f619d540587e805b35d63e2ef8c848e53e1ab87786c9e90587ba0|target=transaction-c6a03d43a04ba00cef80b618bb983ab23f59e27d79ad073f327f119b145660cf;outer-updates6;accepted20;rejected0;recorded20;recurrence494;direct-model18;residual-model2;total-hvp512;failure-inner-structural-budget-total-hvp|capture=passive-on-exact-total-hvp-failure;outer-trial-solve-identity;predicted-current-gradient-u-theta-radius;recurrence-prefix;budget-work-precision-static-adjacency;no-new-work|progress=per-outer-completion-failure-trials-accepted-rejected-inner-hvp-stationarity-available-state;per-trial-work-tier-stationarity-radius-model-ratio-decision-roots;cumulative-completed-plus-interrupted-exact512|classifier=nonfinite;nonpositive-curvature;trust-boundary;forcing-already-reached;safe-near-forcing;safe-progressing;safe-inconclusive|windows=min8;late-min-and-median-less-than-early;near-final-ratio-in-eta-to-2eta-and-late-strict-decrease|controls=r19r9-parent-bytes;r19r8+r19r7+r19r6+r19r5+r19r2-retained;transaction-root;capture-no-work;budget-accounting;progress-root;recurrence-root;static-binding;lifecycle;all-pair0;rollback|routes=total-hvp-boundary-nonfinite;total-hvp-boundary-nonpositive-curvature;total-hvp-boundary-trust-boundary;total-hvp-boundary-forcing-already-reached;total-hvp-boundary-safe-near-forcing;total-hvp-boundary-safe-progressing;total-hvp-boundary-safe-inconclusive|precedence=nonfinite,curvature,boundary,forcing,near,progressing,inconclusive|runs=2-clean-release-builds;1-process-each;byte-exact|work=parent-control-substeps1;diagnostic-copy-only;continued-hvp0;new-model0;new-trial0;new-acceptance0;new-precision0|second-substep=none;macro=none;trajectory=none;timing=none;public-commit=none;physics-mutation=none;total-cap-change=none;production-policy-change=none|credit=one-passive-exact-total-hvp-boundary-diagnostic-only
```

Identity SHA-256:
`52f069f26f5c47f0ea204b227765cea108970da1a87a553173f2655ad42b9fa7`.

## Required command

Add `--nonlocal-al-total-hvp-boundary-diagnostic`. It must:

1. reproduce R9 stdout SHA
   `f1cb461d270e1642e9c0220c60fc59c64cb5bb69039f293ed8e9eade7f6ed1f0`
   and semantic result
   `40e152b8f45f2d2b7b654d0bc629aa76ed160f6c9a9467db1097a5fef877934d`;
2. retain exact R8/R7/R6/R5/R2 roots frozen in the identity projection;
3. retain R9 transaction root
   `c6a03d43a04ba00cef80b618bb983ab23f59e27d79ad073f327f119b145660cf`
   and exact `6/20/0/20/494/18/2/512` boundary facts;
4. derive the diagnostic from the same R9 execution through an optional
   passive capture sink; the ordinary R9 command must remain byte-exact;
5. capture only the first exact `STRUCTURAL_BUDGET_TOTAL_HVP` return and
   require transaction failure `INNER:STRUCTURAL_BUDGET_TOTAL_HVP`;
6. record exact outer/trial/solve identity, inputs, radius, recurrence prefix
   and pre-return structural/lifecycle counters without new physics work;
7. expose every completed outer and trial using the frozen progress fields;
8. account completed recurrence/model work plus interrupted recurrence work
   exactly to the global total of `512`;
9. classify the interrupted recurrence through the frozen route precedence,
   reporting early/late window minimum/median, final ratio, `eta`, recent
   strict decrease and every safety clause;
10. require exact progress and recurrence roots, static binding, finite
    capture, workspace lifecycle, zero all-pair work and rollback;
11. consume zero continuation HVP, new model, trial, acceptance or precision
    audit after the boundary;
12. run no second substep, macro, trajectory, timing lane, public mutation,
    total-cap change or production-policy change;
13. run one fresh process from each of two clean Release builds.

## Hard failures

Identity/parent bytes, target transaction/boundary facts, passive-capture
identity or no-work proof, progress/work accounting, recurrence projection,
classifier precedence, static binding, finite/lifecycle, all-pair count,
rollback, execution scope or build/process repeat mismatch is hard FAIL.

## Routes and precedence

1. `TOTAL_HVP_BOUNDARY_NONFINITE`;
2. `TOTAL_HVP_BOUNDARY_NONPOSITIVE_CURVATURE`;
3. `TOTAL_HVP_BOUNDARY_TRUST_BOUNDARY`;
4. `TOTAL_HVP_BOUNDARY_FORCING_ALREADY_REACHED`;
5. `TOTAL_HVP_BOUNDARY_SAFE_NEAR_FORCING`;
6. `TOTAL_HVP_BOUNDARY_SAFE_PROGRESSING`;
7. `TOTAL_HVP_BOUNDARY_SAFE_INCONCLUSIVE`.

The route classifies the prefix only. It does not estimate or authorize a
larger global budget.

## Authority boundary

This contract grants one passive diagnostic of the exact R9 total-HVP
boundary. It does not authorize recurrence continuation, another trial,
state commit, second substep, macro, trajectory, timing, performance claim,
cap/policy change or runtime/production authority.
