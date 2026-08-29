# NSR3-B4E2D7R18 -- kappa scaling prerequisites contract

Status: `CLOSED / PASS_CLASSIFICATION / DT_KAPPA_NONDIMENSIONAL_MISMATCH / D7R19_BLOCKED / SHARED_HOST_PERFORMANCE_STOP`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r18-kappa-scaling-prerequisites|v1|parent=ae81c21dee45dffeebd88d4a0365e4d96bdf3715:1f368c86ec760a232e0314875d7f61010ecdf37f3a2b80023274855c8c71913b:a2a8de930d41d8e49c255a3fcf987a8794b4da067ddadf658732946fca0ffced|scale=dt-ref0x3f71111111111111;kappa-ref0x4093290000000000;ratio78;ratio2=6084;dt-sub0x3f0c01c01c01c01c;kappa-sub0x415c75a640000000;kappa-dt2=0x3f95cccccccccccd;inertia-ref7200;inertia-sub43804800|api=explicit-finite-positive-dt+kappa;workspace-kappa-bound;phr-energy-gradient-hvp;divided-reduction;long-double;binary128;outer-multiplier+scaled-dual|oracle=tiny-active-same-position+prediction+direction;zero+scaled-multiplier;reference-vs-substep-normalized;dense-sparse-exact-per-scale;one-ulp-kappa-mutation|invalid=zero;negative;nan;+inf;reject-before-static-workspace,pair,precision,hvp,outer|legacy=d7r16-complete-bytes;d7r17-direct-clean-regression|work=no-nominal-solve;no-outer-transaction;all-pair0;workspace-live<=2;timing=none|routes=kappa-propagation-mismatch;dt-kappa-nondimensional-mismatch;kappa-invalid-prework-mismatch;dt-kappa-scaling-prerequisites-confirmed|precedence=propagation,nondimensional,invalid,confirmed|runs=2-clean-release-builds;1-process-each;byte-exact|trajectory=none;nominal-substeps=0;macro=none;public-commit=none;physics-mutation=none;production-kappa-selection=none|credit=explicit-kappa+nondimensional-prerequisite-only
```

Identity SHA-256: `72d05af759eb440959ea6273373eb8851d43ff1a80cb49a8004055e92654925a`.

## Required command

Add `--nonlocal-al-kappa-scaling-prerequisites`. It must:

1. reproduce complete D7R16 stdout bytes without executing D7R17's nominal
   solve internally;
2. introduce one explicit finite-positive `dt`/`kappa` scale through sparse
   workspace PHR energy/gradient, HVP, divided reduction, long-double,
   binary128, multiplier update and scaled-dual accounting;
3. bind every sparse workspace to the exact `kappa` bits used for its active
   coefficient tape;
4. preserve reference dense/sparse evaluation, gradient, HVP, divided and
   precision equivalence on the D7R15 tiny active fixture;
5. prove the frozen ratio, squared ratio, both coefficient bits, exact common
   `kappa*dt^2` bits and exact `7200*6084=43804800` inertia scaling;
6. run the same tiny fixture at scaled `dt`, `kappa` and multiplier, retain
   density/constraint/active membership and pass the predeclared normalized
   PHR/inertia/gradient/HVP/divided/precision comparisons;
7. make a one-ULP scaled-`kappa` mutation change the dimensionless product and
   at least one active output root;
8. reject zero, negative, NaN and positive-infinite `kappa` before static,
   workspace, pair, precision, HVP or outer work;
9. release every workspace, retain at most two live workspaces and perform
   zero candidate all-pair calls;
10. execute in two clean Release builds, one process each, and directly run
    D7R17 once per build as an external byte-exact regression;
11. publish exactly one route under the frozen precedence, without a nominal
    solve, outer transaction, trajectory, timing or state mutation.

The normalized comparison tolerance is `64*epsilon` per scalar/vector maximum
relative error, with zero compared exactly. Active membership, density,
constraint, coefficient bits, scale bits and the dimensionless product use
their exact gates above; the tolerance cannot excuse a scale or branch change.

## Routes

1. `KAPPA_PROPAGATION_MISMATCH`.
2. `DT_KAPPA_NONDIMENSIONAL_MISMATCH`.
3. `KAPPA_INVALID_PREWORK_MISMATCH`.
4. `DT_KAPPA_SCALING_PREREQUISITES_CONFIRMED`.

Identity, D7R16 parent bytes, clean D7R17 regression, non-finite output,
lifecycle, all-pair-call, rollback, process repeat or route-precedence
mismatch is hard FAIL.

The executed prerequisite preserves every dense/sparse and precision pair but
misses the frozen normalized HVP limit: `7.51298e-14` versus
`1.42109e-14`; see the
[dated evidence](../../development/nonlocal-nsr3b4e2d7r18-kappa-scaling-prerequisites-evidence-2026-08-22.md).
D7R19 remains blocked. Research a directly nondimensional
`u=lambda/kappa`, `theta=kappa*dt^2/M` transaction next; do not weaken the HVP
gate or inherit raw absolute-`lambda` admission across the scale change.
