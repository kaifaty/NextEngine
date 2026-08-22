# NSR3-B4E2D7R19R1 -- topology-precision shell replay contract

Status: `FROZEN / IMPLEMENTATION_NEXT / REPLAY_ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r19r1-topology-precision-shell-replay|v1|parent=085056d6593f3685a7d399c620a86cb560d80cb6:f5811bfc7d5e986d72b9130f8e6cb90c5ae21bd347ce7f0b476c171fe8cff7bb:bcc6f588010999f23664209037a0daae961606ca29fdcc0d7a31e661902e185b|target=outer0-trials0,1,2;roots=00188e8e74bf7bc16e9be0d1b7df3a9555f23d4d22f3bad8355b0a36dd4aac02,823812828a7386e3120762cab508b76577fae92c692a39152420ace1ff5d8f95,7298f0f80106642c72a6912736e30b5c55e5cb0898f58f85e8146967b6fb1ded|observed=topology-trials3;membership-observations10989;unique-state-observations7315|min-margin0|shell=abs-r-minus-h<=64eps-h;c2-w-w1-w2-at-h-zero|lanes=live-extended-membership;binary64-owned-membership;horizon-canonicalized-mismatch|precision=long-double-naive+compensated-1024ulp;binary128-naive+compensated-4096ulp;candidate-relative-error<=0.05|controls=parent-bytes;exact-trial-roots;pair-union;membership-counts;shell-bound;kernel-closure;no-acceptance;rollback;all-pair0|routes=nonlocal-topology-mismatch;topology-mismatch-alters-sign;topology-precision-unresolved;runtime-topology-precision-candidate|runs=2-clean-release-builds;1-process-each;byte-exact|nominal-substeps=0;replay-only;hvp=parent-only;public-commit=none;physics-mutation=none;timing=none;runtime-wide-precision=none|credit=precision-policy-discriminator-only
```

Identity SHA-256:
`da91f8ab89ce2d48c400ed0d2cec095827598958c7171be7e2cbd0b99b436be8`.

## Required command

Add `--nonlocal-al-topology-precision-shell-replay`. It must:

1. reproduce D7R19's complete stdout/semantic bytes and exact three target
   long-double roots;
2. reconstruct only outer `0` accepted trials `0..=2` and accept no replay
   state;
3. use the exact D7R19 static binding and sorted current/trial sparse union;
4. enumerate every binary64/extended membership mismatch without all-pair
   fallback;
5. require `abs(r_extended-h) <= 64*epsilon_binary64*h` for every mismatch and
   exact zero `W/W'/W''` at the horizon in binary64, long double and binary128;
6. compare live-extended, binary64-owned and horizon-canonicalized membership
   lanes with naive/compensated long-double and binary128 arithmetic;
7. require `1024` long-double and `4096` binary128 ULP sign resolution and at
   most `5%` candidate-reduction relative error;
8. report exact pair/mismatch counts, maximum shell distance, maximum mismatch
   kernel value/gradient/curvature, every lane sign/root and work/lifecycle;
9. preserve frame-zero and all candidate states exactly, release every
   workspace and execute no second nominal substep, macro or timing lane;
10. run one fresh process from each of two clean Release builds and emit one
    route under frozen precedence.

## Routes

1. `NONLOCAL_TOPOLOGY_MISMATCH`.
2. `TOPOLOGY_MISMATCH_ALTERS_SIGN`.
3. `TOPOLOGY_PRECISION_UNRESOLVED`.
4. `RUNTIME_TOPOLOGY_PRECISION_CANDIDATE`.

Identity, parent bytes, target roots, pair-union/order, structural work,
no-acceptance, rollback, all-pair-call, build/process repeat or route-precedence
mismatch is hard FAIL.

This contract grants one precision-policy discriminator only. It does not
change D7R19, select a full precision policy, increase its watchdog, rerun the
nominal transaction, authorize wider runtime precision or create production
authority.
