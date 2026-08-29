# NSR3-B4E2D7R20R63ZH center-producer decomposition contract -- revision 1

| Field | Value |
|---|---|
| Research ID | `NSR3-B4E2D7R20R63ZH` |
| Parent | reviewed R63ZG result `989886a4...6a5e5` |
| Architecture snapshot | commit `52d7eec62fe94e651a8bdea822d7cb001d30ece3`; SPEC-38/ADR-076 `Proposed`, ADR-081 `Accepted` guardrail |
| Engineering consumer | decide whether to stop certificate repair and require original-equation-preserving operator use |
| Claim class | fixed-profile exact-dyadic/finite center-producer correspondence |
| Claim status target | one reviewed route distinguishing finite center defect from true verifier-model residual |
| Budget | one reviewed R63ZG prefix, two immutable details/solution expansions, 204 exact centers, 102 exact transport identities, no timing |

## Exact claim and negation

On the frozen R63ZG tangent/common final solutions and one immutable R63Y
profile, reconstruct the mathematical sum of every component-center affine
image with an implementation-independent exact dyadic oracle, prove or refute
containment by the existing R63Y image intervals, and test in all 102 rows:

```text
r_s[i] = v[i] - sum_j M[i,j] x_s[j],  s in {t,c}
r_c[i] - r_t[i] = -sum_j M[i,j] (x_c[j] - x_t[j])
```

The positive result is the first identity-, work-, oracle-, containment-,
maximum-row-, transport- and control-closed route deciding whether the common
center is a faithful fixed-profile residual. The negation is the first frozen
identity, solution-expansion, exact-conversion, containment, indexing,
transport, sealing or incomplete-publication failure.

## Frozen model and identity

- Parent commit/result/stdout:
  `52d7eec62fe94e651a8bdea822d7cb001d30ece3`,
  `989886a4...6a5e5`, `ca2a0f80...029c9`.
- Parent executable diff/binary/cache:
  `e42b50c7...fbf9`, `9f63e48c...d98f`, `23dbf605...bb84`.
- Fixture/profile:
  `7780543a...4553`, `b1c43044...a291`, width/dimension `2/102`.
- Tangent/common raw solution roots:
  `38f8d0b2...0baea`, `f7e28b44...f59e8`.
- Tangent/common detail roots:
  `69cc0d3d...a06e`, `3a6549ae...eb02`.
- Tangent/common center-infinity bits:
  `0x3f5067b8ad420000`, `0x42bbe2b2b259004d`.
- Tangent/common radius-infinity bits:
  `0x3db0505f8daea2d2`, `0x3f6be2b2b4492e06`.
- R63ZG observations/work/result roots:
  `0d506330...0804`, `5ce8dd9c...d0ed`, `989886a4...6a5e5`.

R63ZF/R63ZG recurrence, endpoints, detail vectors, certificates, cells,
classification, work and controls remain unchanged.

## Arithmetic model

The candidate is the unchanged strict-binary64 width-two R63Y center and
outward image radius. The reference oracle is a local canonical dyadic:

```text
Dyadic = integer * 2^exponent
```

with arbitrary-width signed `cpp_int` integer. Binary64 values are decoded
directly from sign/exponent/fraction bits. Addition aligns integer exponents;
multiplication multiplies integers and adds exponents; zero has one canonical
form. Equality and absolute ordering normalize powers of two exactly.

The oracle must not call Dot2Err, use an R63Y center/result/error as an input,
use binary64/long-double accumulation, apply epsilon, rescale data, drop terms
or stop at the maximum row. It consumes only immutable profile and solution
component arrays and computes all rows in fixed row/column/part order.

## Private solution-expansion DTO

Add a private report-only `FormulaProbeSolutionExpansion` and wrapper around
the unchanged R63Y solution conversion. It contains:

```text
exact, width, dimension, containments, nonzero_lows
components[dimension*width], radius[dimension]
source_solution_root, component_root, radius_root, root
```

The producer must validate the complete parent fixture and solution shape,
construct the unchanged width-two solution once, copy its components/radii,
bind every field/root and fail without partial publication. It must not expose
or change an operator, certificate policy or solver state. R63ZG DTO and output
remain byte-identical.

## Fixed execution

1. Execute the reviewed R63ZG parent once and require its route, roots, work
   and observations exactly.
2. Produce tangent then common solution expansions and require their roots to
   correspond to the immutable R63ZG solution/detail identities.
3. Decode profile vector/matrix components and both solution component arrays
   into independent exact dyadics.
4. For solution `T` then `C`, row `0..101`, accumulate:

```text
vector_exact = sum_part v_component[row,part]
product_exact = sum_column,mpart,xpart
                  M_component[row,column,mpart]
                * x_component[column,xpart]
center_exact = vector_exact - product_exact
```

5. Require `center_exact` to lie within the closed exact interval obtained by
   decoding `image_center[row] +/- image_radius[row]`. The endpoint arithmetic
   for this check is exact dyadic, not binary64 subtraction.
6. Select the first exact maximum-absolute row with deterministic lowest-index
   tie break. Require it to equal the first maximum of the immutable finite
   center vector by binary64 absolute comparison.
7. For every row compute exact component-center displacement and transport:

```text
delta_x[column,part] = x_c[column,part] - x_t[column,part]
delta_r[row] = center_c_exact[row] - center_t_exact[row]
transport[row] = -sum_column,mpart,xpart
                    M_component[row,column,mpart]
                  * delta_x[column,xpart]
```

   Require literal canonical-dyadic equality `delta_r == transport` in all 102
   rows.
8. At the common maximum row bind vector, tangent/common product, tangent/common
   center, displacement transport, interval and per-column contribution roots.
9. Require the common maximum-row image interval to exclude zero and the
   reviewed common `GC` cells to reject both solutions while common `GR` cells
   pass both.

No row may stop execution of a later row.

## Classification

After ordered gates:

1. `CENTER_PRODUCER_APPARATUS_REJECTED`;
2. `CENTER_PRODUCER_IDENTITY_REJECTED`;
3. `CENTER_PRODUCER_WORK_REJECTED`;
4. `FINITE_CENTER_ARITHMETIC_DEFECT` when any exact center escapes its
   immutable image enclosure;
5. `CENTER_PRODUCER_MAXIMUM_ROW_REJECTED`;
6. `CENTER_PRODUCER_TRANSPORT_REJECTED`;
7. `COMMON_SOLUTION_OUTSIDE_VERIFIER_AFFINE_MODEL` when all exact centers are
   contained, the common maximum-row interval excludes zero, all exact
   transport identities close and the reviewed R63ZG center/radius cells are
   reproduced;
8. otherwise `CENTER_PRODUCER_FURTHER_FACTORIAL_REQUIRED`.

Routes 4 and 7 are mutually exclusive. A route-6 failure invalidates any
operator/certificate interpretation rather than selecting a correction.

## Fixed work

The parent R63ZG work remains exactly unchanged. The R63ZH audit adds:

```text
solution expansions                              2
copied solution components/radii          408 / 204
exact binary64 decodes
  profile vector/matrix                 204 / 20808
  tangent/common solution components      204 / 204
exact center rows/products                 204 / 83232
exact center containments                         204
exact solution-component differences              204
exact transport rows/products              102 / 41616
exact transport equalities                         102
maximum-row scans                                  204
maximum-row per-column contributions               102
candidate/operator/solver/certificate updates  0 / 0 / 0 / 0
```

Integer alignment, normalization and limb operations are implementation work
inside the independent exact oracle and are not credited as candidate solver
work. Their deterministic aggregate count/root must still be published so
early exit or hidden recomputation cannot pass.

## Controls

1. R63ZE, R63ZF, R63ZG, parent cache and kernel-smoke outputs remain
   byte-identical.
2. Bind every DTO field, exact row/vector/product/center, interval result,
   maximum-row record, displacement, transport, work ledger and route.
3. Exhaustively enumerate classifier precedence and all route-4/7 booleans.
4. Mutate and reseal one vector component, matrix component, tangent solution
   component and common solution component; each must change exact/result
   identities. A stale component swap rejects.
5. Deliberately reverse the transport sign and transpose one matrix access;
   both must fail exact transport while the unmodified oracle closes.
6. Shrink one immutable image radius by one representable step without
   resealing and require identity rejection; a separately constructed
   synthetic interval excluding its exact center must exercise route 4.
7. Invalid dimension, nonfinite component, dyadic decode overflow/invalid
   exponent and incomplete expansion reject before exact-row publication with
   exact work prefixes.
8. A corrupted non-profile fixture field rejects at the public solution DTO
   boundary.
9. Freeze commit/source/binary/cache/command/stdout hashes, run Release twice,
   require byte identity and one bounded independent review.

The synthetic route-4 control is apparatus evidence only and cannot be
reported as a defect of the immutable R63Y fixture.

## Resolution and claim ceiling

`COMMON_SOLUTION_OUTSIDE_VERIFIER_AFFINE_MODEL` supports only that the fixed
common endpoint's large R63Y center is a genuine component-center residual of
the frozen verifier affine profile and is exactly transported by its solution
displacement. Combined with reviewed R63ZC, it ends this certificate-repair
lineage and allows research of a common operator only as an
original-equation-preserving accelerator/preconditioner candidate.

`FINITE_CENTER_ARITHMETIC_DEFECT` supports a repair campaign only after review
confirms the exact oracle and immutable interval identity. Neither route
selects a production algorithm, validates a corpus, measures performance or
authorizes CPU/GPU/runtime/production integration.

## Stop and reconsider

Stop `INCONCLUSIVE` on apparatus, identity, work, exact-oracle, transport or
load-bearing review defect. Apply at most one batched repair and one re-review.
Do not widen a radius/tolerance, change an operator/profile/solution, reuse
Dot2Err in the oracle, run timing or open a deeper per-term factorial under
revision 1.

Reconsider this contract only if a frozen R63ZG identity changes or independent
review shows that component-center residual is not the relevant verifier
quantity.
