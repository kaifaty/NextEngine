# NSR3-B4E2D7R19 -- normalized nominal-substep shadow contract

Status: `FROZEN / IMPLEMENTATION_NEXT / ONE_PRIVATE_SUBSTEP_ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r19-normalized-nominal-substep-shadow|v1|parent=0d537bef0e879b28bd36608e350805c564f6370b:5bb8f5abcb6abf0903c681529771ee6814b567f09d6e12f138514c3f0dab0dc3:ffcede5263bdedf21486d3068d5ded6f0d986ae2f05b437410e047535a2494d3|legacy=d7r17-stdouta2a8de930d41d8e49c255a3fcf987a8794b4da067ddadf658732946fca0ffced;d7r17-semantic1f368c86ec760a232e0314875d7f61010ecdf37f3a2b80023274855c8c71913b|alignment=frame0-0d567ba5512ba237a48e5e0b828a670a398f1bf23a35ac269729cad535f374d7;dt0x3f0c01c01c01c01c;kappa0x415c75a640000000;theta0x3fc5cccccccccccd;particles6000;lower-y-clamps400;free5600|predictor=v+dt*g;box-clamp;contact-impulse-separate|solver=r4r2-normalized-static-sparse;u=lambda/kappa;u-next=max(0,u+c);pairwise-precancelled-divided;dimensionless-forcing-frozen-per-trust-solve;private-only;confirmed+warm-holdout|precision=every-accepted-normalized-long-double;candidate-effect-normalized-binary128;runtime-float128-none|required-budget=outer-updates16;inner-trials16;hvp-per-step32;hvp-total512;workspace-builds288;precision-audits64;prework-check;release-exact|ledger=gravity;predictor-contact;kinematic-pressure-m-over-dt;fixed-support-reaction-minus-m-over-dt-times-normalized-gradient;fluid-momentum-identity;pressure-support-residual|gates=finite;mass-exact;u-nonnegative;primal<=1e-8;stationarity<=1e-10;normalized-complementarity<=0x3d6cb1520bd70533;position-update<=1e-8dx;penetration<=1e-12;impulse-closure<=1e-10-scaled;all-pair-calls0;live-workspaces<=2|routes=normalized-nominal-structural-watchdog-exhausted;normalized-nominal-solver-not-confirmed;normalized-nominal-boundary-penetration;normalized-nominal-impulse-ledger-mismatch;normalized-nominal-substep-shadow-confirmed|precedence=watchdog,solver,boundary,ledger,confirmed|controls=r4r2-parent-bytes;d7r17-bytes;alignment;scaled-coefficients;static-binding;invalid-prework;structural-work;precision-ledger;policy-provenance;forced-rollback|runs=2-clean-release-builds;1-process-each;byte-exact;timing=none|trajectory=none;nominal-substeps=1;second-substep=none;macro=none;public-commit=none;physics-mutation=none;projected-contact=none;runtime-wide-precision=none|credit=one-private-normalized-nominal-substep-only
```

Identity SHA-256:
`fa4302fb35a93bb8cbc6372f3358d787493e2276df1739a30a39c9ce5c9a080e`.

## Required command

Add `--nonlocal-al-normalized-nominal-substep-shadow`. It must:

1. reproduce complete R4R2 and D7R17 stdout bytes;
2. derive exact aligned `dt`, scaled `kappa` and normalized `theta` before
   candidate work;
3. decode the exact frame-zero state, rebuild one identity-bound static index,
   form the same clamped predictor and preserve its contact ledger;
4. reject invalid profile, dual, binding or structural budget before nonlinear
   work;
5. execute exactly one private normalized pairwise-precancelled transaction
   with explicit dimensionless forcing under every frozen internal budget;
6. select only a doubly admissible confirmed update with successful warm
   holdout, selecting the confirmed state rather than the holdout;
7. report every outer state, work/policy/precision counter, density range,
   penetration, private root and normalized dual range;
8. reconstruct physical pressure/support impulses with the exact `M/dt`
   mapping and require both scaled impulse closures at `1e-10`;
9. preserve frame-zero/public bytes, release every workspace, keep all-pair
   calls zero and force rollback for every route;
10. execute one process from each of two clean Release builds and emit one
    route under frozen precedence;
11. execute no second substep, macro, trajectory, timing lane, projected
    contact, public commit or runtime mutation.

## Routes

1. `NORMALIZED_NOMINAL_STRUCTURAL_WATCHDOG_EXHAUSTED`.
2. `NORMALIZED_NOMINAL_SOLVER_NOT_CONFIRMED`.
3. `NORMALIZED_NOMINAL_BOUNDARY_PENETRATION`.
4. `NORMALIZED_NOMINAL_IMPULSE_LEDGER_MISMATCH`.
5. `NORMALIZED_NOMINAL_SUBSTEP_SHADOW_CONFIRMED`.

Identity, parent bytes, alignment/scaling, nonfinite state, mass, invalid
prework, policy provenance, precision/lifecycle, all-pair-call, rollback,
build/process repeat or route-precedence mismatch is hard FAIL.

This contract grants one private first-substep execution only. It grants no
second substep, macro, trajectory, timing, projected contact, public state,
runtime integration, GPU work or production authority.
