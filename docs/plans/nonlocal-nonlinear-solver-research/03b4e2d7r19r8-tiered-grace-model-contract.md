# NSR3-B4E2D7R19R8 -- tiered-grace model-image contract

Status: `FROZEN / IMPLEMENTATION_NEXT / REPLAY ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r19r8-tiered-grace-model-image|v1|parent=8b806993c6c8588cc6b4334064b42b06a4628700:db0e5e733b57c9d6b5ffe0ad9fb641de1cee78e2fd05fe8cbf890fcae591fcdb:c8f350b941e57910b1dc5dc93e6a357319d22de6f2fb16fb3a338592f6596c79|legacy=r6-stdout67dfb6781a5840e228b63f3524bf4999ca6d1d9b650f8c461117575a8172611c;r5-stdoutfe0a75877bf539e37e11955f25cebb05821dcbacf7cb50eb120913367f664294|targets=first-prefix-f478832923673956bc98d8067fff9bdeb5c3dab109c0a8e239ad12c6844d60dd-step74a9b58726d5d0279699498c41a70fb0e199f45e531d11befd2bc2dfe692d2bd-hvp33-converged;second-capture97c340ed27b33318fb4d6684a7d385380ddc15e3a8b0358d0c86feb189f48ab3-prefixadf2edcc243060a540d515f6d8687a2b2b0f0e9e2131fbc402f06d9b50b85d08-hvp34-converged|tier1=prefix-safe;ratio32-in-eta-to-1.25eta;last8-decrease;hvp33-must-converge;unchanged|tier2-entry=prefix-safe;ratio32-in-1.25eta-to-2eta;last8-decrease;allow-hvp33|tier2-continuation=hvp33-safe-interior;ratio33-in-eta-to-1.5eta;q33<=0.75;updated-last8-decrease;allow-hvp34;hvp34-must-converge|model=residual-r-final-minus-g;direct-H-step-oracle;image-l2-relative<=1e-10;component-scaled<=1e-10;quadratic-relative<=1e-10;predicted-relative<=1e-10;positive-signs|controls=r19r7-parent-bytes;r19r6+r19r5-retained;two-target-roots;tier1-unchanged;tier2-clauses;hvp34-convergence;model-correspondence;static-binding;finite;work;all-pair0;rollback|routes=tiered-grace-rejected;tiered-grace-residual-model-candidate;direct-model-hvp-required|precedence=guard,residual,direct|runs=2-clean-release-builds;1-process-each;byte-exact|work=parent-replays;new-direct-oracle-hvp1;workspace1;candidate-model-hvp0;precision0;trial0;acceptance0|candidate-substeps=0;transaction=none;second-substep=none;macro=none;trajectory=none;timing=none;public-commit=none;physics-mutation=none;guard-change=none;production-cap-change=none|credit=one-replay-only-tiered-grace-model-discriminator
```

Identity SHA-256:
`15b8442aa24bf86d5358a018c4e4f13258dfb6f3ccf65aa9363415617b37221d`.

## Required command

Add `--nonlocal-al-tiered-grace-model-image-discriminator`. It must:

1. reproduce D7R19R7 stdout SHA
   `db0e5e733b57c9d6b5ffe0ad9fb641de1cee78e2fd05fe8cbf890fcae591fcdb`
   and semantic result
   `c8f350b941e57910b1dc5dc93e6a357319d22de6f2fb16fb3a338592f6596c79`;
2. retain exact R6/R5 roots frozen in the identity projection;
3. bind the original R3 prefix/step and HVP-33 convergence without changing
   its existing tier-1 guard decision;
4. bind the R7 later capture/prefix and exact HVP-34 convergence;
5. evaluate tier-2 entry only when the first 32 records are safe/interior,
   the last eight ratios strictly decrease and
   `ratio32 in (1.25*eta, 2*eta]`;
6. after admitted HVP 33, require safe/interior work,
   `ratio33 in (eta, 1.5*eta]`, `ratio33/ratio32 <= 0.75` and an updated
   last-eight strict decrease before admitting HVP 34;
7. require HVP 34 to converge and expose every tier clause separately;
8. derive the later model image as `r_final-g`, then compare it with one
   direct sparse `H(step)` oracle;
9. require `1e-10` image L2, scaled-component, quadratic and predicted-
   reduction relative bounds plus finite positive direct/candidate models;
10. consume exactly one new workspace and one direct oracle HVP, with zero
    candidate model HVPs, precision audits, trials or acceptances;
11. retain exact target/static binding, finite/lifecycle, zero all-pair work
    and rollback;
12. run no candidate transaction/substep, second substep, macro, trajectory,
    timing lane or public state mutation;
13. run one fresh process from each of two clean Release builds.

## Hard failures

Identity/parent bytes, either target, tier-1 preservation, tier-2 predicate or
HVP-34 convergence, model correspondence/sign, static binding, finite/work/
lifecycle, all-pair count, rollback, execution scope, build/process repeat or
route precedence mismatch is hard FAIL.

## Routes and precedence

1. `TIERED_GRACE_REJECTED`;
2. `TIERED_GRACE_RESIDUAL_MODEL_CANDIDATE`;
3. `DIRECT_MODEL_HVP_REQUIRED`.

## Authority boundary

This contract grants one replay-only tiered-grace/model-image discriminator.
It does not authorize a live guard/cap change, trial, precision audit,
transaction, substep, macro, trajectory, timing, performance claim, public
state or runtime/production authority.
