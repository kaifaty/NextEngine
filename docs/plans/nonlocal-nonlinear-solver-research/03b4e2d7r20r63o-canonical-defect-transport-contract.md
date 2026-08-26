# NSR3-B4E2D7R20R63O canonical-defect transport contract -- revision 1

| Field | Value |
|---|---|
| Research ID | `NSR3-B4E2D7R20R63O` |
| Architecture snapshot | R63N semantic `b8d0c708...f02772`; exact canonical pass and raw arbitrary-product boundary |
| Engineering consumer | Decide whether direct operator PCG may proceed to sparse realization |
| Claim class | Exhaustive finite-matrix defect enclosure and bounded Krylov replay |
| Claim status target | apparatus/transport boundary, retained/export rejection or two-lane transported matrix-free candidate |
| Budget | One capture-only replay; exact R63N reconstruction; one immutable 10,404-entry `E`; two eight-update PCG lanes; no tolerance, sparse/precision change, timing or state update |

## Exact parent

Require:

```text
R63N semantic       b8d0c7081646d94aaeb441982ac22afee4477f5df426e6311d4ce03079f02772
R63N canonical root 3f13e864147e022d47a12c88a8e0911c78b5f8f11fd5ba15123e07470802c28e
R63N wide failure   623feaec490e9f3c424eb5665fbe77f0b3244815db5bdd15adc1f4d70f27e6bb
R63N export failure 1acc1fcee6e90faba50ca8192fd30f2985ee469fc5e7845ba6c941f82e86c4f1
R63M retained root  4a7686b2948b0951302e4f6c1d397681818ff26c89f1c22eadcfbdaff8ce2220
R63M export root    ed31e5f6f933a56e487b275be3beae4a0e22cd53af76fd5e7b4001d4a7dbb0fd
```

Require the same captured `H`, physical `T`, projector scale, R63I starts,
preconditioners, RHS, reference signs and dense verifier as R63N. Reproduce
both negative raw initial slacks before transported comparison.

## Immutable defect matrix

For every canonical column `j` and row `i`:

1. compute the R63N dense and direct products and their bounds;
2. use error-free `two_sum(dense_center, -direct_center)`;
3. set `center_residual_ij` to the upward sum of the absolute high and low
   difference terms;
4. set

```text
E_ij = up_add(center_residual_ij,
              up_add(dense_bound_ij, direct_bound_ij)).
```

Require finite/no-underflow nonnegative values, 10,404 constructions and the
same increasing basis/row order as R63N. Record `E` root, maximum entry,
maximum upward row sum and exact work. Do not shrink, normalize or multiply
`E` by a safety factor.

## Transported comparison

For arbitrary vector `p`, compute for each row:

```text
model_bound_i = up_sum_j(up_multiply(E_ij, abs(p_j))).
combined_i = up_add(raw_dense_bound_i,
                    up_add(raw_direct_bound_i, model_bound_i)).
```

Use the same error-free center residual as the canonical construction and
require `center_residual_i <= combined_i`. Record 10,404 transport terms,
maximum model bound, minimum transported slack and root per product. Also
publish the untransported minimum slack without using it as a gate.

## PCG replay

Clone R63N center arithmetic exactly. The direct rectangular product drives
the initial residual and all eight `H p` operations. The only new data flow is
offline comparison through immutable `E`; no model bound enters PCG.

Both lanes must complete eight updates, have positive finite `r^T z` and
`p^T q`, and contain all nine products after transport. At least one iterate
must independently certify `24+/78-/0`; continue through update 8. Bind first
passing iteration, minimum original-system error, recurrence/direct residual
drift and all roots.

## Fixed work

- one 10,404-entry defect-matrix construction;
- per arbitrary product: 10,404 upward multiply/add transport terms;
- per complete lane: 9 transports = 93,636 terms;
- both complete lanes: 187,272 transport terms;
- candidate matrix-free work remains 8 factor solves, 9 rectangular products,
  3,753 direct Dot2 reductions and 578,340 direct terms per lane;
- dense comparison, `E` transport and original certificates are verification
  work, excluded from candidate algorithm counts.

Count zero fitted tolerances, defect inflation, sparse zero elision, CSR,
precision conversion, new factor/inverse, dense candidate `H` use, timing or
state updates.

## Controls

1. A fixed full-row-rank rectangular `T` and a dense `H` with one known dyadic
   column defect produce exact `E` and contain a nontrivial signed vector.
2. Zeroing the required `E` entry makes that vector fail containment.
3. Transpose reversal remains observable.
4. An eight-eigenvalue SPD control completes all eight matrix-free PCG updates
   and certifies its known solution under transported comparison.
5. Mutating `E`, a transported bound, vector, PCG scalar, certificate or first
   passing iteration changes the relevant root.
6. Classifier precedence covers all five routes.
7. R63B--R63N byte regressions pass.

## Resolution precedence

1. `CANONICAL_DEFECT_TRANSPORT_APPARATUS_REJECTED`.
2. `CANONICAL_DEFECT_TRANSPORT_REJECTED`.
3. `RETAINED_WIDE_TRANSPORTED_MATRIX_FREE_PCG_REJECTED`.
4. `EXPORTED_FACTOR_WIDE_TRANSPORTED_MATRIX_FREE_PCG_REJECTED`.
5. `EXPORTED_FACTOR_WIDE_TRANSPORTED_MATRIX_FREE_PCG_CANDIDATE`.

## Does not count

Changing R63N; a decimal tolerance; an arbitrary safety factor; excluding the
R63I starts; random-vector-only evidence; allowing `E` into the recurrence;
stored-factor operator substitution; sparse/precision changes; norm-only
agreement; runtime stopping, timing, state replacement, GPU or production
inference.

## Stop and reconsider

- Full pass: freeze a fixed-order sparse/zero-elided direct-product
  correspondence before precision engineering.
- Transport failure: identify the first term and verify the linear enclosure;
  do not inflate `E` or retry PCG first.
- Transport pass but PCG failure: preserve correspondence and investigate the
  executed direct-operator recurrence without changing its bounds.
- R64 and R65 remain blocked for every route.
