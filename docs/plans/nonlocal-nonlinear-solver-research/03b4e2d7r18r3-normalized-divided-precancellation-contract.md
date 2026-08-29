# NSR3-B4E2D7R18R3 -- normalized divided precancellation replay contract

Status: `CLOSED / PASS / NORMALIZED_DIVIDED_PRECANCELLATION_CANDIDATE / D7R19_BLOCKED`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r18r3-normalized-divided-precancellation-replay|v1|parent=daf97f3dbc7d37d258ddad0e95b1d3ddc22f582e:3659eac888c22eae5bcf7ae8c5f8a426bcbc12bd24bd3320c23be0c176815d77:69223a86a9a889a5c85fd11b824a4bd178efe505907d6f3877b5a42b8fd0f392|legacy=d7r18r2-complete-bytes;d7r13-stdout514ea1925a85d398a948a2dcbc319689116114a02335a599e51d6703202c18de|fixture=corner-box-2x2x2;active-compressed0.99;initial-equals-prediction;u0|profiles=reference-dt0x3f71111111111111-kappa0x4093290000000000;aligned-dt0x3f0c01c01c01c01c-kappa0x415c75a640000000;theta0x3fc5cccccccccccd;derived-prework|target=active-root-c76bea9f1bff57c16e27a08c4fc51dad7029af1730c11ac80e568ebe112601a5;inner-root-b7f44b27a51e6249b0d88b29cd08af42a7490e69ea5fed4e081cf64f48b44515;outer1;trial2;stationarity0x3dfde62308c06538;radius0x3f8999999999999a;step0x3da69c573653b780;predicted0x3b77e6c5c7652902;raw0xbc16aa0000000000;old-divided0xbc16a9d909f08000|formula=static-sparse-sorted-current-trial-union;d7r10-binary64-kernel-delta;compensated-current-density+density-delta;current-c=density/rest-1;delta-c=density-delta/rest;current-a=u+current-c;delta-a=delta-c;trial-a=current-a+delta-a;piecewise-active-square-delta;phr=-0.5theta*active-square-delta;inertia=-0.5*squared-norm-delta;compensated-center-sums;rounded-active-subtraction-forbidden|oracle=direct-normalized-long-double-naive+compensated-1024ulp;direct-normalized-binary128-naive+compensated-4096ulp;candidate-relative-error<=0.05;predicted-relative-error<=0.05;candidate-ratio>=0.1;pair-membership-exact;runtime-float128-none|correspondence=candidate-repeat-byte-exact;reference-aligned-candidate-root-byte-exact|controls=parent-bytes;target-anchors;inherited-old-divided-exact;static-binding;invalid-prework;work-ledger;workspace-live<=2;all-pair0;forced-rollback|routes=normalized-precancellation-sign-contradiction;normalized-precancellation-bound-required;normalized-model-or-derivative-reclosure-required;normalized-divided-precancellation-candidate|precedence=contradiction,bound,model,candidate|runs=2-clean-release-builds;1-process-each;byte-exact;timing=none|candidate-acceptances=0;outer-updates-by-candidate=0;trajectory=none;nominal-substeps=0;macro=none;public-commit=none;physics-mutation=none;production-scale=none|credit=one-replay-only-normalized-divided-precancellation-discriminator
```

Identity SHA-256:
`d91745fd152876047394d3d668ed9174358b5e6c600fef7b9ba3ddfb8f0a6d6f`.

## Required command

Add `--nonlocal-al-normalized-divided-precancellation-replay`. It must:

1. reproduce complete D7R18R2 stdout bytes and retain its classified route;
2. derive both exact finite-positive `{dt,kappa,theta}` profiles before
   topology, pair, precision or HVP work;
3. replay the unchanged reference active transaction and recover outer `1`,
   trial `2` only if the active/inner roots and all frozen binary64 target
   anchors are exact;
4. obtain the target outer dual from the passed outer-0 update without
   reconstructing it from pressure diagnostics;
5. evaluate the inherited normalized divided formula and reproduce its frozen
   negative result;
6. evaluate the frozen pairwise-precancelled normalized formula twice over
   the sorted static current/trial pair union, with compensated current-density,
   density-delta and center accumulators;
7. evaluate independent direct normalized naive/compensated long-double and
   binary128 oracles over the same exact pair union and resolution rules;
8. require finite exact pair membership, candidate relative error at most
   `0.05`, predicted relative error at most `0.05` and observation-only trust
   ratio at least `0.1` for the candidate route;
9. reevaluate the candidate under the independently derived aligned profile
   and require byte-exact reference/aligned candidate roots;
10. reject invalid `theta`, `u`, workspace layout or profile mismatch before
    candidate pair visits and prove exact work/lifecycle accounting with no
    all-pair candidate calls and at most two live workspaces;
11. leave the R2 replay result and all caller-owned position/dual state exact,
    perform zero candidate acceptances and publish nothing;
12. execute one process from each of two clean Release builds and emit one
    route under the frozen precedence without timing, a nominal substep,
    macro or trajectory.

## Routes

1. `NORMALIZED_PRECANCELLATION_SIGN_CONTRADICTION`.
2. `NORMALIZED_PRECANCELLATION_BOUND_REQUIRED`.
3. `NORMALIZED_MODEL_OR_DERIVATIVE_RECLOSURE_REQUIRED`.
4. `NORMALIZED_DIVIDED_PRECANCELLATION_CANDIDATE`.

Identity, R2 parent bytes, profile derivation, replay anchors, static binding,
inherited formula, pair membership, invalid prework, work/lifecycle, rollback,
process repeat or route-precedence mismatch is hard FAIL.

Only `NORMALIZED_DIVIDED_PRECANCELLATION_CANDIDATE` may authorize
research/freeze of a separately bounded complete normalized private
transaction with the candidate formula. It grants no execution authority for
that transaction, D7R19, a nominal substep, macro, trajectory, timing, public
state, runtime binary128, GPU/runtime or production use.

The command closes with the candidate route. See the
[dated evidence](../../development/nonlocal-nsr3b4e2d7r18r3-normalized-divided-precancellation-evidence-2026-08-22.md).
