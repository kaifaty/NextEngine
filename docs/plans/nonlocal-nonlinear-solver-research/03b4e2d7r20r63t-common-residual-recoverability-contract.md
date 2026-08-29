# NSR3-B4E2D7R20R63T common-residual recoverability contract -- revision 1

| Field | Value |
|---|---|
| Research ID | `NSR3-B4E2D7R20R63T` |
| Architecture snapshot | R63S semantic `97d992f1...54e14`; both common-PCG lanes exact, finite and rejected with 66 unresolved signs |
| Engineering consumer | Decide mathematical recoverability before factor/finite-operator redesign |
| Claim class | Sixteen fixed exact-common-residual refinement updates plus independent common sign certificates |
| Claim status target | apparatus/generation boundary, per-lane rejection or common-residual recoverability candidate |
| Budget | Exact R63S replay; two depth-16 offline `Z` correction lanes; 32 independent updated-center certificates; no replacement, sparse work or timing |

## Exact parent

Require R63S at:

```text
semantic       97d992f18912ed9a223a17080820b72d4da54635534d96dba3dc9e5d71454e14
stdout         15568ae0d50ea3285da6d12cc07ba80d650994bd95b833daaae71868c6404dd1
retained lane  2ec70f637f37e8927a86c071b24029e6d0bf4724968ba40609a46dbc23195376
exported lane  5dcc65f74b8769d5a53cf5db2937f4c2013dc4b92f0333bb42a791006bc3c255
final signs    4dc6f1bc8ba00c9bcfa22589b4b586324aa71cc9c811af04cd49d0b70f04150c
R63R inverse   32508a73b43eab2afd6737761d00b5d166a2b415151fb6ff4d92fd3df0ed4a07
R63R defect    fea187b0a8583cdc08f71521cfcb64f996c4c09c93db71fa94878d24b018b9d5
```

Both parent lanes must be exact, finite, complete, rejected and have exactly
`12 positive / 24 negative / 66 unresolved` final certificates. R63S
candidate recurrence and certificates repeat unmodified.

## Exact common residual projection

For binary128 center `x`, convert all components exactly and form

```text
r_num_i = b_i d - sum_j H_num_ij x_j,
r_i     = round_binary128(r_num_i/d).
```

Require exactly 102 center conversions, 10,404 exact products and 102 finite
binary128 residual projections per update. Convert every projected residual
back to an exact dyadic and bind

```text
projection_error_num_i = abs(r_i d-r_num_i).
```

Bind the residual numerator vector, projected vector, projection-error vector,
infinity values, counts and roots. Mutation of center, RHS, matrix numerator,
`d`, projection or error must change a root. There is no tolerance gate on the
projection error; the independent updated-center certificate decides utility.

## Offline correction transaction

For each lane, start from its exact R63S iteration-8 solution. At updates
`1..16`:

1. compute one fresh exact/projection residual record;
2. compute all 102 rows of `delta=Z*r` by increasing-column binary128 Dot2;
3. require 102 exact/no-underflow finite Dot2 corrections;
4. update all components as `x_new=x+delta` by binary128 Dot2;
5. require 102 exact/no-underflow finite updates;
6. run a fresh R63S exact common certificate on `x_new`.

Record correction/update roots and the independent certificate at every
iteration. Execute all 16 updates with no adaptive exit. Record first passing
iteration in `1..16` with a separate `has_pass` bit. A lane passes only if a
certificate passes and every later one through 16 also passes.

No analytic `rho^k` value, projected residual or correction norm can pass a
lane. Both final independently derived sign vectors must be equal.

## Fixed new work per complete lane

- 16 residual records: 1,632 center conversions, 166,464 exact common
  products, 1,632 residual projections and 1,632 projection-error terms;
- 16 dense verifier applications: 1,632 Dot2 rows and 166,464 Dot2 terms;
- 1,632 center-update Dot2 reductions;
- 16 independent certificates: 1,632 solution conversions, 166,464 exact
  common residual products and 1,632 sign comparisons.

Across two lanes require 32 correction residuals, 32 dense `Z` applications,
32 certificates, 332,928 correction Dot2 terms and 665,856 new exact common
products. Parent R63S work is reported separately. No stored dense `H`, dense
`X`, factor solve, sparse product or timing work is allowed.

## Controls

1. A two-dimensional identity system with `Z=I` recovers an inexact center in
   one update and certifies both signs.
2. A literal `Z=3/4 I` gives `rho=1/4`, contracts monotonically and has a known
   first pass under strict interval comparison.
3. Wrong residual sign/orientation does not reproduce the known correction.
4. Nonpositive `d`, noncontractive defect, nonfinite projection/correction or
   missing update fails closed.
5. Exact residual projection and round-trip error roots react independently to
   mutations.
6. Mutating correction, center, certificate sign or first-pass location
   changes the lane root.
7. Classifier precedence covers every route.
8. R63B--R63S byte regressions pass.

## Resolution precedence

1. `COMMON_RESIDUAL_RECOVERABILITY_APPARATUS_REJECTED`.
2. `COMMON_RESIDUAL_CORRECTION_GENERATION_REJECTED`.
3. `RETAINED_WIDE_COMMON_RESIDUAL_REFINEMENT_REJECTED`.
4. `EXPORTED_FACTOR_COMMON_RESIDUAL_REFINEMENT_REJECTED`.
5. `COMMON_RESIDUAL_RECOVERABILITY_CANDIDATE`.

## Does not count

Using R60 signs or stored dense `H/X`; treating dense `Z` as runtime eligible;
replacing the parent candidate/state; executing a following nonlinear step;
changing factor/operator/precision/rank/rows/regularization; more PCG;
sparse realization; timing; runtime/GPU/production inference.

## Stop and reconsider

- Two-lane candidate: freeze factor-based common-residual refinement next;
  dense `Z` remains verifier/diagnostic only.
- Exported rejection only: isolate exported factor correction arithmetic.
- Both lane rejections after contracting to a floor: research wider
  correction/update arithmetic; do not extend this budget.
- Noncontraction: audit residual projection/orientation and exact common
  algebra before another representation.
- R64 and R65 remain blocked for every route.
