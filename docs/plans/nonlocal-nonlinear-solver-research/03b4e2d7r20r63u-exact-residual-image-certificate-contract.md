# NSR3-B4E2D7R20R63U exact residual-image certificate contract -- revision 1

| Field | Value |
|---|---|
| Research ID | `NSR3-B4E2D7R20R63U` |
| Architecture snapshot | R63T semantic `8aa1e245...fff2c`; rounded correction floor exposes coarse certificate dominance |
| Exact candidate parent | R63S semantic `97d992f1...54e14`; unchanged two iteration-8 candidates |
| Engineering consumer | Decide structured candidate correctness before wider correction/representation work |
| Claim class | Exact dyadic residual-image solution enclosure and independent component signs |
| Claim status target | apparatus/image boundary, per-lane rejection or two-lane residual-image candidate |
| Budget | Exact R63S replay; one common inverse conversion; two final-candidate residual/image certificates; no candidate updates or timing |

## Frozen roots

Require:

```text
R63S semantic       97d992f18912ed9a223a17080820b72d4da54635534d96dba3dc9e5d71454e14
R63S retained lane  2ec70f637f37e8927a86c071b24029e6d0bf4724968ba40609a46dbc23195376
R63S exported lane  5dcc65f74b8769d5a53cf5db2937f4c2013dc4b92f0333bb42a791006bc3c255
R63R inverse         32508a73b43eab2afd6737761d00b5d166a2b415151fb6ff4d92fd3df0ed4a07
R63R defect          fea187b0a8583cdc08f71521cfcb64f996c4c09c93db71fa94878d24b018b9d5
R63R inverse audit   a86c14bad30ffb45cc65491420b885f41f6bcaf5930014efe71b427a4f092aee
R63Q exact matrix    1e935dbe8a29210c3601038812c7b6b18bcb73b7d82c3f64bbfaf628c982998a
exact RHS            9db272a7000883d5f61884f05a5c86c1465d272b585ce6e9f359f91b99c5898e
R63T semantic        8aa1e245058243cf1d33fd00c7d6688082fe7c708c532fb5f1fb9fd0454fff2c
```

R63T is the frozen causal observation, but none of its rounded correction
values or updated centers enters R63U. Reconstruct the physical candidate
lineage only through R63S.

## Exact residual-image certificate

Convert all 10,404 R63R inverse entries from binary128 to exact dyadics once.
For each 102-component candidate:

1. convert all solution components exactly;
2. compute 10,404 exact products for
   `r_num_i=b_i*d-sum_j H_num_ij*x_j`;
3. compute 10,404 exact products for
   `w_i=sum_j Z_ij*r_num_j`;
4. compute exact infinity norms of `r_num` and `w`;
5. set `error_num=||w||inf` and
   `error_den=d-left_infinity_numerator`;
6. require `error_den>0` and classify each sign by strict comparison of
   `abs(x_i)*error_den` with `error_num`;
7. bind counts, minimum resolved separation, all vectors/norms/signs and
   roots.

Require exactly 102 solution conversions, 20,808 exact products, 204 vector
entries and 102 sign comparisons per candidate. A pass is exactly zero
unresolved components and 102 positive/negative components. No expected sign
counts or external sign vector may enter the function.

The report may project exact residual/image/error fractions to binary128 for
readability. Those projections do not enter any comparison.

## Independence and correspondence

The certificate consumes exact `r_num` directly; it does not project residuals
as R63T did. Report the ratio between the old R63S product-norm numerator and
the new exact image numerator, but do not threshold or classify on it.

Both lane sign vectors must be exactly equal. Equality does not repair a lane
with unresolved signs.

## Controls

1. An identity system with exact inverse gives zero residual-image error and
   certifies known positive/negative signs.
2. `H=I`, `Z=3/4 I` and an inexact candidate verifies the nonzero image bound
   and `1/(1-rho)` algebra.
3. A residual orthogonal to the worst inverse row makes the image certificate
   strictly sharper than the product-norm certificate.
4. A zero/near-zero component remains unresolved; no external signs repair it.
5. Nonpositive denominator, wrong image orientation, missing inverse entry or
   noncontractive defect fails closed.
6. Mutating solution, residual numerator, image numerator, bound, sign or lane
   identity changes the relevant root.
7. Classifier precedence covers all routes.
8. R63B--R63T byte regressions pass.

## Resolution precedence

1. `EXACT_RESIDUAL_IMAGE_APPARATUS_REJECTED`.
2. `EXACT_RESIDUAL_IMAGE_CERTIFICATE_REJECTED`.
3. `RETAINED_WIDE_RESIDUAL_IMAGE_REJECTED`.
4. `EXPORTED_FACTOR_RESIDUAL_IMAGE_REJECTED`.
5. `TWO_LANE_EXACT_RESIDUAL_IMAGE_CANDIDATE`.

## Does not count

Rounded R63T correction; candidate update; R60 signs/reference solution;
stored dense `H/X`; componentwise fitted tolerance; wider arithmetic choice;
factor correction; sparse work; timing; state/following transition;
runtime/GPU/production inference.

## Stop and reconsider

- Two-lane candidate: freeze earliest-iterate image certification, then a
  bounded finite residual-image verifier; do not integrate exact dyadics.
- One-lane rejection: isolate candidate arithmetic under the same certificate.
- Two-lane rejection: quantify exact image versus rounded R63T image before
  wider candidate work.
- R64 and R65 remain blocked for every route.
