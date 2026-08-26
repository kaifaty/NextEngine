# NSR3-B4E2D7R20R63N matrix-free correspondence contract -- revision 1

| Field | Value |
|---|---|
| Research ID | `NSR3-B4E2D7R20R63N` |
| Architecture snapshot | R63M semantic `3fb7fdd6...ed424`; two exact eight-update PCG candidate lanes |
| Engineering consumer | Decide whether PCG may advance from dense-Gram research to sparse/direct operator research |
| Claim class | Exhaustive finite-profile basis correspondence plus bounded PCG replay |
| Claim status target | apparatus/product boundary, retained/export rejection or two-lane matrix-free PCG candidate |
| Budget | One capture-only replay; exact R63M reconstruction; 102 canonical products; two eight-update PCG lanes; no sparse elision, precision change, timing or state update |

## Exact parent and inputs

Require:

```text
R63M semantic       3fb7fdd65f887ae2a304dfdf8f1d10996c296d7a85c72ec79655be55e40ed424
retained PCG root   4a7686b2948b0951302e4f6c1d397681818ff26c89f1c22eadcfbdaff8ce2220
export PCG root     ed31e5f6f933a56e487b275be3beae4a0e22cd53af76fd5e7b4001d4a7dbb0fd
captured H root     aa401d0191ad53b7caa7837c827913fa711e1fdc433bc200c020719d297fb09d
tangent value root  114a73ea34534ae25c0ea33a708944a434987374e18b703df542de8130aa73f1
tangent row root    6788422f80d918f39abfb7f9eb3f839e07f4598e408127fce67303314fcf5635
```

Require `T` dimensions `102 x 315`, 17,748 nonzero values, exact projector
state and the same `sigma=1/(1+eta)` used by the captured JVP. Bind R63I starts,
factors, RHS, reference signs and dense-inverse verifier exactly as R63M.

## Canonical product certificate

For each canonical basis vector `e_j`, `j=0..101`:

1. compute `H e_j` with 102 dense Dot2 reductions;
2. compute `u=T^T e_j` with 315 Dot2 reductions;
3. compute `q=sigma T u` with 102 Dot2 reductions;
4. propagate all inner Dot2 bounds through the outer product and final scale;
5. require `|q_i-(H e_j)_i| <= dq_i+dH_i` for all `i=0..101`.

Record all 10,404 component checks, maximum center residual, maximum combined
bound, minimum slack and roots. The minimum slack must be nonnegative. Do not
replace this exhaustive finite basis with random vectors or a matrix norm.

## Matrix-free PCG replay

Clone the exact R63M recurrence, scalar positivity checks, eight-update budget,
factor solve, every-iterate certificate and no-early-exit rule. Replace only
the two sites that apply dense `H`:

- initial `H x0`;
- each of eight `H p_k` products.

The direct rectangular product drives the recurrence. At every site also
compute a dense product only as an independent bounded comparator and require
all 102 component intervals to overlap. Dense comparison must never replace a
recursive residual, direction or iterate.

Lane success still requires eight completed updates and at least one iterate
with all `24+/78-/0` original R60 signs. Record first passing iteration,
minimum certified error, recurrence/direct-original residual drift, product
slack and roots. A changed matrix-free reduction order is a new contract
revision.

## Fixed work

Candidate algorithm per completed lane:

- 8 factor solves, unchanged from R63M;
- 9 rectangular products;
- per product: 417 Dot2 reductions and 64,260 full rectangular terms;
- per lane: 3,753 Dot2 reductions and 578,340 terms;
- across both lanes: 16 factor solves, 7,506 Dot2 reductions and 1,156,680
  rectangular terms.

Offline canonical audit executes 102 additional rectangular and 102 dense
products. Dense per-site comparisons and original-system certificates are
verification work and must be reported separately. Count zero zero-elision,
CSR construction, dense candidate `H` applications, new factorization/inverse,
precision conversion, timing or state updates.

## Controls

1. A fixed non-axis-aligned full-row-rank `2 x 3` tangent matrix with dyadic
   positive `sigma` passes both canonical products and a known PCG solution.
2. Adding `1` to one frozen tangent coefficient fails canonical containment.
3. Swapping `T` and `T^T` traversal on the rectangular control is observable.
4. Mutating one bound, product center, PCG iterate, drift, certificate or first
   passing iteration changes the corresponding root.
5. Classifier precedence covers every frozen route.
6. R63B--R63M byte regressions pass.

## Resolution precedence

1. `MATRIX_FREE_CORRESPONDENCE_APPARATUS_REJECTED`.
2. `DIRECT_RECTANGULAR_PRODUCT_NOT_CONTAINED`.
3. `RETAINED_WIDE_MATRIX_FREE_PCG_REJECTED`.
4. `EXPORTED_FACTOR_WIDE_MATRIX_FREE_PCG_REJECTED`.
5. `EXPORTED_FACTOR_WIDE_MATRIX_FREE_PCG_CANDIDATE`.

## Does not count

Real-arithmetic identity without executed interval correspondence; only the 18
observed Krylov vectors without canonical basis coverage; norm-only agreement;
using stored factor input instead of physical tangent rows; dense-product
replacement; zero elision/CSR; binary64/compensated arithmetic; wall timing;
iteration-2 runtime stopping; state replacement, following decision, GPU or
production inference.

## Stop and reconsider

- Full pass: research fixed-order sparse/zero-elided tangent application and
  only then hardware-available precision.
- Canonical containment failure: localize scale, projector-identity or bound
  transport before any PCG retry.
- Canonical pass but PCG failure: preserve the product correspondence and
  investigate recurrence sensitivity; do not weaken product or sign gates.
- R64 and R65 remain blocked for every route.
