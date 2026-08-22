# NSR3-B4E2D7R18R4 -- full normalized precancelled transaction contract

Status: `FROZEN / IMPLEMENTATION_NEXT / D7R19_BLOCKED`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r18r4-full-normalized-precancelled-private-transaction|v1|parent=27b13e35748f89fd43d81dfb350429b4d5fcdbc8:e0b36e34a047de1d8bb1435fa6f68cf5a53ab8a4260a95719775c03e4608d3e5:884dcda82ec013903db389a6bd873326d2501f452babc8a7d2cedcda1faebf75|legacy=d7r18r3-complete-bytes;d7r18r2-stdout3659eac888c22eae5bcf7ae8c5f8a426bcbc12bd24bd3320c23be0c176815d77;d7r13-stdout514ea1925a85d398a948a2dcbc319689116114a02335a599e51d6703202c18de|fixture=corner-box-2x2x2;active-compressed0.99;inactive1.01;initial-equals-prediction;u0|profiles=reference-dt0x3f71111111111111-kappa0x4093290000000000;aligned-dt0x3f0c01c01c01c01c-kappa0x415c75a640000000;theta0x3fc5cccccccccccd;derived-prework|variables=u=lambda/kappa;theta=kappa*dt2/M;u-next=max(0,u+c)|inner=direct-normalized-static-sparse;steihaug;eta1e-10;64-trials;8-rejects;minimum-radius1e-14;radius-policy-unchanged;r3-pairwise-precancelled-divided-actual-reduction|precision=every-accepted-direct-normalized-long-double-naive+compensated;1024-ulp;resolved-negative-forbidden;candidate-effect=accepted-and-raw-or-r2-rounded-divided-would-reject;every-candidate-effect-direct-normalized-binary128-naive+compensated;4096-ulp;relative-error<=0.05;pair-membership-exact;runtime-float128-none|outer=indices0-through63;primal1e-8;stationarity1e-10;dual-u0x3da1eed347666340;complementarity-u0x3d6cb1520bd70533;position1e-8dx;u-nonnegative;primal-monotone;pressure-diagnostic-only|correspondence=active-provisional11-confirm12-holdout13;active-each-outer14-accepted19-rejected0-hvp38;inactive-provisional0-confirm1-holdout2;inactive-each-outer3-accepted0-rejected0-hvp0;reference-repeat-exact;reference-aligned-transaction-byte-exact|controls=r3-parent-bytes;r2+d7r13-regression;static-binding;invalid-prework;work-ledger;precision-ledger;forced-rollback|work=transactions5;outer48;inner-trials57;hvp114;all-pair0;workspace-live<=2;timing=none|routes=precancelled-accepted-sign-contradiction;precancelled-oracle-or-reduction-bound-required;full-normalized-precancelled-private-state-confirmed;precancelled-inner-policy-still-insufficient;precancelled-outer-state-formulation-required|precedence=contradiction,oracle,confirmed,inner,outer|runs=2-clean-release-builds;1-process-each;byte-exact|trajectory=none;nominal-substeps=0;macro=none;public-commit=none;physics-mutation=none;production-scale=none|credit=one-tiny-full-normalized-precancelled-private-transaction-only
```

Identity SHA-256:
`77f147fb3cb6b963e1873e0fd66afc55712c3dd9bda8bb4ba38e8a35cecec539`.

## Required command

Add `--nonlocal-al-full-normalized-precancelled-private-transaction`. It must:

1. reproduce complete D7R18R3 stdout bytes and directly preserve R2/D7R13;
2. derive exact reference/aligned profiles before topology, workspace,
   precision or HVP work;
3. execute active, active-repeat and inactive reference transactions plus
   active and inactive aligned transactions using R2's normalized solver with
   only the R3 pairwise numerator substituted;
4. leave the inherited R2 divided evaluator and R2 command unchanged;
5. use the frozen R3 sorted pair-union, density-delta, active-square and
   inertia-delta formula for ratio, acceptance and rejected-radius
   interpolation;
6. audit every accepted trial in direct normalized long double and every
   raw-or-R2 candidate-effect acceptance in direct normalized binary128 under
   the frozen sign, membership and 5% gates;
7. require active `11/12/13` and inactive `0/1/2` semantics, exact active work
   `14 outer / 19 accepted / 0 rejected / 38 HVP` per run, exact inactive work
   `3 / 0 / 0 / 0`, reference repeat and byte-exact reference/aligned roots;
8. retain the exact normalized admission, nonnegative-`u`, primal monotonicity,
   static binding and pressure-diagnostic-only rules;
9. reject invalid profile, `theta`, `u`, binding and structural-budget inputs
   before prohibited work and prove exact pair/precision/workspace ledgers;
10. force rollback of every caller-owned active/inactive position and dual
    state and publish nothing;
11. execute one process from each of two clean Release builds and emit one
    route under the frozen precedence without timing, a nominal substep,
    macro or trajectory.

## Routes

1. `PRECANCELLED_ACCEPTED_SIGN_CONTRADICTION`.
2. `PRECANCELLED_ORACLE_OR_REDUCTION_BOUND_REQUIRED`.
3. `FULL_NORMALIZED_PRECANCELLED_PRIVATE_STATE_CONFIRMED`.
4. `PRECANCELLED_INNER_POLICY_STILL_INSUFFICIENT`.
5. `PRECANCELLED_OUTER_STATE_FORMULATION_REQUIRED`.

Identity, R3 parent bytes, R2/D7R13 regression, profile derivation, static
binding, invalid prework, expected correspondence/work, cross-profile root,
pair membership, precision/workspace lifecycle, rollback, process repeat or
route-precedence mismatch is hard FAIL.

Only `FULL_NORMALIZED_PRECANCELLED_PRIVATE_STATE_CONFIRMED` may authorize
research/freeze of D7R19. It grants no execution authority for D7R19, another
substep, macro, trajectory, timing, public state, runtime binary128,
GPU/runtime or production use.
