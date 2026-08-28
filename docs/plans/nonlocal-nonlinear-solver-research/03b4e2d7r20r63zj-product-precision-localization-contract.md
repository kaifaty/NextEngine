# NSR3-B4E2D7R20R63ZJ product-precision localization contract

Revision: `1`.

| Field | Value |
|---|---|
| Research ID | `NSR3-B4E2D7R20R63ZJ` |
| Architecture snapshot | SPEC-38/ADR-076 `Proposed`; ADR-081 `Accepted`; R63ZI `AUTHOR_PASS / REVIEW_NOT_TESTED / BOUNDED_REJECTED` |
| Engineering consumer | decide whether the R63ZI loss is caused by dense width-two operator storage/product arithmetic or by product projection/the remaining K2 recurrence |
| Claim class | fixed-profile numerical localization claim |
| Budget | one three-product hybrid lane, three frozen endpoint controls, two Release repeats, then stop before representation design |

## Exact claim

Use the frozen R63ZI projected K2 RHS/inverse scale, exported factor,
permutation, two-update PCG order and R63Y verifier unchanged. At each of the
three operator sites only:

1. reconstruct the current K2 input center exactly as binary128 `high+low`;
2. apply the frozen binary128 tangent product `sigma*T*(T^T*x)` in its existing
   fixed order;
3. project every binary128 output immediately to canonical K2;
4. return only that sealed K2 product to the unchanged recurrence.

The hybrid must not read verifier state, the exact oracle, the R63ZI dense
artifact center or a baseline solution.

If this hybrid restores reject/reject/pass with state 2 exactly
`24+/78-/0?` and sign root `89b2908b...6094`, while the frozen R63ZI dense K2
lane still rejects, select
`DENSE_TWOFOLD_OPERATOR_PRODUCT_PRECISION_INSUFFICIENT`.

If the valid hybrid still rejects state 2, select
`TWOFOLD_PRODUCT_PROJECTION_OR_RECURRENCE_INSUFFICIENT` and localize product
projection versus updates before choosing a wider representation.

## Fixed correspondence and work

- Dimension `102`, tangent width `315`, exactly three operator calls and three
  sealed states.
- Exact product audit: all `306` hybrid product rows are compared against
  exact dyadic `K_T*x` after the transaction seals.
- Hybrid operator work: `945` inner dots, `306` outer dots, `96,390` inner
  terms, `96,390` outer terms and `306` scale products.
- K2 recurrence work remains exactly R63ZI: three factor solves, two rho dots,
  two denominator dots, three scalar divisions, 204 solution updates, 204
  residual updates and 102 direction updates.
- Frozen endpoints: R63ZC tangent/projected binary128 passes; R63ZI dense K2
  rejects; common K2 rejects.

## Controls and firewall

Require sealed callback identity, exact K2 input reconstruction, product
projection containment, stale/tangent/input/product mutation rejection,
nonfinite failure, result sealing, fixed-work rejection and exhaustive route
precedence. The exact oracle runs only after the candidate transaction seals.

Does not count: a binary128 factor/dot/update, original rather than projected
inputs, an extra iteration, residual replacement, verifier feedback, a stored
width-three matrix, tolerance fitting, timing, successful compilation or an
author-only production claim.

## Stop and next decision

- On passing hybrid: freeze a portable operator-product representation
  discriminator; do not widen unrelated recurrence state.
- On rejecting hybrid: freeze a product-output-width versus K2-update
  factorial before choosing width three.
- On apparatus/control failure: `INCONCLUSIVE`; repair the apparatus rather
  than interpreting the endpoint.
- In all cases dynamic building, corpus, adaptive stopping, runtime/Rust/GPU
  integration, ProductChecks and production promotion remain blocked.
