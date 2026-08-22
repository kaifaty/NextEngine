# NSR3-B4E2D7R18R2 -- full normalized private transaction contract

Status: `FROZEN / IMPLEMENTATION_NEXT / D7R19_BLOCKED`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r18r2-full-normalized-private-transaction|v1|parent=e3e6855740fd666e46d3fbabb0d005324cddecf8:eeb29e67830518104c347105bcf600998eb00a3394e7c4b8169f701863bd570e:b63aa9851e40658c80737ddc2138419bad8833bf8879646de1bab9544d745fbd|legacy=d7r18r1-complete-bytes;d7r13-stdout514ea1925a85d398a948a2dcbc319689116114a02335a599e51d6703202c18de|fixture=corner-box-2x2x2;active-compressed0.99;inactive1.01;initial-equals-prediction;u0|profiles=reference-dt0x3f71111111111111-kappa0x4093290000000000;aligned-dt0x3f0c01c01c01c01c-kappa0x415c75a640000000;theta0x3fc5cccccccccccd;derived-prework|variables=u=lambda/kappa;theta=kappa*dt2/M;u-next=max(0,u+c)|inner=direct-normalized-static-sparse;steihaug;eta1e-10;64-trials;8-rejects;minimum-radius1e-14;radius-policy-unchanged;divided-actual-reduction|precision=every-accepted-direct-normalized-long-double-naive+compensated;1024-ulp;resolved-negative-forbidden;candidate-effect-direct-normalized-binary128-naive+compensated;4096-ulp;relative-error<=0.05;pair-membership-exact;runtime-float128-none|outer=indices0-through63;primal1e-8;stationarity1e-10;dual-u0x3da1eed347666340;complementarity-u0x3d6cb1520bd70533;position1e-8dx;u-nonnegative;primal-monotone;pressure-diagnostic-only|correspondence=active-provisional11-confirm12-holdout13;inactive-provisional0-confirm1-holdout2;reference-repeat-exact;reference-aligned-transaction-byte-exact|controls=static-binding;invalid-prework;work-ledger;precision-ledger;forced-rollback|work=transactions5;outer<=320;inner-trials<=20480;hvp<=512000;all-pair0;workspace-live<=2;timing=none|routes=normalized-accepted-sign-contradiction;normalized-oracle-or-reduction-bound-required;full-normalized-private-state-confirmed;normalized-inner-policy-still-insufficient;normalized-outer-state-formulation-required|precedence=contradiction,oracle,confirmed,inner,outer|runs=2-clean-release-builds;1-process-each;byte-exact|trajectory=none;nominal-substeps=0;macro=none;public-commit=none;physics-mutation=none;production-scale=none|credit=one-tiny-full-normalized-private-transaction-only
```

Identity SHA-256:
`00a883fcbeec514414d1bfe482344b1c9fbaf08c52ae02d25bbc072f61c3a3de`.

## Required command

Add `--nonlocal-al-full-normalized-private-transaction`. It must:

1. reproduce complete D7R18R1 stdout bytes;
2. derive reference and aligned `theta` from the exact finite-positive
   `{dt,kappa}` profiles before topology, workspace, precision or HVP work;
3. execute active, active-repeat and inactive reference transactions plus
   active and inactive aligned transactions using only direct normalized
   `{u,theta}` objective, gradient, HVP, divided reduction and dual update;
4. retain D7R13 Steihaug/radius/acceptance policy, `eta=1e-10`, 64 outer,
   64 inner-trial, eight-reject and `1e-14` minimum-radius bounds;
5. audit every accepted trial with direct normalized naive/compensated long
   double and every candidate-effect acceptance with direct normalized
   naive/compensated binary128 under the frozen sign/error gates;
6. route both precision paths through one static-bound canonical current/trial
   pair union with exact binary64/extended membership and zero all-pair calls;
7. require the active sequence at provisional/confirmation/holdout indices
   `11/12/13`, the inactive sequence at `0/1/2`, exact reference repeat and
   byte-exact reference/aligned normalized transaction roots;
8. use the exact normalized primal, stationarity, dual, complementarity,
   position and nonnegative-`u` gates; pressure change is diagnostic only;
9. reject invalid profile, `theta`, `u`, binding and structural budget inputs
   before prohibited work and prove exact work/precision/lifecycle ledgers;
10. force rollback of every public active/inactive state and publish nothing;
11. directly regress D7R13 once from each clean build;
12. execute one process from each of two clean Release builds and emit one
    route under the frozen precedence without a nominal substep, macro,
    trajectory or timing measurement.

## Routes

1. `NORMALIZED_ACCEPTED_SIGN_CONTRADICTION`.
2. `NORMALIZED_ORACLE_OR_REDUCTION_BOUND_REQUIRED`.
3. `FULL_NORMALIZED_PRIVATE_STATE_CONFIRMED`.
4. `NORMALIZED_INNER_POLICY_STILL_INSUFFICIENT`.
5. `NORMALIZED_OUTER_STATE_FORMULATION_REQUIRED`.

Identity, R1 parent bytes, D7R13 regression, profile derivation, static
binding, invalid prework, cross-profile root, pair membership,
work/precision/lifecycle ledger, rollback, process repeat or route-precedence
mismatch is hard FAIL. A finite transaction that misses the expected
confirmation indices is classified by the inner/outer routes rather than
hidden as a harness failure.

Only `FULL_NORMALIZED_PRIVATE_STATE_CONFIRMED` authorizes research/freeze of
D7R19 as one bounded aligned nominal Dam substep under the normalized
representation. It grants no execution authority for D7R19, another substep,
macro, trajectory, timing, public state, parallel/GPU, runtime or production
path.
