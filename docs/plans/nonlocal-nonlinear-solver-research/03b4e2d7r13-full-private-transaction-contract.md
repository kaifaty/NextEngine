# NSR3-B4E2D7R13 -- full private divided transaction contract

Status: `FROZEN / NOT_RUN / PRIVATE_ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r13-full-private-divided-transaction|v1|parent=9f3fd810af83730469a36c3249b6c921652b8a87:3f612fbc85c4f39f741bf407584747c075ed468a18a59262d401b883ba7e2d03:3c3893b100ae3d0514394a7c265985eeefb545cdbca4a2235636e48fb186aaca|state=corner-box-2x2x2;compressed0.99;zero-multiplier;start-original|lane=eta1e-10;outer0-through63;beta1226.25|inner=d7r10-divided-ratio,acceptance,rejected-radius;64-trials;8-rejects;minimum-radius1e-14;model,hvp,radius-policy-unchanged|audit=every-candidate-effect-accept;independent-binary128-fixed-and-compensated;4096-total-ulp;relative-error<=0.05;pair-membership-exact|gates=primal1e-8;stationarity1e-10;complementarity1e-9;abs-dual1e-8J;pressure8e-5Pa;position1e-8dx;lambda-nonnegative;primal-monotone|confirmation=two-consecutive-admissible;one-warm-holdout;same-gates|controls=d7r12-complete-bytes;active-repeat;inactive-full-transaction;candidate-effect;work-ledger;forced-rollback|routes=binary128-sign-contradiction;oracle-or-reduction-bound-required;full-private-pressure-state-confirmed;inner-policy-still-insufficient;outer-state-formulation-required|precedence=contradiction,oracle,confirmed,inner,outer|runs=2-release-builds;2-processes;byte-exact;timing=none|trajectory=none;public-commit=none;physics-mutation=none;runtime-float128=none|credit=one-full-private-pressure-state-classification-only
```

Identity SHA-256:
`7641bdb9689779c07e4b26ee251f0bbe3abe2e6ba279cf09033328828927f4db`.

## Required command

Add `--nonlocal-al-divided-full-private-transaction`. It must:

1. reproduce complete D7R12 stdout bytes;
2. start active work from the original compressed `0.99` state and zero
   multipliers, with no inherited outer prefix;
3. use `eta=1e-10` and the D7R10 divided inner for every outer update through
   at most index 63;
4. retain every physical formula, trust limit and pressure-state gate;
5. binary128-audit every candidate-effect acceptance and emit the complete
   work/audit ledger;
6. reproduce the complete active transaction twice in process;
7. confirm the inactive `compressed=1.01` control without position or
   multiplier change;
8. require two consecutive active admissible states plus one same-gate
   holdout, force rollback and publish no state;
9. emit exactly one route under the frozen precedence.

## Routes

1. `BINARY128_SIGN_CONTRADICTION`.
2. `ORACLE_OR_REDUCTION_BOUND_REQUIRED`.
3. `FULL_PRIVATE_PRESSURE_STATE_CONFIRMED`.
4. `INNER_POLICY_STILL_INSUFFICIENT`.
5. `OUTER_STATE_FORMULATION_REQUIRED`.

Identity, parent bytes, active repeat, inactive control, formula/profile,
nonfinite, pair-membership, candidate-effect, work/audit ledger, rollback or
route-precedence mismatch is hard FAIL. PASS is one full private pressure-
state classification only. It grants no nominal frame, trajectory, public
state, general error bound, timing, GPU/runtime or production authority.

