# NSR3-B4E2D7R18R4R1 -- normalized Krylov forcing replay contract

Status: `FROZEN / IMPLEMENTATION_NEXT / D7R19_BLOCKED`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r18r4r1-normalized-krylov-forcing-replay|v1|parent=8becfe9e5b22af468fbc4f22873eff8b558d0115:32e4369a19564c769148c9d0bc534bce8f0d22dac4a0eeb085aa191a46dc38c6:91b311aa592a9d5e8f6845c80d98abec98329c1a00c1eb80b559a2d77c1d49e0|legacy=d7r18r4-complete-bytes;d7r18r3-stdoute0b36e34a047de1d8bb1435fa6f68cf5a53ab8a4260a95719775c03e4608d3e5|fixture=corner-box-2x2x2;active-compressed0.99;initial-equals-prediction|profiles=reference-dt0x3f71111111111111-kappa0x4093290000000000;aligned-dt0x3f0c01c01c01c01c-kappa0x415c75a640000000;theta0x3fc5cccccccccccd;alpha0x3f223456789abcdf;derived-prework|target=active-root-9a3a57e7fcb29700c72c710936ef02ea7459cf2470b7d59ca663605df00c5ee9;outer11;trial0;stationarity0x3de517c90da7b861;step0x3d8feecfa8d474e2;predicted0x3b47d038719e0000;divided0x3b47d08b70000000;inherited-hvp3;d7r13-hvp2|replay=outer10-u;target-current-position;one-static-normalized-workspace;first-steihaug-hvp+residual-recurrence-only|forcing=q=norm-r1/norm-r0;inherited=min(0.5,sqrt(norm-r0));mapped-dimensional=min(0.5,sqrt(norm-r0/alpha));dimensionless-sigma=max-particle-gradient-norm/dx;dimensionless=min(0.5,sqrt(sigma));fixed0.5-observable|resolution=abs(q-eta-dimensionless)>=4096-binary64-ulp-of-max-magnitude|correspondence=inherited-fails;mapped-dimensional-passes;reference-repeat-byte-exact;reference-aligned-byte-exact|controls=r4-parent-bytes;target-anchors;static-binding;invalid-prework;work=active-replay-unchanged+candidate-workspaces2+hvp2;workspace-live<=1;all-pair0;forced-rollback|routes=krylov-forcing-mechanism-contradiction;krylov-forcing-bound-required;dimensionless-forcing-retains-second-iteration;dimensionless-forcing-restores-d7r13-work|precedence=contradiction,bound,retains,restores|runs=2-clean-release-builds;1-process-each;byte-exact;timing=none|candidate-acceptances=0;candidate-outer-updates=0;changed-policy-full-transactions=0;trajectory=none;nominal-substeps=0;macro=none;public-commit=none;physics-mutation=none;production-scale=none|credit=one-replay-only-normalized-krylov-forcing-discriminator
```

Identity SHA-256:
`e8ed1ecc5bec03ca592c6d59f50d3b940b1ae58e0fb889d20b4c656008745df2`.

## Required command

Add `--nonlocal-al-normalized-krylov-forcing-replay`. It must:

1. reproduce complete D7R18R4 stdout bytes and its hard work failure;
2. derive exact reference/aligned `{dt,kappa,theta}` and reference
   `alpha=dt^2/M` before candidate work;
3. replay the unchanged reference-active R4 transaction and recover outer
   `11`, trial `0` only under the frozen root and binary64 anchors;
4. obtain target `u` from the passed outer-10 update and build one normalized
   sparse workspace at the exact target current position;
5. execute only the first Steihaug HVP, step coefficient and residual
   recurrence once for reference and once for aligned, without applying the
   step;
6. emit `||r0||`, `||r1||`, `q`, inherited, mapped-dimensional,
   dimensionless and fixed thresholds plus every branch decision;
7. require inherited failure and mapped-dimensional success as the mechanism
   control;
8. resolve the dimensionless decision only outside the frozen 4096-binary64-
   ULP band;
9. repeat the diagnostic under independently derived aligned normalized
   coefficients and require byte-exact roots;
10. reject invalid profile, dual, binding and workspace inputs before HVP work,
    prove two candidate workspaces/two HVPs total, maximum live one, zero
    all-pair calls and exact rollback;
11. perform zero candidate acceptances, candidate outer updates or full
    changed-policy transactions and publish nothing;
12. execute one process from each of two clean Release builds and emit one
    route under frozen precedence without timing, a nominal substep, macro or
    trajectory.

## Routes

1. `KRYLOV_FORCING_MECHANISM_CONTRADICTION`.
2. `KRYLOV_FORCING_BOUND_REQUIRED`.
3. `DIMENSIONLESS_FORCING_RETAINS_SECOND_ITERATION`.
4. `DIMENSIONLESS_FORCING_RESTORES_D7R13_WORK`.

Identity, R4 parent bytes, profile/scale derivation, target anchors, static
binding, invalid prework, repeat/cross-profile roots, HVP/workspace lifecycle,
rollback, process repeat or route-precedence mismatch is hard FAIL.

A resolved dimensionless route authorizes only research/freeze of a separate
complete normalized private transaction with an explicit forcing policy. It
grants no execution authority for that transaction, D7R19, a nominal substep,
macro, trajectory, timing, public state, GPU/runtime or production use.
