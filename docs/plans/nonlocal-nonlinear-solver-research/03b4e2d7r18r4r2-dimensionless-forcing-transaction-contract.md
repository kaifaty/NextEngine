# NSR3-B4E2D7R18R4R2 -- dimensionless-forcing transaction contract

Status: `FROZEN / IMPLEMENTATION_NEXT / D7R19_EXECUTION_BLOCKED`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r18r4r2-dimensionless-forcing-private-transaction|v1|parent=10d1f8f4c34737f779f95c0cf2722524d6ce8a42:26d3bf53272e9eaa2a67d579cdf3a51e5f9ab0435277b1141f92cbcd95298f31:f295cbee3002b1fb872363a4f2288b0640bfcd5c255170d05e94e393303b077c|legacy=d7r18r4-stdout32e4369a19564c769148c9d0bc534bce8f0d22dac4a0eeb085aa191a46dc38c6;d7r18r3-stdoute0b36e34a047de1d8bb1435fa6f68cf5a53ab8a4260a95719775c03e4608d3e5;d7r18r2-stdout3659eac888c22eae5bcf7ae8c5f8a426bcbc12bd24bd3320c23be0c176815d77;d7r13-stdout514ea1925a85d398a948a2dcbc319689116114a02335a599e51d6703202c18de|fixture=corner-box-2x2x2;particles8;active-compressed0.99;inactive1.01;initial-equals-prediction;u0|profiles=reference-dt0x3f71111111111111-kappa0x4093290000000000;aligned-dt0x3f0c01c01c01c01c-kappa0x415c75a640000000;theta0x3fc5cccccccccccd;derived-prework|variables=u=lambda/kappa;theta=kappa*dt2/M;u-next=max(0,u+c)|inner=direct-normalized-static-sparse;steihaug;dimensionless-eta=min(0.5,sqrt(max-particle-gradient-norm/dx));eta-frozen-per-trust-solve;64-trials;8-rejects;minimum-radius1e-14;radius-policy-unchanged;r3-pairwise-precancelled-divided-actual-reduction|work-derivation=norm2-gradient<=sqrt(8)*max-particle-gradient-norm;dx*sqrt(8)<1;eta-dimensionless>=eta-inherited;r4-trials18x-hvp2+1x-hvp3;r4r1-unique-first-recurrence-continues;active-hvp39-preimplementation|precision=every-accepted-direct-normalized-long-double-naive+compensated;1024-ulp;resolved-negative-forbidden;candidate-effect=accepted-and-raw-or-r2-rounded-divided-would-reject;every-candidate-effect-direct-normalized-binary128-naive+compensated;4096-ulp;relative-error<=0.05;pair-membership-exact;runtime-float128-none|outer=indices0-through63;primal1e-8;stationarity1e-10;dual-u0x3da1eed347666340;complementarity-u0x3d6cb1520bd70533;position1e-8dx;u-nonnegative;primal-monotone;pressure-diagnostic-only|correspondence=active-root-9a3a57e7fcb29700c72c710936ef02ea7459cf2470b7d59ca663605df00c5ee9;inactive-root-be761a3c07bc4a7f558486c5e9bc5d3b80e1cc0ba2982e1698e7f5a3d24c2585;active-provisional11-confirm12-holdout13;active-each-outer14-accepted19-rejected0-hvp39;inactive-provisional0-confirm1-holdout2;inactive-each-outer3-accepted0-rejected0-hvp0;reference-repeat-exact;reference-aligned-transaction-byte-exact|controls=r4r1-parent-bytes;r4-hard-fail-bytes;r3+r2+d7r13-regression;explicit-policy-provenance;static-binding;invalid-prework;work-ledger;precision-ledger;forced-rollback|work=transactions5;outer48;inner-trials57;accepted57;rejected0;hvp117;dimensionless-policy-trust-steps57;inherited-policy-trust-steps0;workspaces153;all-pair0;workspace-live<=2;timing=none|routes=dimensionless-forcing-accepted-sign-contradiction;dimensionless-forcing-oracle-or-reduction-bound-required;full-normalized-dimensionless-forcing-state-confirmed;dimensionless-forcing-inner-policy-still-insufficient;dimensionless-forcing-outer-state-formulation-required|precedence=contradiction,oracle,confirmed,inner,outer|runs=2-clean-release-builds;1-process-each;byte-exact|trajectory=none;nominal-substeps=0;macro=none;public-commit=none;physics-mutation=none;production-scale=none|credit=one-tiny-full-normalized-dimensionless-forcing-private-transaction-only
```

Identity SHA-256:
`233195d7c00889b0542643eb855db27ad0356538b528061ddfddf435b6be758f`.

## Required command

Add `--nonlocal-al-full-normalized-dimensionless-forcing-private-transaction`.
It must:

1. reproduce R4R1 stdout bytes and R4's exact hard failure, plus R3/R2/D7R13;
2. derive reference/aligned `{dt,kappa,theta}` before candidate work;
3. prove the frozen eight-particle forcing-dominance inequality before work;
4. execute five complete normalized pairwise-precancelled transactions with
   explicit dimensionless forcing and no other solver change;
5. require exact active/inactive roots, `11/12/13`, `0/1/2` and cross-profile
   correspondence;
6. require each active run to use `14 outer / 19 accepted / 0 rejected / 39
   HVP`, and each inactive run `3 / 0 / 0 / 0`;
7. require exactly 57 candidate dimensionless-policy trust-step selections,
   zero candidate inherited selections, 48 outer, 57 trials, 117 HVP and 153
   workspace builds over all five runs;
8. preserve every R4 normalized long-double/binary128 precision gate and
   forbid resolved-negative accepted effects;
9. reject invalid profile, dual, binding and structural budget before all
   candidate work, and require all-pair zero, maximum live workspace two and
   exact rollback;
10. execute one process from each of two clean Release builds and emit one
    route under frozen precedence without timing;
11. run no nominal substep, macro, trajectory, public commit or production
    mutation.

## Routes

1. `DIMENSIONLESS_FORCING_ACCEPTED_SIGN_CONTRADICTION`.
2. `DIMENSIONLESS_FORCING_ORACLE_OR_REDUCTION_BOUND_REQUIRED`.
3. `FULL_NORMALIZED_DIMENSIONLESS_FORCING_STATE_CONFIRMED`.
4. `DIMENSIONLESS_FORCING_INNER_POLICY_STILL_INSUFFICIENT`.
5. `DIMENSIONLESS_FORCING_OUTER_STATE_FORMULATION_REQUIRED`.

Identity, parent/legacy bytes, profile/inequality derivation, policy provenance,
static binding, correspondence roots, exact work/precision ledgers, invalid
prework, lifecycle, rollback, process repeat or route precedence mismatch is
hard FAIL.

Only a confirmed route may authorize D7R19 research/freeze. This contract does
not authorize D7R19 execution, a nominal substep, macro, trajectory, timing,
runtime/public state, GPU work or production use.
