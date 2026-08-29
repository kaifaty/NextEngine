# NSR3-B4E2D7R20R63S common-operator PCG certificate contract -- revision 1

| Field | Value |
|---|---|
| Research ID | `NSR3-B4E2D7R20R63S` |
| Architecture snapshot | R63R semantic `b8783c53...f382`; exact common operator and two-sided inverse candidate |
| Engineering consumer | Establish structured solver correctness for the immutable RHS |
| Claim class | Exact common residual, inverse-bound solution enclosure and eight-update direct PCG replay |
| Claim status target | apparatus/certificate boundary, retained/export rejection or two-lane common-PCG candidate |
| Budget | Two initial certificates; two eight-update direct PCG lanes; 18 exact common certificates; no dense oracle, sparse work or timing |

## Exact parent

Require:

```text
R63R semantic       b8783c5390382c3169ff49c9d7df7c8facce50a15c7f46804ed10eb0bb65f382
R63R factor         fde9aef5ee0233c920ab76c855c612cc722422c67f44d18598211ae901082ec4
R63R inverse        32508a73b43eab2afd6737761d00b5d166a2b415151fb6ff4d92fd3df0ed4a07
R63R defect         fea187b0a8583cdc08f71521cfcb64f996c4c09c93db71fa94878d24b018b9d5
R63R inverse audit  a86c14bad30ffb45cc65491420b885f41f6bcaf5930014efe71b427a4f092aee
R63Q oracle         221b475869ab34ee919d1d3add4aa993ab361e2dd914be308722f03726c41266
R63I RHS            64be49510b51f8e9898ed2d021fb2d41d98690cecebe5d95e0a108406cf192b1
```

Reconstruct exact R63R, then reconstruct R63I retained/export-wide starts and
preconditioner factors at their frozen roots. Dense `H`, dense `X`, R60
reference solution and R60 sign vector are not passed to the lane or
certificate functions.

## Common certificate

Convert RHS once to exact dyadics. Per candidate solution:

1. convert exactly 102 solution components;
2. compute exactly 10,404 `H_num*x` products;
3. form all 102 residual numerators and their exact infinity norm;
4. set `error_num=||Z||inf * residual_inf_num` and
   `error_den=d-left_inf_num`;
5. classify each component by strict comparison of
   `abs(x_i)*error_den` and `error_num`;
6. bind positive, negative, unresolved, minimum exact separation, residual,
   error projection, sign vector and all roots.

Require positive `error_den`. A pass is exactly `unresolved=0` and
`positive+negative=102`. No external reference signs are allowed.

## PCG lanes

Use the R63N direct product center without dense comparison. For each lane:

- certify immutable start as iteration 0;
- construct initial residual from the direct tangent product;
- run exactly eight positive finite PCG updates;
- use one frozen factor preconditioner solve per update;
- certify every updated solution against exact common `H*`;
- record first passing iteration in `0..8` with a separate `has_pass` bit;
- require every certificate from first pass through iteration 8 to pass.

Both lane final sign vectors must be exactly equal. Continue after first pass;
no certificate value enters the recurrence.

## Fixed work per complete lane

- nine direct products: 2,835 inner and 918 outer Dot2 reductions;
- eight preconditioner solves: 41,208 forward, 41,208 backward terms and
  1,632 divisions;
- 102 initial-residual Dot2 reductions, eight rho and eight denominator dots;
- 816 solution, 816 residual and 714 direction update dots;
- nine exact common certificates: 918 solution conversions, 93,636 exact
  residual products and 918 sign comparisons.

Across both lanes require 18 products, 16 factor solves, 187,272 exact common
residual products and zero dense comparator/certificate work.

## Controls

1. An exact dyadic identity operator certifies known positive/negative signs
   with zero residual.
2. A perturbed solution yields a nonzero exact residual and bounded error.
3. A zero/near-zero component remains unresolved; external signs cannot force
   it resolved.
4. Invalid/nonpositive amplification denominator rejects the certificate.
5. Existing direct-product and PCG breakdown controls remain exact.
6. Mutation of solution, residual, error, sign or first-pass location changes
   the relevant root.
7. Classifier precedence covers every route.
8. R63B--R63R byte regressions pass.

## Resolution precedence

1. `COMMON_OPERATOR_PCG_APPARATUS_REJECTED`.
2. `COMMON_OPERATOR_CERTIFICATE_REJECTED`.
3. `RETAINED_WIDE_COMMON_OPERATOR_PCG_REJECTED`.
4. `EXPORTED_FACTOR_COMMON_OPERATOR_PCG_REJECTED`.
5. `EXPORTED_FACTOR_COMMON_OPERATOR_PCG_CANDIDATE`.

## Does not count

Dense `H` product or `X`; R60 signs; transported dense/direct bounds; a
rounded residual-only test; stopping at first pass; changing factor/operator/
precision/rank/rows; sparse/timing/state/runtime/GPU/production work.

## Stop and reconsider

- Two-lane candidate: qualify lower-precision storage/accumulation against the
  same exact certificate before sparse realization and performance work.
- Certificate failure: audit exact residual orientation and amplification
  algebra; do not fall back to dense signs.
- Lane failure: locate whether recurrence arithmetic or preconditioner storage
  causes the residual floor, then freeze refinement/wider consumption if
  needed.
- R64 and R65 remain blocked for every route.

