# NSR3-B4E2D7R18R1 -- nondimensional AL transaction contract

Status: `FROZEN / IMPLEMENTATION_NEXT / SHARED_HOST_PERFORMANCE_STOP`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r18r1-nondimensional-al-transaction|v1|parent=0b649563729757fce9ffe7df15e664516031511b:871e1ca5b84968261aaa33bca78d5deab92562e2279dadbe1d5e311be0d19084:0267e094641aaddd9b604a71b527db976aaa699f51dacd558624512bd3b1b925|variables=u=lambda/kappa;theta=kappa*dt2/M;theta0x3fc5cccccccccccd;u-next=max(0,u+c)|objective=0.5*norm2(y-yhat)+0.5*theta*sum(max(0,u+c)^2-u^2)|gradient=displacement+theta*sum(max(0,u+c)*J)|hvp=I+theta*sum((Jp)J+max(0,u+c)Hc);direct-normalized;no-postscale|divided=direct-normalized-phr+inertia;long-double+binary128-normalized|reconstruction=dimensional-reference+scaled;componentwise-absolute-bound=gamma(96*max-degree+256)*sum-absolute-terms;observed<=bound|oracle=d7r18-tiny-active;same-position+prediction+direction+u;reference+substep-theta-exact;normalized-dense-sparse-exact;cross-profile-byte-exact;dimensional-reconstruction-certified|admission=primal1e-8;stationarity1e-10;dual-u<=0x3da1eed347666340;complementarity-u<=0x3d6cb1520bd70533;position-dx1e-8;kinematic+impulse-ledger-retained;equivalent-pressure-diagnostic-only|invalid=dt,kappa,theta,u-nonfinite-reject-prework;one-ulp-theta-mutation|legacy=d7r18-complete-bytes;d7r17-clean-regression|work=no-nominal-solve;no-outer-transaction;all-pair0;workspace-live<=2;timing=none|routes=normalized-formula-mismatch;normalized-reconstruction-bound-mismatch;normalized-dual-admission-mismatch;nondimensional-al-transaction-candidate|precedence=formula,reconstruction,admission,candidate|runs=2-clean-release-builds;1-process-each;byte-exact|trajectory=none;nominal-substeps=0;macro=none;public-commit=none;physics-mutation=none;production-scale=none|credit=nondimensional-private-transaction-formulation-only
```

Identity SHA-256: `ff63c33a7f8a2058c4e8a3df3f87d106df26416ffa1728cfbb48382450eebdee`.

## Required command

Add `--nonlocal-al-nondimensional-transaction`. It must:

1. reproduce complete D7R18 stdout bytes without internally executing D7R17;
2. validate finite-positive `dt`, `kappa` and exact finite-positive
   `theta=kappa*dt^2/M` before topology/workspace/precision/HVP work;
3. represent the dual state only as finite `u=lambda/kappa` and implement the
   frozen normalized objective, gradient, HVP, divided difference, long-double,
   binary128 and `u_next` formulas directly;
4. bind the sparse normalized workspace to exact `theta` bits;
5. reproduce an independent dense normalized oracle exactly at reference and
   aligned profiles and emit byte-identical normalized roots across profiles;
6. compare reconstructed dimensional reference/aligned HVP components against
   the existing dimensional path using
   `gamma_(96*maximum_degree+256)*sum_abs_terms`, with every observed absolute
   error no greater than its component bound;
7. prove the exact reference mapping and cross-profile equality of primal,
   normalized stationarity, normalized dual-change, normalized
   complementarity and position-state admission; retain equivalent pressure
   only as a diagnostic and require the later kinematic/impulse ledger gates;
8. make a one-ULP `theta` mutation change the normalized root;
9. reject zero/negative/NaN/infinite `dt`, `kappa`, `theta` and any non-finite
   `u` before static/workspace/pair/precision/HVP/dual-update work;
10. retain zero all-pair candidate calls, at most two live workspaces and exact
    release/rollback;
11. execute in two clean Release builds, one process each, and directly run
    D7R17 once per build as an external byte-exact regression;
12. emit one route under the frozen precedence without a nonlinear solve,
    outer transaction, nominal substep, macro, trajectory, timing or mutation.

Zero normalized terms compare exactly. Dense/sparse and reference/substep
normalized roots compare byte-exactly; the forward certificate is not a
replacement tolerance for those gates.

## Routes

1. `NORMALIZED_FORMULA_MISMATCH`.
2. `NORMALIZED_RECONSTRUCTION_BOUND_MISMATCH`.
3. `NORMALIZED_DUAL_ADMISSION_MISMATCH`.
4. `NONDIMENSIONAL_AL_TRANSACTION_CANDIDATE`.

Identity, D7R18 parent bytes, clean D7R17 regression, invalid pre-work,
non-finite output, lifecycle, all-pair-call, rollback, process repeat or
route-precedence mismatch is hard FAIL.

Only `NONDIMENSIONAL_AL_TRANSACTION_CANDIDATE` authorizes research/freeze of a
tiny full private normalized transaction reproducing D7R13 confirmation,
holdout, accepted-sign precision and rollback. D7R19 remains blocked. No
nominal solve, production scale, macro, trajectory, timing, public state,
parallel/GPU, runtime or production authority is granted.
