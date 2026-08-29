# NSR3-B4E2D7R6 -- cap versus nested-accuracy discriminator contract

Status: `CLOSED / FAIL / INNER_SUBPROBLEM_NUMERICAL_FLOOR / PRIVATE_ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r6-cap-accuracy-discriminator|v1|parent=b9f0e532e064f2316b1f3133fdd3fb7877a7da431c1284d8233f3e3cc5be48b9:46bfccedddada3dc38479e3f510998a7cf00fa68c04f9a37703016bd4d98398e:7ea5489fa90346f385ce8db08dba445f2e2389e65fdf556a13a17e837ba916d7|state=d7-prefix:9bffc61a943f50cab449c052bd0a6791c40128e894d98d9a9589b0969dbb82c2:04a9c03308662d6102b165d8109b23c1b60a3145314603909b7dbfda8385d95e;d7r5-final:1fdbcb763dedce34937c399986691469eaab55565024277ddc70f4dbf8c86546|solver=step-norm-trust-inner-candidate;beta=1226.25;outer=d7r-unchanged|fork=post-outer7-private|inner-stationarity-ladder=1e-8,1e-9,1e-10,1e-11,1e-12;only-stop-threshold-changes|outer-observation-cap=64;original-cap=14|gates=primal1e-8,stationarity-lane-eta,complementarity1e-9,abs-dual1e-8J,pressure8e-5Pa,position1e-8dx,lambda-nonnegative,primal-monotone|confirmation=two-consecutive-admissible;warm-holdout=one-private-update;same-gates|trace=all-lanes;all-outer;inner-work;first-admissible;confirmation;holdout|controls=eta1e-8-first14-d7r5-exact;inactive;reset;forced-rollback;parent-bytes|routes=outer-cap-sufficient;nested-accuracy-sufficient;coupled-cap-and-accuracy-required;pressure-state-formulation-research|precedence=outer-cap,nested-accuracy,coupled,formulation|runs=2-release-builds;2-processes;byte-exact;timing=none|trajectory=none;public-commit=none;physics-mutation=none|credit=accuracy-schedule-or-cap-contract-research-only
```

Identity SHA-256:
`65f1a01ccc110a4e110c2e8b9907d9a29f0ecb58fd1e09a4f3b3a9af0fc5ee0a`.

## Required command

Add `--nonlocal-al-cap-accuracy-discriminator`. It must:

1. reproduce the complete D7R5 parent report and exact post-outer-7 fork;
2. parameterize only the private inner scaled-stationarity stop;
3. run `eta={1e-8,1e-9,1e-10,1e-11,1e-12}` through at most outer index 63;
4. preserve all outer formulas, `beta`, globalization, reject and dimensional
   gates;
5. require the `eta=1e-8` lane through outer 13 to match D7R5 exactly;
6. emit every lane/outer work and KKT record, first admissible index,
   confirmation index and private holdout result;
7. retain inactive/reset/forced rollback controls and zero public commits;
8. emit exactly one route under the frozen precedence.

## Routes

1. `OUTER_CAP_SUFFICIENT`: unchanged `eta=1e-8` confirms after index 13 and
   by index 63, including an admissible warm holdout.
2. `NESTED_ACCURACY_SUFFICIENT`: no preceding route; the loosest tighter eta
   confirms by index 13 with an admissible warm holdout.
3. `COUPLED_CAP_AND_ACCURACY_REQUIRED`: no preceding route; a tighter eta
   confirms only after index 13 and by index 63 with an admissible holdout.
4. `PRESSURE_STATE_FORMULATION_RESEARCH`: every lane is finite, dual-feasible
   and primal-monotone through index 63, but none confirms.

Parent, prefix, D7R5-control, nonfinite, inner-failure, dual-feasibility,
monotonicity, rollback or route-precedence mismatch is hard FAIL. PASS is a
diagnostic classification only. It grants no cap/tolerance selection, public
pressure state, trajectory, performance, runtime, GPU/PhysX or production
authority.
