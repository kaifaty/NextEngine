# NSR3-B4E2D7R12 -- private divided outer continuation contract

Status: `CLOSED / PASS / PRIVATE_PRESSURE_STATE_CONFIRMED`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r12-private-divided-outer-continuation|v1|parent=80b70a831bbda0f216fc6ab5a41f8df310d25d49:f6810759b34599475d61f3f9e6b2cf8322873ee760209086377ebb2590dd5cab:3178c5cde71e3859fdf763b448c50da4bba98ce73914321ea40b2cf49c7e06b0|prefix-parent=d7r5:7ea5489fa90346f385ce8db08dba445f2e2389e65fdf556a13a17e837ba916d7;outer7:9bffc61a943f50cab449c052bd0a6791c40128e894d98d9a9589b0969dbb82c2:04a9c03308662d6102b165d8109b23c1b60a3145314603909b7dbfda8385d95e|lane=eta1e-10;outer8-through63;beta1226.25|inner=d7r10-divided-ratio,acceptance,rejected-radius;64-trials;8-rejects;minimum-radius1e-14;model,hvp,radius-policy-unchanged|audit=every-candidate-effect-accept;independent-binary128-fixed-and-compensated;4096-total-ulp;relative-error<=0.05;pair-membership-exact|gates=primal1e-8;stationarity1e-10;complementarity1e-9;abs-dual1e-8J;pressure8e-5Pa;position1e-8dx;lambda-nonnegative;primal-monotone|confirmation=two-consecutive-admissible;one-warm-holdout;same-gates|controls=d7r11-complete-bytes;d7r5-complete-bytes;prefix-roots;candidate-effect;work-ledger;forced-rollback|routes=binary128-sign-contradiction;oracle-or-reduction-bound-required;private-pressure-state-confirmed;inner-policy-still-insufficient;outer-state-formulation-required|precedence=contradiction,oracle,confirmed,inner,outer|runs=2-release-builds;2-processes;byte-exact;timing=none|trajectory=none;public-commit=none;physics-mutation=none;runtime-float128=none|credit=one-private-pressure-state-classification-only
```

Identity SHA-256:
`c33d7a6f039c44fc6f27ca438dab11d465896ce6eb19d0a0a11fa8bf76474e73`.

## Required command

Add `--nonlocal-al-divided-private-outer-continuation`. It must:

1. reproduce complete D7R11 and D7R5 stdout bytes;
2. reconstruct the exact frozen post-outer-7 prefix roots;
3. run only `eta=1e-10`, outer indices 8 through at most 63;
4. use the D7R10 numerator only for ratio, acceptance and rejected-radius
   interpolation while retaining every other inner/outer formula and limit;
5. emit every outer record and complete inner work/acceptance ledger;
6. binary128-audit every candidate-effect acceptance under the D7R11 rules;
7. preserve every dimensional pressure-state gate, two-record confirmation
   and one same-gate warm holdout;
8. exercise at least one candidate-effect acceptance, force rollback and
   publish no state;
9. emit exactly one route under the frozen precedence.

## Routes

1. `BINARY128_SIGN_CONTRADICTION`: any candidate-effect acceptance resolves
   negative.
2. `ORACLE_OR_REDUCTION_BOUND_REQUIRED`: no preceding route; at least one
   candidate-effect acceptance is unresolved or exceeds 5% oracle error.
3. `PRIVATE_PRESSURE_STATE_CONFIRMED`: no preceding route; two consecutive
   records and the warm holdout pass all frozen gates.
4. `INNER_POLICY_STILL_INSUFFICIENT`: no preceding route; an unchanged inner
   work/radius limit stops the continuation.
5. `OUTER_STATE_FORMULATION_REQUIRED`: no preceding route; finite inner work
   completes, but monotonicity, confirmation, holdout or cap remains invalid.

Identity, parent bytes, prefix roots, formula/profile independence, nonfinite,
pair-membership, candidate-effect, work ledger, rollback or route-precedence
mismatch is hard FAIL. PASS is one private pressure-state classification only.
It grants no trajectory, public state, general error bound, cap/tolerance,
runtime binary128, performance, GPU/runtime or production authority.

Closure evidence:
[NSR3-B4E2D7R12 private divided outer continuation evidence](../../development/nonlocal-nsr3b4e2d7r12-private-outer-evidence-2026-08-22.md).
